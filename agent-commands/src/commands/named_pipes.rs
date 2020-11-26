#![cfg(windows)]
// Let's put this so that it won't open the console
#![windows_subsystem = "windows"]

#[cfg(windows)]
use std::io::Error;
#[cfg(windows)]
use std::ffi::CString;
#[cfg(windows)]
use winapi::shared::minwindef::{BOOL, DWORD};
#[cfg(windows)]
use winapi::ctypes::*;
use winapi::um::winnt::{HANDLE};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::minwinbase::{LPSECURITY_ATTRIBUTES, SECURITY_ATTRIBUTES, LPOVERLAPPED, OVERLAPPED};
use winapi::um::winbase::{CreateNamedPipeA};
use winapi::um::namedpipeapi::ConnectNamedPipe;
use winapi::um::fileapi::WriteFile;
use winapi::um::dbghelp::*;
use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};
use std::mem;
use std::ptr::null_mut;
use std::thread;
use std::sync::mpsc;


// https://github.com/hniksic/rust-subprocess/blob/master/src/win32.rs
// https://github.com/blackbeam/named_pipe/blob/master/src/lib.rs

/*
#[cfg(windows)]
// Get win32 lpstr from &str, converting u8 to u16 and appending '\0'
// See retep998's traits for a more general solution: https://users.rust-lang.org/t/tidy-pattern-to-work-with-lpstr-mutable-char-array/2976/2
fn to_wstring(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
*/


#[cfg(windows)]
fn create_named_pipe(lp_name: &str) -> Result<HANDLE, Error> {
    // https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createnamedpipea

    let lp_name = CString::new(lp_name).unwrap();

    let inherit_handle: bool = false;

    // https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/aa379560(v=vs.85)
    let mut attributes = SECURITY_ATTRIBUTES {
        nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: inherit_handle as BOOL,
    };

    let ret = unsafe {
        CreateNamedPipeA(
            lp_name.as_ptr(),
            3, // PIPE_ACCESS_DUPLEX
            4, // PIPE_TYPE_MESSAGE
            1, 
            1024, 
            1024, 
            0, 
            &mut attributes as LPSECURITY_ATTRIBUTES,
        )
    };

    if ret != INVALID_HANDLE_VALUE {
        Ok(ret)
    } else {
        Err(Error::last_os_error())
    }
}

#[cfg(windows)]
fn connect_named_pipe(h_named_pipe: HANDLE) -> Result<BOOL, Error> {
    // https://docs.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-connectnamedpipe

    let ret = unsafe {
        ConnectNamedPipe(
            h_named_pipe,
            null_mut(),
        )
    };

    if ret != 0{
        Ok(ret)
    } else {
        Err(Error::last_os_error())
    }
}

#[cfg(windows)]
fn write_file(h_named_pipe: HANDLE, buffer: Vec<u8>) -> Result<BOOL, Error> {
    // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-writefile

    let mut bytes_written = 0;

    let ret = unsafe {
        WriteFile(
            h_named_pipe,
            buffer.as_ptr() as *mut c_void,
            buffer.len() as u32,
            &mut bytes_written,
            null_mut(),
        )
    };

    if ret != 0 {
        Ok(ret)
    } else {
        Err(Error::last_os_error())
    }
}

#[cfg(windows)]
pub fn named_pipe_create(cmd_arg: String) -> (String, String){
    //println!("cmd_arg: {:#?}", cmd_arg);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let np: String = format!("\\\\.\\pipe\\{}", cmd_arg);
        let lp_name: &str = &np;
        let h_named_pipe: HANDLE = create_named_pipe(lp_name).unwrap();

        let val = String::from("Pipe well created");
        tx.send(val).unwrap();

        connect_named_pipe(h_named_pipe).unwrap();
    });
    let received = rx.recv().unwrap();

    (received, "".to_string())
}
