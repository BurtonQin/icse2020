extern crate cargo_registry;
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::io::{BufRead, BufReader};
use cargo_registry::category::EncodableCategory;
use cargo_registry::krate::EncodableCrate;
use cargo_registry::keyword::EncodableKeyword;
use cargo_registry::version::EncodableVersion;
use std::cmp::Ordering;
use std::env;

#[derive(Deserialize,Debug)]
struct R {
    #[serde(rename = "crate")]
    krate: EncodableCrate,
    versions: Vec<EncodableVersion>,
    keywords: Vec<EncodableKeyword>,
    categories: Vec<EncodableCategory>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Use: {:?} top input_filename", args[0]);
        panic!("Invalid command line parameters");
    }
    let top_downloaded: usize = args[1].parse::<usize>().unwrap();
    let filename = &args[2];

    let input_file = File::open(filename).expect("File not found");
    let mut reader = BufReader::new(input_file);
    let mut top = Vec::new();
    let order = |c1: &R, c2: &R| {
        if c1.krate.downloads < c2.krate.downloads {
            Ordering::Greater
        } else if c1.krate.downloads == c2.krate.downloads {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    };
    loop {
        let mut line = String::new();
        let len = reader.read_line(&mut line).expect("Error reading file");
        if len == 0 {
            //EOF reached
            break
        } else {
            //process line
            let crate_info: R = serde_json::from_str(&line).unwrap();
            if top.len() < top_downloaded {
                top.push(crate_info);
                top.sort_unstable_by(order);
            } else {
                let last_crate_downloads = {
                    if let Some (last_crate) = top.last() {
                        last_crate.krate.downloads
                    } else {
                        0
                    }
                };
                if last_crate_downloads < crate_info.krate.downloads {
                    // must replace the last element
                    top.pop();
                    top.push(crate_info);
                    top.sort_unstable_by(order);
                }
            }
        }
    }
    for crate_info in top {
        println!("{:?} {:?}", crate_info.krate.name, crate_info.krate.downloads);
    }
}

//{"crate":
//{"id":"md","name":"md","updated_at":"2015-12-11T23:56:40.231265+00:00",
//"versions":[250],"keywords":["parser","markdown"],"categories":[],"badges":[],
//"created_at":"2014-11-20T23:18:27.918437+00:00",
//"downloads":1117,"recent_downloads":40,"max_version":"0.0.1",
//"description":"A pure-Rust Markdown parser implementation, CommonMark-compatible",
//"homepage":null,"documentation":null,
//"repository":"https://github.com/netvl/md.rs",
//"links":{"version_downloads":"/api/v1/crates/md/downloads","versions":null,"owners":"/api/v1/crates/md/owners","owner_team":"/api/v1/crates/md/owner_team","owner_user":"/api/v1/crates/md/owner_user","reverse_dependencies":"/api/v1/crates/md/reverse_dependencies"},"exact_match":false},
//"versions":[{"id":250,"crate":"md","num":"0.0.1","dl_path":"/api/v1/crates/md/0.0.1/download",
//"readme_path":"/api/v1/crates/md/0.0.1/readme",
//"updated_at":"2017-11-30T02:30:04.843739+00:00",
//"created_at":"2014-11-20T23:18:27.944900+00:00","downloads":1117,"features":{},"yanked":false,"license":"MIT","links":{"dependencies":"/api/v1/crates/md/0.0.1/dependencies","version_downloads":"/api/v1/crates/md/0.0.1/downloads","authors":"/api/v1/crates/md/0.0.1/authors"},"crate_size":null}],"keywords":[{"id":"parser","keyword":"parser","created_at":"2014-11-14T20:00:40.785931+00:00","crates_cnt":247},{"id":"markdown","keyword":"markdown","created_at":"2014-11-20T23:18:27.960919+00:00","crates_cnt":44}],"categories":[]}
//
