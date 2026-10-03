use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::Deserialize;
use crate::models::AppMetadata;
use crate::traits::AppRegistry;

#[derive(Debug, Deserialize)]
struct RawAppConfig {
    app_name: Option<String>,
    environment: Option<String>,
    platform: Option<String>,
    language: Option<String>,
    framework: Option<String>,
    repo_url: Option<String>,
    default_branch: Option<String>,
    verification_command: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawConfigFile {
    applications: Option<HashMap<String, RawAppConfig>>,
}

pub struct YamlAppRegistry {
    apps: HashMap<String, AppMetadata>,
}

impl YamlAppRegistry {
    pub fn new() -> Self {
        Self {
            apps: HashMap::new(),
        }
    }

    /// Loads app metadata from a single YAML file or a directory of app YAML files (e.g., `apps.d/`).
    pub fn load_from_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        let p = path.as_ref();
        if p.is_dir() {
            self.load_directory(p);
        } else if p.is_file() {
            self.load_single_file(p);
        }
        self
    }

    /// Recursively/flatly scans a directory (e.g. `apps.d/`) and loads every `*.yaml` and `*.yml` file.
    pub fn load_directory(&mut self, dir_path: &Path) {
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if ext == "yaml" || ext == "yml" {
                        self.load_single_file(&path);
                    }
                }
            }
        }
    }

    /// Loads a single file. Supports both `{ applications: { app1: ... } }` and standalone app definitions.
    pub fn load_single_file(&mut self, file_path: &Path) {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        // Try format 1: { applications: { ... } }
        if let Ok(parsed) = serde_yaml::from_str::<RawConfigFile>(&content) {
            if let Some(applications) = parsed.applications {
                for (name, conf) in applications {
                    self.insert_raw(name, conf);
                }
                return;
            }
        }

        // Try format 2: Standalone app definition in its own file (e.g., apps.d/toko-api.yaml)
        if let Ok(conf) = serde_yaml::from_str::<RawAppConfig>(&content) {
            let file_stem = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let app_name = conf.app_name.clone().unwrap_or(file_stem);
            self.insert_raw(app_name, conf);
        }
    }

    fn insert_raw(&mut self, name: String, conf: RawAppConfig) {
        let meta = AppMetadata {
            app_name: name.clone(),
            environment: conf.environment.unwrap_or_else(|| "production".to_string()),
            platform: conf.platform.unwrap_or_else(|| "coolify".to_string()),
            language: conf.language.unwrap_or_else(|| "unknown".to_string()),
            framework: conf.framework.unwrap_or_else(|| "unknown".to_string()),
            repo_url: conf.repo_url.unwrap_or_else(|| format!("https://github.com/myorg/{}", name)),
            default_branch: conf.default_branch.unwrap_or_else(|| "main".to_string()),
            verification_command: conf.verification_command.unwrap_or_else(|| "npm test".to_string()),
        };
        self.apps.insert(name, meta);
    }

    pub fn count(&self) -> usize {
        self.apps.len()
    }
}

impl Default for YamlAppRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AppRegistry for YamlAppRegistry {
    fn get_app(&self, app_name: &str) -> AppMetadata {
        if let Some(meta) = self.apps.get(app_name) {
            return meta.clone();
        }

        let default_org = std::env::var("DEFAULT_GITHUB_ORG").unwrap_or_else(|_| "your-org".to_string());
        AppMetadata {
            app_name: app_name.to_string(),
            environment: "production".to_string(),
            platform: "coolify".to_string(),
            language: "unknown".to_string(),
            framework: "unknown".to_string(),
            repo_url: format!("https://github.com/{}/{}", default_org, app_name),
            default_branch: "main".to_string(),
            verification_command: "npm test".to_string(),
        }
    }
}
