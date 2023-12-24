use crate::ffi::cuda::{
    CUcontext, CUresult, cuDevicePrimaryCtxRetain, cuCtxPushCurrent_v2, cuCtxPopCurrent_v2
};
use super::device::CuDevice;

pub struct CuPrimaryContext {
    pub context: CUcontext,
}

impl CuPrimaryContext {
    pub fn retain(device: &CuDevice) -> Result<CuPrimaryContext, CUresult> {
        let mut ctx = CuPrimaryContext { context: std::ptr::null_mut() };
        let res = unsafe { cuDevicePrimaryCtxRetain(&mut ctx.context, device.device) };

        wrap!(ctx, res)
    }

    pub fn push(&self) -> Result<(), CUresult> {
        let res = unsafe { cuCtxPushCurrent_v2(self.context) };

        wrap!((), res)
    }

    pub fn pop(&self) -> Result<(), CUresult> {
        let res = unsafe { cuCtxPopCurrent_v2(std::ptr::null_mut()) };

        wrap!((), res)
    }
}

pub struct CuPrimaryContextGuard {
    pub context: CuPrimaryContext,
}

impl CuPrimaryContextGuard {
    pub fn new(context: CuPrimaryContext) -> Result<CuPrimaryContextGuard, CUresult> {
        context.push()?;
        Ok(CuPrimaryContextGuard { context })
    }
}

impl Drop for CuPrimaryContextGuard {
    fn drop(&mut self) {
        self.context.pop().unwrap();
    }
}
