#![feature(plugin)]

#![plugin(hidden_unsafe)]

fn safe_fn() -> () {
    println!("safe_fn");
}

unsafe fn unsafe_fn() -> () {
    println!("unsafe_fn");
}

unsafe trait UnsafeTrait {
    fn safe_method_unsafe_trait(&self) -> ();
    unsafe fn unsafe_method_unsafe_trait(&self) -> ();
}

trait Trait {
    unsafe fn unsafe_method_safe_trait(&self) -> ();
    fn safe_method_safe_trait(&self) -> ();
}

struct SafeImpl {}

// impl SafeImpl {
//     fn nested_methods() {
//         fn inner() {
//             let s1 = UnsafeImpl{};
//             unsafe{s1.unsafe_method_unsafe_trait();}    
//         }
//         inner();
//     }
// }

//the impl must be unsafe if the trait is
unsafe impl UnsafeTrait for SafeImpl {
    fn safe_method_unsafe_trait(&self) -> () {
        println!("safe_method_unsafe_trait::safe impl");
    }
    
    unsafe fn unsafe_method_unsafe_trait(&self) -> () {
        println!("unsafe_method_unsafe_trait::safe impl");
    }
}

impl Trait for SafeImpl {
    unsafe fn unsafe_method_safe_trait(&self) -> () {
        println!("unsafe_method_safe_trait::safe impl");
    }
    
    fn safe_method_safe_trait(&self) -> () {
        println!("safe_method_safe_trait::safe impl");
    }
}

struct UnsafeImpl {}

impl UnsafeImpl {
    fn no_self_safe() {
        let s1 = UnsafeImpl{};
        two_calls(&s1);
    }
}

unsafe impl UnsafeTrait for UnsafeImpl {
    fn safe_method_unsafe_trait(&self) -> () {
        println!("safe_method_unsafe_trait::unsafe impl");
    }
    
    unsafe fn unsafe_method_unsafe_trait(&self) -> () {
        println!("unsafe_method_unsafe_trait::unsafe impl");
    }
}

impl Trait for UnsafeImpl {
    unsafe fn unsafe_method_safe_trait(&self) -> () {
        println!("unsafe_method_safe_trait::unsafe impl");
    }
    
    fn safe_method_safe_trait(&self) -> () {
        println!("safe_method_safe_trait::unsafe impl");
    }
}

// stil requires unsafe impl
// unsafe trait UnsafeTraitSafeMethodsOnly {
//     fn only_safe_method_unsafe_trait(&self) -> ();
// }

// impl UnsafeTraitSafeMethodsOnly for SafeImpl {
//     fn only_safe_method_unsafe_trait(&self) -> () {
//         println!("only_safe_method_unsafe_trait::safe impl");
//     }
// }

fn unsafe_in_closure() {
    let assign_two = |mut x:i32| {
        let ptr:*mut i32 = &mut x as *mut i32;
        unsafe{ *ptr = 2; }
    };
    let x = 5;
    assign_two(x);
}

fn call_unsafe(s1: &UnsafeImpl) {
    unsafe{s1.unsafe_method_unsafe_trait();}
    unsafe{s1.unsafe_method_safe_trait();}
    let mut v = Vec::new();
    v.push(1);
}

fn call_unsafe_and_safe(s1: &UnsafeImpl) {
    s1.safe_method_unsafe_trait();
    unsafe{s1.unsafe_method_unsafe_trait();}
    unsafe{s1.unsafe_method_safe_trait();}
    s1.safe_method_safe_trait();
}

fn two_calls(s1: &UnsafeImpl) {
    call_unsafe(s1);
}


fn main() {
    println!("Hello, world!");
    safe_fn();
    unsafe{unsafe_fn();}
    let s1 = UnsafeImpl {};
    s1.safe_method_unsafe_trait();
    unsafe{s1.unsafe_method_unsafe_trait();}
    unsafe{s1.unsafe_method_safe_trait();}
    s1.safe_method_safe_trait();
    let s2 = SafeImpl {};
    s2.safe_method_safe_trait();
    unsafe{s2.unsafe_method_safe_trait();}
    s2.safe_method_unsafe_trait();
    unsafe{s2.unsafe_method_unsafe_trait();}
    unsafe_in_closure();
    call_unsafe(&s1);
    call_unsafe_and_safe(&s1);
    two_calls(&s1);
    UnsafeImpl::no_self_safe();
    //SafeImpl::nested_methods();
}
