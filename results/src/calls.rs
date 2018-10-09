use Abi;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalCall {
    pub abi: Abi,
    pub def_path: String,
    pub name: String,
}