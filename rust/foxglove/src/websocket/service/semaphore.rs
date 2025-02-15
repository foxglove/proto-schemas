//! A sempahore for admission control

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// A non-blocking counting semaphore for concurrency control.
///
/// Decrements the inner counter when acquired.
#[derive(Debug, Clone)]
pub(crate) struct Semaphore(Arc<AtomicUsize>);

impl Semaphore {
    /// Constructs a new semaphore.
    pub fn new(count: usize) -> Self {
        Self(Arc::new(AtomicUsize::new(count)))
    }

    /// Attempts to acquire the semaphore.
    pub fn try_acquire(&self) -> Option<SemaphoreGuard> {
        loop {
            let current = self.0.load(Ordering::Acquire);
            if current == 0 {
                return None;
            }
            if self
                .0
                .compare_exchange(current, current - 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Some(SemaphoreGuard(self.0.clone()));
            }
        }
    }
}

/// A counting semaphore guard.
///
/// Increments the inner counter when dropped.
#[derive(Debug)]
pub(crate) struct SemaphoreGuard(Arc<AtomicUsize>);

impl Drop for SemaphoreGuard {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Release);
    }
}
