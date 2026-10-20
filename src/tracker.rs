use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use walkdir::WalkDir;
use winreg::enums::*;
use winreg::RegKey;

use crate::model::{AppType, InstalledApp, ResidualArtifact, ArtifactKind, TrackedAppProfile, UninstallReport, StepLog};

/// Root folders to monitor for file creation
fn get_monitor_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(pf) = std::env::var("ProgramFiles") {
        roots.push(PathBuf::from(pf));
    }
    if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
        roots.push(PathBuf::from(pf86));
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        roots.push(PathBuf::from(appdata));
    }
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        roots.push(PathBuf::from(local_appdata));
    }
    if let Ok(prog_data) = std::env::var("ProgramData") {
        roots.push(PathBuf::from(prog_data));
    }

    // Desktop folders
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        roots.push(PathBuf::from(&userprofile).join("Desktop"));
        roots.push(PathBuf::from(&userprofile).join("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs"));
    }
    roots.push(PathBuf::from("C:\\Users\\Public\\Desktop"));
    roots.push(PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"));

    roots.retain(|p| p.exists());
    roots
}

/// Noise paths that should never be tracked as part of a third-party application
fn is_noise_path(path: &Path) -> bool {
    let p_str = path.to_string_lossy().to_lowercase();

    // Windows system noise
    p_str.contains("\\prefetch\\")
        || p_str.contains("\\windows\\temp\\")
        || p_str.contains("\\appdata\\local\\temp\\")
        || p_str.contains("\\microsoft\\windows\\webcache")
        || p_str.contains("\\microsoft\\windows\\explorer")
        || p_str.contains("\\cryptneturlcache\\")
        || p_str.contains("\\microsoft\\windows\\recent")
        || p_str.contains("\\google\\chrome\\user data")
        || p_str.contains("\\microsoft\\edge\\user data")
        || p_str.contains("\\bravesoftware\\brave-browser\\user data")
        || p_str.contains("winraze\\tracked")
        || p_str.contains("derust\\backups")
}

/// Noise registry subkeys that frequently fluctuate during normal Windows operation
fn is_noise_reg(key: &str) -> bool {
    let k_lower = key.to_lowercase();
    k_lower.contains("cryptneturlcache")
        || k_lower.contains("userassist")
        || k_lower.contains("shell folders")
        || k_lower.contains("recentdocs")
        || k_lower.contains("windows\\currentversion\\explorer\\streammrulist")
        || k_lower.contains("appcompatflags")
        || k_lower.contains("deliveryoptimization")
}

#[derive(Default, Clone)]
pub struct FsSnapshot {
    pub files: HashSet<PathBuf>,
    pub directories: HashSet<PathBuf>,
}

#[derive(Default, Clone)]
pub struct RegSnapshot {
    pub keys: HashSet<String>,
}

#[derive(Default, Clone)]
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub fs: FsSnapshot,
    pub reg: RegSnapshot,
}

pub struct InstallTracker;

impl InstallTracker {
    /// Takes a complete snapshot of file system target directories and key registry hives
    pub fn take_snapshot() -> SystemSnapshot {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 1. FS Snapshot
        let mut fs_snap = FsSnapshot::default();
        let roots = get_monitor_roots();

        for root in roots {
            for entry in WalkDir::new(&root).max_depth(4).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if is_noise_path(path) {
                    continue;
                }
                if path.is_file() {
                    fs_snap.files.insert(path.to_path_buf());
                } else if path.is_dir() {
                    fs_snap.directories.insert(path.to_path_buf());
                }
            }
        }

        // 2. Registry Snapshot
        let mut reg_snap = RegSnapshot::default();
        Self::snapshot_registry_hive(&mut reg_snap, &RegKey::predef(HKEY_CURRENT_USER), "HKCU\\Software");
        Self::snapshot_registry_hive(&mut reg_snap, &RegKey::predef(HKEY_LOCAL_MACHINE), "HKLM\\Software");
        Self::snapshot_registry_hive(&mut reg_snap, &RegKey::predef(HKEY_LOCAL_MACHINE), "HKLM\\Software\\WOW6432Node");

