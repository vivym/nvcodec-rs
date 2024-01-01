pub extern crate nvcodec_sys as ffi;

#[macro_use]
mod macros;

pub mod codec;
pub mod error;
pub mod decoder;
pub mod demuxer;
pub mod surface;
