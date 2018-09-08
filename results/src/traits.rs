#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeTrait {
    name: String,
}

impl UnsafeTrait {
    pub fn new(name: String) -> Self {
        UnsafeTrait { name }
    }
}
