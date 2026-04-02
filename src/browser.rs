use std::process::{Child, Command};

pub struct BrowserInstance {
    pub id: usize,
    pub name: String,
    pub process: Child,
    pub profile_dir: String,
    pub is_ephemeral: bool,
    pub debug_port: u16,
}

pub fn find_chromium() -> Option<String> {
    let binaries = ["chromium", "google-chrome-stable", "google-chrome", "brave", "vivaldi"];
    for bin in binaries {
        if Command::new(bin).arg("--version").output().is_ok() {
            return Some(bin.to_string());
        }
    }
    None
}