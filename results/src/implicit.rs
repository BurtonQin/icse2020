use util;

static IMPLICIT_FILENAME: &'static str = "10_unsafe_in_call_tree";
static IMPLICIT_TRAIT_FILENAME: &'static str = "11_unsafe_trait_safe_method_in_call_tree";

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeInBody  {
    pub fn_name: String,
    pub has_unsafe: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeTraitSafeMethodInBody {
    pub fn_name: String,
    pub has_unsafe: bool,
}

pub fn get_implicit_unsafe_file(crate_name: String,
                                  crate_version: String) -> util::FileOps {
    util::FileOps::new( crate_name, crate_version, IMPLICIT_FILENAME)
}

pub fn get_implicit_trait_unsafe_file(crate_name: String,
                                crate_version: String) -> util::FileOps {
    util::FileOps::new( crate_name, crate_version, IMPLICIT_TRAIT_FILENAME)
}
