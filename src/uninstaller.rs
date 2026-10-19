use std::fs;
use std::path::Path;
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
        };

        println!("\n[+] Starting Uninstalr-RS 9-Step Pipeline for: {}", app.name);

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
        // Step 7 & 8: Deep Registry & Residuals Clean
        // ==========================================
        let mut step8_reg_details = Vec::new();
        for art in &app.artifacts {
            if art.kind == ArtifactKind::RegistryKey {
                if self.options.dry_run {
                    step8_reg_details.push(format!("[DRY RUN] Would delete registry key: {}", art.path_or_target));
                } else if Self::delete_registry_key_recursive(&art.path_or_target) {
                    step8_reg_details.push(format!("Deleted registry key: {}", art.path_or_target));
                    report.deleted_reg_count += 1;
                }
            }
        }
        report.steps.push(StepLog {
            step_number: 7,
            step_name: "Deep Registry Cleanup".to_string(),
            details: step8_reg_details,
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
        // Step 9: Post-Processing & Empty Dirs
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
        } else if lower.contains("uninstall.exe") || lower.contains("uninst.exe") {
            // NSIS or generic
            if !lower.contains("/s") {
                format!("{} /S", cmd)
            } else {
                cmd.to_string()
            }
        } else if lower.contains("msiexec") {
            Self::format_silent_msi_command(cmd)
        } else {
            cmd.to_string()
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
