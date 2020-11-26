
extern crate libc;

use std::error::Error;
use std::io;
use std::fmt;
use libc::{c_void, c_int};
use std::ops::Drop;
use std::ptr;
use self::MemoryMapKind::*;
use self::MapOption::*;
use self::MapError::*;

#[cfg(windows)]
use std::mem;

fn errno() -> i32 {
    io::Error::last_os_error().raw_os_error().unwrap_or(-1)
}

#[cfg(unix)]
fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

#[cfg(windows)]
fn page_size() -> usize {
    unsafe {
        let mut info = mem::zeroed();
        libc::GetSystemInfo(&mut info);
        return info.dwPageSize as usize;
    }
}

/// A memory mapped file or chunk of memory. This is a very system-specific
/// interface to the OS's memory mapping facilities (`mmap` on POSIX,
/// `VirtualAlloc`/`CreateFileMapping` on Windows). It makes no attempt at
/// abstracting platform differences, besides in error values returned. Consider
/// yourself warned.
///
/// The memory map is released (unmapped) when the destructor is run, so don't
/// let it leave scope by accident if you want it to stick around.
pub struct MemoryMap {
    data: *mut u8,
    len: usize,
    kind: MemoryMapKind,
}

/// Type of memory map
#[allow(raw_pointer_derive)]
#[derive(Copy, Clone)]
pub enum MemoryMapKind {
    /// Virtual memory map. Usually used to change the permissions of a given
    /// chunk of memory.  Corresponds to `VirtualAlloc` on Windows.
    MapFile(*const u8),
    /// Virtual memory map. Usually used to change the permissions of a given
    /// chunk of memory, or for allocation. Corresponds to `VirtualAlloc` on
    /// Windows.
    MapVirtual
}

/// Options the memory map is created with
#[allow(raw_pointer_derive)]
#[derive(Copy, Clone)]
pub enum MapOption {
    /// The memory should be readable
    MapReadable,
    /// The memory should be writable
    MapWritable,
    /// The memory should be executable
    MapExecutable,
    /// Create a map for a specific address range. Corresponds to `MAP_FIXED` on
    /// POSIX.
    MapAddr(*const u8),
    /// Create a memory mapping for a file with a given HANDLE.
    #[cfg(windows)]
    MapFd(libc::HANDLE),
    /// Create a memory mapping for a file with a given fd.
    #[cfg(not(windows))]
    MapFd(c_int),
    /// When using `MapFd`, the start of the map is `usize` bytes from the start
    /// of the file.
    MapOffset(usize),
    /// On POSIX, this can be used to specify the default flags passed to
    /// `mmap`. By default it uses `MAP_PRIVATE` and, if not using `MapFd`,
    /// `MAP_ANON`. This will override both of those. This is platform-specific
    /// (the exact values used) and ignored on Windows.
    MapNonStandardFlags(c_int),
}

/// Possible errors when creating a map.
#[derive(Copy, Clone, Debug)]
pub enum MapError {
    /// # The following are POSIX-specific
    ///
    /// fd was not open for reading or, if using `MapWritable`, was not open for
    /// writing.
    ErrFdNotAvail,
    /// fd was not valid
    ErrInvalidFd,
    /// Either the address given by `MapAddr` or offset given by `MapOffset` was
    /// not a multiple of `MemoryMap::granularity` (unaligned to page size).
    ErrUnaligned,
    /// With `MapFd`, the fd does not support mapping.
    ErrNoMapSupport,
    /// If using `MapAddr`, the address + `min_len` was outside of the process's
    /// address space. If using `MapFd`, the target of the fd didn't have enough
    /// resources to fulfill the request.
    ErrNoMem,
    /// A zero-length map was requested. This is invalid according to
    /// [POSIX](http://pubs.opengroup.org/onlinepubs/9699919799/functions/mmap.html).
    /// Not all platforms obey this, but this wrapper does.
    ErrZeroLength,
    /// Unrecognized error. The inner value is the unrecognized errno.
    ErrUnknown(isize),
    /// # The following are Windows-specific
    ///
    /// Unsupported combination of protection flags
    /// (`MapReadable`/`MapWritable`/`MapExecutable`).
    ErrUnsupProt,
    /// When using `MapFd`, `MapOffset` was given (Windows does not support this
    /// at all)
    ErrUnsupOffset,
    /// When using `MapFd`, there was already a mapping to the file.
    ErrAlreadyExists,
    /// Unrecognized error from `VirtualAlloc`. The inner value is the return
    /// value of GetLastError.
    ErrVirtualAlloc(i32),
    /// Unrecognized error from `CreateFileMapping`. The inner value is the
    /// return value of `GetLastError`.
    ErrCreateFileMappingW(i32),
    /// Unrecognized error from `MapViewOfFile`. The inner value is the return
    /// value of `GetLastError`.
    ErrMapViewOfFile(i32)
}

