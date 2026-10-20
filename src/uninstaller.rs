use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use winreg::enums::*;
use winreg::RegKey;

use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT};

use crate::model::{AppType, ArtifactKind, InstalledApp, StepLog, UninstallOptions, UninstallReport};
use crate::process_killer::ProcessKiller;

pub struct UninstallationEngine {
    options: UninstallOptions,
}

impl UninstallationEngine {
    pub fn new(options: UninstallOptions) -> Self {
        Self { options }
    }

    pub fn execute_uninstallation(&self, app: &InstalledApp) -> UninstallReport {
        let mut report = UninstallReport {
            app_name: app.name.clone(),
            steps: Vec::new(),
            deleted_files_count: 0,
            deleted_reg_count: 0,
            terminated_procs_count: 0,
            stopped_services_count: 0,
            total_freed_bytes: 0,
            requires_reboot: false,
            restore_point_created: false,
            registry_backup_path: None,
        };

        println!("\n[+] Starting DeRust Deep Clean Pipeline for: {}", app.name);

        // ==========================================
        // Step 0: Windows System Restore Point Creation
        // ==========================================
        let mut step0_details = Vec::new();
        let mut restore_created = false;
        if self.options.create_restore_point {
            if self.options.dry_run {
                step0_details.push(format!("[DRY RUN] Would create System Restore Point: DeRust Pre-Uninstall: {}", app.name));
                restore_created = true;
            } else {
                println!("[*] Creating Windows System Restore Point...");
                if Self::create_windows_restore_point(&app.name) {
                    step0_details.push(format!("Created Windows System Restore Point for: {}", app.name));
                    restore_created = true;
                } else {
                    step0_details.push("System Restore Point creation skipped (disabled or insufficient privileges)".to_string());
                }
            }
            report.steps.push(StepLog {
                step_number: 0,
                step_name: "System Restore Point Creation".to_string(),
                details: step0_details,
                success: restore_created,
            });
        }
        report.restore_point_created = restore_created;

        // ==========================================
        // Step 1 & 2: Process & Service Termination
        // ==========================================
        let mut step1_details = Vec::new();
        if self.options.auto_kill && !self.options.dry_run {
            let svcs = ProcessKiller::stop_related_services(&app.name);
            for s in &svcs {
                step1_details.push(format!("Stopped Windows Service: {}", s));
            }
            report.stopped_services_count += svcs.len();

            let procs = ProcessKiller::terminate_app_processes(&app.name, app.install_location.as_deref());
            for p in &procs {
                step1_details.push(format!("Terminated Process: {}", p));
            }
            report.terminated_procs_count += procs.len();
        } else if self.options.dry_run {
            step1_details.push("[DRY RUN] Would scan and terminate running processes & services".to_string());
        }

        report.steps.push(StepLog {
            step_number: 1,
            step_name: "Process & Service Termination (AutoKill)".to_string(),
            details: step1_details,
            success: true,
        });

        // ==========================================
        // Step 3: Shortcut Removal
        // ==========================================
        let mut step3_details = Vec::new();
        for art in &app.artifacts {
            if art.kind == ArtifactKind::Shortcut {
                let p = Path::new(&art.path_or_target);
                if p.exists() {
                    if self.options.dry_run {
                        step3_details.push(format!("[DRY RUN] Would delete shortcut: {}", art.path_or_target));
                    } else if fs::remove_file(p).is_ok() {
                        step3_details.push(format!("Deleted shortcut: {}", art.path_or_target));
                        report.deleted_files_count += 1;
                    }
                }
            }
        }

        report.steps.push(StepLog {
            step_number: 3,
            step_name: "Shortcut Cleaning".to_string(),
            details: step3_details,
            success: true,
        });

        // ==========================================
        // Step 4: Silent MSI Execution
        // ==========================================
        let mut step4_details = Vec::new();
        if app.app_type == AppType::Msi {
            if let Some(ref uninst_cmd) = app.uninstall_string {
                if self.options.dry_run {
                    step4_details.push(format!("[DRY RUN] Would execute silent MSI: {}", uninst_cmd));
                } else {
                    let quiet_cmd = Self::format_silent_msi_command(uninst_cmd);
                    step4_details.push(format!("Executing MSI: {}", quiet_cmd));
                    let _ = run_shell_command(&quiet_cmd, self.options.timeout_seconds);
                }
            }
        }
        report.steps.push(StepLog {
            step_number: 4,
            step_name: "Windows Installer (MSI) Silent Uninstallation".to_string(),
            details: step4_details,
            success: true,
        });

        // ==========================================
        // Step 5: Main Native Uninstaller Execution
        // ==========================================
        let mut step5_details = Vec::new();
        if app.app_type == AppType::Win32 {
            let cmd_to_run = app.quiet_uninstall_string.as_ref()
                .or(app.uninstall_string.as_ref());

            if let Some(cmd) = cmd_to_run {
                let silent_cmd = Self::inject_silent_switches(cmd);
                if self.options.dry_run {
                    step5_details.push(format!("[DRY RUN] Would run native uninstaller: {}", silent_cmd));
                } else {
                    step5_details.push(format!("Executing: {}", silent_cmd));
                    let _ = run_shell_command(&silent_cmd, self.options.timeout_seconds);
                    sleep(Duration::from_millis(500));
                }
            }
        }
        report.steps.push(StepLog {
            step_number: 5,
            step_name: "Native Uninstaller Execution".to_string(),
            details: step5_details,
            success: true,
        });

        // ==========================================
        // Step 6: AppX / MSIX Removal
        // ==========================================
        let mut step6_details = Vec::new();
        if app.app_type == AppType::AppXStore {
            if self.options.dry_run {
                step6_details.push(format!("[DRY RUN] Would remove AppX package: {}", app.id));
            } else {
                let ps_cmd = format!("Remove-AppxPackage -Package \"{}\"", app.id);
                step6_details.push(format!("Invoking: {}", ps_cmd));
                let _ = Command::new("powershell")
                    .args(&["-NoProfile", "-Command", &ps_cmd])
                    .output();
            }
        }
        report.steps.push(StepLog {
            step_number: 6,
            step_name: "AppX / Store Package Removal".to_string(),
            details: step6_details,
            success: true,
        });

        // ==========================================
        // Step 7: Deep Registry & Residuals Clean (with Rollback Backup)
        // ==========================================
        let mut step7_reg_details = Vec::new();
        for art in &app.artifacts {
            if art.kind == ArtifactKind::RegistryKey {
                if self.options.dry_run {
                    step7_reg_details.push(format!("[DRY RUN] Would backup and delete registry key: {}", art.path_or_target));
                } else {
                    if self.options.backup_registry {
                        if let Some(bk_path) = Self::backup_registry_key(&art.path_or_target, &app.name) {
                            step7_reg_details.push(format!("Exported .reg rollback backup: {}", bk_path));
                            if report.registry_backup_path.is_none() {
                                report.registry_backup_path = Some(bk_path);
                            }
                        }
                    }
                    if Self::delete_registry_key_recursive(&art.path_or_target) {
                        step7_reg_details.push(format!("Deleted registry key: {}", art.path_or_target));
                        report.deleted_reg_count += 1;
                    }
                }
            } else if art.kind == ArtifactKind::RegistryValue {
                if self.options.dry_run {
                    step7_reg_details.push(format!("[DRY RUN] Would remove registry value: {}", art.path_or_target));
                } else if Self::delete_registry_value(&art.path_or_target) {
                    step7_reg_details.push(format!("Removed registry value: {}", art.path_or_target));
                    report.deleted_reg_count += 1;
                }
            }
        }
        report.steps.push(StepLog {
            step_number: 7,
            step_name: "Deep Registry Cleanup & Rollback Backup".to_string(),
            details: step7_reg_details,
            success: true,
        });

        // ==========================================
        // Step 8: Filesystem Cleanup & Delayed Deletion
        // ==========================================
        let mut step8_file_details = Vec::new();
        for art in &app.artifacts {
            if art.kind == ArtifactKind::Directory || art.kind == ArtifactKind::File {
                let target_path = Path::new(&art.path_or_target);
                if target_path.exists() {
                    let size = art.size_bytes.unwrap_or(0);
                    if self.options.dry_run {
                        step8_file_details.push(format!("[DRY RUN] Would delete: {}", art.path_or_target));
                    } else if target_path.is_dir() {
                        match fs::remove_dir_all(target_path) {
                            Ok(_) => {
                                step8_file_details.push(format!("Removed directory: {}", art.path_or_target));
                                report.deleted_files_count += 1;
                                report.total_freed_bytes += size;
                            }
                            Err(_) => {
                                // Locked files - schedule for reboot via MoveFileExW
                                Self::schedule_deletion_on_reboot(target_path);
                                step8_file_details.push(format!("Locked! Scheduled deletion on reboot: {}", art.path_or_target));
                                report.requires_reboot = true;
                            }
                        }
                    } else if fs::remove_file(target_path).is_ok() {
                        step8_file_details.push(format!("Removed file: {}", art.path_or_target));
                        report.deleted_files_count += 1;
                        report.total_freed_bytes += size;
                    }
                }
            }
        }
        report.steps.push(StepLog {
            step_number: 8,
            step_name: "Filesystem Cleanup".to_string(),
            details: step8_file_details,
            success: true,
        });

        // ==========================================
        // Step 9: Post-Processing & Residual Pruning
        // ==========================================
        let mut step9_details = Vec::new();
        if self.options.clean_prefetch && !self.options.dry_run {
            for art in &app.artifacts {
                if art.kind == ArtifactKind::Prefetch {
                    let p = Path::new(&art.path_or_target);
                    if p.exists() && fs::remove_file(p).is_ok() {
                        step9_details.push(format!("Cleaned prefetch file: {}", art.path_or_target));
                        report.deleted_files_count += 1;
                    }
                }
            }
        }

        // Clean Crash Dumps
        if !self.options.dry_run {
            for art in &app.artifacts {
                if art.kind == ArtifactKind::CrashDump {
                    let p = Path::new(&art.path_or_target);
                    if p.exists() && fs::remove_file(p).is_ok() {
                        step9_details.push(format!("Cleaned residual crash dump: {}", art.path_or_target));
                        report.deleted_files_count += 1;
                    }
                }
            }
        }

        if self.options.clean_empty_dirs && !self.options.dry_run {
            if let Some(ref loc) = app.install_location {
                if let Some(parent) = loc.parent() {
                    if parent.exists() && is_dir_empty(parent) {
                        let _ = fs::remove_dir(parent);
                        step9_details.push(format!("Pruned empty parent directory: {:?}", parent));
                    }
                }
            }
        }

        report.steps.push(StepLog {
            step_number: 9,
            step_name: "Post-Processing & Residual Pruning".to_string(),
            details: step9_details,
            success: true,
        });

        report
    }

