use std::process::{Child, Command, Stdio};
use std::fs;
use std::path::Path;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Clone, PartialEq, Debug)]
pub enum BrowserType {
    Chromium,
    Firefox,
}

pub struct BrowserInstance {
    pub id: usize,
    pub name: String,
    pub process: Child,
    pub profile_dir: String,
    pub is_ephemeral: bool,
    pub debug_port: u16,
    pub b_type: BrowserType,
    pub ws_url: String,
}

pub fn find_executable(b_type: &BrowserType) -> Option<String> {
    let candidates = match b_type {
        BrowserType::Chromium => vec!["chromium", "google-chrome-stable", "google-chrome", "brave", "microsoft-edge-stable"],
        BrowserType::Firefox => vec!["firefox", "firefox-developer-edition"],
    };

    for c in candidates {
        if let Ok(out) = Command::new("which").arg(c).output() {
            if out.status.success() {
                return Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }
    }
    None
}

pub fn launch(
    id: usize,
    b_type: BrowserType,
    profile_dir: &str,
    urls: Vec<&str>,
    is_ephemeral: bool,
    name_prefix: &str,
) -> Result<BrowserInstance, String> {
    let bin = find_executable(&b_type).ok_or("Navegador não encontrado")?;
    let debug_port = 9222 + id as u16;

    let _ = fs::create_dir_all(profile_dir);
    let mut cmd = Command::new(&bin);

    match b_type {
        BrowserType::Chromium => {
            cmd.arg(format!("--user-data-dir={}", profile_dir))
               .arg(format!("--remote-debugging-port={}", debug_port))
               .arg("--no-first-run")
               .arg("--no-default-browser-check")
               .arg("--disable-sync");
            
            for u in &urls { if !u.is_empty() { cmd.arg(u); } }
        }
        BrowserType::Firefox => {
            let prefs_path = Path::new(profile_dir).join("user.js");
            let prefs = "user_pref(\"remote.active-protocols\", 2);\n\
                         user_pref(\"devtools.debugger.remote-enabled\", true);\n\
                         user_pref(\"devtools.chrome.enabled\", true);\n";
            let _ = fs::write(prefs_path, prefs);

            cmd.arg("--profile")
               .arg(profile_dir)
               .arg(format!("--remote-debugging-port={}", debug_port))
               .arg("--new-instance");
            
            for u in &urls { if !u.is_empty() { cmd.arg(u); } }
        }
    }

    cmd.stdout(Stdio::null()).stderr(Stdio::piped());
    let mut process = cmd.spawn().map_err(|e| e.to_string())?;

    let stderr = process.stderr.take().ok_or("Erro ao capturar stderr")?;
    let (tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(l) = line {
                if let Some(idx) = l.find("ws://") {
                    let _ = tx.send(l[idx..].trim().to_string());
                    break;
                }
            } else {
                break;
            }
        }
    });

    let mut ws_url = rx.recv_timeout(Duration::from_secs(3)).unwrap_or_else(|_| String::new());
    ws_url = ws_url.replace("localhost", "127.0.0.1");

    Ok(BrowserInstance {
        id,
        name: format!("{} #{}", name_prefix, id),
        process,
        profile_dir: profile_dir.to_string(),
        is_ephemeral,
        debug_port,
        b_type,
        ws_url,
    })
}