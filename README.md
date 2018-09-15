# external_calls

<h1>Tools Needed</h1>
1. cargo install clone

<h1>Download top N crates from crates.io</h1>

Execute: cd select-crates

Clean up to run a fresh counter download:<br> rm crates.io-fixed

Download top N crates (N is the parameter passed): <br>
./crates_select_and_download.sh N

If the file crates.io-fixed exists then it uses it, otherwise it is
created. This file contains the information downloaded from crates.io
for each crate from crates.io-index repository.

The script parses the file and retains the top N crate names and the
downloads in top-N-crates.io.

Next, it downloads each crate in top-N-crates.io in the directory:
${HOME}/unsafe_analysis/crates.io-downloads.It uses cargo clone.

<h1>Compilation</h1>

cd unsafe-analysis/; ./compile.sh build <br>

<h1>Run Analysis</h1>
export PROJECT_HOME="$HOME/work/unsafe_study" #change this to your path<br>
cd $PROJECT_HOME/select-crates<br>
./crates_select_and_download.sh 500<br>
./compile.sh<br>

cd $PROJECT_HOME/github-downloads<br>
./download.sh<br>
./compile.sh<br>

<h1>Run the plugin on one crate</h1>

rustup override set nightly-2018-08-29<br>
export PROJECT_HOME="$HOME/work/unsafe_study" #change this to your path<br>
export RUSTFLAGS="--extern unsafe_analysis=$PROJECT_HOME/unsafe-analysis/target/debug/libunsafe-analysis.so -Z extra-plugins=unsafe-analysis --emit mir"<br>
cargo build

<h1>Run examples from repository</h1>

rustup override set nightly-2018-08-29<br>
export PROJECT_HOME="$HOME/work/unsafe_study" #change this to your path<br>
export RUSTFLAGS="--extern unsafe_analysis=$PROJECT_HOME/unsafe-analysis/target/debug/libunsafe_analysis.so -Z extra-plugins=unsafe_analysis --emit mir"<br>

cd $PROJECT_HOME/examples/elf2tbf; cargo build

cd $PROJECT_HOME/examples/tests; ./compile.sh

<h1>Issues:</h1> 

<ol>
  <item> Compiling advapi32-sys Output ${HOME}/unsafe_analysis/compiler_output/advapi32-sys Error ${HOME}/unsafe_analysis/compiler_output/advapi32-sys_error
error: failed to load source for a dependency on `winapi`

Caused by:
  Unable to update file://${HOME}/unsafe_analysis

Caused by:
  failed to read `${HOME}/unsafe_analysis/Cargo.toml`

Caused by:
  No such file or directory (os error 2)
Compiling aho-corasick Output ${HOME}/unsafe_analysis/compiler_output/aho-corasick Error ${HOME}/unsafe_analysis/compiler_output/aho-corasick_error

<item>    Compiling log v0.4.5
ERROR 2018-09-10T01:42:02Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<sip::SipHasher13 as std::hash::Hasher>::finish"
<item>    Compiling phf v0.7.23
ERROR 2018-09-10T01:42:03Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _8
ERROR 2018-09-10T01:42:04Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _8

<item> Downloading discard v1.0.3                                                     
ERROR 2018-09-10T01:42:10Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _11
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: ty::InstanceDef:: NOT handled CloneShim(DefId(2/0:1254 ~ core[15e1]::clone[0]::Clone[0]::clone[0]), ())
  
<item>  Downloading syn v0.15.3                                                        
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _9
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _7
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _5
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _14
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _3
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: ty::InstanceDef:: NOT handled CloneShim(DefId(2/0:1254 ~ core[15e1]::clone[0]::Clone[0]::clone[0]), [u32; 16])
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: ty::InstanceDef:: NOT handled CloneShim(DefId(2/0:1254 ~ core[15e1]::clone[0]::Clone[0]::clone[0]), [u32; 1024])
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: ty::InstanceDef:: NOT handled CloneShim(DefId(2/0:1254 ~ core[15e1]::clone[0]::Clone[0]::clone[0]), [std::num::Wrapping<u32>; 256])
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: ty::InstanceDef:: NOT handled CloneShim(DefId(2/0:1254 ~ core[15e1]::clone[0]::Clone[0]::clone[0]), [std::num::Wrapping<u64>; 256])
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: ty::InstanceDef:: NOT handled CloneShim(DefId(2/0:1254 ~ core[15e1]::clone[0]::Clone[0]::clone[0]), [T; 256])
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _5
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _8
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _4
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _9
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _29
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _38
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _5
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _16
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::implicit_analysis: Error external call NOT found "RngCore::next_u32"
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::implicit_analysis: Error external call NOT found "RngCore::next_u32"
ERROR 2018-09-10T01:42:11Z: unsafe_analysis::implicit_analysis: Error external call NOT found "RngCore::next_u64"

<item>    Compiling phf_generator v0.7.23
    Updating registry `https://github.com/rust-lang/crates.io-index`
ERROR 2018-09-10T01:42:17Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<XorShiftRng as SeedableRng>::from_seed"
ERROR 2018-09-10T01:42:17Z: unsafe_analysis::implicit_analysis: Error external call NOT found "Rng::gen"

<item>   Compiling phf_codegen v0.7.23
    Updating registry `https://github.com/rust-lang/crates.io-index`
    Blocking waiting for file lock on the registry index
    Updating registry `https://github.com/rust-lang/crates.io-index`
    Blocking waiting for file lock on the registry index
