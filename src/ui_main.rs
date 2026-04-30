use eframe::egui;
use std::fs;
use crate::app::DevBrowsersApp;

pub fn render(app: &mut DevBrowsersApp, ui: &mut egui::Ui) {
    let mut instances_to_remove = Vec::new();

    for instance in &mut app.instances {
        if let Ok(Some(_)) = instance.process.try_wait() {
            instances_to_remove.push(instance.id);
        }
    }
    
    app.instances.retain(|inst| !instances_to_remove.contains(&inst.id));

    ui.horizontal(|ui| {
        if ui.button("▶ Abrir Limpo").clicked() {
            let profile_dir = format!("/tmp/devbrowser_profile_{}", app.next_id);
            let b_name = match app.default_browser { crate::browser::BrowserType::Chromium => "Chromium", crate::browser::BrowserType::Firefox => "Firefox" };
            
            if let Ok(inst) = crate::browser::launch(app.next_id, app.default_browser.clone(), &profile_dir, vec![&app.url_input], true, b_name) {
                app.instances.push(inst);
                app.next_id += 1;
            }
        }

        ui.add(egui::TextEdit::singleline(&mut app.url_input).desired_width(300.0).hint_text("URL inicial..."));

        if ui.button("⏹ Fechar Todas").clicked() {
            for instance in &mut app.instances {
                let _ = instance.process.kill();
                if instance.is_ephemeral {
                    let _ = fs::remove_dir_all(&instance.profile_dir);
                }
            }
            app.instances.clear();
        }
    });

    ui.separator();
    ui.heading("Navegadores em Execução");
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for instance in &mut app.instances {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let icon = match instance.b_type {
                        crate::browser::BrowserType::Chromium => "🌐",
                        crate::browser::BrowserType::Firefox => "🦊",
                    };
                    
                    ui.label(egui::RichText::new(format!("{} {}", icon, instance.name)).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕ Fechar").clicked() {
                            let _ = instance.process.kill();
                            if instance.is_ephemeral {
                                let _ = fs::remove_dir_all(&instance.profile_dir);
                            }
                            instances_to_remove.push(instance.id);
                        }

                        if ui.button("🔑 Injetar Credencial").clicked() {
                            println!("--- INICIANDO INJEÇÃO {} INSTÂNCIA {} ---", icon, instance.id);
                            
                            for cred in &mut app.credentials {
                                if cred.selected && !cred.fetched {
                                    app.keepass.fetch_details(cred);
                                }
                            }

                            if instance.b_type == crate::browser::BrowserType::Chromium {
                                if instance.ws_url.is_empty() { println!("-> ERRO: URL WebSocket não capturada. Feche e abra o navegador novamente."); }
                                match crate::cdp::get_targets(&instance.ws_url, instance.debug_port) {
                                    Ok(targets) => {
                                        for target in targets {
                                            if target.url.starts_with("http") {
                                                let mut best_cred = None;
                                                let clean_target = target.url.replace("https://", "").replace("http://", "").replace("www.", "");
                                                let target_domain = clean_target.split('/').next().unwrap_or("").split('?').next().unwrap_or("");

                                                for cred in &app.credentials {
                                                    if cred.selected && cred.fetched && !cred.url.is_empty() {
                                                        let clean_cred = cred.url.replace("https://", "").replace("http://", "").replace("www.", "");
                                                        let cred_domain = clean_cred.split('/').next().unwrap_or("").split('?').next().unwrap_or("");
                                                        if !target_domain.is_empty() && target_domain == cred_domain {
                                                            best_cred = Some(cred.clone());
                                                            break;
                                                        }
                                                    }
                                                }

                                                if best_cred.is_none() {
                                                    let selected_creds: Vec<_> = app.credentials.iter().filter(|c| c.selected && c.fetched).collect();
                                                    if selected_creds.len() == 1 { best_cred = Some(selected_creds[0].clone()); }
                                                }

                                                if let Some(cred) = best_cred {
                                                    if let Some(ws_url) = &target.web_socket_debugger_url {
                                                        println!("-> Injetando na aba {}", target.url);
                                                        let safe_user = cred.username.replace("'", "\\'");
                                                        let safe_pass = cred.password.replace("'", "\\'");
                                                        
                                                        let js_code = format!(
                                                            r#"(function() {{ let passField = document.querySelector('input[type="password"]'); if (!passField) return 'ERRO'; let form = passField.closest('form'); let userField = form ? form.querySelector('input[type="text"], input[type="email"], input:not([type="password"]):not([type="hidden"])') : document.querySelector('input[type="text"], input[type="email"]'); if (userField) {{ userField.value = '{}'; userField.dispatchEvent(new Event('input', {{ bubbles: true }})); userField.dispatchEvent(new Event('change', {{ bubbles: true }})); }} passField.value = '{}'; passField.dispatchEvent(new Event('input', {{ bubbles: true }})); passField.dispatchEvent(new Event('change', {{ bubbles: true }})); return 'SUCESSO'; }})();"#, safe_user, safe_pass
                                                        );
                                                        
                                                        let params = serde_json::json!({ "expression": js_code });
                                                        match crate::cdp::send_cdp_command(ws_url, "Runtime.evaluate", params) {
                                                            Ok(res) => println!("-> Resposta CDP: {}", res),
                                                            Err(e) => println!("-> ERRO no envio CDP: {}", e),
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    Err(e) => println!("-> ERRO CDP: {}", e),
                                }
                            } else if instance.b_type == crate::browser::BrowserType::Firefox {
                                match crate::bidi::get_targets(instance.debug_port) {
                                    Ok(targets) => {
                                        for target in targets {
                                            if target.url.starts_with("http") {
                                                let mut best_cred = None;
                                                let clean_target = target.url.replace("https://", "").replace("http://", "").replace("www.", "");
                                                let target_domain = clean_target.split('/').next().unwrap_or("").split('?').next().unwrap_or("");

                                                for cred in &app.credentials {
                                                    if cred.selected && cred.fetched && !cred.url.is_empty() {
                                                        let clean_cred = cred.url.replace("https://", "").replace("http://", "").replace("www.", "");
                                                        let cred_domain = clean_cred.split('/').next().unwrap_or("").split('?').next().unwrap_or("");
                                                        if !target_domain.is_empty() && target_domain == cred_domain {
                                                            best_cred = Some(cred.clone());
                                                            break;
                                                        }
                                                    }
                                                }

                                                if best_cred.is_none() {
                                                    let selected_creds: Vec<_> = app.credentials.iter().filter(|c| c.selected && c.fetched).collect();
                                                    if selected_creds.len() == 1 { best_cred = Some(selected_creds[0].clone()); }
                                                }

                                                if let Some(cred) = best_cred {
                                                    println!("-> Injetando na aba {}", target.url);
                                                    let safe_user = cred.username.replace("'", "\\'");
                                                    let safe_pass = cred.password.replace("'", "\\'");
                                                    
                                                    let js_code = format!(
                                                        r#"(function() {{ let passField = document.querySelector('input[type="password"]'); if (!passField) return 'ERRO'; let form = passField.closest('form'); let userField = form ? form.querySelector('input[type="text"], input[type="email"], input:not([type="password"]):not([type="hidden"])') : document.querySelector('input[type="text"], input[type="email"]'); if (userField) {{ userField.value = '{}'; userField.dispatchEvent(new Event('input', {{ bubbles: true }})); userField.dispatchEvent(new Event('change', {{ bubbles: true }})); }} passField.value = '{}'; passField.dispatchEvent(new Event('input', {{ bubbles: true }})); passField.dispatchEvent(new Event('change', {{ bubbles: true }})); return 'SUCESSO'; }})();"#, safe_user, safe_pass
                                                    );
                                                    
                                                    match crate::bidi::inject_js(instance.debug_port, &target.id, &js_code) {
                                                        Ok(res) => println!("-> Resposta BiDi: {}", res),
                                                        Err(e) => println!("-> ERRO no envio BiDi: {}", e),
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    Err(e) => println!("-> ERRO BiDi: {}", e),
                                }
                            }
                        }
                    });
                });
            });
            ui.add_space(4.0);
        }
    });
}