    fn create_windows_restore_point(app_name: &str) -> bool {
        let escaped_name = app_name.replace('\'', "''").replace('"', "");
        let ps_cmd = format!(
            "Checkpoint-Computer -Description 'DeRust Pre-Uninstall: {}' -RestorePointType 'APPLICATION_UNINSTALL' -ErrorAction SilentlyContinue",
            escaped_name
        );
        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", &ps_cmd])
            .output();
        output.map_or(false, |o| o.status.success())
    }

    fn backup_registry_key(key_str: &str, app_name: &str) -> Option<String> {
        let local_appdata = std::env::var("LOCALAPPDATA").ok()?;
        let backup_dir = PathBuf::from(local_appdata).join("DeRust").join("Backups").join("Registry");
        let _ = fs::create_dir_all(&backup_dir);

        let sanitized_app: String = app_name.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();
        let sanitized_key: String = key_str.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let filename = format!("{}_{}_{}.reg", sanitized_app, sanitized_key, timestamp);
        let backup_file = backup_dir.join(filename);
        let backup_file_str = backup_file.to_string_lossy().to_string();

        let status = Command::new("reg")
            .args(&["export", key_str, &backup_file_str, "/y"])
            .output();

        if status.map_or(false, |o| o.status.success()) {
            Some(backup_file_str)
        } else {
            None
        }
    }

