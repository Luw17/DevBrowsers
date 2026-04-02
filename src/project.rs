use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectService {
    pub nome: String,
    pub url: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub id: String,
    pub nome: String,
    pub servicos: Vec<ProjectService>,
    pub tecnologias: Vec<String>,
}

pub fn get_projects_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("devbrowsers");
    path.push("projetos");
    fs::create_dir_all(&path).unwrap_or_default();
    path
}

pub fn load_projects() -> Vec<ProjectConfig> {
    let mut projects = Vec::new();
    let dir = get_projects_dir();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let mut config_path = entry.path();
                config_path.push("config.json");

                if let Ok(content) = fs::read_to_string(config_path) {
                    if let Ok(config) = serde_json::from_str::<ProjectConfig>(&content) {
                        projects.push(config);
                    }
                }
            }
        }
    }
    projects
}

pub fn create_project(config: &ProjectConfig, vault_pass: &str) -> Result<(), String> {
    let mut dir = get_projects_dir();
    dir.push(&config.id);

    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut config_path = dir.clone();
    config_path.push("config.json");

    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(config_path, json).map_err(|e| e.to_string())?;

    let mut notes_path = dir.clone();
    notes_path.push("notas.md");
    if !notes_path.exists() {
        let _ = fs::write(
            notes_path,
            format!("# {}\n\nAnotações do projeto...", config.nome),
        );
    }

    if !vault_pass.is_empty() {
        let mut vault_path = dir.clone();
        vault_path.push("vault.kdbx");
        if !vault_path.exists() {
            let _ = crate::vault::KeePassIntegration::create_db(vault_path.to_str().unwrap(), vault_pass);
        }
    }

    Ok(())
}