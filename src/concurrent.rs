///
///
/// Some concurrent structures...
///
///

use std::ops::Deref;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// An AtomicUsize encapsulated in Arc<>
pub struct ConcurrentAtomicUsize {
    c: Arc<AtomicUsize>
}

impl ConcurrentAtomicUsize {
    pub fn new(i: usize) -> Self {
        ConcurrentAtomicUsize {
            c: Arc::new(AtomicUsize::new(i))
        }
    }

    pub fn new2(i: usize) -> (Self, Self) {
        let r = Self::new(i);
        (r.clone(), r)
    }

    pub fn clone2(&self) -> (Self, Self) {
        (self.clone(), self.clone())
    }
}

impl Clone for ConcurrentAtomicUsize {
    fn clone(&self) -> Self {
        ConcurrentAtomicUsize { c: self.c.clone() }
    }
}

impl Deref for ConcurrentAtomicUsize {
    type Target = AtomicUsize;
    fn deref(&self) -> &AtomicUsize {
        &self.c
    }
}
