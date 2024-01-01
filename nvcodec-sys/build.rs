use std::env;
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

fn main() {
    let cuda_include = find_dir(
        "CUDA_INCLUDE_PATH",
        vec!["/opt/cuda/include", "/usr/local/cuda/include"],
    ).expect("Could not find CUDA include path");

    let nvcodec_include = find_dir(
        "NVIDIA_VIDEO_CODEC_INCLUDE_PATH",
        vec!["/opt/nvidia-video-codec/include", "/usr/local/nvidia-video-codec/include"],
    ).expect("Could not find Nvidia Video Codec SDK include path");

    println!("cargo:rustc-link-lib=dylib=cuda");
    println!("cargo:rustc-link-lib=dylib=nvcuvid");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", nvcodec_include.to_string_lossy()))
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .header(nvcodec_include.join("nvcuvid.h").to_string_lossy())
        .blocklist_function("strtold")
        .blocklist_function("qecvt")
        .blocklist_function("qfcvt")
        .blocklist_function("qgcvt")
        .blocklist_function("qecvt_r")
        .blocklist_function("qfcvt_r")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
