use eframe::egui;
use std::process::Command;
use crate::app::DevBrowsersApp;
use crate::project::{create_project, load_projects, ProjectConfig, ProjectService};

pub fn render(app: &mut DevBrowsersApp, ui: &mut egui::Ui) {
    let collapse_title = if app.editing_project_id.is_some() { "✏ Editar Projeto" } else { "➕ Criar Novo Projeto" };
    
    ui.collapsing(collapse_title, |ui| {
        ui.add(egui::TextEdit::singleline(&mut app.new_proj_id).hint_text("ID da Pasta (ex: cliente_x)"));
        ui.add(egui::TextEdit::singleline(&mut app.new_proj_nome).hint_text("Nome de Exibição"));
        
        if app.editing_project_id.is_none() {
            ui.add(egui::TextEdit::singleline(&mut app.new_proj_vault_pass)
                .password(true)
                .hint_text("Senha para criar o Cofre do Projeto (Opcional)"));
        }
        
        ui.separator();
        ui.label("Serviços (URLs):");
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut app.new_svc_nome).hint_text("Nome (ex: WP Admin)").desired_width(120.0));
            ui.add(egui::TextEdit::singleline(&mut app.new_svc_url).hint_text("URL").desired_width(300.0));

            if ui.button("➕ Adicionar").clicked() && !app.new_svc_url.is_empty() {
                app.new_proj_servicos.push(ProjectService {
                    nome: app.new_svc_nome.clone(),
                    url: app.new_svc_url.clone(),
                });
                app.new_svc_nome.clear();
                app.new_svc_url.clear();
            }
        });

        for (idx, svc) in app.new_proj_servicos.clone().iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{} - {}", svc.nome, svc.url));
                if ui.button("🗑").clicked() {
                    app.new_proj_servicos.remove(idx);
                }
            });
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button(if app.editing_project_id.is_some() { "Salvar Alterações" } else { "Salvar Projeto" }).clicked() {
                if !app.new_proj_id.is_empty() && !app.new_proj_nome.is_empty() {
                    let config = ProjectConfig {
                        id: app.new_proj_id.clone(),
                        nome: app.new_proj_nome.clone(),
                        servicos: app.new_proj_servicos.clone(),
                        tecnologias: vec!["php".to_string(), "wordpress".to_string()],
                    };

                    if let Ok(_) = create_project(&config, &app.new_proj_vault_pass) {
                        app.new_proj_id.clear();
                        app.new_proj_nome.clear();
                        app.new_proj_vault_pass.clear();
                        app.new_proj_servicos.clear();
                        app.editing_project_id = None;
                        app.projects = load_projects();
                    }
                }
            }

            if app.editing_project_id.is_some() {
                if ui.button("Cancelar").clicked() {
                    app.new_proj_id.clear();
                    app.new_proj_nome.clear();
                    app.new_proj_vault_pass.clear();
                    app.new_proj_servicos.clear();
                    app.editing_project_id = None;
                }
            }
        });
    });

    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for proj in &app.projects {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(&proj.nome);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📦 Exportar").clicked() {
                            let mut proj_dir = crate::project::get_projects_dir();
                            proj_dir.push(&proj.id);
                            if let Some(download_dir) = dirs::download_dir() {
                                let export_path = format!("{}/{}.tar.gz", download_dir.display(), proj.id);
                                let _ = Command::new("tar")
                                    .arg("-czvf")
                                    .arg(&export_path)
                                    .arg("-C")
                                    .arg(proj_dir.parent().unwrap())
                                    .arg(&proj.id)
                                    .status();
                            }
                        }
                        if ui.button("✏ Editar").clicked() {
                            app.editing_project_id = Some(proj.id.clone());
                            app.new_proj_id = proj.id.clone();
                            app.new_proj_nome = proj.nome.clone();
                            app.new_proj_servicos = proj.servicos.clone();
                        }

                        if ui.button("▶ Abrir Tudo").clicked() {
                            let mut profile_dir = dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
                            profile_dir.push("devbrowsers");
                            profile_dir.push("profiles");
                            profile_dir.push(&proj.id);
                            let profile_str = profile_dir.to_string_lossy().to_string();
                            
                            let urls: Vec<&str> = proj.servicos.iter().map(|s| s.url.as_str()).collect();

                            if let Ok(inst) = crate::browser::launch(app.next_id, app.default_browser.clone(), &profile_str, urls, false, &proj.nome) {
                                app.instances.push(inst);
                                app.next_id += 1;
                            }

                            if !app.quick_vault_pass.is_empty() {
                                let mut vault_path = crate::project::get_projects_dir();
                                vault_path.push(&proj.id);
                                vault_path.push("vault.kdbx");
                                
                                app.keepass.db_path = vault_path.to_string_lossy().to_string();
                                app.keepass.master_pass = app.quick_vault_pass.clone();
                                
                                if app.keepass.test_unlock() {
                                    let mut creds = app.keepass.list_all_entries();
                                    for c in &mut creds { app.keepass.fetch_details(c); }
                                    app.credentials = creds;
                                }
                                app.quick_vault_pass.clear();
                            }
                        }
                        
                        ui.add(egui::TextEdit::singleline(&mut app.quick_vault_pass)
                            .password(true)
                            .hint_text("Senha do Cofre")
                            .desired_width(120.0));
                        
                        if ui.button("🔑 Abrir Cofre").clicked() {
                            let mut vault_path = crate::project::get_projects_dir();
                            vault_path.push(&proj.id);
                            vault_path.push("vault.kdbx");
                            
                            app.keepass.db_path = vault_path.to_string_lossy().to_string();
                            app.keepass.is_unlocked = false;
                            app.current_tab = crate::app::AppTab::Vault;
                        }
                    });
                });
                
                ui.add_space(5.0);
                for svc in &proj.servicos {
                    ui.horizontal(|ui| {
                        ui.label(format!("🔗 {}", svc.nome));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Limpo").clicked() {
                                let profile_dir = format!("/tmp/devbrowser_profile_{}", app.next_id);
                                if let Ok(inst) = crate::browser::launch(app.next_id, app.default_browser.clone(), &profile_dir, vec![&svc.url], true, &svc.nome) {
                                    app.instances.push(inst);
                                    app.next_id += 1;
                                }
                            }
                            if ui.button("Persistente").clicked() {
                                let mut profile_dir = dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
                                profile_dir.push("devbrowsers");
                                profile_dir.push("profiles");
                                profile_dir.push(&proj.id);
                                let profile_str = profile_dir.to_string_lossy().to_string();
                                
                                if let Ok(inst) = crate::browser::launch(app.next_id, app.default_browser.clone(), &profile_str, vec![&svc.url], false, &svc.nome) {
                                    app.instances.push(inst);
                                    app.next_id += 1;
                                }
                            }
                        });
                    });
                }
            });
            ui.add_space(5.0);
        }
    });
}