impl fmt::Display for MapError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            ErrFdNotAvail => "fd not available for reading or writing",
            ErrInvalidFd => "Invalid fd",
            ErrUnaligned => {
                "Unaligned address, invalid flags, negative length or \
                 unaligned offset"
            }
            ErrNoMapSupport=> "File doesn't support mapping",
            ErrNoMem => "Invalid address, or not enough available memory",
            ErrUnsupProt => "Protection mode unsupported",
            ErrUnsupOffset => "Offset in virtual memory mode is unsupported",
            ErrAlreadyExists => "File mapping for specified file already exists",
            ErrZeroLength => "Zero-length mapping not allowed",
            ErrUnknown(code) => {
                return write!(out, "Unknown error = {}", code)
            },
            ErrVirtualAlloc(code) => {
                return write!(out, "VirtualAlloc failure = {}", code)
            },
            ErrCreateFileMappingW(code) => {
                return write!(out, "CreateFileMappingW failure = {}", code)
            },
            ErrMapViewOfFile(code) => {
                return write!(out, "MapViewOfFile failure = {}", code)
            }
        };
        write!(out, "{}", str)
    }
}

impl Error for MapError {
    fn description(&self) -> &str { "memory map error" }
}

// Round up `from` to be divisible by `to`
fn round_up(from: usize, to: usize) -> usize {
    let r = if from % to == 0 {
        from
    } else {
        from + to - (from % to)
    };
    if r == 0 {
        to
    } else {
        r
    }
}

#[cfg(unix)]
impl MemoryMap {
    /// Create a new mapping with the given `options`, at least `min_len` bytes
    /// long. `min_len` must be greater than zero; see the note on
    /// `ErrZeroLength`.
    pub fn new(min_len: usize, options: &[MapOption]) -> Result<MemoryMap, MapError> {
        use libc::off_t;

        if min_len == 0 {
            return Err(ErrZeroLength)
        }
        let mut addr: *const u8 = ptr::null();
        let mut prot = 0;
        let mut flags = libc::MAP_PRIVATE;
        let mut fd = -1;
        let mut offset = 0;
        let mut custom_flags = false;
        let len = round_up(min_len, page_size());

        for &o in options {
            match o {
                MapReadable => { prot |= libc::PROT_READ; },
                MapWritable => { prot |= libc::PROT_WRITE; },
                MapExecutable => { prot |= libc::PROT_EXEC; },
                MapAddr(addr_) => {
                    flags |= libc::MAP_FIXED;
                    addr = addr_;
                },
                MapFd(fd_) => {
                    flags |= libc::MAP_FILE;
                    fd = fd_;
                },
                MapOffset(offset_) => { offset = offset_ as off_t; },
                MapNonStandardFlags(f) => { custom_flags = true; flags = f },
            }
        }
        if fd == -1 && !custom_flags { flags |= libc::MAP_ANON; }

        let r = unsafe {
            libc::mmap(addr as *mut c_void, len as libc::size_t, prot, flags,
                       fd, offset)
        };
        if r == libc::MAP_FAILED {
            Err(match errno() {
                libc::EACCES => ErrFdNotAvail,
                libc::EBADF => ErrInvalidFd,
                libc::EINVAL => ErrUnaligned,
                libc::ENODEV => ErrNoMapSupport,
                libc::ENOMEM => ErrNoMem,
                code => ErrUnknown(code as isize)
            })
        } else {
            Ok(MemoryMap {
               data: r as *mut u8,
               len: len,
               kind: if fd == -1 {
                   MapVirtual
               } else {
                   MapFile(ptr::null())
               }
            })
        }
    }

