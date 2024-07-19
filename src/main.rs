use demo::my_box;

mod demo;

fn main() {
    let mut s = my_box::MyBox::new(String::from("hello, "));
    my_box::display(&mut s);

    let rc1 = demo::my_rc::MyRc::new(5);
    let rc2 = rc1.clone();
    let rc3 = rc1.clone();

    println!("rc1: {}", *rc1);
    println!("rc2: {}", *rc2);
    println!("rc3: {}", *rc3);
}
