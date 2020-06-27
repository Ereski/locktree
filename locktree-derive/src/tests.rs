use crate::locktree_impl;
use pretty_assertions::assert_eq;
use proc_macro2::TokenStream;

#[test]
fn should_output_nothing_with_empty_input() {
    compare_input_output("", "");
}

#[test]
fn should_output_barebones_with_empty_struct() {
    compare_input_output(
        "Main {}",
        "
        struct MainLockTree {}

        impl MainLockTree {
            pub fn new() -> Self {
                Self {}
            }
        }
        ",
    );
}

#[test]
fn should_handle_a_single_mutex() {
    compare_input_output(
        "
        Main {
            mutex: Mutex<()>
        }
        ",
        "
        struct MainLockTree {
            mutex: ::std::sync::Mutex<()>,
        }

        impl MainLockTree {
            pub fn new(mutex_value: ()) -> Self {
                Self {
                    mutex: ::locktree::New::new(mutex_value),
                }
            }

            pub fn lock_mutex<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedMutexGuard<'a, ::std::sync::Mutex<()>>,
                MainLockTreeMutex<'a>
            ) {
                (::locktree::Mutex::lock(&self.mutex), MainLockTreeMutex { locks: self })
            }
        }

        struct MainLockTreeMutex<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeMutex<'b> {}
        ",
    );
}

#[test]
fn should_handle_a_mutex_with_an_explicit_hkt() {
    compare_input_output(
        "
        Main {
            mutex: Mutex(SuperMutex)<()>
        }
        ",
        "
        struct MainLockTree {
            mutex: SuperMutex<()>,
        }

        impl MainLockTree {
            pub fn new(mutex_value: ()) -> Self {
                Self {
                    mutex: ::locktree::New::new(mutex_value),
                }
            }

            pub fn lock_mutex<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedMutexGuard<'a, SuperMutex<()>>,
                MainLockTreeMutex<'a>
            ) {
                (::locktree::Mutex::lock(&self.mutex), MainLockTreeMutex { locks: self })
            }
        }

        struct MainLockTreeMutex<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeMutex<'b> {}
        ",
    );
}

#[test]
fn should_handle_an_async_mutex() {
    compare_input_output(
        "
        Main {
            mutex: async Mutex(SuperMutex)<()>
        }
        ",
        "
        struct MainLockTree {
            mutex: SuperMutex<()>,
        }

        impl MainLockTree {
            pub fn new(mutex_value: ()) -> Self {
                Self {
                    mutex: ::locktree::New::new(mutex_value),
                }
            }

            pub fn lock_mutex<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedAsyncMutexGuard<'a, SuperMutex<()>>,
                MainLockTreeMutex<'a>
            ) {
                (::locktree::AsyncMutex::lock(&self.mutex), MainLockTreeMutex { locks: self })
            }
        }

        struct MainLockTreeMutex<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeMutex<'b> {}
        ",
    );
}

#[test]
fn should_handle_a_single_rw_lock() {
    compare_input_output(
        "
        Main {
            rw_lock: RwLock<()>
        }
        ",
        "
        struct MainLockTree {
            rw_lock: ::std::sync::RwLock<()>,
        }

        impl MainLockTree {
            pub fn new(rw_lock_value: ()) -> Self {
                Self {
                    rw_lock: ::locktree::New::new(rw_lock_value),
                }
            }

            pub fn read_rw_lock<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedRwLockReadGuard<'a, ::std::sync::RwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                (::locktree::RwLock::read(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }

            pub fn write_rw_lock<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedRwLockWriteGuard<'a, ::std::sync::RwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                (::locktree::RwLock::write(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }
        }

        struct MainLockTreeRwLock<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeRwLock<'b> {}
        ",
    );
}

#[test]
fn should_handle_an_rw_lock_with_an_explicit_hkt() {
    compare_input_output(
        "
        Main {
            rw_lock: RwLock(SuperRwLock)<()>
        }
        ",
        "
        struct MainLockTree {
            rw_lock: SuperRwLock<()>,
        }

        impl MainLockTree {
            pub fn new(rw_lock_value: ()) -> Self {
                Self {
                    rw_lock: ::locktree::New::new(rw_lock_value),
                }
            }

            pub fn read_rw_lock<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedRwLockReadGuard<'a, SuperRwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                (::locktree::RwLock::read(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }

            pub fn write_rw_lock<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedRwLockWriteGuard<'a, SuperRwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                (::locktree::RwLock::write(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }
        }

        struct MainLockTreeRwLock<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeRwLock<'b> {}
        ",
    );
}

#[test]
fn should_handle_an_async_rw_lock() {
    compare_input_output(
        "
        Main {
            rw_lock: async RwLock(SuperRwLock)<()>
        }
        ",
        "
        struct MainLockTree {
            rw_lock: SuperRwLock<()>,
        }

        impl MainLockTree {
            pub fn new(rw_lock_value: ()) -> Self {
                Self {
                    rw_lock: ::locktree::New::new(rw_lock_value),
                }
            }

            pub fn read_rw_lock<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedAsyncRwLockReadGuard<'a, SuperRwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                (::locktree::AsyncRwLock::read(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }

            pub fn write_rw_lock<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedAsyncRwLockWriteGuard<'a, SuperRwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                (::locktree::AsyncRwLock::write(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }
        }

        struct MainLockTreeRwLock<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeRwLock<'b> {}
        ",
    );
}

#[test]
fn should_handle_two_locks() {
    compare_input_output(
        "
        Main {
            mutex0: Mutex<()>,
            mutex1: Mutex<()>,
        }
        ",
        "
        struct MainLockTree {
            mutex0: ::std::sync::Mutex<()>,
            mutex1: ::std::sync::Mutex<()>,
        }

        impl MainLockTree {
            pub fn new(mutex0_value: (), mutex1_value: ()) -> Self {
                Self {
                    mutex0: ::locktree::New::new(mutex0_value),
                    mutex1: ::locktree::New::new(mutex1_value),
                }
            }

            pub fn lock_mutex0<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedMutexGuard<'a, ::std::sync::Mutex<()>>,
                MainLockTreeMutex0<'a>
            ) {
                (::locktree::Mutex::lock(&self.mutex0), MainLockTreeMutex0 { locks: self })
            }

            pub fn lock_mutex1<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedMutexGuard<'a, ::std::sync::Mutex<()>>,
                MainLockTreeMutex1<'a>
            ) {
                (::locktree::Mutex::lock(&self.mutex1), MainLockTreeMutex1 { locks: self })
            }
        }

        struct MainLockTreeMutex0<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeMutex0<'b> {
            pub fn lock_mutex1<'a>(
                &'a mut self
            ) -> (
                ::locktree::PluggedMutexGuard<'a, ::std::sync::Mutex<()>>,
                MainLockTreeMutex1<'a>
            ) {
                (::locktree::Mutex::lock(&self.locks.mutex1), MainLockTreeMutex1 { locks: self.locks })
            }
        }

        struct MainLockTreeMutex1<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeMutex1<'b> {}
        ",
    );
}

fn compare_input_output(input: &str, output: &str) {
    assert_eq!(
        locktree_impl(syn::parse_str(input).unwrap())
            .to_string()
            .replace(" '", "'")
            .replace(" ,", ",")
            .replace(" >", ">"),
        syn::parse_str::<TokenStream>(output)
            .unwrap()
            .to_string()
            .replace(" ,", ",")
            .replace(" >", ">")
    );
}
