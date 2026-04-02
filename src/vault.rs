use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Clone)]
pub struct Credential {
    pub entry_path: String,
    pub site_name: String,
    pub username: String,
    pub password: String,
    pub selected: bool,
    pub fetched: bool,
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
            db_path: String::new(),
            master_pass: String::new(),
            is_unlocked: false,
            error_msg: None,
        }
    }
}

impl KeePassIntegration {
    pub fn create_db(path: &str, master_pass: &str) -> Result<String, String> {
        let mut child = Command::new("keepassxc-cli")
            .arg("db-create")
            .arg("-p")
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(master_pass.as_bytes());
            let _ = stdin.write_all(b"\n");
            let _ = stdin.write_all(master_pass.as_bytes());
            let _ = stdin.write_all(b"\n");
        }

        let output = child.wait_with_output().map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok("Cofre criado".into())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    pub fn run_cli(&self, args: &[&str]) -> Result<String, String> {
        let mut child = Command::new("keepassxc-cli")
            .args(args)
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
        if self.db_path.is_empty() { return false; }
        match self.run_cli(&["ls", "-q", &self.db_path]) {
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
        if let Ok(output) = self.run_cli(&["ls", "-R", &self.db_path]) {
            for line in output.lines() {
                let clean_line = line.trim();
                if !clean_line.is_empty() && !clean_line.ends_with('/') {
                    entries.push(Credential {
                        entry_path: clean_line.to_string(),
                        site_name: clean_line.split('/').last().unwrap_or(clean_line).to_string(),
                        username: String::new(),
                        password: String::new(),
                        selected: false,
                        fetched: false,
                    });
                }
            }
        }
        entries
    }

    pub fn fetch_details(&self, entry: &mut Credential) {
        if let Ok(out) = self.run_cli(&["show", "-q", "-s", "-a", "UserName", &self.db_path, &entry.entry_path]) {
            entry.username = out;
        } else {
            println!("-> ERRO: Falha ao pegar usuário");
        }
        
        if let Ok(out) = self.run_cli(&["show", "-q", "-s", "-a", "Password", &self.db_path, &entry.entry_path]) {
            entry.password = out;
        } else {
            println!("-> ERRO: Falha ao pegar senha");
        }
        entry.fetched = true;
    }

    pub fn add_entry(&self, title: &str, user: &str, pass: &str, url: &str) -> Result<String, String> {
        let mut child = Command::new("keepassxc-cli")
            .arg("add")
            .arg("-p")
            .arg("-u").arg(user)
            .arg("--url").arg(url)
            .arg(&self.db_path)
            .arg(title)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        if let Some(mut stdin) = child.stdin.take() {
            // 1. Senha mestre do banco
            let _ = stdin.write_all(self.master_pass.as_bytes());
            let _ = stdin.write_all(b"\n");
            // 2. Senha da nova entrada
            let _ = stdin.write_all(pass.as_bytes());
            let _ = stdin.write_all(b"\n");
            // 3. Repetir a senha da nova entrada
            let _ = stdin.write_all(pass.as_bytes());
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