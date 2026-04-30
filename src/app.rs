use eframe::egui;
use crate::browser::{BrowserInstance, BrowserType};
use crate::project::{load_projects, ProjectConfig, ProjectService};
use crate::vault::{Credential, KeePassIntegration};

#[derive(PartialEq)]
pub enum AppTab {
    Main,
    Projects,
    Vault,
}

pub struct DevBrowsersApp {
    pub current_tab: AppTab,
    pub default_browser: BrowserType,
    pub instances: Vec<BrowserInstance>,
    pub url_input: String,
    pub next_id: usize,
    
    pub credentials: Vec<Credential>,
    pub keepass: KeePassIntegration,
    pub new_title: String,
    pub new_user: String,
    pub new_pass: String,
    pub new_url: String,

    pub projects: Vec<ProjectConfig>,
    pub editing_project_id: Option<String>,
    pub new_proj_id: String,
    pub new_proj_nome: String,
    pub new_proj_vault_pass: String,
    pub quick_vault_pass: String,
    pub new_proj_servicos: Vec<ProjectService>,
    pub new_svc_nome: String,
    pub new_svc_url: String,
}

impl Default for DevBrowsersApp {
    fn default() -> Self {
        Self {
            current_tab: AppTab::Main,
            default_browser: BrowserType::Chromium,
            instances: Vec::new(),
            url_input: String::from("http://localhost:3000"),
            next_id: 1,
            
            credentials: Vec::new(),
            keepass: KeePassIntegration::default(),
            new_title: String::new(),
            new_user: String::new(),
            new_pass: String::new(),
            new_url: String::new(),

            projects: load_projects(),
            editing_project_id: None,
            new_proj_id: String::new(),
            new_proj_nome: String::new(),
            new_proj_vault_pass: String::new(),
            quick_vault_pass: String::new(),
            new_proj_servicos: Vec::new(),
            new_svc_nome: String::new(),
            new_svc_url: String::new(),
        }
    }
}

impl eframe::App for DevBrowsersApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, AppTab::Main, "🚀 Instâncias e CDP");
                ui.selectable_value(&mut self.current_tab, AppTab::Projects, "📂 Projetos");
                ui.selectable_value(&mut self.current_tab, AppTab::Vault, "⚙ Cofre de Senhas");
                
                ui.separator();
                ui.label("🌐 Navegador:");
                egui::ComboBox::from_id_source("browser_select")
                    .selected_text(match self.default_browser {
                        BrowserType::Chromium => "Chromium",
                        BrowserType::Firefox => "Firefox",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.default_browser, BrowserType::Chromium, "Chromium");
                        ui.selectable_value(&mut self.default_browser, BrowserType::Firefox, "Firefox");
                    });
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                AppTab::Main => crate::ui_main::render(self, ui),
                AppTab::Projects => crate::ui_projects::render(self, ui),
                AppTab::Vault => crate::ui_vault::render(self, ui),
            }
        });
    }
}