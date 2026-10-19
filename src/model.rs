use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppType {
    Win32,
    Msi,
    AppXStore,
    Portable,
    Leftover,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    Process,
    Service,
    Shortcut,
    RegistryKey,
    RegistryValue,
    File,
    Directory,
    Prefetch,
    CrashDump,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualArtifact {
    pub kind: ArtifactKind,
    pub path_or_target: String,
    pub description: String,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub install_location: Option<PathBuf>,
    pub main_executable: Option<PathBuf>,
    pub main_shortcut: Option<PathBuf>,
    pub uninstall_string: Option<String>,
    pub quiet_uninstall_string: Option<String>,
    pub registry_key: Option<String>,
    pub app_type: AppType,
    pub estimated_size_bytes: u64,
    pub install_date: Option<String>,
    pub artifacts: Vec<ResidualArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub include_store_apps: bool,
    pub include_portable_apps: bool,
    pub detect_leftovers: bool,
    pub deep_scan_paths: Vec<PathBuf>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            include_store_apps: true,
            include_portable_apps: true,
            detect_leftovers: true,
            deep_scan_paths: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallOptions {
    pub auto_kill: bool,
    pub auto_reboot: bool,
    pub dry_run: bool,
    pub clean_empty_dirs: bool,
    pub clean_broken_shortcuts: bool,
    pub clean_prefetch: bool,
    pub timeout_seconds: u64,
}

impl Default for UninstallOptions {
    fn default() -> Self {
        Self {
            auto_kill: true,
            auto_reboot: false,
            dry_run: false,
            clean_empty_dirs: true,
            clean_broken_shortcuts: true,
            clean_prefetch: true,
            timeout_seconds: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepLog {
    pub step_number: u8,
    pub step_name: String,
    pub details: Vec<String>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallReport {
    pub app_name: String,
    pub steps: Vec<StepLog>,
    pub deleted_files_count: usize,
    pub deleted_reg_count: usize,
    pub terminated_procs_count: usize,
    pub stopped_services_count: usize,
    pub total_freed_bytes: u64,
    pub requires_reboot: bool,
}
