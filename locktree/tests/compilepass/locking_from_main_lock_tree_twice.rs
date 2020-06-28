use locktree::{locktree, New, Mutex, plug::PlugLifetime, PluggedGuard};
use std::marker::PhantomData;

locktree! {
    Main {
        // Use a dummy lock that does nothing becase will deadlock by trying to
        // acquire the same lock twice
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

fn main() {
    let locks = MainLockTree::new(());
    let _a = locks.lock_mutex();
    let _ = locks.lock_mutex();
}
