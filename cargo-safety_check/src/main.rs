extern crate cargo;

use cargo::Config;
use cargo::core::shell::Shell;
use cargo::core::Workspace;
use cargo::ops;

fn main() {

    let mut cargo_config = match Config::default() {
        Ok(cfg) => cfg,
        Err(e) => {
            let mut shell = Shell::new();
            cargo::exit_with_error(e.into(), &mut shell)
        }
    };

    let _ = cargo_config.configure(0u32, &Some (false),
                                   &None, false, false, &[],
                                    &[] //unstable flags
    );

    let workspace = Workspace::new(config.manifest.as_path(), &cargo_config).map_err(|_| 1i32)?;


    // want to put dependencies in env

    let rustflags = "RUSTFLAGS";
    let mut value = " -C relocation-model=dynamic-no-pic -C link-dead-code -C opt-level=0 ".to_string();
    if let Ok(vtemp) = env::var(rustflags) {
        value.push_str(vtemp.as_ref());
    }
    env::set_var(rustflags, value);


    // execute cargo build
    let mut copt = ops::CompileOptions::default(&cargo_config, ops::CompileMode::Build);
    let compilation = ops::compile(&workspace, &copt);


}