    /// Granularity that the offset or address must be for `MapOffset` and
    /// `MapAddr` respectively.
    pub fn granularity() -> usize {
        page_size()
    }
}

#[cfg(unix)]
impl Drop for MemoryMap {
    /// Unmap the mapping. Panics the task if `munmap` panics.
    fn drop(&mut self) {
        if self.len == 0 { /* workaround for dummy_stack */ return; }

        unsafe {
            // `munmap` only panics due to logic errors
            libc::munmap(self.data as *mut c_void, self.len as libc::size_t);
        }
    }
}

#[cfg(windows)]
impl MemoryMap {
    /// Create a new mapping with the given `options`, at least `min_len` bytes long.
    pub fn new(min_len: usize, options: &[MapOption]) -> Result<MemoryMap, MapError> {
        use libc::types::os::arch::extra::{LPVOID, DWORD, SIZE_T, HANDLE};

        let mut lp_address: LPVOID = ptr::null_mut();
        let mut readable = false;
        let mut writable = false;
        let mut executable = false;
        let mut handle: HANDLE = libc::INVALID_HANDLE_VALUE;
        let mut offset: usize = 0;
        let len = round_up(min_len, page_size());

        for &o in options {
            match o {
                MapReadable => { readable = true; },
                MapWritable => { writable = true; },
                MapExecutable => { executable = true; }
                MapAddr(addr_) => { lp_address = addr_ as LPVOID; },
                MapFd(handle_) => { handle = handle_; },
                MapOffset(offset_) => { offset = offset_; },
                MapNonStandardFlags(..) => {}
            }
        }

        let fl_protect = match (executable, readable, writable) {
            (false, false, false) if handle == libc::INVALID_HANDLE_VALUE => libc::PAGE_NOACCESS,
            (false, true, false) => libc::PAGE_READONLY,
            (false, true, true) => libc::PAGE_READWRITE,
            (true, false, false) if handle == libc::INVALID_HANDLE_VALUE => libc::PAGE_EXECUTE,
            (true, true, false) => libc::PAGE_EXECUTE_READ,
            (true, true, true) => libc::PAGE_EXECUTE_READWRITE,
            _ => return Err(ErrUnsupProt)
        };

        if handle == libc::INVALID_HANDLE_VALUE {
            if offset != 0 {
                return Err(ErrUnsupOffset);
            }
            let r = unsafe {
                libc::VirtualAlloc(lp_address,
                                   len as SIZE_T,
                                   libc::MEM_COMMIT | libc::MEM_RESERVE,
                                   fl_protect)
            };
            match r as usize {
                0 => Err(ErrVirtualAlloc(errno())),
                _ => Ok(MemoryMap {
                   data: r as *mut u8,
                   len: len,
                   kind: MapVirtual
                })
            }
        } else {
            let dw_desired_access = match (executable, readable, writable) {
                (false, true, false) => libc::FILE_MAP_READ,
                (false, true, true) => libc::FILE_MAP_WRITE,
                (true, true, false) => libc::FILE_MAP_READ | libc::FILE_MAP_EXECUTE,
                (true, true, true) => libc::FILE_MAP_WRITE | libc::FILE_MAP_EXECUTE,
                _ => return Err(ErrUnsupProt) // Actually, because of the check above,
                                              // we should never get here.
            };
            unsafe {
                let h_file = handle;
                let mapping = libc::CreateFileMappingW(h_file,
                                                       ptr::null_mut(),
                                                       fl_protect,
                                                       0,
                                                       0,
                                                       ptr::null());
                if mapping == ptr::null_mut() {
                    return Err(ErrCreateFileMappingW(errno()));
                }
                if errno() as c_int == libc::ERROR_ALREADY_EXISTS {
                    return Err(ErrAlreadyExists);
                }
                let r = libc::MapViewOfFile(mapping,
                                            dw_desired_access,
                                            ((len as u64) >> 32) as DWORD,
                                            (offset & 0xffff_ffff) as DWORD,
                                            0);
                match r as usize {
                    0 => Err(ErrMapViewOfFile(errno())),
                    _ => Ok(MemoryMap {
                       data: r as *mut u8,
                       len: len,
                       kind: MapFile(mapping as *const u8)
                    })
                }
            }
        }
    }

