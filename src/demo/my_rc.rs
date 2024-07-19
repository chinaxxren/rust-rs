use std::ops::Deref;
use std::ptr::NonNull;

// 使用 NonNull 来表示非空指针。
// 实现 Deref trait 以便 Rc 可以像普通引用一样被解引用。比如 *rc
// Box::into_raw 将一个 Box<T> 转换成一个裸指针。原来的 Box<T>
// 实例不再负责管理那块内存。稍后使用 Box::from_raw 重新获取所有权，
// 从而离开作用域才能释放。
// clone 时增加计数，drop 时减少计数。
pub(crate) struct MyRc<T> {
    ptr: NonNull<Inner<T>>,
}

struct Inner<T> {
    value: T,
    ref_count: usize,
}

impl<T> MyRc<T> {
    pub(crate) fn new(value: T) -> Self {
        let inner = Box::new(Inner {
            value,
            ref_count: 1,
        });
        MyRc {
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
        }
    }

    pub fn clone(&self) -> Self {
        unsafe {
            (*self.ptr.as_ptr()).ref_count += 1;
        }
        MyRc { ptr: self.ptr }
    }
}

impl<T> Deref for MyRc<T> {
    type Target = T;

   fn deref(&self) -> &Self::Target {
        unsafe { &(*self.ptr.as_ptr()).value }
    }
}

impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).ref_count -= 1;
            if (*inner).ref_count == 0 {
                let _ = Box::from_raw(inner); // 自动调用 drop 来释放内存
            }
        }
    }
}
