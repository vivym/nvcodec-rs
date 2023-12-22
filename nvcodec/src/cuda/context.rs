use crate::ffi::cuda::{
    CUcontext, CUresult, cuDevicePrimaryCtxRetain, cuCtxPushCurrent_v2, cuCtxPopCurrent_v2
};
use super::device::CuDevice;

pub struct CuPrimaryContext {
    context: CUcontext,
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

pub struct PrimaryContextGuard {
    context: CuPrimaryContext,
}

impl PrimaryContextGuard {
    pub fn new(context: CuPrimaryContext) -> Result<PrimaryContextGuard, CUresult> {
        context.push()?;
        Ok(PrimaryContextGuard { context })
    }
}

impl Drop for PrimaryContextGuard {
    fn drop(&mut self) {
        self.context.pop().unwrap();
    }
}
