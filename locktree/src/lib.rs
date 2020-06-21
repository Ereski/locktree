//! Provable, composable, compile-time deadlock-freedom.
//!
//! # How to Use `locktree`
//!
//! This crate revolves around a macro: `locktree`.
//!
//! # How `locktree` Works (TODO: update with latest format)
//!
//! `locktree` (ab)uses Rust's type system to guarantee that locks are always
//! taken in the same order. Locks under `locktree`'s management are organized
//! into a linear sequence. Locks can only be acquired by moving forward into
//! this sequence, and locks must be released when moving back. Thus it is
//! statically impossible for two threads to acquire the same set of locks in
//! different orders, and so deadlocks are impossible *as long as all locks are
//! managed by `locktree`*.
//!
//! To achieve this, `locktree` relies on a mix of aliasing rules for mutable
//! references, lifetimes, and separate types for each point in the sequence.
//! In the simplest case with a single lock:
//!
//! ```
//! # use locktree::locktree;
//! locktree! {
//!   Main {
//!     main: StdMutex<String>,
//!   }
//! }
//! ```
//!
//! The macro will generate an "entry point" with which you can lock anything
//! (only `main` in this case, with explicit lifetimes for illustration
//! purposes):
//!
//! ```
//! struct MainLockTree {
//!   main: ::std::sync::Mutex<String>,
//! }
//!
//! impl MainLockTree {
//!   fn lock_main<'a>(
//!     &'a mut self
//!   ) -> (::std::sync::MutexGuard<'a, String>, MainLockTreeMain<'a>) {
//!     // ...
//!     # unimplemented!()
//!   }
//! }
//! # struct MainLockTreeMain<'a>(std::marker::PhantomData<&'a ()>);
//! ```
//!
//! All lock functions take `&mut self` and return the appropriate lock and a
//! *forward locktree*. Both are tied through their lifetimes to the
//! `MainLockTree` instance, and thus it is impossible (in safe Rust) to use
//! that instance to lock anything else. Further locking can only happen
//! through the forward locktree. In this case, the `MainLockTreeMain` is
//! completely empty and thus no further locks can be acquired:
//!
//! ```
//! # struct MainLockTreeMain<'a>(::std::marker::PhantomData<&'a ()>);
//! impl<'b> MainLockTreeMain<'b> {}
//! ```
//!
//! This happens because there are no further locks after `main`. If instead we
//! had two locks:
//!
//! ```
//! # use locktree::locktree;
//! locktree! {
//!   Main {
//!     first: StdMutex<String>,
//!     second: StdRwLock<Vec<usize>>,
//!   }
//! }
//! ```
//!
//! We would be able to lock any of those from the entry point:
//!
//! ```
//! struct MainLockTree {
//!   first: ::std::sync::Mutex<String>,
//!   second: ::std::sync::RwLock<Vec<usize>>,
//! }
//!
//! impl MainLockTree {
//!   fn lock_first<'a>(
//!     &'a mut self
//!   ) -> (::std::sync::MutexGuard<'a, String>, MainLockTreeFirst<'a>) {
//!     // ...
//!     # unimplemented!()
//!   }
//!
//!   fn read_second<'a>(
//!     &'a mut self
//!   ) -> (::std::sync::RwLockReadGuard<'a, Vec<usize>>, MainLockTreeSecond<'a>) {
//!     // ...
//!     # unimplemented!()
//!   }
//!
//!   fn write_second<'a>(
//!     &'a mut self
//!   ) -> (::std::sync::RwLockWriteGuard<'a, Vec<usize>>, MainLockTreeSecond<'a>) {
//!     // ...
//!     # unimplemented!()
//!   }
//! }
//! ```
//!
//! `MainLockTreeSecond` is again empty since it is the last in the sequence.
//! However, `MainLockTreeFirst` allows `second` (but not `fist`) to be locked
//! in sequence:
//!
//! ```
//! struct MainLockTreeSecond<'a>(&'a MainLockTree);
//!
//! impl MainLockTree {
//!   fn read_second<'a>(
//!     &'a mut self
//!   ) -> (::std::sync::RwLockReadGuard<'a, Vec<usize>>, MainLockTreeSecond<'a>) {
//!     // ...
//!     # unimplemented!()
//!   }
//!
//!   fn write_second<'a>(
//!     &'a mut self
//!   ) -> (::std::sync::RwLockWriteGuard<'a, Vec<usize>>, MainLockTreeSecond<'a>) {
//!     // ...
//!     # unimplemented!()
//!   }
//! }
//! ```
//!
//! And thus a proper locking sequence is enforced. Note that you can choose
//! not to lock anything between the current state and a target lock, but that
//! will have to be dropped and reacquired if your code needs to lock anything
//! that was skipped.
//!
//! # Composing
//!
//! TODO

/// `locktree!` macro. See the module-level documentation for details.
pub use locktree_derive::locktree;

mod plug;

use plug::*;

pub trait Mutex {
    type Guard: for<'a> PlugLifetime<'a>;

    fn lock(&self) -> <Self::Guard as PlugLifetime>::Type;
}

impl<T> Mutex for std::sync::Mutex<T>
where
    T: 'static,
{
    type Guard = H1MutexLockGuard<T>;

    fn lock(&self) -> <Self::Guard as PlugLifetime>::Type {
        (self as &std::sync::Mutex<T>).lock().unwrap()
    }
}

pub trait RwLock {
    type ReadGuard: for<'a> PlugLifetime<'a>;
    type WriteGuard: for<'a> PlugLifetime<'a>;

    fn read(&self) -> <Self::ReadGuard as PlugLifetime>::Type;
    fn write(&self) -> <Self::WriteGuard as PlugLifetime>::Type;
}

impl<T> RwLock for std::sync::RwLock<T>
where
    T: 'static,
{
    type ReadGuard = H1RwLockReadGuard<T>;
    type WriteGuard = H1RwLockWriteGuard<T>;

    fn read(&self) -> <Self::ReadGuard as PlugLifetime>::Type {
        (self as &std::sync::RwLock<T>).read().unwrap()
    }

    fn write(&self) -> <Self::WriteGuard as PlugLifetime>::Type {
        (self as &std::sync::RwLock<T>).write().unwrap()
    }
}

impl<T> RwLock for T
where
    T: Mutex,
{
    type ReadGuard = T::Guard;
    type WriteGuard = T::Guard;

    fn read(&self) -> <Self::ReadGuard as PlugLifetime>::Type {
        self.lock()
    }

    fn write(&self) -> <Self::WriteGuard as PlugLifetime>::Type {
        self.lock()
    }
}
