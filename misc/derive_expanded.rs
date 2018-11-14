#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use ::std::prelude::v1::*;
#[macro_use]
extern crate std;

#[structural_match]
#[rustc_copy_clone_marker]
struct Point {
    x: u32,
    y: u32,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::marker::Copy for Point { }
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::clone::Clone for Point {
    #[inline]
    fn clone(&self) -> Point {
        {
            let _: ::std::clone::AssertParamIsClone<u32>;
            let _: ::std::clone::AssertParamIsClone<u32>;
            *self
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::Eq for Point {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<u32>;
            let _: ::std::cmp::AssertParamIsEq<u32>;
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::PartialEq for Point {
    #[inline]
    fn eq(&self, other: &Point) -> bool {
        match *other {
            Point { x: ref __self_1_0, y: ref __self_1_1 } =>
            match *self {
                Point { x: ref __self_0_0, y: ref __self_0_1 } =>
                (*__self_0_0) == (*__self_1_0) &&
                    (*__self_0_1) == (*__self_1_1),
            },
        }
    }
    #[inline]
    fn ne(&self, other: &Point) -> bool {
        match *other {
            Point { x: ref __self_1_0, y: ref __self_1_1 } =>
            match *self {
                Point { x: ref __self_0_0, y: ref __self_0_1 } =>
                (*__self_0_0) != (*__self_1_0) ||
                    (*__self_0_1) != (*__self_1_1),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for Point {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Point { x: ref __self_0_0, y: ref __self_0_1 } => {
                let mut debug_trait_builder = f.debug_struct("Point");
                let _ = debug_trait_builder.field("x", &&(*__self_0_0));
                let _ = debug_trait_builder.field("y", &&(*__self_0_1));
                debug_trait_builder.finish()
            }
        }
    }
}

#[structural_match]
#[rustc_copy_clone_marker]
enum Type { Safe, Unsafe, }
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::marker::Copy for Type { }
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::clone::Clone for Type {
    #[inline]
    fn clone(&self) -> Type { { *self } }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::Eq for Type {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () { { } }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::PartialEq for Type {
    #[inline]
    fn eq(&self, other: &Type) -> bool {
        {
            let __self_vi =
                unsafe { ::std::intrinsics::discriminant_value(&*self) } as
                    isize;
            let __arg_1_vi =
                unsafe { ::std::intrinsics::discriminant_value(&*other) } as
                    isize;
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) { _ => true, }
            } else { false }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for Type {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match (&*self,) {
            (&Type::Safe,) => {
                let mut debug_trait_builder = f.debug_tuple("Safe");
                debug_trait_builder.finish()
            }
            (&Type::Unsafe,) => {
                let mut debug_trait_builder = f.debug_tuple("Unsafe");
                debug_trait_builder.finish()
            }
        }
    }
}
