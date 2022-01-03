use once_cell::sync::OnceCell;
use std::{fmt, marker::PhantomData, sync::Mutex};

/// A cell to store **immutable** reference.
///
/// # Safety
/// Because the implementation internally converts raw pointers to usize and shares them between threads,
/// fetching references is essentially unsafe. Please verify its safety before using it.
pub struct GlobalRef<T> {
    inner: OnceCell<Mutex<Option<usize>>>,
    _marker: PhantomData<T>,
}

impl<T: fmt::Debug> fmt::Debug for GlobalRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GlobalRef").finish()
    }
}

impl<T> Default for GlobalRef<T> {
    fn default() -> Self {
        GlobalRef::new()
    }
}

impl<T> GlobalRef<T> {
    /// Create a new instance.
    pub const fn new() -> Self {
        GlobalRef {
            inner: OnceCell::new(),
            _marker: PhantomData,
        }
    }

    /// Set a reference so that other functions can obtain it through GlobalRef.
    /// It is recommended to use `with()` instead.
    /// **Be sure to call `clear()` after it is used.**
    pub unsafe fn set(&self, item: &T) {
        let mutex = self.inner.get_or_init(|| None.into());
        mutex.lock().unwrap().replace(item as *const T as usize);
    }

    /// Clear the registered reference.
    pub fn clear(&self) {
        let mutex = self.inner.get_or_init(|| None.into());
        *mutex.lock().unwrap() = None;
    }

    /// Set a reference and clear the reference after calling the given closure.
    pub fn with<F, R>(&self, item: &T, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        unsafe {
            self.set(item);
        }
        let res = f();
        self.clear();
        res
    }

    /// Get a immutable reference. Panics if `set()` or `with()` has not been called before.
    pub fn get(&self) -> &T {
        self.try_get().expect("Call set() before calling get()!")
    }

    /// Get a immutable reference. Returns None if `set()` or `with()` has not been called before.
    pub fn try_get(&self) -> Option<&T> {
        let inner = self.inner.get()?.lock().unwrap();
        unsafe { inner.and_then(|p| (p as *const T).as_ref()) }
    }
}

/// A cell to store **mutable** reference.
///
/// # Safety
/// Because the implementation internally converts raw pointers to usize and shares them between threads,
/// fetching references is essentially unsafe. Please verify its safety before using it.
pub struct GlobalMut<T> {
    inner: OnceCell<Mutex<Option<usize>>>,
    _marker: PhantomData<T>,
}

impl<T: fmt::Debug> fmt::Debug for GlobalMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GlobalMut").finish()
    }
}

impl<T> Default for GlobalMut<T> {
    fn default() -> Self {
        GlobalMut::new()
    }
}

impl<T> GlobalMut<T> {
    /// Create a new instance.
    pub const fn new() -> Self {
        GlobalMut {
            inner: OnceCell::new(),
            _marker: PhantomData,
        }
    }

    /// Set a reference so that other functions can obtain it through GlobalMut.
    /// It is recommended to use `with()` instead.
    /// **Be sure to call `clear()` after it is used.**
    pub unsafe fn set(&self, item: &mut T) {
        let mutex = self.inner.get_or_init(|| None.into());
        mutex.lock().unwrap().replace(item as *mut T as usize);
    }

    /// Clear the registered reference.
    pub fn clear(&self) {
        let mutex = self.inner.get_or_init(|| None.into());
        *mutex.lock().unwrap() = None;
    }

    /// Set a reference and clear the reference after calling the given closure.
    pub fn with<F, R>(&self, item: &mut T, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        unsafe {
            self.set(item);
        }
        let res = f();
        self.clear();
        res
    }

    /// Get a immutable reference. Panics if `set()` or `with()` has not been called before.
    pub fn get(&self) -> &T {
        self.try_get().expect("Call set() before calling get()!")
    }

    /// Get a immutable reference. Returns None if `set()` or `with()` has not been called before.
    pub fn try_get(&self) -> Option<&T> {
        let inner = self.inner.get()?.lock().unwrap();
        unsafe { inner.and_then(|p| (p as *mut T).as_ref()) }
    }

    /// Get a mutable reference. Panics if `set()` or `with()` has not been called before.
    pub fn get_mut(&self) -> &mut T {
        self.try_get_mut()
            .expect("Call set() before calling get_mut()!")
    }

    /// Get a mutable reference. Returns None if `set()` or `with()` has not been called before.
    pub fn try_get_mut(&self) -> Option<&mut T> {
        let inner = self.inner.get()?.lock().unwrap();
        unsafe { inner.and_then(|p| (p as *mut T).as_mut()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn global_ref() {
        static GLOBAL: GlobalRef<i32> = GlobalRef::new();

        let content = -1;
        unsafe {
            GLOBAL.set(&content);
        }
        assert_eq!(GLOBAL.get().abs(), 1);
        GLOBAL.clear();
        assert!(GLOBAL.try_get().is_none());
    }

    #[test]
    fn global_mut() {
        static GLOBAL: GlobalMut<i32> = GlobalMut::new();

        let mut content = 0;
        unsafe {
            GLOBAL.set(&mut content);
        }
        *GLOBAL.get_mut() += 1;
        assert_eq!(*GLOBAL.get(), 1);
        GLOBAL.clear();
        assert!(GLOBAL.try_get().is_none());
    }

    #[test]
    fn multi_thread() {
        static GLOBAL: GlobalMut<i32> = GlobalMut::new();

        let mut content = 0;

        GLOBAL.with(&mut content, || {
            fn add_one() {
                *GLOBAL.get_mut() += 1;
            }

            let handle = thread::spawn(add_one);
            handle.join().unwrap();
            assert_eq!(*GLOBAL.get(), 1);
        });

        assert!(GLOBAL.try_get().is_none());
    }
}
