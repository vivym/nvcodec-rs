macro_rules! wrap {
    ($val:expr, $res:ident) => (
        if $res == crate::ffi::cuda::cudaError_enum_CUDA_SUCCESS {
            Ok($val)
        } else {
            Err($res)
        }
    )
}
