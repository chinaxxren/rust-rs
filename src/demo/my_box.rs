// Deref 可以说是 Rust 中最常见的隐式类型转换，而且它可以连续的实现如
// Box<String> -> String -> &str 的隐式转换，只要链条上的类型实现了 Deref 特征

pub struct MyBox<T> {
    v: T,
}

impl<T> MyBox<T> {
    pub fn new(x: T) -> MyBox<T> {
        MyBox { v: x }
    }
}

// 规则如下：
// 当 T: Deref<Target=U>，可以将 &T 转换成 &U
// 当 T: DerefMut<Target=U>，可以将 &mut T 转换成 &mut U
// 当 T: Deref<Target=U>，可以将 &mut T 转换成 &U

use std::ops::Deref;

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
}

use std::ops::DerefMut;

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.v
    }
}

pub fn display(s: &mut String) {
    s.push_str("world");
    println!("{}", s);
}
