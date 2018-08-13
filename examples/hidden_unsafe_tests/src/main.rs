#![feature(plugin)]
#![feature(asm)]
#![plugin(hidden_unsafe)]

pub unsafe fn with_asm() {
    asm!("nop");
}

pub fn with_asm_unsafe_block() {
    unsafe {
        asm!("nop");
    }
}

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
        let s1 = UnsafeImpl {};
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
    let assign_two = |mut x: i32| {
        let ptr: *mut i32 = &mut x as *mut i32;
        unsafe {
            *ptr = 2;
        }
    };
    let x = 5;
    assign_two(x);
}

fn call_unsafe(s1: &UnsafeImpl) {
    unsafe {
        s1.unsafe_method_unsafe_trait();
    }
    unsafe {
        s1.unsafe_method_safe_trait();
    }
    with_asm_unsafe_block();
    let mut v = Vec::new();
    v.push(1);
}

fn call_unsafe_and_safe(s1: &UnsafeImpl) {
    s1.safe_method_unsafe_trait();
    unsafe {
        s1.unsafe_method_unsafe_trait();
    }
    unsafe {
        s1.unsafe_method_safe_trait();
    }
    s1.safe_method_safe_trait();
}

fn two_calls(s1: &UnsafeImpl) {
    call_unsafe(s1);
}

extern "C" {
    // Our C function definitions!
    pub fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8;
    pub fn puts(s: *const u8) -> i32;
}

fn call_C() {
    let x = b"Hello, world!\0"; // our string to copy
    let mut y = [0u8; 32]; // declare some space on the stack to copy the string into
    unsafe {
        // calling C code is definitely unsafe. it could be doing ANYTHING
        strcpy(y.as_mut_ptr(), x.as_ptr()); // we need to call .as_ptr() to get a pointer for C to use
        puts(y.as_ptr());
    }
}

unsafe fn call_unsafe_1() {
    unsafe_fn();
}

unsafe fn call_unsafe_2() {
    let s1 = UnsafeImpl {};
    s1.unsafe_method_unsafe_trait();
}

fn main() {
    println!("Hello, world!");
    safe_fn();
    unsafe {
        unsafe_fn();
    }
    let s1 = UnsafeImpl {};
    s1.safe_method_unsafe_trait();
    unsafe {
        s1.unsafe_method_unsafe_trait();
    }
    unsafe {
        s1.unsafe_method_safe_trait();
    }
    s1.safe_method_safe_trait();
    let s2 = SafeImpl {};
    s2.safe_method_safe_trait();
    unsafe {
        s2.unsafe_method_safe_trait();
    }
    s2.safe_method_unsafe_trait();
    unsafe {
        s2.unsafe_method_unsafe_trait();
    }
    unsafe_in_closure();
    call_unsafe(&s1);
    call_unsafe_and_safe(&s1);
    two_calls(&s1);
    UnsafeImpl::no_self_safe();
    unsafe {
        with_asm();
    }
    call_C();
    unsafe {
        call_unsafe_1();
        call_unsafe_2();
    }
    //SafeImpl::nested_methods();
}
