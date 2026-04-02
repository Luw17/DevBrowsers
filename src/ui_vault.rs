use eframe::egui;
use crate::app::DevBrowsersApp;

pub fn render(app: &mut DevBrowsersApp, ctx: &egui::Context) {
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("vault_viewport"),
        egui::ViewportBuilder::default().with_title("Cofre").with_inner_size([350.0, 500.0]),
        |ctx, _| {
            crate::clipboard::handle_global_events(ctx);

            egui::CentralPanel::default().show(ctx, |ui| {
                if !app.keepass.is_unlocked {
                    ui.heading("🔓 Destrancar");
                    ui.add(egui::TextEdit::singleline(&mut app.keepass.db_path).hint_text("Caminho do arquivo .kdbx"));
                    ui.add(egui::TextEdit::singleline(&mut app.keepass.master_pass).password(true).hint_text("Senha Mestre"));
                    if ui.button("Entrar").clicked() { app.keepass.test_unlock(); }
                } else {
                    ui.horizontal(|ui| {
                        ui.heading("🔒 Aberto");
                        if ui.button("🔄").clicked() { app.credentials = app.keepass.list_all_entries(); }
                        if ui.button("🔒").clicked() { app.keepass.is_unlocked = false; }
                    });

                    ui.collapsing("➕ Nova Entrada", |ui| {
                        ui.add(egui::TextEdit::singleline(&mut app.new_title).hint_text("Título (ex: WP Admin)"));
                        ui.add(egui::TextEdit::singleline(&mut app.new_user).hint_text("Usuário"));
                        ui.add(egui::TextEdit::singleline(&mut app.new_pass).password(true).hint_text("Senha"));
                        ui.add(egui::TextEdit::singleline(&mut app.new_url).hint_text("URL"));
                        
                        if ui.button("Salvar").clicked() {
                            if let Ok(_) = app.keepass.add_entry(&app.new_title, &app.new_user, &app.new_pass, &app.new_url) {
                                app.new_title.clear(); app.new_user.clear(); app.new_pass.clear(); app.new_url.clear();
                                app.credentials = app.keepass.list_all_entries();
                            }
                        }
                    });

                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for cred in &mut app.credentials {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut cred.selected, "");
                                ui.label(&cred.site_name);
                                if cred.fetched {
                                    ui.label(egui::RichText::new(" (Pronto)").color(egui::Color32::GREEN));
                                }
                            });
                        }
                    });
                }
            });
            if ctx.input(|i| i.viewport().close_requested()) { app.show_vault = false; }
        },
    );
}