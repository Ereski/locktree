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

use crate::plug::*;
#[cfg(feature = "async")]
use std::future::Future;

/// `locktree!` macro. See the module-level documentation for details.
pub use locktree_derive::locktree;

mod plug;

pub type PluggedGuard<'a, T> = <T as PlugLifetime<'a>>::Type;

pub type PluggedMutexGuard<'a, T> = PluggedGuard<'a, <T as Mutex>::Guard>;

pub type PluggedRwLockReadGuard<'a, T> =
    PluggedGuard<'a, <T as RwLock>::ReadGuard>;

pub type PluggedRwLockWriteGuard<'a, T> =
    PluggedGuard<'a, <T as RwLock>::WriteGuard>;

#[cfg(feature = "async")]
pub type PluggedAsyncGuard<'a, T> =
    Box<dyn Future<Output = <T as PlugLifetime<'a>>::Type> + 'a>;

pub trait New<T> {
    fn new(value: T) -> Self;
}

impl<T> New<T> for ::std::sync::Mutex<T> {
    fn new(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> New<T> for ::std::sync::RwLock<T> {
    fn new(value: T) -> Self {
        Self::new(value)
    }
}

#[cfg(feature = "tokio")]
impl<T> New<T> for ::tokio::sync::Mutex<T> {
    fn new(value: T) -> Self {
        Self::new(value)
    }
}

#[cfg(feature = "tokio")]
impl<T> New<T> for ::tokio::sync::RwLock<T> {
    fn new(value: T) -> Self {
        Self::new(value)
    }
}

pub trait Mutex {
    type Guard: for<'a> PlugLifetime<'a>;

    fn lock(&self) -> PluggedGuard<Self::Guard>;
}

impl<T> Mutex for std::sync::Mutex<T>
where
    T: 'static,
{
    type Guard = H1MutexLockGuard<T>;

    fn lock(&self) -> PluggedGuard<Self::Guard> {
        std::sync::Mutex::<T>::lock(self).unwrap()
    }
}

#[cfg(feature = "async")]
pub trait AsyncMutex {
    type Guard: for<'a> PlugLifetime<'a>;

    fn lock(&self) -> PluggedAsyncGuard<Self::Guard>;
}

#[cfg(feature = "tokio")]
impl<T> AsyncMutex for tokio::sync::Mutex<T>
where
    T: 'static,
{
    type Guard = H1TokioMutexLockGuard<T>;

    fn lock(&self) -> PluggedAsyncGuard<Self::Guard> {
        Box::new(tokio::sync::Mutex::<T>::lock(self))
    }
}

pub trait RwLock {
    type ReadGuard: for<'a> PlugLifetime<'a>;
    type WriteGuard: for<'a> PlugLifetime<'a>;

    fn read(&self) -> PluggedGuard<Self::ReadGuard>;
    fn write(&self) -> PluggedGuard<Self::WriteGuard>;
}

impl<T> RwLock for std::sync::RwLock<T>
where
    T: 'static,
{
    type ReadGuard = H1RwLockReadGuard<T>;
    type WriteGuard = H1RwLockWriteGuard<T>;

    fn read(&self) -> PluggedGuard<Self::ReadGuard> {
        std::sync::RwLock::<T>::read(self).unwrap()
    }

    fn write(&self) -> PluggedGuard<Self::WriteGuard> {
        std::sync::RwLock::<T>::write(self).unwrap()
    }
}

impl<T> RwLock for T
where
    T: Mutex,
{
    type ReadGuard = T::Guard;
    type WriteGuard = T::Guard;

    fn read(&self) -> PluggedGuard<Self::ReadGuard> {
        self.lock()
    }

    fn write(&self) -> PluggedGuard<Self::WriteGuard> {
        self.lock()
    }
}

#[cfg(feature = "async")]
pub trait AsyncRwLock {
    type ReadGuard: for<'a> PlugLifetime<'a>;
    type WriteGuard: for<'a> PlugLifetime<'a>;

    fn read(&self) -> PluggedAsyncGuard<Self::ReadGuard>;
    fn write(&self) -> PluggedAsyncGuard<Self::WriteGuard>;
}

#[cfg(feature = "tokio")]
impl<T> AsyncRwLock for tokio::sync::RwLock<T>
where
    T: 'static,
{
    type ReadGuard = H1TokioRwLockReadGuard<T>;
    type WriteGuard = H1TokioRwLockWriteGuard<T>;

    fn read(&self) -> PluggedAsyncGuard<Self::ReadGuard> {
        Box::new(tokio::sync::RwLock::<T>::read(self))
    }

    fn write(&self) -> PluggedAsyncGuard<Self::WriteGuard> {
        Box::new(tokio::sync::RwLock::<T>::write(self))
    }
}

#[cfg(feature = "async")]
impl<T> AsyncRwLock for T
where
    T: AsyncMutex,
{
    type ReadGuard = T::Guard;
    type WriteGuard = T::Guard;

    fn read(&self) -> PluggedAsyncGuard<Self::ReadGuard> {
        self.lock()
    }

    fn write(&self) -> PluggedAsyncGuard<Self::WriteGuard> {
        self.lock()
    }
}
