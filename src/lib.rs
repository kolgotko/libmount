#![feature(try_from)]

extern crate libc;

use std::ffi::*;
use std::error::Error;
use std::collections::HashMap;
use std::io::Error as IOError;
use std::convert::{ TryFrom, TryInto };
use std::fmt;

pub mod libc_mount;

pub type MntParams = HashMap<MntParam, MntParam>;

#[derive(Debug)]
pub enum LibMountError {
    IOError(IOError),
    NulError(NulError),
}

impl fmt::Display for LibMountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LibMountError::IOError(error) => {
                error.fmt(f)
            },
            LibMountError::NulError(error) => {
                error.fmt(f)
            },
        }
    }
}

impl Error for LibMountError {}

impl From<IOError> for LibMountError {
    fn from(value: IOError) -> LibMountError {
        LibMountError::IOError(value)
    }
}

impl From<NulError> for LibMountError {
    fn from(value: NulError) -> LibMountError {
        LibMountError::NulError(value)
    }
}

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

impl TryFrom<String> for MntParam {
    type Error = NulError;

    fn try_from(value: String) -> Result<MntParam, Self::Error> {
        Ok(MntParam::CString(CString::new(value)?))
    }
}

impl TryFrom<&str> for MntParam {
    type Error = NulError;

    fn try_from(value: &str) -> Result<MntParam, Self::Error> {
        Ok(MntParam::CString(CString::new(value)?))
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

pub fn nmount(params: MntParams, flags: Option<i32>) -> Result<(), LibMountError> {

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
        Err(IOError::last_os_error())?
    } else {
        Ok(())
    }

}

pub fn unmount(dir: impl Into<String>, flags: Option<i32>) -> Result<(), LibMountError> {

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
        Err(IOError::last_os_error())?
    } else {
        Ok(())
    }

}

pub fn mount_nullfs<P>(target: P, mount_point: P, options: Option<MntParams>, flags: Option<i32>)
    -> Result<(), LibMountError> 
    where P: TryInto<MntParam, Error=LibMountError> {

        let mut params: MntParams = HashMap::new();

        if let Some(options) = options {
            params.extend(options);
        }

        params.insert("fstype".try_into()?, "nullfs".try_into()?);
        params.insert("fspath".try_into()?, mount_point.try_into().unwrap());
        params.insert("target".try_into()?, target.try_into()?);

        nmount(params, flags)

    }

macro_rules! new_mount {
    ($fn_name:ident, $fs_type:expr) => {

        pub fn $fn_name<P>(mount_point: P, options: Option<MntParams>, flags: Option<i32>)
            -> Result<(), LibMountError> 
            where P: TryInto<MntParam, Error=LibMountError> {

                let mut params: HashMap<MntParam, MntParam> = HashMap::new();

                if let Some(options) = options {
                    params.extend(options);
                }

                params.insert("fstype".try_into()?, $fs_type.try_into()?);
                params.insert("fspath".try_into()?, mount_point.try_into()?);

                nmount(params, flags)

            }

    }
}

new_mount!(mount_procfs, "procfs");

new_mount!(mount_devfs, "devfs");

new_mount!(mount_fdescfs, "fdescfs");

#[macro_export]
macro_rules! mount_options {
    ({ $($key:expr => $value:expr),+ }) => {
        {
            let mut options: MntParams = HashMap::new();
            $(
                options.insert($key.try_into()?, $value.try_into()?);
            )+
            Some(options)
        }
    }
}