ERROR 2018-09-10T01:42:18Z: unsafe_analysis::implicit_analysis: Error external call NOT found "Tokens::new"
ERROR 2018-09-10T01:42:18Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<&\'a T as ToTokens>::to_tokens"
ERROR 2018-09-10T01:42:18Z: unsafe_analysis::implicit_analysis: Error external call NOT found "Tokens::append"
ERROR 2018-09-10T01:42:18Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<Tokens as ToTokens>::to_tokens"
ERROR 2018-09-10T01:42:18Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<proc_macro2::Term as ToTokens>::to_tokens"
ERROR 2018-09-10T01:42:18Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<u64 as ToTokens>::to_tokens"
ERROR 2018-09-10T01:42:18Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<u32 as ToTokens>::to_tokens"
ERROR 2018-09-10T01:42:18Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<proc_macro2::TokenStream as ToTokens>::to_tokens"
ERROR 2018-09-10T01:42:18Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<proc_macro2::TokenTree as ToTokens>::to_tokens"

<item>    Updating registry `https://github.com/rust-lang/crates.io-index`
ERROR 2018-09-10T01:42:24Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _37
ERROR 2018-09-10T01:42:24Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _31
ERROR 2018-09-10T01:42:24Z: unsafe_analysis::calls: calls.rs::Operand Type NOT handled move _9
ERROR 2018-09-10T01:42:25Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<punctuated::Punctuated<T, P>>::iter"
ERROR 2018-09-10T01:42:25Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<punctuated::Punctuated<T, P>>::len"
ERROR 2018-09-10T01:42:25Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<punctuated::Punctuated<T, P>>::iter"
ERROR 2018-09-10T01:42:25Z: unsafe_analysis::implicit_analysis: Error external call NOT found "Meta::name"
ERROR 2018-09-10T01:42:25Z: unsafe_analysis::implicit_analysis: Error external call NOT found "LitStr::value"
ERROR 2018-09-10T01:42:25Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<&\'a punctuated::Punctuated<T, P> as std::iter::IntoIterator>::into_iter"
ERROR 2018-09-10T01:42:25Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<punctuated::Iter<\'a, T> as std::iter::Iterator>::next"

<item>    Compiling serde_json v1.0.27
ERROR 2018-09-10T01:42:30Z: unsafe_analysis::implicit_analysis: Error external call NOT found "ser::impls::<impl Serialize for str>::serialize"
ERROR 2018-09-10T01:42:30Z: unsafe_analysis::implicit_analysis: Error external call NOT found "de::impls::<impl Deserialize<\'de> for std::string::String>::deserialize"

<item>     Updating registry `https://github.com/rust-lang/crates.io-index`
ERROR 2018-09-10T01:42:32Z: unsafe_analysis::implicit_analysis: Error external call NOT found "de::Visitor::visit_f64"
ERROR 2018-09-10T01:42:32Z: unsafe_analysis::implicit_analysis: Error external call NOT found "de::Visitor::visit_u64"
ERROR 2018-09-10T01:42:32Z: unsafe_analysis::implicit_analysis: Error external call NOT found "de::Visitor::visit_i64"
ERROR 2018-09-10T01:42:32Z: unsafe_analysis::implicit_analysis: Error external call NOT found "de::Visitor::visit_unit"
ERROR 2018-09-10T01:42:32Z: unsafe_analysis::implicit_analysis: Error external call NOT found "de::Visitor::visit_bool"
ERROR 2018-09-10T01:42:32Z: unsafe_analysis::implicit_analysis: Error external call NOT found "de::Visitor::visit_borrowed_str"

<item>    Compiling html5ever v0.22.3
    Updating registry `https://github.com/rust-lang/crates.io-index`
ERROR 2018-09-10T01:42:37Z: unsafe_analysis::implicit_analysis: Error external call NOT found "parse_file"
ERROR 2018-09-10T01:42:37Z: unsafe_analysis::implicit_analysis: Error external call NOT found "fold::Fold::fold_file"
ERROR 2018-09-10T01:42:37Z: unsafe_analysis::implicit_analysis: Error external call NOT found "ToTokens::into_tokens"
ERROR 2018-09-10T01:42:37Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<token::Lt as synom::Synom>::parse"
ERROR 2018-09-10T01:42:37Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<token::Div as synom::Synom>::parse"
ERROR 2018-09-10T01:42:37Z: unsafe_analysis::implicit_analysis: Error external call NOT found "ident::parsing::<impl synom::Synom for Ident>::parse"
ERROR 2018-09-10T01:42:37Z: unsafe_analysis::implicit_analysis: Error external call NOT found "<token::Underscore as synom::Synom>::parse"
  <item>   Compiling markup5ever v0.7.2
    Updating registry `https://github.com/rust-lang/crates.io-index`
ERROR 2018-09-10T01:42:44Z: unsafe_analysis::implicit_analysis: Error external call NOT found "from_reader"
ERROR 2018-09-10T01:42:44Z: unsafe_analysis::implicit_analysis: Error external call NOT found "named_entities_to_phf::_IMPL_DESERIALIZE_FOR_CharRef::_Deserializer::deserialize_struct"
ERROR 2018-09-10T01:42:44Z: unsafe_analysis::implicit_analysis: Error external call NOT found "named_entities_to_phf::_IMPL_DESERIALIZE_FOR_CharRef::_de::Error::invalid_value"
ERROR 2018-09-10T01:42:44Z: unsafe_analysis::implicit_analysis: Error external call NOT found "named_entities_to_phf::_IMPL_DESERIALIZE_FOR_CharRef::_Deserializer::deserialize_identifier"


  
</ol>
