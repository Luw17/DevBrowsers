use eframe::egui;
use crate::browser::BrowserInstance;
use crate::project::{load_projects, ProjectConfig, ProjectService};
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
    pub new_url: String,

    pub show_projects: bool,
    pub projects: Vec<ProjectConfig>,
    pub editing_project_id: Option<String>,
    pub new_proj_id: String,
    pub new_proj_nome: String,
    pub new_proj_vault_pass: String,
    pub new_proj_servicos: Vec<ProjectService>,
    pub new_svc_nome: String,
    pub new_svc_url: String,
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
            new_url: String::new(),

            show_projects: false,
            projects: load_projects(),
            editing_project_id: None,
            new_proj_id: String::new(),
            new_proj_nome: String::new(),
            new_proj_vault_pass: String::new(),
            new_proj_servicos: Vec::new(),
            new_svc_nome: String::new(),
            new_svc_url: String::new(),
        }
    }
}

impl eframe::App for DevBrowsersApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::clipboard::handle_global_events(ctx);

        crate::ui_main::render(self, ctx);

        if self.show_projects {
            crate::ui_projects::render(self, ctx);
        }

        if self.show_vault {
            crate::ui_vault::render(self, ctx);
        }
    }
}