    fn format_silent_msi_command(cmd: &str) -> String {
        let mut res = cmd.to_string();
        if !res.to_lowercase().contains("/quiet") && !res.to_lowercase().contains("/qn") {
            res.push_str(" /quiet /norestart /passive");
        }
        res
    }

    fn inject_silent_switches(cmd: &str) -> String {
        let lower = cmd.to_lowercase();
        if lower.contains("unins000.exe") || lower.contains("inno") {
            // Inno Setup
            format!("{} /VERYSILENT /SUPPRESSMSGBOXES /NORESTART", cmd)
        } else if lower.contains("uninstall.exe") || lower.contains("uninst.exe") || lower.contains("setup.exe") {
            // NSIS, InstallShield, or generic setup
            if lower.contains("installshield") || lower.contains("issetup") {
                format!("{} /s /v\"/qn /norestart\"", cmd)
            } else if !lower.contains("/s") && !lower.contains("-s") && !lower.contains("/quiet") {
                format!("{} /S", cmd)
            } else {
                cmd.to_string()
            }
        } else if lower.contains("msiexec") {
            Self::format_silent_msi_command(cmd)
        } else if lower.contains("update.exe") && (lower.contains("squirrel") || lower.contains("discord") || lower.contains("slack")) {
            format!("{} --uninstall -s", cmd)
        } else {
            // Generic installer fallback
            if !lower.contains("/quiet") && !lower.contains("/qn") && !lower.contains("/s") && !lower.contains("/silent") {
                format!("{} /silent /norestart", cmd)
            } else {
                cmd.to_string()
            }
        }
    }

