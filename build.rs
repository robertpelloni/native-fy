use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=VERSION.md");
    println!("cargo:rerun-if-changed=CHANGELOG.md");
    println!("cargo:rerun-if-changed=TODO.md");
    println!("cargo:rerun-if-changed=ROADMAP.md");

    let status = Command::new("node")
        .arg("scripts/protocol_sync.js")
        .status()
        .expect("Failed to execute protocol_sync.js");

    if !status.success() {
        panic!("protocol_sync.js failed with status: {}", status);
    }
}
