use eframe::egui;
use std::process::{Command, Stdio};
use crate::browser::{find_chromium, BrowserInstance};
use crate::vault::{Credential, KeePassIntegration};

pub struct DevBrowsersApp {
    pub instances: Vec<BrowserInstance>,
    pub url_input: String,
    pub next_id: usize,
    pub show_vault: bool,
    pub credentials: Vec<Credential>,
    pub keepass: KeePassIntegration,
    pub new_title: String,
    pub new_user: String,
    pub new_pass: String,
}

impl Default for DevBrowsersApp {
    fn default() -> Self {
        Self {
            instances: Vec::new(),
            url_input: String::from("http://localhost:3000"),
            next_id: 1,
            show_vault: false,
            credentials: Vec::new(),
            keepass: KeePassIntegration::default(),
            new_title: String::new(),
            new_user: String::new(),
            new_pass: String::new(),
        }
    }
}

impl eframe::App for DevBrowsersApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut instances_to_remove = Vec::new();

        for instance in &mut self.instances {
            if let Ok(Some(_)) = instance.process.try_wait() {
                instances_to_remove.push(instance.id);
            }
        }
        
        self.instances.retain(|inst| !instances_to_remove.contains(&inst.id));

        let mut frame_style = egui::Frame::window(&ctx.style());
        frame_style.inner_margin = egui::Margin::same(12.0);
        frame_style.rounding = egui::Rounding::same(8.0);

        egui::CentralPanel::default().frame(frame_style).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("▶ Chromium").clicked() {
                    if let Some(browser_bin) = find_chromium() {
                        let profile_dir = format!("/tmp/devbrowser_profile_{}", self.next_id);
                        let _ = std::fs::create_dir_all(&profile_dir);

                        let child = Command::new(&browser_bin)
                            .arg(format!("--user-data-dir={}", profile_dir))
                            .arg("--no-first-run")
                            .arg("--no-default-browser-check")
                            .arg("--disable-sync")
                            .arg(&self.url_input)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn();

                        if let Ok(process) = child {
                            self.instances.push(BrowserInstance {
                                id: self.next_id,
                                name: format!("Instância #{}", self.next_id),
                                process,
                                profile_dir,
                            });
                            self.next_id += 1;
                        }
                    }
                }

                ui.add(egui::TextEdit::singleline(&mut self.url_input).desired_width(200.0));

                if ui.button("⏹ Tudo").clicked() {
                    for instance in &mut self.instances {
                        let _ = instance.process.kill();
                        let _ = std::fs::remove_dir_all(&instance.profile_dir);
                    }
                    self.instances.clear();
                }

                ui.separator();
                if ui.button("⚙ Cofre").clicked() { self.show_vault = !self.show_vault; }
                ui.separator();

                for instance in &mut self.instances {
                    ui.group(|ui| {
                        ui.label(&instance.name);
                        if ui.button("✕").clicked() {
                            let _ = instance.process.kill();
                            let _ = std::fs::remove_dir_all(&instance.profile_dir);
                            instances_to_remove.push(instance.id);
                        }
                    });
                }
                
                self.instances.retain(|inst| !instances_to_remove.contains(&inst.id));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });
            });
        });

        if self.show_vault {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("vault_viewport"),
                egui::ViewportBuilder::default().with_title("Cofre").with_inner_size([350.0, 500.0]),
                |ctx, _| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if !self.keepass.is_unlocked {
                            ui.heading("🔓 Destrancar");
                            ui.text_edit_singleline(&mut self.keepass.db_path);
                            ui.add(egui::TextEdit::singleline(&mut self.keepass.master_pass).password(true));
                            if ui.button("Entrar").clicked() { self.keepass.test_unlock(); }
                        } else {
                            ui.horizontal(|ui| {
                                ui.heading("🔒 Aberto");
                                if ui.button("🔄").clicked() { self.credentials = self.keepass.list_all_entries(); }
                                if ui.button("🔒").clicked() { self.keepass.is_unlocked = false; }
                            });

                            ui.collapsing("➕ Nova Entrada", |ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.new_title).hint_text("Título"));
                                ui.add(egui::TextEdit::singleline(&mut self.new_user).hint_text("Usuário"));
                                ui.add(egui::TextEdit::singleline(&mut self.new_pass).password(true).hint_text("Senha"));
                                
                                if ui.button("Salvar").clicked() {
                                    match self.keepass.add_entry(&self.new_title, &self.new_user, &self.new_pass) {
                                        Ok(_) => {
                                            self.new_title.clear();
                                            self.new_user.clear();
                                            self.new_pass.clear();
                                            self.credentials = self.keepass.list_all_entries();
                                        },
                                        Err(e) => {
                                            self.keepass.error_msg = Some(e);
                                        }
                                    }
                                }
                            });

                            ui.separator();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for cred in &mut self.credentials {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut cred.selected, "");
                                        ui.label(&cred.site_name);
                                        if cred.selected && cred.username.is_empty() {
                                            self.keepass.fetch_details(cred);
                                        }
                                    });
                                }
                            });
                        }
                    });
                    if ctx.input(|i| i.viewport().close_requested()) { self.show_vault = false; }
                },
            );
        }
    }
}