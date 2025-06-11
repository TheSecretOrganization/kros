use core::sync::atomic::{AtomicBool, Ordering};

/// A simple spinlock that uses atomic operations for synchronization.
///
/// This spinlock provides mutual exclusion by spinning in a loop until it can acquire a lock.
/// It is suitable for scenarios where contention is low or critical sections are very short.
///
/// # Safety
/// - This implementation is `Sync` but **not `Send`**, so it's safe to share between threads, but not to move the data across them unless `T: Send`.
/// - Internally uses `UnsafeCell` and `unsafe` code to permit mutable access.
pub struct Spinlock<T> {
    lock: AtomicBool,
    data: core::cell::UnsafeCell<T>,
}

// Safe because we ensure only one thread can access `data` at a time.
unsafe impl<T> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    /// Creates a new spinlock protecting the given data.
    ///
    /// # Example
    /// ```
    /// let lock = Spinlock::new(42);
    /// ```
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: core::cell::UnsafeCell::new(data),
        }
    }

    /// Acquires the spinlock, blocking the current thread until it is available.
    ///
    /// Returns a guard which releases the lock when dropped.
    ///
    /// # Blocking Behavior
    /// This call will **spin in a tight loop** until the lock becomes available.
    ///
    /// # Atomic Ordering
    /// - `compare_exchange` uses `Ordering::Acquire` on success: ensures that subsequent
    ///   operations are not reordered before the lock is acquired.
    /// - Uses `Ordering::Relaxed` on failure: no synchronization needed while spinning.
    ///
    /// # Example
    /// ```
    /// let lock = Spinlock::new(0);
    /// let mut guard = lock.lock();
    /// *guard += 1;
    /// ```
    pub fn lock<'a>(&'a self) -> SpinlockGuard<'a, T> {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        SpinlockGuard { lock: self }
    }
}

/// A RAII guard that releases the associated `Spinlock` when dropped.
///
/// Implements `Deref` and `DerefMut` to access the inner data.
pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
}

impl<'a, T> core::ops::Deref for SpinlockGuard<'a, T> {
    type Target = T;

    /// Provides shared access to the locked data.
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for SpinlockGuard<'a, T> {
    /// Provides mutable access to the locked data.
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
    /// Releases the spinlock when the guard is dropped.
    ///
    /// # Atomic Ordering
    /// - Uses `Ordering::Release` to ensure that all modifications to the data
    ///   are visible to the next thread that acquires the lock.
    fn drop(&mut self) {
        self.lock.lock.store(false, Ordering::Release);
    }
}
