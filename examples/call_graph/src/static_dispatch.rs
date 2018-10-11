trait Trait {
    fn m1(self:&Self) {
        println!("Trait::m1");
        self.m2();
    }

    fn m2(self:&Self);
}

trait Trait2 <T1:Trait, T2:Trait> {
    fn m3(self:Self, t:&T1, x:&T2);
}

#[derive(Clone,Copy)]
struct Impl{}

impl Trait for Impl {
    fn m2(self:&Self) {
        println!("Impl as Trait::m2");
    }
}

impl <T1:Trait,T2:Trait> Trait2<T1,T2> for Impl {
    fn m3(self:Self, t:&T1, x:&T2) {
        println!("Impl as Trait2::m3");
        t.m1();
        x.m2();
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

trait TraitGeneric  {
    fn m4<T:Trait>(&self, t:T);
}

impl TraitGeneric for Impl {
    fn m4<T:Trait>(&self, t:T) {
        t.m1();
    }
}

//trait Add<T> {
//    fn add(x:&mut T,y:& T);
//}
//struct Point<X,Y> {
//    x: Add<X>,
//    y: Add<Y>,
//}
//
//fn add<X:Add,Y:Add>(p1:&mut Point<X,Y>,p2:&Point<X,Y>) {
//    Add::add(p1.x,p2.x);
//    Add::add(p1.y,p2.y);
//}

pub fn m() {
    let o = Impl{};
    let a = A{};
    o.m1();
    println!("---------------------");
    o.m3(&a, &o);
    o.m4(a);
}

