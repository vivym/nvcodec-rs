use futures::stream::Stream;
use futures::task::AtomicWaker;
use std::{
    io,
    path::Path,
    pin::Pin,
    sync::{Arc, mpsc::{self, Receiver}},
    task::{Context, Poll},
    thread,
};
use ffmpeg_next::codec::context::Context as CodecContext;
use ffmpeg_next::util::frame::Video as VideoFrame;

pub struct FFmpegDemuxStream{
    waker: Arc<AtomicWaker>,
    rx: Receiver<io::Result<VideoFrame>>,
}

impl FFmpegDemuxStream {
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        let waker = Arc::new(AtomicWaker::new());
        let (tx, rx) =
            mpsc::sync_channel::<io::Result<VideoFrame>>(8);

        let waker_in_thread = waker.clone();
        let path = (*path.as_ref()).to_owned();
        thread::spawn(move || {
            let mut ctx = match ffmpeg_next::format::input(&path) {
                Ok(ctx) => ctx,
                Err(e) => {
                    if let Err(_) = tx.send(Err(e.into())) {
                        return;
                    }
                    waker_in_thread.wake();
                    return;
                }
            };

            let stream = match ctx
                .streams()
                .best(ffmpeg_next::media::Type::Video)
                .ok_or(ffmpeg_next::Error::StreamNotFound)
            {
                Ok(stream) => stream,
                Err(e) => {
                    if let Err(_) = tx.send(Err(e.into())) {
                        return;
                    }
                    waker_in_thread.wake();
                    return;
                }
            };
            let video_stream_index = stream.index();

            let decoder_ctx = match CodecContext::from_parameters(stream.parameters()) {
                Ok(decoder_ctx) => decoder_ctx,
                Err(e) => {
                    if let Err(_) = tx.send(Err(e.into())) {
                        return;
                    }
                    waker_in_thread.wake();
                    return;
                }
            };

            let mut decoder = match decoder_ctx.decoder().video() {
                Ok(decoder) => decoder,
                Err(e) => {
                    if let Err(_) = tx.send(Err(e.into())) {
                        return;
                    }
                    waker_in_thread.wake();
                    return;
                }
            };

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

        FFmpegDemuxStream{ waker, rx }
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
