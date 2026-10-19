use std::path::Path;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

pub struct ProcessKiller;

impl ProcessKiller {
    /// Terminates any processes whose executable path or name matches the target application
    pub fn terminate_app_processes(app_name: &str, _install_dir: Option<&Path>) -> Vec<String> {
        let mut terminated = Vec::new();
        let target_name_lower = app_name.to_lowercase();

        unsafe {
            let snapshot: HANDLE = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(h) => h,
                Err(_) => return terminated,
            };

            let mut entry = PROCESSENTRY32W::default();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let exe_name = String::from_utf16_lossy(&entry.szExeFile)
                        .trim_matches('\0')
                        .to_string();
                    let exe_lower = exe_name.to_lowercase();

                    let mut should_kill = false;

                    // Match by executable name containing app name (e.g. "driverbooster.exe" for "Driver Booster")
                    let normalized_app = target_name_lower.replace(' ', "");
                    let normalized_exe = exe_lower.replace(".exe", "").replace(' ', "");

                    if !normalized_app.is_empty() && (normalized_exe.contains(&normalized_app) || normalized_app.contains(&normalized_exe)) {
                        should_kill = true;
                    }

                    if should_kill && entry.th32ProcessID != std::process::id() {
                        if let Ok(proc_handle) = OpenProcess(PROCESS_TERMINATE, false, entry.th32ProcessID) {
                            if TerminateProcess(proc_handle, 1).is_ok() {
                                terminated.push(format!("PID {} ({})", entry.th32ProcessID, exe_name));
                            }
                            let _ = CloseHandle(proc_handle);
                        }
                    }

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
        }

        terminated
    }

    /// Stops any Windows service related to the app name
    pub fn stop_related_services(app_name: &str) -> Vec<String> {
        let mut stopped = Vec::new();
        let _name_lower = app_name.to_lowercase().replace(' ', "");

        // Query services via powershell
        let cmd = format!(
            "Get-Service | Where-Object {{ $_.Name -like '*{}*' -or $_.DisplayName -like '*{}*' }} | Select-Object -ExpandProperty Name",
            app_name, app_name
        );

        if let Ok(output) = std::process::Command::new("powershell")
            .args(&["-NoProfile", "-Command", &cmd])
            .output()
        {
            let svc_names = String::from_utf8_lossy(&output.stdout);
            for svc in svc_names.lines() {
                let svc_name = svc.trim();
                if !svc_name.is_empty() {
                    let stop_res = std::process::Command::new("sc")
                        .args(&["stop", svc_name])
                        .output();
                    if stop_res.is_ok() {
                        stopped.push(svc_name.to_string());
                    }
                }
            }
        }

        stopped
    }
}
