use crate::locktree_impl;
use pretty_assertions::assert_eq;
use proc_macro2::TokenStream;

#[test]
fn should_output_nothing_with_empty_input() {
    compare_input_output("", "");
}

#[test]
fn should_output_barebones_with_valid_empty_code() {
    compare_input_output(
        "Main {}",
        "
        struct MainLockTree {
            glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool
        }

        impl MainLockTree {
            pub fn new() -> Self {
                Self {
                    glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool::new(false)
                }
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
            glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool
        }

        impl MainLockTree {
            pub fn new(mutex_value: ()) -> Self {
                Self {
                    mutex: ::locktree::New::new(mutex_value),
                    glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool::new(false)
                }
            }

            pub fn lock_mutex<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedMutexGuard<'a, ::std::sync::Mutex<()>>,
                MainLockTreeMutex<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::Mutex::lock(&self.mutex), MainLockTreeMutex { locks: self })
            }
        }

        struct MainLockTreeMutex<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeMutex<'b> {}

        impl<'b> Drop for MainLockTreeMutex<'b> {
            fn drop(&mut self) {
                self.locks.glsBWeCagvYcGEd.store(false, ::std::sync::atomic::Ordering::Release)
            }
        }
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
            glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool
        }

        impl MainLockTree {
            pub fn new(mutex_value: ()) -> Self {
                Self {
                    mutex: ::locktree::New::new(mutex_value),
                    glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool::new(false)
                }
            }

            pub fn lock_mutex<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedMutexGuard<'a, SuperMutex<()>>,
                MainLockTreeMutex<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::Mutex::lock(&self.mutex), MainLockTreeMutex { locks: self })
            }
        }

        struct MainLockTreeMutex<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeMutex<'b> {}

        impl<'b> Drop for MainLockTreeMutex<'b> {
            fn drop(&mut self) {
                self.locks.glsBWeCagvYcGEd.store(false, ::std::sync::atomic::Ordering::Release)
            }
        }
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
            glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool
        }

        impl MainLockTree {
            pub fn new(mutex_value: ()) -> Self {
                Self {
                    mutex: ::locktree::New::new(mutex_value),
                    glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool::new(false)
                }
            }

            pub fn lock_mutex<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedAsyncMutexGuard<'a, SuperMutex<()>>,
                MainLockTreeMutex<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::AsyncMutex::lock(&self.mutex), MainLockTreeMutex { locks: self })
            }
        }

        struct MainLockTreeMutex<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeMutex<'b> {}

        impl<'b> Drop for MainLockTreeMutex<'b> {
            fn drop(&mut self) {
                self.locks.glsBWeCagvYcGEd.store(false, ::std::sync::atomic::Ordering::Release)
            }
        }
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
            glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool
        }

        impl MainLockTree {
            pub fn new(rw_lock_value: ()) -> Self {
                Self {
                    rw_lock: ::locktree::New::new(rw_lock_value),
                    glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool::new(false)
                }
            }

            pub fn read_rw_lock<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedRwLockReadGuard<'a, ::std::sync::RwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::RwLock::read(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }

            pub fn write_rw_lock<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedRwLockWriteGuard<'a, ::std::sync::RwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::RwLock::write(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }
        }

        struct MainLockTreeRwLock<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeRwLock<'b> {}

        impl<'b> Drop for MainLockTreeRwLock<'b> {
            fn drop(&mut self) {
                self.locks.glsBWeCagvYcGEd.store(false, ::std::sync::atomic::Ordering::Release)
            }
        }
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
            glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool
        }

        impl MainLockTree {
            pub fn new(rw_lock_value: ()) -> Self {
                Self {
                    rw_lock: ::locktree::New::new(rw_lock_value),
                    glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool::new(false)
                }
            }

            pub fn read_rw_lock<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedRwLockReadGuard<'a, SuperRwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::RwLock::read(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }

            pub fn write_rw_lock<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedRwLockWriteGuard<'a, SuperRwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::RwLock::write(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }
        }

        struct MainLockTreeRwLock<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeRwLock<'b> {}

        impl<'b> Drop for MainLockTreeRwLock<'b> {
            fn drop(&mut self) {
                self.locks.glsBWeCagvYcGEd.store(false, ::std::sync::atomic::Ordering::Release)
            }
        }
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
            glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool
        }

        impl MainLockTree {
            pub fn new(rw_lock_value: ()) -> Self {
                Self {
                    rw_lock: ::locktree::New::new(rw_lock_value),
                    glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool::new(false)
                }
            }

            pub fn read_rw_lock<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedAsyncRwLockReadGuard<'a, SuperRwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::AsyncRwLock::read(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }

            pub fn write_rw_lock<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedAsyncRwLockWriteGuard<'a, SuperRwLock<()>>,
                MainLockTreeRwLock<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::AsyncRwLock::write(&self.rw_lock), MainLockTreeRwLock { locks: self })
            }
        }

        struct MainLockTreeRwLock<'b> {
            locks: &'b MainLockTree
        }

        impl<'b> MainLockTreeRwLock<'b> {}

        impl<'b> Drop for MainLockTreeRwLock<'b> {
            fn drop(&mut self) {
                self.locks.glsBWeCagvYcGEd.store(false, ::std::sync::atomic::Ordering::Release)
            }
        }
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
            glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool
        }

        impl MainLockTree {
            pub fn new(mutex0_value: (), mutex1_value: ()) -> Self {
                Self {
                    mutex0: ::locktree::New::new(mutex0_value),
                    mutex1: ::locktree::New::new(mutex1_value),
                    glsBWeCagvYcGEd: ::std::sync::atomic::AtomicBool::new(false)
                }
            }

            pub fn lock_mutex0<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedMutexGuard<'a, ::std::sync::Mutex<()>>,
                MainLockTreeMutex0<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

                (::locktree::Mutex::lock(&self.mutex0), MainLockTreeMutex0 { locks: self })
            }

            pub fn lock_mutex1<'a>(
                &'a self
            ) -> (
                ::locktree::PluggedMutexGuard<'a, ::std::sync::Mutex<()>>,
                MainLockTreeMutex1<'a>
            ) {
                if self.glsBWeCagvYcGEd.compare_and_swap(false, true, ::std::sync::atomic::Ordering::AcqRel) {
                    panic!(\"potential deadlock detected: MainLockTree was locked twice\");
                }

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

        impl<'b> Drop for MainLockTreeMutex0<'b> {
            fn drop(&mut self) {
                self.locks.glsBWeCagvYcGEd.store(false, ::std::sync::atomic::Ordering::Release)
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
