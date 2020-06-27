//! See the [locktree crate](https://crates.io/crates/locktree).

#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{
    braced, custom_keyword, parenthesized,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token::Paren,
    AngleBracketedGenericArguments, Ident, Path, Token,
};

struct LockTree {
    map: HashMap<proc_macro2::Ident, LockSequence>,
}

impl Parse for LockTree {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut map = HashMap::new();
        while !input.is_empty() {
            let name = input.parse::<Ident>()?;
            let seq;
            braced!(seq in input);
            map.insert(name, seq.parse::<LockSequence>()?);
        }

        Ok(LockTree { map })
    }
}

struct LockSequence {
    seq: Vec<Lock>,
}

impl Parse for LockSequence {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            seq: Punctuated::<Lock, Token![,]>::parse_terminated(input)?
                .into_iter()
                .collect(),
        })
    }
}

struct Lock {
    name: String,
    ty: LockType,
}

impl Lock {
    fn fragment(&self, struct_prefix: &str) -> Fragment {
        let forward = self.forward(struct_prefix);
        let name =
            proc_macro2::Ident::new(&self.name, proc_macro2::Span::call_site());
        let type_declaraction = self.ty.declaration();
        let init_var = proc_macro2::Ident::new(
            &format!("{}_value", &self.name),
            proc_macro2::Span::call_site(),
        );
        let generics = self.ty.generics();

        Fragment {
            main_accessors: self
                .ty
                .accessor_functions(&self.name, &forward, true),
            forward_accessors: self
                .ty
                .accessor_functions(&self.name, &forward, false),
            forward,
            lock_declaration: quote! {
                #name: #type_declaraction,
            },
            init_arg: quote! {
                #init_var: #generics
            },
            init_statement: quote! {
                #name: ::locktree::New::new(#init_var),
            },
        }
    }

    fn forward(&self, struct_prefix: &str) -> String {
        format!("{}{}", struct_prefix, snake_to_camel_case(&self.name))
    }
}

impl Parse for Lock {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?.to_string();
        input.parse::<Token![:]>()?;
        let ty = input.parse::<LockType>()?;

        Ok(Self { name, ty })
    }
}

struct LockType {
    is_async: bool,
    declaration: TokenStream,
    generics: TokenStream,
    interface: LockInterface,
}

impl LockType {
    fn accessor_functions(
        &self,
        name: &str,
        forward: &str,
        is_entry_point: bool,
    ) -> TokenStream {
        let name =
            proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
        let forward =
            proc_macro2::Ident::new(&forward, proc_macro2::Span::call_site());
        let accessor = if is_entry_point {
            quote! {
                self
            }
        } else {
            quote! {
                self.locks
            }
        };

        self.interface.accessor_functions(
            self.is_async,
            &name,
            &forward,
            &accessor,
            &self.declaration,
        )
    }

    fn declaration(&self) -> &TokenStream {
        &self.declaration
    }

    fn generics(&self) -> &TokenStream {
        &self.generics
    }
}

impl Parse for LockType {
    fn parse(input: ParseStream) -> Result<Self> {
        let is_async = input.peek(Token![async]);
        if is_async {
            input.parse::<Token![async]>().unwrap();
        }

        let interface = input.parse::<LockInterface>()?;
        let hkt = if input.peek(Paren) {
            let hkt;
            parenthesized!(hkt in input);

            hkt.parse::<Path>()?.into_token_stream()
        } else {
            if is_async {
                return Err(syn::Error::new(
                    input.span(),
                    "async locks must have an explicit HKT",
                ));
            }

            interface.default_concrete_type()
        };
        let generics = input
            .parse::<AngleBracketedGenericArguments>()?
            .args
            .to_token_stream();

        Ok(Self {
            is_async,
            declaration: quote! {
                #hkt<#generics>
            },
            generics,
            interface,
        })
    }
}

#[derive(Clone, Copy)]
enum LockInterface {
    Mutex,
    RwLock,
}

impl LockInterface {
    fn default_concrete_type(&self) -> TokenStream {
        match self {
            Self::Mutex => quote! {
                ::std::sync::Mutex
            },
            Self::RwLock => quote! {
                ::std::sync::RwLock
            },
        }
    }