    fn delete_registry_key_recursive(key_str: &str) -> bool {
        let parts: Vec<&str> = key_str.splitn(2, '\\').collect();
        if parts.len() != 2 {
            return false;
        }

        let hive = match parts[0].to_uppercase().as_str() {
            "HKLM" | "HKEY_LOCAL_MACHINE" => HKEY_LOCAL_MACHINE,
            "HKCU" | "HKEY_CURRENT_USER" => HKEY_CURRENT_USER,
            "HKCR" | "HKEY_CLASSES_ROOT" => HKEY_CLASSES_ROOT,
            _ => return false,
        };

        let root = RegKey::predef(hive);
        root.delete_subkey_all(parts[1]).is_ok()
    }

    fn delete_registry_value(val_str: &str) -> bool {
        let parts: Vec<&str> = val_str.split(" -> ").collect();
        if parts.len() != 2 {
            return false;
        }
        let key_parts: Vec<&str> = parts[0].splitn(2, '\\').collect();
        if key_parts.len() != 2 {
            return false;
        }
        let hive = match key_parts[0].to_uppercase().as_str() {
            "HKLM" | "HKEY_LOCAL_MACHINE" => HKEY_LOCAL_MACHINE,
            "HKCU" | "HKEY_CURRENT_USER" => HKEY_CURRENT_USER,
            "HKCR" | "HKEY_CLASSES_ROOT" => HKEY_CLASSES_ROOT,
            _ => return false,
        };
        let root = RegKey::predef(hive);
        if let Ok(key) = root.open_subkey_with_flags(key_parts[1], KEY_SET_VALUE) {
            key.delete_value(parts[1]).is_ok()
        } else {
            false
        }
    }

    fn schedule_deletion_on_reboot(path: &Path) {
        use std::os::windows::ffi::OsStrExt;
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        wide.push(0);

        unsafe {
            let _ = MoveFileExW(
                PCWSTR(wide.as_ptr()),
                PCWSTR(std::ptr::null()),
                MOVEFILE_DELAY_UNTIL_REBOOT,
            );
        }
    }

    pub fn force_clean_path(&self, target_path: &Path) -> UninstallReport {
        let folder_name = target_path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Target".to_string());

        let mut report = UninstallReport {
            app_name: format!("Zorla Kaldır / Force Clean: {}", folder_name),
            steps: Vec::new(),
            deleted_files_count: 0,
            deleted_reg_count: 0,
            terminated_procs_count: 0,
            stopped_services_count: 0,
            total_freed_bytes: 0,
            requires_reboot: false,
            restore_point_created: false,
            registry_backup_path: None,
        };

        if !target_path.exists() {
            report.steps.push(StepLog {
                step_number: 1,
                step_name: "Yol Doğrulama / Path Verification".to_string(),
                details: vec![format!("Hata: Belirtilen hedef yol bulunamadı: {:?}", target_path)],
                success: false,
            });
            return report;
        }

        let freed_size = crate::scanner::calculate_dir_size(target_path);

        if self.options.create_restore_point && !self.options.dry_run {
            let res = Self::create_windows_restore_point(&format!("DeRust Force Clean: {}", folder_name));
            report.restore_point_created = res;
        }

        let mut step1_details = Vec::new();
        if self.options.auto_kill && !self.options.dry_run {
            let procs = ProcessKiller::terminate_app_processes(&folder_name, Some(target_path));
            for p in &procs {
                step1_details.push(format!("Sonlandırılan Süreç / Terminated: {}", p));
            }
            report.terminated_procs_count += procs.len();
        }
        report.steps.push(StepLog {
            step_number: 1,
            step_name: "Süreç & Kilit Açma / Process & Handle Termination".to_string(),
            details: step1_details,
            success: true,
        });

        let mut step2_details = Vec::new();
        let hives = [
            (HKEY_CURRENT_USER, "Software"),
            (HKEY_LOCAL_MACHINE, "Software"),
            (HKEY_LOCAL_MACHINE, "Software\\Wow6432Node"),
        ];
        for (hive, sub) in hives {
            let root = RegKey::predef(hive);
            if let Ok(key) = root.open_subkey_with_flags(sub, KEY_READ) {
                for subname in key.enum_keys().filter_map(|r| r.ok()) {
                    if subname.eq_ignore_ascii_case(&folder_name) {
                        let full_reg = format!("{}\\{}\\{}", if hive == HKEY_CURRENT_USER { "HKCU" } else { "HKLM" }, sub, subname);
                        if !self.options.dry_run {
                            if self.options.backup_registry {
                                let _ = Self::backup_registry_key(&full_reg, &folder_name);
                            }
                            if Self::delete_registry_key_recursive(&full_reg) {
                                step2_details.push(format!("Silinen Kayıt Anahtarı: {}", full_reg));
                                report.deleted_reg_count += 1;
                            }
                        } else {
                            step2_details.push(format!("[DRY RUN] Would delete: {}", full_reg));
                        }
                    }
                }
            }
        }
        report.steps.push(StepLog {
            step_number: 2,
            step_name: "İlişkili Kayıt Defteri Temizliği / Registry Clean".to_string(),
            details: step2_details,
            success: true,
        });

        let mut step3_details = Vec::new();
        if self.options.dry_run {
            step3_details.push(format!("[DRY RUN] Would remove: {:?}", target_path));
        } else if target_path.is_dir() {
            match fs::remove_dir_all(target_path) {
                Ok(_) => {
                    step3_details.push(format!("Hedef klasör silindi: {:?}", target_path));
                    report.deleted_files_count += 1;
                    report.total_freed_bytes += freed_size;
                }
                Err(_) => {
                    Self::schedule_deletion_on_reboot(target_path);
                    step3_details.push(format!("Kilitli! Yeniden başlatmada silinecek: {:?}", target_path));
                    report.requires_reboot = true;
                }
            }
        } else if fs::remove_file(target_path).is_ok() {
            step3_details.push(format!("Hedef dosya silindi: {:?}", target_path));
            report.deleted_files_count += 1;
            report.total_freed_bytes += freed_size;
        }

        if let Some(parent) = target_path.parent() {
            if parent.exists() && is_dir_empty(parent) {
                let _ = fs::remove_dir(parent);
                step3_details.push(format!("Boş kalan üst klasör budandı: {:?}", parent));
            }
        }

        report.steps.push(StepLog {
            step_number: 3,
            step_name: "Dosya Sistemi İmhası / Filesystem Eradication".to_string(),
            details: step3_details,
            success: true,
        });

        report
    }

