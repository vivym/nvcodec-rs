macro_rules! wrap {
    ($val:expr, $res:ident) => (
        if $res == crate::ffi::cudaError_enum_CUDA_SUCCESS {
            Ok($val)
        } else {
            use cuda_rs::error::CuError;
            Err(CuError::from($res))
        }
    )
}
