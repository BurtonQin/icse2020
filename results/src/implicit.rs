#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsafeInBody {
    pub fn_name: String,
    pub has_unsafe: bool,
}

impl UnsafeInBody {
    pub fn new(fn_name: String, has_unsafe:bool ) -> Self {
        UnsafeInBody {
            fn_name,
            has_unsafe,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeTraitSafeMethodInBody {
    pub fn_name: String,
    pub has_unsafe: bool,
}
//
//impl UnsafeTraitSafeMethodInBody {
//    fn new(fn_name: String) -> Self {
//        UnsafeTraitSafeMethodInBody {
//            fn_name,
//            has_unsafe: false,
//        }
//    }
//}
