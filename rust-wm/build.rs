use std::process::Command;

fn main() {
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xinerama");
    Command::new("sudo")
        .arg("cp")
        .arg("-f")
        .arg("./target/release/rust-wm")
        .arg("/usr/local/bin");
}
