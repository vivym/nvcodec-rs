use crate::error::NVCodecResult;
use futures::{
    stream::Stream,
    task::AtomicWaker,
};
use std::{
    io,
    ffi::CString,
    path::Path,
    pin::Pin,
    sync::{Arc, mpsc::{self, Receiver}},
    task::{Context, Poll},
    thread,
};
use ffmpeg_next::{
    codec::{
        Id as CodecId,
        packet::{Packet as AVPacket, Mut},
        Parameters,
    },
    util::color::{Range, Space},
};

pub struct Packet {
    av_packet: AVPacket,
    color_space: Space,
    color_range: Range,
}

impl Packet {
    pub fn data(&self) -> Option<&[u8]> {
        self.av_packet.data()
    }

    pub fn data_mut(&mut self) -> Option<&mut [u8]> {
        self.av_packet.data_mut()
    }

    pub fn is_key(&self) -> bool {
        self.av_packet.is_key()
    }

    pub fn is_corrupt(&self) -> bool {
        self.av_packet.is_corrupt()
    }

    pub fn pts(&self) -> Option<i64> {
        self.av_packet.pts()
    }

    pub fn dts(&self) -> Option<i64> {
        self.av_packet.dts()
    }

    pub fn duration(&self) -> i64 {
        self.av_packet.duration()
    }

    pub fn position(&self) -> isize {
        self.av_packet.position()
    }

    pub fn size(&self) -> usize {
        self.av_packet.size()
    }

    pub fn color_space(&self) -> Space {
        self.color_space
    }

    pub fn color_range(&self) -> Range {
        self.color_range
    }
}

pub struct FFmpegDemuxStream {
    pub codec_id: CodecId,
    pub total_frames: i64,
    pub color_space: Space,
    pub color_range: Range,
    waker: Arc<AtomicWaker>,
    rx: Receiver<NVCodecResult<Packet>>,
}

impl FFmpegDemuxStream {
    pub fn new<P: AsRef<Path>>(path: &P) -> NVCodecResult<Self> {
        let waker = Arc::new(AtomicWaker::new());
        let (tx, rx) =
            mpsc::sync_channel::<NVCodecResult<Packet>>(8);

        let mut ctx = ffmpeg_next::format::input(path)?;

        let stream = ctx
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;
        let video_stream_index = stream.index();
        let total_frames = stream.frames();
        let stream_params = stream.parameters();
        let codec_ctx = stream.codec();
        let codec_id = codec_ctx.id();

        let (color_range, color_space) = unsafe {
            let params = *stream_params.as_ptr();
            (params.color_range, params.color_space)
        };

        let waker_in_thread: Arc<AtomicWaker> = waker.clone();

        thread::spawn(move || {
            let bsf_name = match codec_id {
                CodecId::H264 => Some("h264_mp4toannexb"),
                CodecId::HEVC => Some("hevc_mp4toannexb"),
                _ => None,
            };
            let bsf_ctx = match bsf_name {
                Some(name) => {
                    match BSFContext::new(name, stream_params) {
                        Ok(ctx) => Some(ctx),
                        Err(e) => {
                            if tx.send(Err(e)).is_err() {
                                return;
                            }
                            waker_in_thread.wake();
                            return;
                        }
                    }
                }
                None => None,
            };

            for (stream, mut packet) in ctx.packets() {
                if stream.index() == video_stream_index {
                    let packet = match bsf_ctx {
                        Some(ref bsf_ctx) => {
                            match bsf_ctx.send_packet(&mut packet) {
                                Ok(_) => (),
                                Err(e) => {
                                    if tx.send(Err(e)).is_err() {
                                        return;
                                    }
                                    waker_in_thread.wake();
                                    return;
                                }
                            }
                            match bsf_ctx.receive_packet() {
                                Ok(packet) => packet,
                                Err(e) => {
                                    if tx.send(Err(e)).is_err() {
                                        return;
                                    }
                                    waker_in_thread.wake();
                                    return;
                                }
                            }
                        }
                        None => packet,
                    };
                    let packet = Packet {
                        av_packet: packet,
                        color_space: color_space.into(),
                        color_range: color_range.into(),
                    };

                    if tx.send(Ok(packet)).is_err() {
                        return;
                    }
                    waker_in_thread.wake();
                }
            }

            std::mem::drop(tx);
            waker_in_thread.wake();
        });

        Ok(Self {
            codec_id,
            total_frames,
            color_space: color_space.into(),
            color_range: color_range.into(),
            waker,
            rx,
        })
    }

}

impl Stream for FFmpegDemuxStream {
    type Item = NVCodecResult<Packet>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.waker.register(cx.waker());

        match self.rx.try_recv() {
            Ok(v) => {
                Poll::Ready(Some(v))
            }
            Err(mpsc::TryRecvError::Empty) => {
                Poll::Pending
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                Poll::Ready(None)
            }
        }
    }
}

struct BSFContext {
    bsf_ctx: *mut ffmpeg_next::ffi::AVBSFContext,
}

impl BSFContext {
    pub fn new(name: &str, params: Parameters) -> NVCodecResult<Self> {
        unsafe {
            let name = CString::new(name).unwrap();
            let to_annex_b =
                ffmpeg_next::ffi::av_bsf_get_by_name(name.as_ptr());
            if to_annex_b.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "av_bsf_get_by_name failed",
                ).into());
            }

            let mut bsf_ctx = std::ptr::null_mut();
            let res = ffmpeg_next::ffi::av_bsf_alloc(to_annex_b, &mut bsf_ctx);
            if res != 0 {
                return Err(ffmpeg_next::Error::from(res).into());
            }

            let res = ffmpeg_next::ffi::avcodec_parameters_copy(
                (*bsf_ctx).par_in, params.as_ptr()
            );
            if res != 0 {
                return Err(ffmpeg_next::Error::from(res).into());
            }

            let res = ffmpeg_next::ffi::av_bsf_init(bsf_ctx);
            if res != 0 {
                return Err(ffmpeg_next::Error::from(res).into());
            }

            Ok(Self { bsf_ctx })
        }
    }

    pub fn send_packet(&self, packet: &mut AVPacket) -> NVCodecResult<()> {
        unsafe {
            let res = ffmpeg_next::ffi::av_bsf_send_packet(
                self.bsf_ctx, packet.as_mut_ptr()
            );
            if res != 0 {
                return Err(ffmpeg_next::Error::from(res).into());
            }
        }

        Ok(())
    }

    pub fn receive_packet(&self) -> NVCodecResult<AVPacket> {
        let mut packet = AVPacket::empty();

        unsafe {
            let res = ffmpeg_next::ffi::av_bsf_receive_packet(
                self.bsf_ctx, packet.as_mut_ptr()
            );
            if res != 0 {
                return Err(ffmpeg_next::Error::from(res).into());
            }
        }

        Ok(packet)
    }
}

impl Drop for BSFContext {
    fn drop(&mut self) {
        unsafe {
            ffmpeg_next::ffi::av_bsf_free(&mut self.bsf_ctx);
        }
    }
}
