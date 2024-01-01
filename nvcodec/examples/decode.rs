use clap::Parser;
use cuda_rs::{stream::CuStream, context::CuContext, device::CuDevice};
use futures::StreamExt;
use nvcodec::{
    demuxer::ffmpeg::FFmpegDemuxStream,
    decoder::NVDecoder,
};
use indicatif::ProgressBar;
use std::{path::Path, io::Write as _};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_video: String,

    #[arg(short, long)]
    output_dir: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let Args {
        input_video,
        output_dir,
    } = args;

    let output_dir = Path::new(&output_dir);

    // let bar = ProgressBar::new();

    cuda_rs::init().unwrap();

    let device = CuDevice::new(0).unwrap();
    let ctx = CuContext::retain_primary_context(&device).unwrap();
    let _guard = ctx.guard().unwrap();

    let mut demuxer = FFmpegDemuxStream::new(&input_video).unwrap();
    println!("codec_id: {:?}", demuxer.codec_id);
    println!("total_frames: {:?}", demuxer.total_frames);

    let bar = ProgressBar::new(demuxer.total_frames as u64);

    let stream = CuStream::new().unwrap();

    let mut decoder = NVDecoder::new(
        &stream,
        demuxer.codec_id.into(),
        None,
        None,
        false,
    ).unwrap();

    let mut i = 0;
    loop {
        tokio::select! {
            res = demuxer.next() => {
                match res {
                    Some(res) => {
                        match res {
                            Ok(packet) => {
                                decoder.decode(Some(&packet)).unwrap();
                            },
                            Err(e) => {
                                eprintln!("demux error: {:?}", e);
                                break;
                            }
                        }
                    }
                    None => {
                        decoder.decode(None).unwrap();
                        break;
                    }
                }
            }
            Some(res) = decoder.next() => {
                match res {
                    Ok(frame) => {
                        let host_mem = frame.buf.to_host().unwrap();
                        stream.synchronize().unwrap();

                        let slice = host_mem.as_slice();

                        std::fs::OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(output_dir.join(format!("frame_{}.yuv", i)))
                            .unwrap()
                            .write_all(slice)
                            .unwrap();

                        bar.inc(1);
                        i += 1;
                    },
                    Err(e) => {
                        eprintln!("decode error: {:?}", e);
                        break;
                    }
                }
            }
            else => break,
        }
    }

    while let Some(res) = decoder.next().await {
        match res {
            Ok(frame) => {
                let host_mem = frame.buf.to_host().unwrap();
                stream.synchronize().unwrap();

                let slice = host_mem.as_slice();

                std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(output_dir.join(format!("frame_{}.yuv", i)))
                    .unwrap()
                    .write_all(slice)
                    .unwrap();

                bar.inc(1);
                i += 1;
            },
            Err(e) => {
                eprintln!("decode error: {:?}", e);
                break;
            }
        }
    }

    bar.finish();

    println!("done");
}
