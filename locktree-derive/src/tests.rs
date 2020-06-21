use crate::locktree_impl;
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
                    mutex: ::std::sync::Mutex::new(mutex_value),
                }
            }

            pub fn lock_mutex<'a>(
                &'a mut self
            ) -> (::std::sync::MutexGuard<'a, ()>, MainLockTreeMutex<'a>) {
                (self.mutex.lock().unwrap(), MainLockTreeMutex { locks: self })
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
                    rw_lock: ::std::sync::RwLock::new(rw_lock_value),
                }
            }

            pub fn read_rw_lock<'a>(
                &'a mut self
            ) -> (::std::sync::RwLockReadGuard<'a, ()>, MainLockTreeRwLock<'a>) {
                (self.rw_lock.read().unwrap(), MainLockTreeRwLock { locks: self })
            }

            pub fn write_rw_lock<'a>(
                &'a mut self
            ) -> (::std::sync::RwLockWriteGuard<'a, ()>, MainLockTreeRwLock<'a>) {
                (self.rw_lock.write().unwrap(), MainLockTreeRwLock { locks: self })
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
                    mutex0: ::std::sync::Mutex::new(mutex0_value),
                    mutex1: ::std::sync::Mutex::new(mutex1_value),
                }
            }

            pub fn lock_mutex0<'a>(
                &'a mut self
            ) -> (::std::sync::MutexGuard<'a, ()>, MainLockTreeMutex0<'a>) {
                (self.mutex0.lock().unwrap(), MainLockTreeMutex0 { locks: self })
            }

            pub fn lock_mutex1<'a>(
                &'a mut self
            ) -> (::std::sync::MutexGuard<'a, ()>, MainLockTreeMutex1<'a>) {
                (self.mutex1.lock().unwrap(), MainLockTreeMutex1 { locks: self })
            }
        }

        struct MainLockTreeMutex0<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeMutex0<'b> {
            pub fn lock_mutex1<'a>(
                &'a mut self
            ) -> (::std::sync::MutexGuard<'a, ()>, MainLockTreeMutex1<'a>) {
                (self.locks.mutex1.lock().unwrap(), MainLockTreeMutex1 { locks: self.locks })
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
            .replace(" ,", ","),
        syn::parse_str::<TokenStream>(output)
            .unwrap()
            .to_string()
            .replace(" ,", ",")
    );
}
