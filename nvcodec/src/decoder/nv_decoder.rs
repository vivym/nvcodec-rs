use crate::{
    cuda::{
        context::{
            CuPrimaryContext,
            CuPrimaryContextGuard,
        },
        device::CuDevice,
    },
    demuxer::ffmpeg::FFmpegDemuxStream,
    ffi::nvcuvid::{
        CUVIDPARSERPARAMS,
        cuvidCtxLockCreate,
        cuvidCreateVideoParser,
        cudaVideoCodec,
        cudaVideoCodec_enum_cudaVideoCodec_MPEG1,
        cudaVideoCodec_enum_cudaVideoCodec_MPEG2,
        cudaVideoCodec_enum_cudaVideoCodec_MPEG4,
        cudaVideoCodec_enum_cudaVideoCodec_VC1,
        cudaVideoCodec_enum_cudaVideoCodec_H264,
        cudaVideoCodec_enum_cudaVideoCodec_JPEG,
        cudaVideoCodec_enum_cudaVideoCodec_HEVC,
        cudaVideoCodec_enum_cudaVideoCodec_VP8,
        cudaVideoCodec_enum_cudaVideoCodec_VP9,
        cudaVideoCodec_enum_cudaVideoCodec_AV1,
        CUresult,
        CUvideoparser,
        CUvideoctxlock,
        CUVIDEOFORMAT,
        CUVIDPICPARAMS,
        CUVIDPARSERDISPINFO,
        CUVIDOPERATINGPOINTINFO,
    },
};
use futures::stream::Stream;
use ffmpeg_next::codec::Id as CodecId;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

fn to_cuda_codec_id(codec_id: CodecId) -> cudaVideoCodec {
    match codec_id {
        CodecId::MPEG1VIDEO => cudaVideoCodec_enum_cudaVideoCodec_MPEG1,
        CodecId::MPEG2VIDEO => cudaVideoCodec_enum_cudaVideoCodec_MPEG2,
        CodecId::MPEG4 => cudaVideoCodec_enum_cudaVideoCodec_MPEG4,
        CodecId::VC1 => cudaVideoCodec_enum_cudaVideoCodec_VC1,
        CodecId::H264 => cudaVideoCodec_enum_cudaVideoCodec_H264,
        CodecId::JPEG2000 => cudaVideoCodec_enum_cudaVideoCodec_JPEG,
        CodecId::HEVC => cudaVideoCodec_enum_cudaVideoCodec_HEVC,
        CodecId::VP8 => cudaVideoCodec_enum_cudaVideoCodec_VP8,
        CodecId::VP9 => cudaVideoCodec_enum_cudaVideoCodec_VP9,
        CodecId::AV1 => cudaVideoCodec_enum_cudaVideoCodec_AV1,
        _ => panic!("unsupported codec id: {:?}", codec_id),
    }
}

struct Inner {
    parser: CUvideoparser,
    ctx_lock: CUvideoctxlock,
}

pub struct NVDecoder {
}

impl NVDecoder {
    pub fn new(demuxer: &FFmpegDemuxStream) -> Result<Self, CUresult> {
        let device = CuDevice::new(0).unwrap();
        let ctx = CuPrimaryContext::retain(&device).unwrap();
        let _guard = CuPrimaryContextGuard::new(ctx).unwrap();

        let mut parser = std::ptr::null_mut();
        let mut ctx_lock = std::ptr::null_mut();

        unsafe {
            let res = cuvidCtxLockCreate(&mut ctx_lock, _guard.context.context as _);
            wrap!(res, res)?;
        }

        let mut inner = Box::new(Inner {
            parser,
            ctx_lock,
        });

        let mut params: CUVIDPARSERPARAMS = unsafe { std::mem::zeroed() };
        params.CodecType = to_cuda_codec_id(demuxer.codec_id);
        params.ulMaxNumDecodeSurfaces = 1;
        params.ulClockRate = 0;
        params.ulErrorThreshold = 100;
        params.ulMaxDisplayDelay = 1;
        params.pUserData = (&mut *inner as *mut Inner) as *mut std::os::raw::c_void;
        params.pfnSequenceCallback = Some(handle_video_sequence_proc);
        params.pfnDecodePicture = Some(handle_picture_decode_proc);
        params.pfnDisplayPicture = Some(handle_picture_display_proc);
        params.pfnGetOperatingPoint = Some(handle_operating_point_proc);

        unsafe {
            let res = cuvidCreateVideoParser(&mut parser, &mut params);
            wrap!(res, res)?;
        }
        inner.parser = parser;

        Ok(Self {
        })
    }
}

impl Inner {
    fn sequence_callback(&mut self, video_format: *mut CUVIDEOFORMAT) -> i32 {
        let fmt = unsafe { &*video_format };
        0
    }

    fn picture_decode_callback(&mut self, display_info: *mut CUVIDPICPARAMS) -> i32 {
        0
    }

    fn picture_display_callback(&mut self, display_info: *mut CUVIDPARSERDISPINFO) -> i32 {
        0
    }

    fn operating_point_callback(&self, _op_info: *mut CUVIDOPERATINGPOINTINFO) -> i32 {
        0
    }
}

impl Stream for NVDecoder {
    type Item = io::Result<()>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(None)
    }
}

pub unsafe extern "C" fn handle_video_sequence_proc(
    user_data: *mut std::os::raw::c_void,
    video_format: *mut CUVIDEOFORMAT,
) -> i32 {
    let decoder = user_data as *mut Inner;
    let decoder = &mut *decoder;

    decoder.sequence_callback(video_format)
}

pub unsafe extern "C" fn handle_picture_decode_proc(
    user_data: *mut std::os::raw::c_void,
    pic_params: *mut CUVIDPICPARAMS,
) -> i32 {
    let decoder = user_data as *mut Inner;
    let decoder = &mut *decoder;

    decoder.picture_decode_callback(pic_params)
}

pub unsafe extern "C" fn handle_picture_display_proc(
    user_data: *mut std::os::raw::c_void,
    display_info: *mut CUVIDPARSERDISPINFO,
) -> i32 {
    let decoder = user_data as *mut Inner;
    let decoder = &mut *decoder;

    decoder.picture_display_callback(display_info)
}

pub unsafe extern "C" fn handle_operating_point_proc(
    user_data: *mut std::os::raw::c_void,
    op_info: *mut CUVIDOPERATINGPOINTINFO,
) -> i32 {
    let decoder = user_data as *mut Inner;
    let decoder = &*decoder;

    decoder.operating_point_callback(op_info)
}
