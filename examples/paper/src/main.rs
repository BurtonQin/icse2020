trait Animal {
    fn name(&self) -> &'static str;
    fn sound(&self) -> &'static str;
    fn says(&self) -> String {
        let mut buffer = String::new();
        write!(buffer, "The {:?} says {:?}", self.name(), self.sound());
        buffer
    }
}

struct Mouse{}
impl Animal for Mouse {
    fn name(&self) -> &'static str {
        "mouse"
    }
    fn sound(&self) -> &'static str {
        unsafe {"squeak"}
    }
}

struct Fox{}
impl Animal for Fox {
    fn name(&self) -> &'static str {
        "fox"
    }
    fn sound(&self) -> &'static str {
        ""
    }
    fn says(&self) -> String {
        "What does the fox say?".to_string()
    }
}

<<<<<<< HEAD
fn chases<A:Animal,B:Animal>(predator:&A, prey:&B) -> String {
    let mut buffer = String::new();
    let predator_name = predator.name();
    let prey_name = prey.name();
    write!(buffer, "{:?} chases {:?}",predator_name,prey_name);
=======
fn chases<A:Animal,B:Animal>(predator:&A, prey:&B) -> {
    let buffer = String::new();
    write!(buffer, "{:?} chases {:?}",predator,prey);
>>>>>>> 7d0aff54d755afc00056ebbe5b8d400582b18bd0
    buffer
}

fn mini_zoo() {
<<<<<<< HEAD
    let mouse = Mouse{};
    let fox = Fox{};
    let mouse_says = mouse.says();
    let fox_says = fox.says();
    let message = chases(&fox,&mouse);
    println!("{:?}",mouse_says);
    println!("{:?}",fox_says);
    println!("{:?}",message);
=======
    let dog = Mouse{};
    let fox = Fox{};
    println!("{:?}",dog.says());
    println!("{:?}",fox.says());
    println(chases(fox,mouse));
>>>>>>> 7d0aff54d755afc00056ebbe5b8d400582b18bd0
}

// trait Trait {
//     fn m1(self:&Self) {
//         self.m2();
//     }

//     fn m2(self:&Self);
// }
// trait Trait2 <T1:Trait, T2:Trait> {
//     fn m3(self:Self, t:&T1, x:&T2);
// }
// struct Impl{}
// impl Trait for Impl {
//     fn m2(self:&Self) {}
// }
// impl <T1:Trait,T2:Trait> Trait2<T1,T2> for Impl {
//     fn m3(self:Self, t:&T1, x:&T2) {
//         t.m1();
//         x.m2();
//     }
// }
// struct A{}
// impl Trait for A {
//     fn m1(self:&Self) {}

//     fn m2(self:&Self) {}
// }
// pub fn m() {
//     let o = Impl{};
//     let o1 = Impl{};
//     let a = A{};
//     o.m1();
//     o.m3(&a, &o1);
// }


// trait Foo {
//     fn method(&self) -> String;
// }
// impl Foo for u8 {
//     fn method(&self) -> String {
//         format!("u8: {}", *self)
//     }
// }
// impl Foo for String {
//     fn method(&self) -> String { format!("string: {}", *self) }
// }
// fn do_something(x: &Foo) {
//     x.method();
// }
// fn m1() {
//     let x = 5u8;
//     do_something(&x as &Foo);
// }


// fn m_with_ptr<F>( f: F ) -> i32
//     where F: Fn(i32) -> i32 {
//     f(0)
// }
// fn with_closure() {
//     m_with_ptr(|x| {
//         unsafe{ x }
//     });
// }
// fn id(x:i32) -> i32 {
//     unsafe {x}
// }
// fn with_fn() {
//     m_with_ptr(id);
// }

// fn main() {
//     println!("Hello, world!");
// }

// fn main() {
//     let mut hello = String::new();
//     hello.push_str("Hello");
//     let mut world = hello;
//     world.push_str(" world!");
//     println!("{:?}", hello);
//     println!("{:?}", world);
// }


// fn main() {
//     let mut hello = String::new();
//     hello.push_str("Hello");
//     let world = &mut hello;
//     world.push_str(" world!");
//     println!("{:?}", hello);
//     println!("{:?}", world);
// }

fn main() {
}
