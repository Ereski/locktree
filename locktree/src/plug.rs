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
