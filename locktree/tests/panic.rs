use locktree::{locktree, New, Mutex, plug::PlugLifetime, PluggedGuard};
use std::marker::PhantomData;

locktree! {
    Main {
        // Use a dummy lock that does nothing becase otherwise `main()` will
        // deadlock by if `lock_mutex()` doesn't panic
        mutex: Mutex(DummyLock)<()>
    }
}

struct DummyLock<T>(PhantomData<T>);

impl<T> New<T> for DummyLock<T> {
    fn new(_: T) -> Self {
        Self(PhantomData)
    }
}

impl<T> Mutex for DummyLock<T> {
    type Guard = DummyGuard;

    fn lock(&self) -> PluggedGuard<Self::Guard> {
        DummyGuard
    }
}

struct DummyGuard;

impl<'a> PlugLifetime<'a> for DummyGuard {
    type Type = DummyGuard;
}

#[test]
#[should_panic]
fn main_lock_tree_should_panic_on_double_lock() {
    let locks = MainLockTree::new(());
    let _a = locks.lock_mutex();
    // Will panic
    let _ = locks.lock_mutex();
}

#[test]
#[should_panic]
fn main_lock_tree_should_panic_on_double_lock_after_dropping_the_guard() {
    let locks = MainLockTree::new(());
    let _a = locks.lock_mutex().1;
    // Will panic
    let _ = locks.lock_mutex();
}

#[test]
#[should_panic]
fn main_lock_tree_should_panic_on_double_lock_after_dropping_the_forward() {
    let locks = MainLockTree::new(());
    let _a = locks.lock_mutex().0;
    // Will panic
    let _ = locks.lock_mutex();
}
