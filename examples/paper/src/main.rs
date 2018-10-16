trait Trait {
    fn m1(self:&Self) {
        self.m2();
    }

    fn m2(self:&Self);
}
trait Trait2 <T1:Trait, T2:Trait> {
    fn m3(self:Self, t:&T1, x:&T2);
}
struct Impl{}
impl Trait for Impl {
    fn m2(self:&Self) {}
}
impl <T1:Trait,T2:Trait> Trait2<T1,T2> for Impl {
    fn m3(self:Self, t:&T1, x:&T2) {
        t.m1();
        x.m2();
    }
}
struct A{}
impl Trait for A {
    fn m1(self:&Self) {}

    fn m2(self:&Self) {}
}
pub fn m() {
    let o = Impl{};
    let o1 = Impl{};
    let a = A{};
    o.m1();
    o.m3(&a, &o1);
}


trait Foo {
    fn method(&self) -> String;
}
impl Foo for u8 {
    fn method(&self) -> String {
        format!("u8: {}", *self)
    }
}
impl Foo for String {
    fn method(&self) -> String { format!("string: {}", *self) }
}
fn do_something(x: &Foo) {
    x.method();
}
fn m1() {
    let x = 5u8;
    do_something(&x as &Foo);
}


fn m_with_ptr<F>( f: F ) -> i32
    where F: Fn(i32) -> i32 {
    f(0)
}
fn with_closure() {
    m_with_ptr(|x| {
        unsafe{ x }
    });
}
fn id(x:i32) -> i32 {
    unsafe {x}
}
fn with_fn() {
    m_with_ptr(id);
}

fn main() {
    println!("Hello, world!");
}
