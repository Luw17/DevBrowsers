use eframe::egui;
use crate::app::DevBrowsersApp;

pub fn render(app: &mut DevBrowsersApp, ui: &mut egui::Ui) {
    if !app.keepass.is_unlocked {
        ui.heading("🔓 Destrancar Cofre");
        ui.add_space(8.0);
        
        ui.horizontal(|ui| {
            ui.label("Caminho do .kdbx:");
            ui.add(egui::TextEdit::singleline(&mut app.keepass.db_path).desired_width(300.0));
        });
        
        ui.horizontal(|ui| {
            ui.label("Senha Mestre:");
            ui.add(egui::TextEdit::singleline(&mut app.keepass.master_pass).password(true).desired_width(200.0));
            if ui.button("Entrar").clicked() { app.keepass.test_unlock(); }
        });
    } else {
        ui.horizontal(|ui| {
            ui.heading("🔒 Cofre Aberto");
            if ui.button("🔄 Recarregar").clicked() { app.credentials = app.keepass.list_all_entries(); }
            if ui.button("🔒 Trancar").clicked() { app.keepass.is_unlocked = false; }
        });

        ui.add_space(8.0);

        ui.collapsing("➕ Nova Entrada de Senha", |ui| {
            ui.horizontal(|ui| {
                ui.label("Título:");
                ui.add(egui::TextEdit::singleline(&mut app.new_title).desired_width(150.0));
            });
            ui.horizontal(|ui| {
                ui.label("Usuário:");
                ui.add(egui::TextEdit::singleline(&mut app.new_user).desired_width(150.0));
            });
            ui.horizontal(|ui| {
                ui.label("Senha:");
                ui.add(egui::TextEdit::singleline(&mut app.new_pass).password(true).desired_width(150.0));
            });
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut app.new_url).desired_width(250.0));
            });
            
            if ui.button("Salvar no KeePass").clicked() {
                if let Ok(_) = app.keepass.add_entry(&app.new_title, &app.new_user, &app.new_pass, &app.new_url) {
                    app.new_title.clear(); app.new_user.clear(); app.new_pass.clear(); app.new_url.clear();
                    app.credentials = app.keepass.list_all_entries();
                }
            }
        });

        ui.separator();
        
        let mut entry_to_remove = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for cred in &mut app.credentials {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut cred.selected, "");
                    ui.label(egui::RichText::new(&cred.site_name).strong());
                    if cred.fetched {
                        ui.label(egui::RichText::new(" (Pronto para Injetar)").color(egui::Color32::GREEN));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").clicked() {
                            entry_to_remove = Some(cred.entry_path.clone());
                        }
                    });
                });
            }
        });

        if let Some(path) = entry_to_remove {
            let _ = app.keepass.rm_entry(&path);
            app.credentials = app.keepass.list_all_entries();
        }
    }
}