    /// Granularity of MapAddr() and MapOffset() parameter values.
    /// This may be greater than the value returned by page_size().
    pub fn granularity() -> usize {
        use mem;
        unsafe {
            let mut info = mem::zeroed();
            libc::GetSystemInfo(&mut info);

            return info.dwAllocationGranularity as usize;
        }
    }
}

#[cfg(windows)]
impl Drop for MemoryMap {
    /// Unmap the mapping. Panics the task if any of `VirtualFree`,
    /// `UnmapViewOfFile`, or `CloseHandle` fail.
    fn drop(&mut self) {
        use libc::types::os::arch::extra::{LPCVOID, HANDLE};
        use libc::consts::os::extra::FALSE;
        if self.len == 0 { return }

        unsafe {
            match self.kind {
                MapVirtual => {
                    if libc::VirtualFree(self.data as *mut c_void, 0,
                                         libc::MEM_RELEASE) == 0 {
                        println!("VirtualFree failed: {}", errno());
                    }
                },
                MapFile(mapping) => {
                    if libc::UnmapViewOfFile(self.data as LPCVOID) == FALSE {
                        println!("UnmapViewOfFile failed: {}", errno());
                    }
                    if libc::CloseHandle(mapping as HANDLE) == FALSE {
                        println!("CloseHandle failed: {}", errno());
                    }
                }
            }
        }
    }
}

impl MemoryMap {
    /// Returns the pointer to the memory created or modified by this map.
    #[inline(always)]
    pub fn data(&self) -> *mut u8 { self.data }

    /// Returns the number of bytes this map applies to.
    #[inline(always)]
    pub fn len(&self) -> usize { self.len }

    /// Returns the type of mapping this represents.
    pub fn kind(&self) -> MemoryMapKind { self.kind }
}

#[cfg(test)]
mod tests {
    extern crate libc;
    extern crate tempdir;

    use super::{MemoryMap, MapOption};

    #[test]
    fn memory_map_rw() {
        let chunk = match MemoryMap::new(16, &[
            MapOption::MapReadable,
            MapOption::MapWritable
        ]) {
            Ok(chunk) => chunk,
            Err(msg) => panic!("{:?}", msg)
        };
        assert!(chunk.len >= 16);

        unsafe {
            *chunk.data = 0xBE;
            assert!(*chunk.data == 0xBE);
        }
    }

    #[test]
    fn memory_map_file() {
        use std::fs;
        use std::io::{Seek, SeekFrom, Write};

        #[cfg(unix)]
        use std::os::unix::io::AsRawFd;

        #[cfg(unix)]
        fn get_fd(file: &fs::File) -> libc::c_int {
            file.as_raw_fd()
        }

        #[cfg(windows)]
        fn get_fd(file: &fs::File) -> libc::HANDLE {
            file.as_raw_handle()
        }

        let tmpdir = tempdir::TempDir::new("").unwrap();
        let mut path = tmpdir.path().to_path_buf();
        path.push("mmap_file.tmp");
        let size = MemoryMap::granularity() * 2;

        let mut file = fs::OpenOptions::new()
                        .create(true)
                        .read(true)
                        .write(true)
                        .open(&path)
                        .unwrap();
        file.seek(SeekFrom::Start(size as u64)).unwrap();
        file.write(b"\0").unwrap();
        let fd = get_fd(&file);

        let chunk = MemoryMap::new(size / 2, &[
            MapOption::MapReadable,
            MapOption::MapWritable,
            MapOption::MapFd(fd),
            MapOption::MapOffset(size / 2)
        ]).unwrap();
        assert!(chunk.len > 0);

        unsafe {
            *chunk.data = 0xbe;
            assert!(*chunk.data == 0xbe);
        }
        drop(chunk);

        fs::remove_file(&path).unwrap();
    }
}
