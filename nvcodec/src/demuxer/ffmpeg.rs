use futures::{
    stream::Stream,
    task::AtomicWaker,
};
use std::{
    io,
    path::Path,
    pin::Pin,
    sync::{Arc, mpsc::{self, Receiver}},
    task::{Context, Poll},
    thread,
};
use ffmpeg_next::{
    codec::{
        Id as CodecId,
        context::Context as CodecContext,
    },
    util::{
        frame::Video as VideoFrame,
        format::Pixel as PixelFormat,
    },
};

pub struct FFmpegDemuxStream{
    pub codec_id: CodecId,
    pub pixel_format: PixelFormat,
    waker: Arc<AtomicWaker>,
    rx: Receiver<io::Result<VideoFrame>>,
}

impl FFmpegDemuxStream {
    pub fn new<P: AsRef<Path>>(path: &P) -> io::Result<Self> {
        let waker = Arc::new(AtomicWaker::new());
        let (tx, rx) =
            mpsc::sync_channel::<io::Result<VideoFrame>>(8);

        let mut ctx = ffmpeg_next::format::input(path)?;

        let stream = ctx
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;
        let video_stream_index = stream.index();

        let codec_ctx = CodecContext::from_parameters(stream.parameters())?;
        let codec_id = codec_ctx.id();
        let mut decoder = codec_ctx.decoder().video()?;
        let pixel_format = decoder.format();

        let waker_in_thread: Arc<AtomicWaker> = waker.clone();
        thread::spawn(move || {
            for (stream, packet) in ctx.packets() {
                if stream.index() == video_stream_index {
                    match decoder.send_packet(&packet) {
                        Ok(()) => {
                            let mut decoded_frame = VideoFrame::empty();
                            while decoder.receive_frame(&mut decoded_frame).is_ok() {
                                if let Err(_) = tx.send(Ok(decoded_frame.clone())) {
                                    return;
                                }
                                waker_in_thread.wake();
                            }
                        },
                        Err(e) => {
                            if let Err(_) = tx.send(Err(e.into())) {
                                return;
                            }
                            waker_in_thread.wake();
                            return;
                        }
                    }
                }
            }

            std::mem::drop(tx);
            waker_in_thread.wake();
        });

        Ok(Self{ codec_id, pixel_format, waker, rx })
    }

}

impl Stream for FFmpegDemuxStream {
    type Item = io::Result<VideoFrame>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.waker.register(cx.waker());

        match self.rx.try_recv() {
            Ok(v) => {
                return Poll::Ready(Some(v));
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
