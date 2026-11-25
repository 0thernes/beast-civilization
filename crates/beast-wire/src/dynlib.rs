//! Zero-dependency dynamic library loader (Win64 `kernel32.dll` / POSIX `libdl`).

use std::ffi::{c_void, CString};
use std::path::Path;

pub struct DynLib {
    handle: *mut c_void,
}

unsafe impl Send for DynLib {}
unsafe impl Sync for DynLib {}

#[cfg(windows)]
mod os {
    use std::ffi::c_void;
    unsafe extern "system" {
        pub fn LoadLibraryA(lpLibFileName: *const i8) -> *mut c_void;
        pub fn GetProcAddress(hModule: *mut c_void, lpProcName: *const i8) -> *mut c_void;
        pub fn FreeLibrary(hLibModule: *mut c_void) -> i32;
    }
}

#[cfg(unix)]
mod os {
    use std::ffi::c_void;
    pub const RTLD_NOW: i32 = 2;
    unsafe extern "C" {
        pub fn dlopen(filename: *const i8, flags: i32) -> *mut c_void;
        pub fn dlsym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
        pub fn dlclose(handle: *mut c_void) -> i32;
    }
}

impl DynLib {
    pub fn open(path: &Path) -> Result<Self, String> {
        let path_str = path.to_str().ok_or_else(|| "non-UTF8 library path".to_string())?;
        let cstr = CString::new(path_str).map_err(|_| "interior NUL in library path".to_string())?;
        #[cfg(windows)]
        let handle = unsafe { os::LoadLibraryA(cstr.as_ptr()) };
        #[cfg(unix)]
        let handle = unsafe { os::dlopen(cstr.as_ptr(), os::RTLD_NOW) };
        if handle.is_null() {
            Err(format!("failed to load dynamic library: {path_str}"))
        } else {
            Ok(Self { handle })
        }
    }

    pub fn symbol_raw(&self, name: &str) -> Result<*mut c_void, String> {
        let cstr = CString::new(name).map_err(|_| "interior NUL in symbol".to_string())?;
        #[cfg(windows)]
        let ptr = unsafe { os::GetProcAddress(self.handle, cstr.as_ptr()) };
        #[cfg(unix)]
        let ptr = unsafe { os::dlsym(self.handle, cstr.as_ptr()) };
        if ptr.is_null() {
            Err(format!("missing symbol: {name}"))
        } else {
            Ok(ptr)
        }
    }
}

impl Drop for DynLib {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            #[cfg(windows)]
            unsafe {
                os::FreeLibrary(self.handle);
            }
            #[cfg(unix)]
            unsafe {
                os::dlclose(self.handle);
            }
        }
    }
}
