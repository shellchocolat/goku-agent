use std::io::prelude::*;
use std::fs::File;

pub fn download(cmd_arg: String) -> (String, String) {
    //println!("cmd_arg: {:#?}", cmd_arg);
    //let path = cmd_arg;

    //println!("{:#?}", agent_encryption::upload_encoding(&data));

    let mut iter = cmd_arg.split_whitespace();
    let local_path = iter.next();
    let remote_filename = iter.next();

    let local_path_str: &str = match local_path {
        None => {"mjollnir"},
        Some(ref path) => {path},
    };

    let remote_filename_str: &str = match remote_filename {
        None => {"mjollnir"},
        Some(ref name) => {name},
    };

    let mut file = File::open(local_path_str.clone()).expect("file not found");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("cannot read file");

    let scan_result: String = format!("{} {}", remote_filename_str, agent_encryption::upload_encoding(&data));
    let scan_error: String = "".to_string();
    (scan_result, scan_error)
}
