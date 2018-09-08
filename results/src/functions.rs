use unsafety_sources::Source;

#[derive(Serialize, Deserialize, Debug)]
pub struct LongFnInfo {
    name: String,
    node_id: String,
    location: String,
    // pairs (name,node_id)
    local_calls: Vec<(String,String)>,
    external_calls: Vec<(String,Vec<String>)>,
}

impl LongFnInfo {
    pub fn new(name: String,
               node_id: String,
               location: String,
               // pairs (name,node_id)
               local_calls: Vec<(String,String)>,
               external_calls: Vec<(String,Vec<String>)> ) -> Self {
        LongFnInfo{
            name, node_id, location, local_calls, external_calls
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShortFnInfo {
    name: String,
    node_id: String,
    location: String,
}

impl ShortFnInfo {
    pub fn new(name: String,
               node_id: String,
               location: String) -> Self {
        ShortFnInfo {
            name,
            node_id,
            location
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Summary{
    unsafe_no: usize,
    total: usize,
}

impl Summary {
    pub fn new(unsafe_no: usize,
               total: usize) -> Self {
        Summary{ unsafe_no, total }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeFnUsafetySources {
    name: String,
    from_trait: bool,
    arguments: Vec<Argument>,
    sources: Vec<Source>,
}

impl UnsafeFnUsafetySources {
    pub fn name(&self) -> &String {&self.name}
    pub fn from_trait(&self) -> bool {self.from_trait}
    pub fn arguments(&self) -> &Vec<Argument> {&self.arguments}
    pub fn sources(&self) -> &Vec<Source> {&self.sources}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Argument {
    type_info: String,
    kind: ArgumentKind,
}

impl Argument {
    pub fn new(type_info: String,
               kind: ArgumentKind) -> Self {
        Self{ type_info, kind }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArgumentKind {
    RawPointer,
    UnsafeFunction,
}

impl UnsafeFnUsafetySources {
    pub fn new(name: String, from_trait: bool) -> Self {
        UnsafeFnUsafetySources {
            name,
            from_trait,
            arguments: Vec::new(),
            sources: Vec::new(),
        }
    }

    pub fn add_argument(&mut self, arg: Argument) {
        self.arguments.push(arg);
    }

    pub fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalCallsSummary {
    calls: Vec<(String,usize)>,
}

impl ExternalCallsSummary {
    pub fn new() -> Self {
        ExternalCallsSummary{ calls: Vec::new() }
    }

    pub fn push(&mut self, fn_name: String, count: usize) {
        self.calls.push((fn_name,count));
    }
}
