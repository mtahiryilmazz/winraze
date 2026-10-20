use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

const BUILTIN_TR: &str = include_str!("../locales/tr.json");
const BUILTIN_EN: &str = include_str!("../locales/en.json");
const BUILTIN_DE: &str = include_str!("../locales/de.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleFile {
    pub locale_name: String,
    pub locale_code: String,
    #[serde(flatten)]
    pub translations: Translations,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Translations {
    pub app_title: String,
    pub app_subtitle: String,
    pub search_hint: String,
    pub stat_apps: String,
    pub stat_size: String,
    pub stat_selected: String,
    pub stat_leftovers: String,
    pub tab_all: String,
    pub tab_win32: String,
    pub tab_msi: String,
    pub tab_store: String,
    pub tab_leftovers: String,
    pub tab_tracked: String,
    pub btn_uninstall: String,
    pub btn_review: String,
    pub btn_rescan: String,
    pub btn_settings: String,
    pub btn_select_all: String,
    pub btn_deselect_all: String,
    pub btn_close: String,
    pub btn_save: String,
    pub col_name: String,
    pub col_publisher: String,
    pub col_version: String,
    pub col_size: String,
    pub col_type: String,
    pub settings_title: String,
    pub settings_autokill: String,
    pub settings_restorepoint: String,
    pub settings_regbackup: String,
    pub settings_emptydirs: String,
    pub settings_shortcuts: String,
    pub settings_prefetch: String,
    pub settings_autoreboot: String,
    pub settings_dryrun: String,
    pub settings_lang: String,
    pub badge_protected: String,
    pub warn_protected_modal: String,
    pub inspect_title: String,
    pub inspect_publisher: String,
    pub inspect_version: String,
    pub inspect_location: String,
    pub inspect_command: String,
    pub inspect_size: String,
    pub inspect_artifacts: String,
    pub modal_uninstalling: String,
    pub modal_live_log: String,
    pub modal_complete: String,
    pub modal_reboot_notice: String,
    pub metric_freed: String,
    pub metric_files: String,
    pub metric_keys: String,
    pub metric_procs: String,
    pub metric_services: String,
    pub ready_status: String,
    pub scanning_status: String,
    pub nav_dashboard: String,
    pub nav_apps: String,
    pub nav_tracker: String,
    pub nav_debloater: String,
    pub nav_extensions: String,
    pub nav_leftovers: String,
    pub nav_force_clean: String,
    pub nav_settings: String,
    pub health_title: String,
    pub health_score_desc: String,
    pub health_opt_btn: String,
    pub health_crash_dumps: String,
    pub health_shortcuts: String,
    pub health_protected: String,
    pub tracker_title: String,
    pub tracker_desc: String,
    pub tracker_path_hint: String,
    pub tracker_btn_browse: String,
    pub tracker_btn_install: String,
    pub tracker_status_idle: String,
    pub tracker_status_presnap: String,
    pub tracker_status_running: String,
    pub tracker_status_postsnap: String,
    pub tracker_status_done: String,
    pub debloat_title: String,
    pub debloat_desc: String,
    pub debloat_telemetry_btn: String,
    pub debloat_temp_btn: String,
    pub debloat_raze_all_btn: String,
    pub debloat_success: String,
    pub ext_title: String,
    pub ext_desc: String,
    pub ext_btn_remove: String,
    pub force_title: String,
    pub force_desc: String,
    pub force_path_hint: String,
    pub force_btn_browse: String,
    pub force_btn_shred: String,
    pub force_success: String,
    pub modal_confirm_title: String,
    pub modal_confirm_btn: String,
    pub modal_cancel_btn: String,
}

pub struct LocaleManager {
    locales: HashMap<String, LocaleFile>,
    current_code: String,
    active_translations: Arc<Translations>,
}

impl LocaleManager {
    pub fn init() -> Self {
        let mut locales = HashMap::new();

        // 1. Load built-in fallback translations (guarantees standalone single-binary operation)
        if let Ok(tr) = serde_json::from_str::<LocaleFile>(BUILTIN_TR) {
            locales.insert(tr.locale_code.clone(), tr);
        }
        if let Ok(en) = serde_json::from_str::<LocaleFile>(BUILTIN_EN) {
            locales.insert(en.locale_code.clone(), en);
        }
        if let Ok(de) = serde_json::from_str::<LocaleFile>(BUILTIN_DE) {
            locales.insert(de.locale_code.clone(), de);
        }

        // 2. Scan external directory `./locales/`
        Self::scan_directory(&PathBuf::from("locales"), &mut locales);

        // 3. Scan user roaming directory `%LOCALAPPDATA%\WinRaze\Locales\`
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let user_locales = PathBuf::from(local_appdata).join("WinRaze").join("Locales");
            let _ = fs::create_dir_all(&user_locales);
            Self::scan_directory(&user_locales, &mut locales);
        }

        let default_active = Arc::new(
            locales
                .get("tr")
                .map(|l| l.translations.clone())
                .unwrap_or_default(),
        );

        Self {
            locales,
            current_code: "tr".to_string(),
            active_translations: default_active,
        }
    }

    fn scan_directory(dir: &PathBuf, locales: &mut HashMap<String, LocaleFile>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(locale_file) = serde_json::from_str::<LocaleFile>(&content) {
                            locales.insert(locale_file.locale_code.clone(), locale_file);
                        }
                    }
                }
            }
        }
    }

    pub fn t(&self) -> Arc<Translations> {
        self.active_translations.clone()
    }

    pub fn current_code(&self) -> &str {
        &self.current_code
    }

    pub fn current_name(&self) -> &str {
        self.locales
            .get(&self.current_code)
            .map(|l| l.locale_name.as_str())
            .unwrap_or("Türkçe")
    }

    pub fn set_language(&mut self, code: &str) {
        if let Some(locale) = self.locales.get(code) {
            self.current_code = code.to_string();
            self.active_translations = Arc::new(locale.translations.clone());
        }
    }

    /// Returns list of (locale_code, locale_name)
    pub fn get_available_locales(&self) -> Vec<(String, String)> {
        let mut list: Vec<(String, String)> = self
            .locales
            .values()
            .map(|l| (l.locale_code.clone(), l.locale_name.clone()))
            .collect();
        list.sort_by(|a, b| a.1.cmp(&b.1));
        list
    }
}
