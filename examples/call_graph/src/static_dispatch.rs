trait Trait {
    fn m1(self:&Self) {
        println!("Trait::m1");
        self.m2();
    }

    fn m2(self:&Self);
}

trait Trait2 <T:Trait> {
    fn m3(self:Self, t:&T);
}

#[derive(Clone,Copy)]
struct Impl{}

impl Trait for Impl {
    fn m2(self:&Self) {
        println!("Impl as Trait::m2");
    }
}

impl <T:Trait> Trait2<T> for Impl {
    fn m3(self:Self, t:&T) {
        println!("Impl as Trait2::m3");
        t.m1();
    }
}

struct A{}

impl Trait for A {
    fn m1(self:&Self) {
        println!("A as Trait::m1");
    }

    fn m2(self:&Self) {
        println!("A as Trait::m2");
    }
}

pub fn m() {
    let o = Impl{};
    let a = A{};
    o.m1();
    println!("---------------------");
    o.m3(&a);
}

