use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Clone)]
pub struct Credential {
    pub entry_path: String,
    pub site_name: String,
    pub username: String,
    pub password: String,
    pub selected: bool,
}

pub struct KeePassIntegration {
    pub db_path: String,
    pub master_pass: String,
    pub is_unlocked: bool,
    pub error_msg: Option<String>,
}

impl Default for KeePassIntegration {
    fn default() -> Self {
        Self {
            db_path: String::from("/home/luw/senhas.kdbx"),
            master_pass: String::new(),
            is_unlocked: false,
            error_msg: None,
        }
    }
}

impl KeePassIntegration {
    pub fn run_cli(&self, args: &[&str]) -> Result<String, String> {
        let mut child = Command::new("keepassxc-cli")
            .args(args)
            .arg(&self.db_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Erro ao executar CLI: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(self.master_pass.as_bytes());
            let _ = stdin.write_all(b"\n");
        }

        let output = child.wait_with_output().map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    pub fn test_unlock(&mut self) -> bool {
        match self.run_cli(&["ls", "-q"]) {
            Ok(_) => {
                self.is_unlocked = true;
                self.error_msg = None;
                true
            }
            Err(e) => {
                self.is_unlocked = false;
                self.error_msg = Some(e);
                false
            }
        }
    }

    pub fn list_all_entries(&self) -> Vec<Credential> {
        let mut entries = Vec::new();
        if let Ok(output) = self.run_cli(&["ls", "-R"]) {
            for line in output.lines() {
                let clean_line = line.trim();
                if !clean_line.is_empty() && !clean_line.ends_with('/') {
                    entries.push(Credential {
                        entry_path: clean_line.to_string(),
                        site_name: clean_line.split('/').last().unwrap_or(clean_line).to_string(),
                        username: String::new(),
                        password: String::new(),
                        selected: false,
                    });
                }
            }
        }
        entries
    }

    pub fn fetch_details(&self, entry: &mut Credential) {
        if let Ok(out) = self.run_cli(&["show", "-a", "UserName", &entry.entry_path]) {
            entry.username = out;
        }
        if let Ok(out) = self.run_cli(&["show", "-a", "Password", &entry.entry_path]) {
            entry.password = out;
        }
    }

    pub fn add_entry(&self, title: &str, user: &str, pass: &str) -> Result<String, String> {
        let mut child = Command::new("keepassxc-cli")
            .arg("add")
            .arg("-p")
            .arg(&self.db_path)
            .arg(title)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(self.master_pass.as_bytes());
            let _ = stdin.write_all(b"\n");
            let _ = stdin.write_all(user.as_bytes());
            let _ = stdin.write_all(b"\n");
            let _ = stdin.write_all(pass.as_bytes());
            let _ = stdin.write_all(b"\n");
            let _ = stdin.write_all(pass.as_bytes());
            let _ = stdin.write_all(b"\n");
            let _ = stdin.write_all(b"\n");
            let _ = stdin.write_all(b"\n");
        }

        let output = child.wait_with_output().map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok("Sucesso".into())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }
}