    pub fn quick_health_cleanup(&self) -> UninstallReport {
        let mut report = UninstallReport {
            app_name: "Sistem Sağlığı & Derin Bakım / System Health Optimization".to_string(),
            steps: Vec::new(),
            deleted_files_count: 0,
            deleted_reg_count: 0,
            terminated_procs_count: 0,
            stopped_services_count: 0,
            total_freed_bytes: 0,
            requires_reboot: false,
            restore_point_created: false,
            registry_backup_path: None,
        };

        let mut dump_details = Vec::new();
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let dumps = PathBuf::from(local).join("CrashDumps");
            if dumps.exists() {
                if let Ok(entries) = fs::read_dir(&dumps) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() {
                            let size = entry.metadata().map_or(0, |m| m.len());
                            if fs::remove_file(&path).is_ok() {
                                dump_details.push(format!("Silinen Crash Dump: {}", path.to_string_lossy()));
                                report.deleted_files_count += 1;
                                report.total_freed_bytes += size;
                            }
                        }
                    }
                }
            }
        }
        report.steps.push(StepLog {
            step_number: 1,
            step_name: "Çökme Raporları Temizliği (Crash Dumps)".to_string(),
            details: dump_details,
            success: true,
        });

        let mut pf_details = Vec::new();
        let pf_dir = Path::new("C:\\Windows\\Prefetch");
        if pf_dir.exists() {
            if let Ok(entries) = fs::read_dir(pf_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() {
                        let size = entry.metadata().map_or(0, |m| m.len());
                        if fs::remove_file(&path).is_ok() {
                            pf_details.push(format!("Temizlenen Prefetch: {}", path.to_string_lossy()));
                            report.deleted_files_count += 1;
                            report.total_freed_bytes += size;
                        }
                    }
                }
            }
        }
        report.steps.push(StepLog {
            step_number: 2,
            step_name: "Windows Prefetch Önbellek Temizliği".to_string(),
            details: pf_details,
            success: true,
        });

        report
    }
}

fn run_shell_command(cmd: &str, timeout_secs: u64) -> bool {
    let child = Command::new("cmd")
        .args(&["/C", cmd])
        .spawn();

    if let Ok(mut c) = child {
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(timeout_secs) {
            if let Ok(Some(status)) = c.try_wait() {
                return status.success();
            }
            sleep(Duration::from_millis(250));
        }
        let _ = c.kill();
    }
    false
}

fn is_dir_empty(path: &Path) -> bool {
    if let Ok(mut entries) = fs::read_dir(path) {
        entries.next().is_none()
    } else {
        false
    }
}
