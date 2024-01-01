use crate::ffi;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VideoSurfaceFormat {
    NV12 = ffi::cudaVideoSurfaceFormat_enum_cudaVideoSurfaceFormat_NV12 as isize,
    P016 = ffi::cudaVideoSurfaceFormat_enum_cudaVideoSurfaceFormat_P016 as isize,
    YUV444 = ffi::cudaVideoSurfaceFormat_enum_cudaVideoSurfaceFormat_YUV444 as isize,
    YUV444_16Bit = ffi::cudaVideoSurfaceFormat_enum_cudaVideoSurfaceFormat_YUV444_16Bit as isize,
}
