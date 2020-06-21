use std::marker::PhantomData;

pub trait PlugLifetime<'a> {
    type Type;
}

pub struct H1MutexLockGuard<T>(PhantomData<T>);

impl<'a, T> PlugLifetime<'a> for H1MutexLockGuard<T>
where
    T: 'static,
{
    type Type = std::sync::MutexGuard<'a, T>;
}

#[cfg(feature = "tokio")]
pub struct H1TokioMutexLockGuard<T>(PhantomData<T>);

#[cfg(feature = "tokio")]
impl<'a, T> PlugLifetime<'a> for H1TokioMutexLockGuard<T>
where
    T: 'static,
{
    type Type = tokio::sync::MutexGuard<'a, T>;
}

pub struct H1RwLockReadGuard<T>(PhantomData<T>);

impl<'a, T> PlugLifetime<'a> for H1RwLockReadGuard<T>
where
    T: 'static,
{
    type Type = std::sync::RwLockReadGuard<'a, T>;
}

pub struct H1RwLockWriteGuard<T>(PhantomData<T>);

impl<'a, T> PlugLifetime<'a> for H1RwLockWriteGuard<T>
where
    T: 'static,
{
    type Type = std::sync::RwLockWriteGuard<'a, T>;
}

#[cfg(feature = "tokio")]
pub struct H1TokioRwLockReadGuard<T>(PhantomData<T>);

#[cfg(feature = "tokio")]
impl<'a, T> PlugLifetime<'a> for H1TokioRwLockReadGuard<T>
where
    T: 'static,
{
    type Type = tokio::sync::RwLockReadGuard<'a, T>;
}

#[cfg(feature = "tokio")]
pub struct H1TokioRwLockWriteGuard<T>(PhantomData<T>);

#[cfg(feature = "tokio")]
impl<'a, T> PlugLifetime<'a> for H1TokioRwLockWriteGuard<T>
where
    T: 'static,
{
    type Type = tokio::sync::RwLockWriteGuard<'a, T>;
}
