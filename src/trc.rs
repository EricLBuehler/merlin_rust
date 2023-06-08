#![allow(dead_code)]
#![allow(clippy::mut_from_ref)]

use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::RwLock,
};

pub struct AtomicThreadTrc<T> {
    atomicref: RwLock<usize>,
    pub data: T,
}

struct LocalThreadTrc<T> {
    atomicref: NonNull<AtomicThreadTrc<T>>,
    threadref: usize,
}

/// `Trc` is a smart pointer for sharing data across threads is a thread-safe manner without putting locks on the data.
/// `Trc` stands for: Thread Reference Counted
/// It impkements biased reference counting, which is based on the observation that most objects are only used by one thread.
/// This means that two refernce counts can be created: one for thread-local use, and one atomic one (with a lock) for sharing.
/// This implementation of biased reference counting sets the atomic reference count to the number of threads using the data.
/// When the thread drops the object, the atomic refernce count is decremented, and the data is freed.
///
/// For ease of developer use, `Trc` comes with `Deref` and `DerefMut` implemented to allow internal mutation.
///
/// Example in a single thread:
/// ```
/// let trc = Trc::new(100);
/// println!("{}", trc);
/// *trc = 200;
/// println!("{}", trc);
/// ```
///
/// Example with multiple threads:
/// ```
/// use std::thread;
///
/// let trc = Trc::new(100);
/// let mut trc2 = trc.clone_across_thread();
///
/// let handle = thread::spawn(move || {
///     println!("{}", *trc2);
///     *trc2 = 200;
/// });
///
/// handle.join().unwrap();
/// println!("{}", *trc);
/// assert_eq!(*trc, 200);
/// ```
///
#[derive(PartialEq, Eq)]
pub struct Trc<T> {
    data: NonNull<LocalThreadTrc<T>>,
}

impl<T> Trc<T> {
    /// Creates a new `Trc` from the provided data.
    /// ```
    /// let trc = Trc::new(100);
    /// ```
    #[inline]
    pub fn new(value: T) -> Self {
        let atomicthreadata = AtomicThreadTrc {
            atomicref: RwLock::from(1),
            data: value,
        };

        let abx = Box::new(atomicthreadata);

        let localthreadtrc = LocalThreadTrc {
            atomicref: NonNull::from(Box::leak(abx)),
            threadref: 1,
        };

        let tbx = Box::new(localthreadtrc);

        Trc {
            data: NonNull::from(Box::leak(tbx)),
        }
    }

    /// Return the local thread count of the object. This is how many `Trc`s are using the data referenced by this `Trc`.
    /// ```
    /// let trc = Trc::new(100);
    /// assert!(Trc::thread_count(trc) == 1)
    /// ```
    #[inline]
    pub fn thread_count(this: &Self) -> usize {
        return this.inner().threadref;
    }

    /// Return the atomic reference count of the object. This is how many threads are using the data referenced by this `Trc`./// ```
    /// use std::thread;
    ///
    /// let trc = Trc::new(100);
    /// let mut trc2 = trc.clone_across_thread();
    ///
    /// let handle = thread::spawn(move || {
    ///     println!("{}", *trc2);
    ///     *trc2 = 200;
    ///     assert_eq!(Trc::atomic_count(&trc), 2);
    /// });
    ///
    /// handle.join().unwrap();
    /// assert_eq!(Trc::atomic_count(&trc), 1);
    /// println!("{}", *trc);
    /// assert_eq!(*trc, 200);
    /// ```
    #[inline]
    pub fn atomic_count(this: &Self) -> usize {
        let mut readlock = this.inner_atomic().atomicref.try_write();

        while readlock.is_err() {
            readlock = this.inner_atomic().atomicref.try_write();
        }
        *readlock.unwrap()
    }

    #[inline]
    fn inner(&self) -> &LocalThreadTrc<T> {
        return unsafe { self.data.as_ref() };
    }

    #[inline]
    fn inner_atomic(&self) -> &AtomicThreadTrc<T> {
        return unsafe { self.data.as_ref().atomicref.as_ref() };
    }

    #[inline]
    fn inner_mut(&self) -> &mut LocalThreadTrc<T> {
        unsafe { &mut *self.data.as_ptr() }
    }

    #[inline]
    fn inner_atomic_mut(&self) -> &mut AtomicThreadTrc<T> {
        unsafe { &mut *(*self.data.as_ptr()).atomicref.as_ptr() }
    }

    /// Clone a `Trc` across threads. This is necessary because otherwise the atomic reference count will not be incremented.use std::thread;
    /// ```
    /// let trc = Trc::new(100);
    /// let trc2 = trc.clone_across_thread();
    /// ```
    #[inline]
    pub fn clone_across_thread(&self) -> Self {
        let mut writelock = self.inner_atomic().atomicref.try_write();

        while writelock.is_err() {
            writelock = self.inner_atomic().atomicref.try_write();
        }
        let mut writedata = writelock.unwrap();

        *writedata += 1;

        let localthreadtrc = LocalThreadTrc {
            atomicref: self.inner().atomicref,
            threadref: 1,
        };

        let tbx = Box::new(localthreadtrc);

        return Trc {
            data: NonNull::from(Box::leak(tbx)),
        };
    }

    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        return this.inner().atomicref.as_ptr() == other.inner().atomicref.as_ptr();
    }

    #[inline]
    pub fn as_ptr(this: &Self) -> *mut AtomicThreadTrc<T> {
        return this.inner().atomicref.as_ptr();
    }
}

impl<T> Deref for Trc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner_atomic().borrow().data
    }
}

impl<T> DerefMut for Trc<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner_atomic_mut().borrow_mut().data
    }
}

impl<T> Drop for Trc<T> {
    #[inline]
    fn drop(&mut self) {
        self.inner_mut().threadref -= 1;
        if self.inner().threadref == 0 {
            let mut writelock = self.inner_atomic().atomicref.try_write();

            while writelock.is_err() {
                writelock = self.inner_atomic().atomicref.try_write();
            }
            let mut writedata = writelock.unwrap();

            *writedata -= 1;

            if *writedata == 0 {
                std::mem::drop(writedata);

                unsafe { Box::from_raw(self.inner().atomicref.as_ptr()) };
                unsafe { Box::from_raw(self.data.as_ptr()) };
            }
        }
    }
}

impl<T> Clone for Trc<T> {
    ///Be sure to call `clone_across_thread` for threads
    #[inline]
    fn clone(&self) -> Self {
        self.inner_mut().threadref += 1;

        Trc { data: self.data }
    }
}

unsafe impl<T> Send for Trc<T> {}
unsafe impl<T> Sync for Trc<T> {}