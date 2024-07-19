use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

// Arc 的主要区别在于线程安全的，因此需要使用 AtomicUsize 而不是 Cell<usize> 来管理引用计数。
struct Arc<T> {
    ptr: NonNull<Inner<T>>,
}

struct Weak<T> {
    ptr: NonNull<Inner<T>>,
}

struct Inner<T> {
    value: T,
    strong_count: AtomicUsize,
    weak_count: AtomicUsize,
}

impl<T> Arc<T> {
    fn new(value: T) -> Self {
        let inner = Box::new(Inner {
            value,
            strong_count: AtomicUsize::new(1),
            weak_count: AtomicUsize::new(0),
        });
        Arc {
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
        }
    }

    fn downgrade(&self) -> Weak<T> {
        self.inner().weak_count.fetch_add(1, Ordering::Relaxed);
        Weak { ptr: self.ptr }
    }

    fn strong_count(&self) -> usize {
        self.inner().strong_count.load(Ordering::Relaxed)
    }

    fn weak_count(&self) -> usize {
        self.inner().weak_count.load(Ordering::Relaxed)
    }

    fn inner(&self) -> &Inner<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        self.inner().strong_count.fetch_add(1, Ordering::Relaxed);
        Arc { ptr: self.ptr }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner().value
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.inner().strong_count.fetch_sub(1, Ordering::Release) == 1 {
            std::sync::atomic::fence(Ordering::Acquire);
            if self.weak_count() == 0 {
                unsafe {
                    Box::from_raw(self.ptr.as_ptr());
                } // 释放 Inner
            } else {
                self.inner().strong_count.store(0, Ordering::Relaxed);
            }
        }
    }
}

impl<T> Weak<T> {
    fn upgrade(&self) -> Option<Arc<T>> {
        let strong_count = self.strong_count();
        if strong_count == 0 {
            None
        } else {
            self.inner().strong_count.fetch_add(1, Ordering::Relaxed);
            Some(Arc { ptr: self.ptr })
        }
    }

    fn strong_count(&self) -> usize {
        self.inner().strong_count.load(Ordering::Relaxed)
    }

    fn weak_count(&self) -> usize {
        self.inner().weak_count.load(Ordering::Relaxed)
    }

    fn inner(&self) -> &Inner<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        self.inner().weak_count.fetch_add(1, Ordering::Relaxed);
        Weak { ptr: self.ptr }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.inner().weak_count.fetch_sub(1, Ordering::Release) == 1 {
            std::sync::atomic::fence(Ordering::Acquire);
            if self.strong_count() == 0 {
                unsafe {
                    Box::from_raw(self.ptr.as_ptr());
                } // 释放 Inner
            }
        }
    }
}

fn main() {
    let arc1 = Arc::new(5);
    let weak1 = arc1.downgrade();
    let arc2 = arc1.clone();

    println!("Strong count: {}", arc1.strong_count());
    println!("Weak count: {}", arc1.weak_count());

    if let Some(arc3) = weak1.upgrade() {
        println!("Upgraded value: {}", *arc3);
    } else {
        println!("Upgrade failed");
    }

    drop(arc1);
    drop(arc2);

    if let Some(arc3) = weak1.upgrade() {
        println!("Upgraded value after drop: {}", *arc3);
    } else {
        println!("Upgrade failed after drop");
    }
}
