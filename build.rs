use std::env;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=tasks.json");
    let input_path: PathBuf = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("tasks.json");
    let output_path = {
        let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
        let build_type = env::var("PROFILE").unwrap();
        let path = Path::new(&manifest_dir_string).join("target").join(build_type);
        Path::new(&PathBuf::from(path)).join("tasks.json")
    };
    let _ = std::fs::copy(input_path, output_path);
}