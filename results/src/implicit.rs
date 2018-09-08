#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeInBody {
    pub fn_name: String,
    pub has_unsafe: bool,
}

impl UnsafeInBody {
    pub fn new(fn_name: String) -> Self {
        UnsafeInBody {
            fn_name: fn_name,
            has_unsafe: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeTraitSafeMethodInBody {
    pub fn_name: String,
    pub has_unsafe: bool,
}

impl UnsafeTraitSafeMethodInBody {
    fn new(fn_name: String) -> Self {
        UnsafeTraitSafeMethodInBody {
            fn_name,
            has_unsafe: false,
        }
    }
}
