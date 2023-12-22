use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn find_dir(env_key: &'static str, candidates: Vec<&'static str>) -> Option<PathBuf> {
    match env::var_os(env_key) {
        Some(val) => Some(PathBuf::from(&val)),
        _ => {
            for candidate in candidates {
                let path = PathBuf::from(candidate);
                if path.exists() {
                    return Some(path);
                }
            }

            None
        }
    }
}

fn common_builder() -> bindgen::Builder {
    bindgen::Builder::default()
        .raw_line("#![allow(dead_code)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(non_upper_case_globals)]")
}

fn write_builder(builder: bindgen::Builder, output_path: &str) {
    let s = builder
        .generate()
        .expect("Unable to generate bindings")
        .to_string()
        .replace("/**", "/*")
        .replace("/*!", "/*");

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_path)
        .unwrap();

    let _ = file.write(s.as_bytes());
}

fn main() {
    let cuda_include = find_dir(
        "CUDA_INCLUDE_PATH",
        vec!["/opt/cuda/include", "/usr/local/cuda/include"],
    ).expect("Could not find CUDA include path");

    let nvc_include = find_dir(
        "NVIDIA_VIDEO_CODEC_INCLUDE_PATH",
        vec!["/opt/nvidia-video-codec/include", "/usr/local/nvidia-video-codec/include"],
    ).expect("Could not find Nvidia Video Codec SDK include path");

    println!("cargo:rustc-link-lib=dylib={}", "cuda");
    println!("cargo:rustc-link-lib=dylib={}", "nvcuvid");

    let cuda_builder = common_builder()
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .header(cuda_include.join("cuda.h").to_string_lossy());

    write_builder(cuda_builder, "src/cuda.rs");

    let cuda_builder = common_builder()
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .clang_arg(format!("-I{}", nvc_include.to_string_lossy()))
        .header(nvc_include.join("nvcuvid.h").to_string_lossy());

    write_builder(cuda_builder, "src/nvcuvid.rs");
}
