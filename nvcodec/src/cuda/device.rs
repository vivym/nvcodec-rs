use crate::ffi::cuda::{CUdevice, CUresult, cuDeviceGet, cuDeviceGetCount};

use std::os::raw::c_int;

pub struct CuDevice {
    pub device: CUdevice,
}

impl CuDevice {
    pub fn new(ordinal: u32) -> Result<CuDevice, CUresult> {
        let mut d = CuDevice { device: 0 };
        let res = unsafe { cuDeviceGet(&mut d.device as *mut i32, ordinal as c_int) };

        wrap!(d, res)
    }
}

pub fn get_device_count() -> Result<u32, CUresult> {
    let mut val: c_int = 0;
    let res = unsafe { cuDeviceGetCount(&mut val as *mut i32) };

    wrap!(val as u32, res)
}
