//trait Trait {
//    fn m1(self:&Self) {
//        //println!("Trait::m1");
//        self.m2();
//    }
//
//    fn m2(self:&Self);
//}
//
//trait Trait2 <T1:Trait, T2:Trait> {
//    fn m3(self:Self, t:&T1, x:&T2);
//}
//
//#[derive(Clone,Copy)]
//struct Impl{}
//
//impl Trait for Impl {
//    fn m2(self:&Self) {
//        //println!("Impl as Trait::m2");
//    }
//}
//
//impl <T1:Trait,T2:Trait> Trait2<T1,T2> for Impl {
//    fn m3(self:Self, t:&T1, x:&T2) {
//        //println!("Impl as Trait2::m3");
//        t.m1();
//        x.m2();
//    }
//}
//
//struct A{}
//
//impl Trait for A {
//    fn m1(self:&Self) {
//        //println!("A as Trait::m1");
//    }
//
//    fn m2(self:&Self) {
//        //println!("A as Trait::m2");
//    }
//}
//
//trait TraitGeneric  {
//    fn m4<T:Trait>(&self, t:T);
//}
//
//impl TraitGeneric for Impl {
//    fn m4<T:Trait>(&self, t:T) {
//        t.m1();
//    }
//}
//
//pub fn m() {
//    let o = Impl{};
//    let a = A{};
//    o.m1();
//    //println!("---------------------");
//    o.m3(&a, &o);
//    o.m4(a);
//}

//pub fn rec() {
//    rec();
//}
//
//pub fn recu() {
//    unsafe {
//        recu();
//    }
//}

pub fn rec1() {
    rec2();
}

pub fn rec2() {
    rec1();
}

//pub fn rec3() {
//    rec4();
//}
//
//pub fn rec4() {
//    rec5();
//    m6();
//}
//
//pub fn rec5() {
//    rec3();
//}
//
//pub fn m6() {
//    unsafe {
//        m6();
//    }
//}
//
//pub fn rec7() {
//    rec8();
//}
//
//pub fn rec8() {
//    m6();
//    rec9();
//}
//
//pub fn rec9() {
//    rec7();
//}