    fn accessor_functions(
        &self,
        is_async: bool,
        name: &proc_macro2::Ident,
        forward: &proc_macro2::Ident,
        accessor: &TokenStream,
        declaration: &TokenStream,
    ) -> TokenStream {
        match self {
            Self::Mutex => {
                let lock_fn_name = proc_macro2::Ident::new(
                    &format!("lock_{}", name),
                    proc_macro2::Span::call_site(),
                );
                let async_keyword = if is_async { "Async" } else { "" };
                let guard = proc_macro2::Ident::new(
                    &format!("Plugged{}MutexGuard", async_keyword),
                    proc_macro2::Span::call_site(),
                );
                let lock = proc_macro2::Ident::new(
                    &format!("{}Mutex", async_keyword),
                    proc_macro2::Span::call_site(),
                );

                quote! {
                    pub fn #lock_fn_name<'a>(
                        &'a mut self
                    ) -> (
                        ::locktree::#guard<'a, #declaration>,
                        #forward<'a>
                    ) {
                        (::locktree::#lock::lock(&#accessor.#name), #forward { locks: #accessor })
                    }
                }
            }
            Self::RwLock => {
                let read_fn_name = proc_macro2::Ident::new(
                    &format!("read_{}", name),
                    proc_macro2::Span::call_site(),
                );
                let write_fn_name = proc_macro2::Ident::new(
                    &format!("write_{}", name),
                    proc_macro2::Span::call_site(),
                );
                let async_keyword = if is_async { "Async" } else { "" };
                let read_guard = proc_macro2::Ident::new(
                    &format!("Plugged{}RwLockReadGuard", async_keyword),
                    proc_macro2::Span::call_site(),
                );
                let write_guard = proc_macro2::Ident::new(
                    &format!("Plugged{}RwLockWriteGuard", async_keyword),
                    proc_macro2::Span::call_site(),
                );
                let lock = proc_macro2::Ident::new(
                    &format!("{}RwLock", async_keyword),
                    proc_macro2::Span::call_site(),
                );

                quote! {
                    pub fn #read_fn_name<'a>(
                        &'a mut self
                    ) -> (
                        ::locktree::#read_guard<'a, #declaration>,
                        #forward<'a>
                    ) {
                        (::locktree::#lock::read(&#accessor.#name), #forward { locks: #accessor })
                    }

                    pub fn #write_fn_name<'a>(
                        &'a mut self
                    ) -> (
                        ::locktree::#write_guard<'a, #declaration>,
                        #forward<'a>
                    ) {
                        (::locktree::#lock::write(&#accessor.#name), #forward { locks: #accessor })
                    }
                }
            }
        }
    }
}

impl Parse for LockInterface {
    fn parse(input: ParseStream) -> Result<Self> {
        custom_keyword!(Mutex);
        custom_keyword!(RwLock);

        let lookahead = input.lookahead1();
        if lookahead.peek(Mutex) {
            input.parse::<Mutex>().unwrap();

            Ok(Self::Mutex)
        } else if lookahead.peek(RwLock) {
            input.parse::<RwLock>().unwrap();

            Ok(Self::RwLock)
        } else {
            Err(lookahead.error())
        }
    }
}

struct Fragment {
    main_accessors: TokenStream,
    forward_accessors: TokenStream,
    forward: String,
    lock_declaration: TokenStream,
    init_arg: TokenStream,
    init_statement: TokenStream,
}

#[proc_macro]
pub fn locktree(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    locktree_impl(input.into()).into()
}

fn locktree_impl(input: TokenStream) -> TokenStream {
    let map = syn::parse2::<LockTree>(input).unwrap().map;
    let mut code = TokenStream::new();
    for (struct_name, LockSequence { seq }) in map {
        let struct_prefix = format!("{}LockTree", struct_name);
        let main_struct = proc_macro2::Ident::new(
            &struct_prefix,
            proc_macro2::Span::call_site(),
        );
        let fragments = seq
            .into_iter()
            .map(|x| x.fragment(&struct_prefix))
            .collect::<Vec<_>>();

        let init_args = fragments.iter().map(|x| &x.init_arg);
        let init_statements = fragments.iter().map(|x| &x.init_statement);
        let init_fn = quote! {
            pub fn new(#(#init_args),*) -> Self {
                Self {
                    #(#init_statements)*
                }
            }
        };

        let main_accessors = fragments.iter().map(|x| &x.main_accessors);
        let lock_declarations = fragments.iter().map(|x| &x.lock_declaration);
        code.extend(quote! {
            struct #main_struct {
                #(#lock_declarations)*
            }

            impl #main_struct {
                #init_fn

                #(#main_accessors)*
            }
        });

        for (i, fragment) in fragments.iter().enumerate() {
            let name = proc_macro2::Ident::new(
                &fragment.forward,
                proc_macro2::Span::call_site(),
            );
            let forward_accessors =
                fragments[i + 1..].iter().map(|x| &x.forward_accessors);
            code.extend(quote! {
                struct #name<'b> {
                    locks: &'b #main_struct
                }

                impl<'b> #name<'b> {
                    #(#forward_accessors)*
                }
            });
        }
    }

    code
}

fn snake_to_camel_case(x: &str) -> String {
    let mut camel = String::new();
    for word in x.split('_') {
        camel.extend(word.chars().next().unwrap().to_uppercase());
        camel.push_str(&word[1..])
    }

    camel
}
