extern crate libc;

use std::ffi::*;
use std::error::Error;
use std::collections::HashMap;
use std::io::Error as IoError;

pub mod libc_mount;

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum MntParam {
    CString(CString),
    Buffer(Vec<u8>),
}

impl MntParam {

    fn as_iov(&self) -> libc::iovec {

        match self {

            MntParam::CString(value) => {
                libc::iovec {
                    iov_base: value.as_ptr() as *mut _,
                    iov_len: value.as_bytes_with_nul().len(),
                }
            },
            MntParam::Buffer(value) => {
                libc::iovec {
                    iov_base: value.as_ptr() as *mut _,
                    iov_len: value.len(),
                }
            },

        }

    }

}

impl From<String> for MntParam {
    fn from(value: String) -> MntParam {
        MntParam::CString(CString::new(value).unwrap())
    }
}

impl From<&str> for MntParam {
    fn from(value: &str) -> MntParam {
        MntParam::CString(CString::new(value).unwrap())
    }
}

pub trait AsIovec {
    fn as_iovec(&self) -> Vec<libc::iovec>;
}

impl AsIovec for HashMap<MntParam, MntParam> {
    fn as_iovec(&self) -> Vec<libc::iovec> {

        let mut iovec_vec = Vec::new();

        for (key, value) in self.iter() {

            iovec_vec.push(key.as_iov());
            iovec_vec.push(value.as_iov());

        }

        iovec_vec

    }
}

pub fn nmount(params: HashMap<MntParam, MntParam>, flags: Option<i32>) -> Result<(), Box<Error>> {

    let iovec_params = params.as_iovec();

    let flags = if let Some(value) = flags {
        value
    } else {
        0
    };

    let result = unsafe {
        libc_mount::nmount(iovec_params.as_ptr() as *mut _, iovec_params.len() as u32, flags)
    };

    if result < 0 {
        Err(IoError::last_os_error())?
    } else {
        Ok(())
    }

}

pub fn unmount(dir: impl Into<String>, flags: Option<i32>) -> Result<(), Box<Error>> {

    let dir = CString::new(dir.into())?;

    let flags = if let Some(value) = flags {
        value
    } else {
        0
    };

    let result = unsafe {
        libc_mount::unmount(dir.as_ptr(), flags)
    };

    if result < 0 {
        Err(IoError::last_os_error())?
    } else {
        Ok(())
    }

}

pub fn mount_nullfs<P: Into<MntParam>>(target: P, mount_point: P, flags: Option<i32>)
    -> Result<(), Box<Error>> {

        let mut params: HashMap<MntParam, MntParam> = HashMap::new();
        params.insert("fstype".into(), "nullfs".into());
        params.insert("fspath".into(), mount_point.into());
        params.insert("target".into(), target.into());

        nmount(params, flags)

    }

macro_rules! new_mount {
    ($fn_name:ident, $fs_type:expr) => {

        pub fn $fn_name<P: Into<MntParam>>(mount_point: P, flags: Option<i32>)
            -> Result<(), Box<Error>> {

                let mut params: HashMap<MntParam, MntParam> = HashMap::new();
                params.insert("fstype".into(), $fs_type.into());
                params.insert("fspath".into(), mount_point.into());

                nmount(params, flags)

            }

    }
}

new_mount!(mount_procfs, "procfs");

new_mount!(mount_devfs, "devfs");

new_mount!(mount_fdescfs, "fdescfs");
