#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FnType {
    Safe,
    Unsafe,
    NormalNotSafe,
    Parametric,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsafeInBody {
    pub def_path: String,
    pub fn_type: FnType,
    pub name: String,
}

impl UnsafeInBody {
    pub fn new(def_path: String, fn_type: FnType, name: String ) -> Self {
        UnsafeInBody {
            def_path,
            fn_type,
            name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeTraitSafeMethodInBody {
    pub fn_name: String,
    pub has_unsafe: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TraitBound {
    pub trait_def_path: String,
    pub generic: String,
}

