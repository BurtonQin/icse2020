#![feature(asm)]
#![feature(unboxed_closures)]

extern crate byteorder;

mod external_calls;

pub unsafe fn with_asm() {
    asm!("nop");
}

pub fn with_asm_unsafe_block(b:bool) {
    unsafe {
        asm!("nop");
    }
    unsafe {
        if b {
            asm!("nop");
        } else {
            asm!("nop");
        }
    }
}

fn safe_fn() -> () {
    let mut i = 1;
    i += 1;
}

unsafe fn unsafe_fn() -> () {
    let mut i = 1;
    i += 1;
}

unsafe trait UnsafeTrait {
    fn safe_method_unsafe_trait(&self) -> ();
    unsafe fn unsafe_method_unsafe_trait(&self) -> ();
    fn m1() -> () {
        let mut i = 1;
        i += 1;
    }
    unsafe fn m2() -> () {
        let mut i = 1;
        i += 1;
    }
}

trait Trait {
    unsafe fn unsafe_method_safe_trait(&self) -> ();
    fn safe_method_safe_trait(&self) -> ();
}

struct SafeImpl {}

fn nested_methods(test: bool) {
    let s1 = UnsafeImpl {};
    unsafe {
        if test {
            s1.unsafe_method_unsafe_trait();
        } else {
            UnsafeImpl::m2();
        }
    }

    fn inner() {
        let s1 = UnsafeImpl {};
        unsafe {
            s1.unsafe_method_unsafe_trait();
        }
    }
    inner();
}

//the impl must be unsafe if the trait is
unsafe impl UnsafeTrait for SafeImpl {
    fn safe_method_unsafe_trait(&self) -> () {
        let mut i = 1;
        i += 1;
    }

    unsafe fn unsafe_method_unsafe_trait(&self) -> () {
        let mut i = 1;
        i += 1;
    }
}

impl Trait for SafeImpl {
    unsafe fn unsafe_method_safe_trait(&self) -> () {
        let mut i = 1;
        i += 1;
    }

    fn safe_method_safe_trait(&self) -> () {
        let mut i = 1;
        i += 1;
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
        let mut i = 1;
        i += 1;
    }

    unsafe fn unsafe_method_unsafe_trait(&self) -> () {
        let mut i = 1;
        i += 1;
    }
}

impl Trait for UnsafeImpl {
    unsafe fn unsafe_method_safe_trait(&self) -> () {
        let mut i = 1;
        i += 1;
    }

    fn safe_method_safe_trait(&self) -> () {
        let mut i = 1;
        i += 1;
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
    with_asm_unsafe_block(true);
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
    nested_methods(true);

    UnsafeImpl::m1();
    unsafe {
        UnsafeImpl::m2();
    }

    external_calls::use_trait();
}
