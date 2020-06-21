//! See the [locktree crate](https://crates.io/crates/locktree).

#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{
    braced, custom_keyword,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    AngleBracketedGenericArguments, Ident, Token,
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
        let inner_type = self.ty.inner_type();
        let new_fn = self.ty.new_fn();

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
                #init_var: #inner_type
            },
            init_statement: quote! {
                #name: #new_fn(#init_var),
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

enum LockType {
    Mutex(TokenStream),
    RwLock(TokenStream),
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
        match self {
            Self::Mutex(generics) => {
                let lock_fn_name = proc_macro2::Ident::new(
                    &format!("lock_{}", name),
                    proc_macro2::Span::call_site(),
                );

                quote! {
                    pub fn #lock_fn_name<'a>(&'a mut self) -> (::std::sync::MutexGuard<'a, #generics>, #forward<'a>) {
                        (#accessor.#name.lock().unwrap(), #forward { locks: #accessor })
                    }
                }
            }
            Self::RwLock(generics) => {
                let read_fn_name = proc_macro2::Ident::new(
                    &format!("read_{}", name),
                    proc_macro2::Span::call_site(),
                );
                let write_fn_name = proc_macro2::Ident::new(
                    &format!("write_{}", name),
                    proc_macro2::Span::call_site(),
                );

                quote! {
                    pub fn #read_fn_name<'a>(&'a mut self) -> (::std::sync::RwLockReadGuard<'a, #generics>, #forward<'a>) {
                        (#accessor.#name.read().unwrap(), #forward { locks: #accessor })
                    }

                    pub fn #write_fn_name<'a>(&'a mut self) -> (::std::sync::RwLockWriteGuard<'a, #generics>, #forward<'a>) {
                        (#accessor.#name.write().unwrap(), #forward { locks: #accessor })
                    }
                }
            }
        }
    }

    fn declaration(&self) -> TokenStream {
        match self {
            Self::Mutex(generics) => quote! {
                ::std::sync::Mutex<#generics>
            },
            Self::RwLock(generics) => quote! {
                ::std::sync::RwLock<#generics>
            },
        }
    }

    fn inner_type(&self) -> &TokenStream {
        match self {
            Self::Mutex(generics) | Self::RwLock(generics) => generics,
        }
    }

    fn new_fn(&self) -> TokenStream {
        match self {
            Self::Mutex(_) => quote! {
                ::std::sync::Mutex::new
            },
            Self::RwLock(_) => quote! {
                ::std::sync::RwLock::new
            },
        }
    }
}

impl Parse for LockType {
    fn parse(input: ParseStream) -> Result<Self> {
        custom_keyword!(Mutex);
        custom_keyword!(RwLock);

        let lookahead = input.lookahead1();
        let constructor: fn(TokenStream) -> Self;
        if lookahead.peek(Mutex) {
            input.parse::<Mutex>().unwrap();
            constructor = |generics| Self::Mutex(generics);
        } else if lookahead.peek(RwLock) {
            input.parse::<RwLock>().unwrap();
            constructor = |generics| Self::RwLock(generics);
        } else {
            return Err(lookahead.error());
        }
        let generics = input
            .parse::<AngleBracketedGenericArguments>()?
            .args
            .to_token_stream();

        Ok(constructor(generics))
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
