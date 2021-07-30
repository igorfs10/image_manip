use std::path::{Path, PathBuf};
use std::{env, fs};

const SETTINGS_FILE: &str = "image_manip_config.json";
fn main() {
    println!("cargo:rerun-if-changed=src/ui.fl");
    let g = fl2rust::Generator::default();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    g.in_out("src/ui.fl", out_path.join("ui.rs").to_str().unwrap())
        .expect("Failed to generate rust from fl file!");

    let target_dir_path = env::var("OUT_DIR").unwrap();
    copy(&target_dir_path, SETTINGS_FILE);
}

fn copy<S: AsRef<std::ffi::OsStr> + ?Sized, P: Copy + AsRef<Path>>(
    target_dir_path: &S,
    file_name: P,
) {
    fs::copy(
        file_name,
        Path::new(&target_dir_path).join("../../..").join(file_name),
    )
    .unwrap();
}
