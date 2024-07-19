use std::cell::Cell;
use std::ops::Deref;
use std::ptr::NonNull;

// 增加弱引用计数：用于跟踪有多少个弱引用指向相同的对象。
// Weak 结构体：用于表示弱引用。
// 管理弱引用的生命周期：在强引用计数和弱引用计数都为零时释放资源
struct Rc<T> {
    ptr: NonNull<Inner<T>>,
}

struct Weak<T> {
    ptr: NonNull<Inner<T>>,
}

struct Inner<T> {
    value: T,
    strong_count: Cell<usize>,
    weak_count: Cell<usize>,
}

impl<T> Rc<T> {
    fn new(value: T) -> Self {
        let inner = Box::new(Inner {
            value,
            strong_count: Cell::new(1),
            weak_count: Cell::new(0),
        });
        Rc {
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
        }
    }

    fn downgrade(&self) -> Weak<T> {
        self.inner().weak_count.set(self.weak_count() + 1);
        Weak { ptr: self.ptr }
    }

    fn strong_count(&self) -> usize {
        self.inner().strong_count.get()
    }

    fn weak_count(&self) -> usize {
        self.inner().weak_count.get()
    }

    fn inner(&self) -> &Inner<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        self.inner().strong_count.set(self.strong_count() + 1);
        Rc { ptr: self.ptr }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner().value
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let strong_count = self.strong_count();
        if strong_count > 1 {
            self.inner().strong_count.set(strong_count - 1);
        } else {
            let weak_count = self.weak_count();
            if weak_count == 0 {
                unsafe {
                    Box::from_raw(self.ptr.as_ptr());
                } // 释放 Inner
            } else {
                self.inner().strong_count.set(0);
            }
        }
    }
}

impl<T> Weak<T> {
    fn upgrade(&self) -> Option<Rc<T>> {
        let strong_count = self.strong_count();
        if strong_count == 0 {
            None
        } else {
            self.inner().strong_count.set(strong_count + 1);
            Some(Rc { ptr: self.ptr })
        }
    }

    fn strong_count(&self) -> usize {
        self.inner().strong_count.get()
    }

    fn weak_count(&self) -> usize {
        self.inner().weak_count.get()
    }

    fn inner(&self) -> &Inner<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        self.inner().weak_count.set(self.weak_count() + 1);
        Weak { ptr: self.ptr }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        let weak_count = self.weak_count();
        if weak_count > 1 {
            self.inner().weak_count.set(weak_count - 1);
        } else {
            let strong_count = self.strong_count();
            if strong_count == 0 {
                unsafe {
                    Box::from_raw(self.ptr.as_ptr());
                } // 释放 Inner
            } else {
                self.inner().weak_count.set(0);
            }
        }
    }
}

fn main() {
    let rc1 = Rc::new(5);
    let weak1 = rc1.downgrade();
    let rc2 = rc1.clone();

    println!("Strong count: {}", rc1.strong_count());
    println!("Weak count: {}", rc1.weak_count());

    if let Some(rc3) = weak1.upgrade() {
        println!("Upgraded value: {}", *rc3);
    } else {
        println!("Upgrade failed");
    }

    drop(rc1);
    drop(rc2);

    if let Some(rc3) = weak1.upgrade() {
        println!("Upgraded value after drop: {}", *rc3);
    } else {
        println!("Upgrade failed after drop");
    }
}
