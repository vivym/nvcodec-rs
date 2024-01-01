use crate::ffi;
use ffmpeg_next::codec::Id as CodecId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CuVideoCodecType {
    MPEG1 = ffi::cudaVideoCodec_enum_cudaVideoCodec_MPEG1 as isize,
    MPEG2 = ffi::cudaVideoCodec_enum_cudaVideoCodec_MPEG2 as isize,
    MPEG4 = ffi::cudaVideoCodec_enum_cudaVideoCodec_MPEG4 as isize,
    VC1 = ffi::cudaVideoCodec_enum_cudaVideoCodec_VC1 as isize,
    H264 = ffi::cudaVideoCodec_enum_cudaVideoCodec_H264 as isize,
    JPEG = ffi::cudaVideoCodec_enum_cudaVideoCodec_JPEG as isize,
    HEVC = ffi::cudaVideoCodec_enum_cudaVideoCodec_HEVC as isize,
    VP8 = ffi::cudaVideoCodec_enum_cudaVideoCodec_VP8 as isize,
    VP9 = ffi::cudaVideoCodec_enum_cudaVideoCodec_VP9 as isize,
    AV1 = ffi::cudaVideoCodec_enum_cudaVideoCodec_AV1 as isize,
}

impl From<CodecId> for CuVideoCodecType {
    fn from(codec_id: CodecId) -> Self {
        match codec_id {
            CodecId::MPEG1VIDEO => CuVideoCodecType::MPEG1,
            CodecId::MPEG2VIDEO => CuVideoCodecType::MPEG2,
            CodecId::MPEG4 => CuVideoCodecType::MPEG4,
            CodecId::VC1 => CuVideoCodecType::VC1,
            CodecId::H264 => CuVideoCodecType::H264,
            CodecId::JPEG2000 => CuVideoCodecType::JPEG,
            CodecId::HEVC => CuVideoCodecType::HEVC,
            CodecId::VP8 => CuVideoCodecType::VP8,
            CodecId::VP9 => CuVideoCodecType::VP9,
            CodecId::AV1 => CuVideoCodecType::AV1,
            _ => panic!("unsupported codec id: {:?}", codec_id),
        }
    }
}

// impl Into<CodecId> for CuVideoCodecType {
//     fn into(self) -> CodecId {
//         match self {
//             CuVideoCodecType::MPEG1 => CodecId::MPEG1VIDEO,
//             CuVideoCodecType::MPEG2 => CodecId::MPEG2VIDEO,
//             CuVideoCodecType::MPEG4 => CodecId::MPEG4,
//             CuVideoCodecType::VC1 => CodecId::VC1,
//             CuVideoCodecType::H264 => CodecId::H264,
//             CuVideoCodecType::JPEG => CodecId::JPEG2000,
//             CuVideoCodecType::HEVC => CodecId::HEVC,
//             CuVideoCodecType::VP8 => CodecId::VP8,
//             CuVideoCodecType::VP9 => CodecId::VP9,
//             CuVideoCodecType::AV1 => CodecId::AV1,
//         }
//     }
// }
