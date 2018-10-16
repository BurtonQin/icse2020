#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsafeInBody {
    pub def_path: String,
    pub has_unsafe: bool,
    pub name: String,
}

impl UnsafeInBody {
    pub fn new(def_path: String, has_unsafe:bool, name: String ) -> Self {
        UnsafeInBody {
            def_path,
            has_unsafe,
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

#[derive(Serialize, Deserialize, Debug)]
pub enum CallTypes {
    // Identifier of function
    Resolved(String),
    // Identifier of trait method
    SelfCall(String),
    // formal argument
    FnPtr(String),
    //formal argument, trait, method
    TraitObject(String,String,String),
    // type variable, method
    ParametricCall(String,String),
    // crate, def path
    Unresolved(String,String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnresolvedFn {
    pub generics: Vec<TraitBound>,
    pub calls: Vec<CallTypes>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UnsafeResults {
    Resolved(String,bool),
    Unresolved(String, UnresolvedFn),
}