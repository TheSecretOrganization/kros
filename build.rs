use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let asm_dir = Path::new("src");

    for entry in fs::read_dir(asm_dir).expect("Failed to read src directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if let Some(ext) = path.extension()
            && ext == "asm"
        {
            let asm_path = path.to_str().expect("Invalid path");
            let obj_path = path.with_extension("o");
            let obj_str = obj_path.to_str().expect("Invalid object path");

            println!("cargo:rerun-if-changed={asm_path}");

            let status = Command::new("nasm")
                .args(["-f", "elf32", asm_path, "-o", obj_str])
                .status()
                .expect("Failed to assemble .asm file");

            if !status.success() {
                panic!("nasm failed on {asm_path}");
            }

            println!("cargo:rustc-link-arg={obj_str}");
        }
    }
}
