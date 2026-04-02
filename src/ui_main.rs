use eframe::egui;
use std::process::{Command, Stdio};
use crate::app::DevBrowsersApp;
use crate::browser::{find_chromium, BrowserInstance};

pub fn render(app: &mut DevBrowsersApp, ctx: &egui::Context) {
    let mut instances_to_remove = Vec::new();

    for instance in &mut app.instances {
        if let Ok(Some(_)) = instance.process.try_wait() {
            instances_to_remove.push(instance.id);
        }
    }
    
    app.instances.retain(|inst| !instances_to_remove.contains(&inst.id));

    let mut frame_style = egui::Frame::window(&ctx.style());
    frame_style.inner_margin = egui::Margin::same(12.0);
    frame_style.rounding = egui::Rounding::same(8.0);

    egui::CentralPanel::default().frame(frame_style).show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("▶ Chromium").clicked() {
                if let Some(browser_bin) = find_chromium() {
                    let profile_dir = format!("/tmp/devbrowser_profile_{}", app.next_id);
                    let _ = std::fs::create_dir_all(&profile_dir);
                    let debug_port = 9222 + app.next_id as u16;

                    let child = Command::new(&browser_bin)
                        .arg(format!("--user-data-dir={}", profile_dir))
                        .arg(format!("--remote-debugging-port={}", debug_port))
                        .arg("--no-first-run")
                        .arg("--no-default-browser-check")
                        .arg("--disable-sync")
                        .arg(&app.url_input)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn();

                    if let Ok(process) = child {
                        app.instances.push(BrowserInstance {
                            id: app.next_id,
                            name: format!("Instância #{}", app.next_id),
                            process,
                            profile_dir,
                            is_ephemeral: true,
                            debug_port,
                        });
                        app.next_id += 1;
                    }
                }
            }

            ui.add(egui::TextEdit::singleline(&mut app.url_input).desired_width(200.0));

            if ui.button("⏹ Tudo").clicked() {
                for instance in &mut app.instances {
                    let _ = instance.process.kill();
                    if instance.is_ephemeral {
                        let _ = std::fs::remove_dir_all(&instance.profile_dir);
                    }
                }
                app.instances.clear();
            }

            ui.separator();
            
            if ui.button("📂 Projetos").clicked() { 
                app.show_projects = !app.show_projects; 
                if app.show_projects {
                    app.projects = crate::project::load_projects();
                }
            }
            
            if ui.button("⚙ Cofre").clicked() { 
                app.show_vault = !app.show_vault; 
            }
            
            ui.separator();

            for instance in &mut app.instances {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(&instance.name);
                        if ui.button("🔑 Injetar Credencial").clicked() {
                            println!("--- INICIANDO INJEÇÃO INSTÂNCIA {} ---", instance.id);
                            
                            let mut selected_idx = None;
                            for (i, c) in app.credentials.iter().enumerate() {
                                if c.selected {
                                    selected_idx = Some(i);
                                    break;
                                }
                            }
                            
                            if let Some(idx) = selected_idx {
                                if !app.credentials[idx].fetched {
                                    println!("-> Descriptografando senha no KeePass...");
                                    let mut cred = app.credentials[idx].clone();
                                    app.keepass.fetch_details(&mut cred);
                                    app.credentials[idx] = cred;
                                }

                                let cred = &app.credentials[idx];
                                println!("-> Credencial marcada: {}", cred.site_name);
                                println!("-> Possui senha carregada? {}", !cred.password.is_empty());

                                match crate::cdp::get_targets(instance.debug_port) {
                                    Ok(targets) => {
                                        for target in targets {
                                            if target.url.starts_with("http") {
                                                if let Some(ws_url) = &target.web_socket_debugger_url {
                                                    let safe_user = cred.username.replace("'", "\\'");
                                                    let safe_pass = cred.password.replace("'", "\\'");
                                                    
                                                    let js_code = format!(
                                                        r#"
                                                        (function() {{
                                                            let passField = document.querySelector('input[type="password"]');
                                                            if (!passField) return 'ERRO: Campo de senha não encontrado';
                                                            let form = passField.closest('form');
                                                            let userField = form ? form.querySelector('input[type="text"], input[type="email"], input:not([type="password"]):not([type="hidden"])') : document.querySelector('input[type="text"], input[type="email"]');
                                                            
                                                            if (userField) {{
                                                                userField.value = '{}';
                                                                userField.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                                                userField.dispatchEvent(new Event('change', {{ bubbles: true }}));
                                                            }}
                                                            passField.value = '{}';
                                                            passField.dispatchEvent(new Event('input', {{ bubbles: true }}));
                                                            passField.dispatchEvent(new Event('change', {{ bubbles: true }}));
                                                            return 'SUCESSO: Credencial Injetada!';
                                                        }})();
                                                        "#, safe_user, safe_pass
                                                    );
                                                    
                                                    let params = serde_json::json!({
                                                        "expression": js_code
                                                    });
                                                    let _ = crate::cdp::send_cdp_command(ws_url, "Runtime.evaluate", params);
                                                }
                                            }
                                        }
                                    },
                                    Err(e) => println!("-> ERRO HTTP: {}", e),
                                }
                            } else {
                                println!("-> Nenhuma credencial marcada na janela do cofre!");
                            }
                        }
                        if ui.button("✕").clicked() {
                            let _ = instance.process.kill();
                            if instance.is_ephemeral {
                                let _ = std::fs::remove_dir_all(&instance.profile_dir);
                            }
                            instances_to_remove.push(instance.id);
                        }
                    });
                });
            }
            
            app.instances.retain(|inst| !instances_to_remove.contains(&inst.id));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
            });
        });
    });
}