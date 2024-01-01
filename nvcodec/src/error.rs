use cuda_rs::error::CuError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NVCodecError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("FFmpeg Error: {0}")]
    FFmpegError(#[from] ffmpeg_next::Error),
    #[error("CuError: {0}")]
    CuError(#[from] CuError),
    #[error("NotSupported Error: {0}")]
    NotSupported(String),
    #[error("Decoder not initialized")]
    DecoderNotInitialized,
    #[error("Decode error")]
    DecodeError,
    #[error("Parser error")]
    ParserError,
    #[error("Surface shape mismatch")]
    SurfaceShapeMismatch,
    #[error("Reconfigure failed")]
    ReconfigureFailed,
}

pub type NVCodecResult<T> = Result<T, NVCodecError>;