        SystemSnapshot {
            timestamp: ts,
            fs: fs_snap,
            reg: reg_snap,
        }
    }

    fn snapshot_registry_hive(reg_snap: &mut RegSnapshot, root: &RegKey, subkey_path: &str) {
        let prefix = subkey_path.split('\\').next().unwrap_or("HKLM");
        let rel_path = if subkey_path.len() > prefix.len() {
            &subkey_path[prefix.len() + 1..]
        } else {
            ""
        };

        if let Ok(key) = root.open_subkey(rel_path) {
            reg_snap.keys.insert(subkey_path.to_string());
            for sub_name in key.enum_keys().filter_map(|k| k.ok()) {
                let full_sub = format!("{}\\{}", subkey_path, sub_name);
                if !is_noise_reg(&full_sub) {
                    reg_snap.keys.insert(full_sub.clone());
                    // 1 level deeper inspection
                    if let Ok(child_key) = key.open_subkey(&sub_name) {
                        for grand_child in child_key.enum_keys().filter_map(|g| g.ok()) {
                            let grand_sub = format!("{}\\{}", full_sub, grand_child);
                            if !is_noise_reg(&grand_sub) {
                                reg_snap.keys.insert(grand_sub);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Computes diff between pre-install and post-install snapshots
    pub fn compute_diff(
        pre: &SystemSnapshot,
        post: &SystemSnapshot,
        app_name: String,
        installer_path: Option<String>,
    ) -> TrackedAppProfile {
        let mut created_files: Vec<PathBuf> = post
            .fs
            .files
            .difference(&pre.fs.files)
            .cloned()
            .filter(|p| !is_noise_path(p))
            .collect();

        let created_directories: Vec<PathBuf> = post
            .fs
            .directories
            .difference(&pre.fs.directories)
            .cloned()
            .filter(|p| !is_noise_path(p))
            .collect();

        let created_registry_keys: Vec<String> = post
            .reg
            .keys
            .difference(&pre.reg.keys)
            .cloned()
            .filter(|k| !is_noise_reg(k))
            .collect();

        // Calculate total size
        let mut total_size_bytes = 0u64;
        for f in &created_files {
            if let Ok(meta) = fs::metadata(f) {
                total_size_bytes += meta.len();
            }
        }

        created_files.sort();

        let id = format!(
            "tracked_{}_{}",
            app_name.to_lowercase().replace(' ', "_"),
            post.timestamp
        );

        TrackedAppProfile {
            id,
            name: app_name,
            version: None,
            install_date: chrono_like_now(),
            installer_path,
            created_files,
            created_directories,
            created_registry_keys,
            created_registry_values: Vec::new(),
            total_size_bytes,
        }
    }

    /// Run the target installer and wait for its completion
    pub fn execute_installer_and_wait(installer_path: &Path) -> Result<bool, String> {
        if !installer_path.exists() {
            return Err(format!("Installer file not found: {:?}", installer_path));
        }

        let mut child = Command::new(installer_path)
            .spawn()
            .map_err(|e| format!("Failed to launch installer: {}", e))?;

        let status = child
            .wait()
            .map_err(|e| format!("Failed while waiting for installer: {}", e))?;

        Ok(status.success())
    }

    /// Profile storage directory: `%LOCALAPPDATA%\WinRaze\Tracked\`
    pub fn get_storage_dir() -> PathBuf {
        let base = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"));
        let dir = base.join("WinRaze").join("Tracked");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    /// Save tracked profile to disk
    pub fn save_profile(profile: &TrackedAppProfile) -> Result<PathBuf, String> {
        let dir = Self::get_storage_dir();
        let file_path = dir.join(format!("{}.json", profile.id));
        let data = serde_json::to_string_pretty(profile)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&file_path, data)
            .map_err(|e| format!("Failed to write profile: {}", e))?;
        Ok(file_path)
    }

    /// Load all tracked installation profiles
    pub fn load_all_profiles() -> Vec<TrackedAppProfile> {
        let dir = Self::get_storage_dir();
        let mut profiles = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(profile) = serde_json::from_str::<TrackedAppProfile>(&content) {
                            profiles.push(profile);
                        }
                    }
                }
            }
        }

        profiles
    }

    /// Convert a TrackedAppProfile into an InstalledApp item for GUI/CLI presentation
    pub fn profile_to_installed_app(profile: &TrackedAppProfile) -> InstalledApp {
        let mut artifacts = Vec::new();

        for f in &profile.created_files {
            artifacts.push(ResidualArtifact {
                kind: ArtifactKind::File,
                path_or_target: f.to_string_lossy().to_string(),
                description: "Tracked Installation File".to_string(),
                size_bytes: fs::metadata(f).ok().map(|m| m.len()),
            });
        }

        for d in &profile.created_directories {
            artifacts.push(ResidualArtifact {
                kind: ArtifactKind::Directory,
                path_or_target: d.to_string_lossy().to_string(),
                description: "Tracked Installation Directory".to_string(),
                size_bytes: None,
            });
        }

        for k in &profile.created_registry_keys {
            artifacts.push(ResidualArtifact {
                kind: ArtifactKind::RegistryKey,
                path_or_target: k.clone(),
                description: "Tracked Installation Registry Key".to_string(),
                size_bytes: None,
            });
        }

        let main_exec = profile
            .created_files
            .iter()
            .find(|f| f.extension().and_then(|s| s.to_str()) == Some("exe"))
            .cloned();

        let install_location = profile
            .created_directories
            .first()
            .cloned()
            .or_else(|| main_exec.as_ref().and_then(|e| e.parent().map(|p| p.to_path_buf())));

        InstalledApp {
            id: profile.id.clone(),
            name: profile.name.clone(),
            publisher: Some("Tracked Installation (WinRaze)".to_string()),
            version: profile.version.clone(),
            install_location,
            main_executable: main_exec,
            main_shortcut: None,
            uninstall_string: Some(format!("winraze tracked-remove {}", profile.id)),
            quiet_uninstall_string: Some(format!("winraze tracked-remove {} --yes", profile.id)),
            registry_key: profile.created_registry_keys.first().cloned(),
            app_type: AppType::Tracked,
            estimated_size_bytes: profile.total_size_bytes,
            install_date: Some(profile.install_date.clone()),
            artifacts,
            is_protected: false,
        }
    }

    /// Execute 100% clean uninstallation using tracked profile log
    pub fn uninstall_profile(profile: &TrackedAppProfile, dry_run: bool) -> UninstallReport {
        let mut steps = Vec::new();
        let mut deleted_files = 0;
        let mut deleted_regs = 0;
        let mut freed_bytes = 0u64;

        // Step 1: Remove created files
        let mut file_logs = Vec::new();
        for file in &profile.created_files {
            if file.exists() {
                if let Ok(meta) = fs::metadata(file) {
                    freed_bytes += meta.len();
                }
                if dry_run {
                    file_logs.push(format!("[Simülasyon] Silinecek dosya: {:?}", file));
                    deleted_files += 1;
                } else {
                    match fs::remove_file(file) {
                        Ok(_) => {
                            file_logs.push(format!("Silindi: {:?}", file));
                            deleted_files += 1;
                        }
                        Err(e) => {
                            file_logs.push(format!("Dosya silinemedi: {:?} ({})", file, e));
                        }
                    }
                }
            }
        }
        steps.push(StepLog {
            step_number: 1,
            step_name: "İzlenen Dosyaların Temizlenmesi".to_string(),
            details: file_logs,
            success: true,
        });

        // Step 2: Remove created directories (reverse order for nested deletion)
        let mut dir_logs = Vec::new();
        let mut dirs = profile.created_directories.clone();
        dirs.sort_by_key(|b| std::cmp::Reverse(b.as_os_str().len()));

        for dir in dirs {
            if dir.exists() {
                if dry_run {
                    dir_logs.push(format!("[Simülasyon] Silinecek klasör: {:?}", dir));
                } else {
                    let _ = fs::remove_dir_all(&dir);
                    dir_logs.push(format!("Klasör kaldırıldı: {:?}", dir));
                }
            }
        }
        steps.push(StepLog {
            step_number: 2,
            step_name: "İzlenen Dizinlerin Kaldırılması".to_string(),
            details: dir_logs,
            success: true,
        });

        // Step 3: Remove created registry keys
        let mut reg_logs = Vec::new();
        for key_str in &profile.created_registry_keys {
            if dry_run {
                reg_logs.push(format!("[Simülasyon] Silinecek Registry Anahtarı: {}", key_str));
                deleted_regs += 1;
            } else {
                let deleted = Self::delete_registry_key_by_path(key_str);
                if deleted {
                    reg_logs.push(format!("Registry anahtarı silindi: {}", key_str));
                    deleted_regs += 1;
                }
            }
        }
        steps.push(StepLog {
            step_number: 3,
            step_name: "İzlenen Kayıt Defteri Anahtarlarının Temizlenmesi".to_string(),
            details: reg_logs,
            success: true,
        });

        // Step 4: Remove profile JSON file
        if !dry_run {
            let profile_file = Self::get_storage_dir().join(format!("{}.json", profile.id));
            let _ = fs::remove_file(profile_file);
        }

        UninstallReport {
            app_name: profile.name.clone(),
            steps,
            deleted_files_count: deleted_files,
            deleted_reg_count: deleted_regs,
            terminated_procs_count: 0,
            stopped_services_count: 0,
            total_freed_bytes: freed_bytes,
            requires_reboot: false,
            restore_point_created: false,
            registry_backup_path: None,
        }
    }

    fn delete_registry_key_by_path(key_path: &str) -> bool {
        let (root_hive, rel_path) = if key_path.starts_with("HKLM\\") {
            (RegKey::predef(HKEY_LOCAL_MACHINE), &key_path[5..])
        } else if key_path.starts_with("HKCU\\") {
            (RegKey::predef(HKEY_CURRENT_USER), &key_path[5..])
        } else {
            return false;
        };

        if let Some(last_slash) = rel_path.rfind('\\') {
            let parent_path = &rel_path[..last_slash];
            let sub_name = &rel_path[last_slash + 1..];
            if let Ok(parent_key) = root_hive.open_subkey_with_flags(parent_path, KEY_WRITE) {
                return parent_key.delete_subkey_all(sub_name).is_ok();
            }
        }
        false
    }
}

fn chrono_like_now() -> String {
    // Simple timestamp format YYYY-MM-DD
    let now = SystemTime::now();
    let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    let days = duration.as_secs() / 86400;
    // Approximated Gregorian date for zero-dependency simplicity
    let year = 1970 + (days / 365);
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
}
