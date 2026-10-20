use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use winreg::enums::*;
use winreg::{RegKey, HKEY};

use crate::model::{AppType, ArtifactKind, InstalledApp, ResidualArtifact, ScanConfig, SystemHealthStats};

pub struct SystemScanner {
    config: ScanConfig,
}

impl SystemScanner {
    pub fn new(config: ScanConfig) -> Self {
        Self { config }
    }

    pub fn scan_all(&self) -> Vec<InstalledApp> {
        let mut apps = Vec::new();

        // 1. Scan Standard Registry Uninstall keys
        self.scan_registry_hive(HKEY_LOCAL_MACHINE, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall", false, &mut apps);
        self.scan_registry_hive(HKEY_LOCAL_MACHINE, "Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall", true, &mut apps);
        self.scan_registry_hive(HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall", false, &mut apps);

        // 2. Scan Store / AppX packages if enabled
        if self.config.include_store_apps {
            self.scan_store_apps(&mut apps);
        }

        // 3. Scan for Leftovers if enabled
        if self.config.detect_leftovers {
            self.scan_orphaned_leftovers(&mut apps);
        }

        // 4. Scan Browser Extensions (Chrome, Edge, Brave)
        self.scan_browser_extensions(&mut apps);

        // Deduplicate apps by name & install location
        let mut deduplicated: Vec<InstalledApp> = Vec::new();
        for app in apps {
            if let Some(existing) = deduplicated.iter_mut().find(|a| {
                a.name.eq_ignore_ascii_case(&app.name) && 
                (a.install_location == app.install_location || app.install_location.is_none())
            }) {
                // Merge artifacts
                for art in app.artifacts {
                    if !existing.artifacts.iter().any(|ea| ea.path_or_target == art.path_or_target) {
                        existing.artifacts.push(art);
                    }
                }
            } else {
                deduplicated.push(app);
            }
        }

        // Deepen artifacts collection (shortcuts, appdata, prefetch, registry)
        for app in &mut deduplicated {
            if app.app_type != AppType::Tracked {
                self.enrich_app_artifacts(app);
            }
        }

        // 5. Append Tracked Installation Profiles
        let tracked_profiles = crate::tracker::InstallTracker::load_all_profiles();
        for profile in tracked_profiles {
            deduplicated.push(crate::tracker::InstallTracker::profile_to_installed_app(&profile));
        }

        deduplicated.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        deduplicated
    }

    fn scan_registry_hive(&self, hive: HKEY, subkey_path: &str, _is_wow64: bool, results: &mut Vec<InstalledApp>) {
        let root = RegKey::predef(hive);
        let uninstall_key = match root.open_subkey_with_flags(subkey_path, KEY_READ) {
            Ok(k) => k,
            Err(_) => return,
        };

        for subkey_name in uninstall_key.enum_keys().filter_map(|r| r.ok()) {
            if let Ok(app_key) = uninstall_key.open_subkey_with_flags(&subkey_name, KEY_READ) {
                let display_name: String = match app_key.get_value("DisplayName") {
                    Ok(val) => val,
                    Err(_) => continue, // Without DisplayName, Windows doesn't show it in Apps list
                };

                let display_name = display_name.trim();
                if display_name.is_empty() {
                    continue;
                }

                // Check SystemComponent flag
                let is_system_comp: u32 = app_key.get_value("SystemComponent").unwrap_or(0);
                if is_system_comp == 1 {
                    continue;
                }

                let publisher: Option<String> = app_key.get_value("Publisher").ok();
                let version: Option<String> = app_key.get_value("DisplayVersion").ok();
                let install_loc_str: Option<String> = app_key.get_value("InstallLocation").ok();
                let uninstall_str: Option<String> = app_key.get_value("UninstallString").ok();
                let quiet_str: Option<String> = app_key.get_value("QuietUninstallString").ok();
                let install_date: Option<String> = app_key.get_value("InstallDate").ok();

                let mut estimated_size: u64 = app_key.get_value::<u32, _>("EstimatedSize").unwrap_or(0) as u64 * 1024;

                let install_location = install_loc_str
                    .as_ref()
                    .map(|s| PathBuf::from(s.trim_matches('"')))
                    .filter(|p| p.exists());

                // If size is 0 and install_location exists, compute directory size
                if estimated_size == 0 {
                    if let Some(ref path) = install_location {
                        estimated_size = calculate_dir_size(path);
                    }
                }

                let mut app_type = AppType::Win32;
                let windows_installer: u32 = app_key.get_value("WindowsInstaller").unwrap_or(0);
                if windows_installer == 1 || uninstall_str.as_ref().map_or(false, |s| s.to_lowercase().contains("msiexec")) {
                    app_type = AppType::Msi;
                }

                let full_reg_path = format!(
                    "{}\\{}\\{}",
                    if hive == HKEY_LOCAL_MACHINE { "HKLM" } else { "HKCU" },
                    subkey_path,
                    subkey_name
                );

                let mut artifacts = Vec::new();
                artifacts.push(ResidualArtifact {
                    kind: ArtifactKind::RegistryKey,
                    path_or_target: full_reg_path.clone(),
                    description: "Main Uninstall Registry Key".to_string(),
                    size_bytes: None,
                });

                if let Some(ref loc) = install_location {
                    artifacts.push(ResidualArtifact {
                        kind: ArtifactKind::Directory,
                        path_or_target: loc.to_string_lossy().to_string(),
                        description: "Installation Root Directory".to_string(),
                        size_bytes: Some(estimated_size),
                    });
                }

                let is_protected = is_system_protected_component(&display_name, publisher.as_deref());

                results.push(InstalledApp {
                    id: subkey_name,
                    name: display_name.to_string(),
                    publisher,
                    version,
                    install_location,
                    main_executable: None,
                    main_shortcut: None,
                    uninstall_string: uninstall_str,
                    quiet_uninstall_string: quiet_str,
                    registry_key: Some(full_reg_path),
                    app_type,
                    estimated_size_bytes: estimated_size,
                    install_date,
                    artifacts,
                    is_protected,
                });
            }
        }
    }

    fn scan_store_apps(&self, results: &mut Vec<InstalledApp>) {
        // Enumerate Windows Store apps via PowerShell query
        let output = std::process::Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-Command",
                "Get-AppxPackage | Where-Object { -not $_.IsFramework -and $_.NonRemovable -ne $true } | Select-Object Name, PackageFullName, InstallLocation, Publisher | ConvertTo-Json"
            ])
            .output();

        if let Ok(out) = output {
            if let Ok(json_str) = String::from_utf8(out.stdout) {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    let items = if let Some(arr) = value.as_array() {
                        arr.clone()
                    } else if value.is_object() {
                        vec![value]
                    } else {
                        vec![]
                    };

                    for item in items {
                        let name = item["Name"].as_str().unwrap_or("").to_string();
                        let package_full_name = item["PackageFullName"].as_str().unwrap_or("").to_string();
                        let install_loc = item["InstallLocation"].as_str().unwrap_or("").to_string();
                        let publisher = item["Publisher"].as_str().map(|s| s.to_string());

                        if name.is_empty() || package_full_name.is_empty() {
                            continue;
                        }

                        let loc_path = if !install_loc.is_empty() {
                            Some(PathBuf::from(&install_loc))
                        } else {
                            None
                        };

                        let size = loc_path.as_ref().map_or(0, |p| calculate_dir_size(p));

                        let mut artifacts = Vec::new();
                        if let Some(ref p) = loc_path {
                            artifacts.push(ResidualArtifact {
                                kind: ArtifactKind::Directory,
                                path_or_target: p.to_string_lossy().to_string(),
                                description: "AppX Install Directory".to_string(),
                                size_bytes: Some(size),
                            });
                        }

                        results.push(InstalledApp {
                            id: package_full_name.clone(),
                            name,
                            publisher,
                            version: None,
                            install_location: loc_path,
                            main_executable: None,
                            main_shortcut: None,
                            uninstall_string: Some(format!("powershell -Command Remove-AppxPackage -Package {}", package_full_name)),
                            quiet_uninstall_string: Some(format!("powershell -Command Remove-AppxPackage -Package {}", package_full_name)),
                            registry_key: None,
                            app_type: AppType::AppXStore,
                            estimated_size_bytes: size,
                            install_date: None,
                            artifacts,
                            is_protected: false,
                        });
                    }
                }
            }
        }
    }

    fn scan_orphaned_leftovers(&self, results: &mut Vec<InstalledApp>) {
        // Look in AppData and ProgramData for orphaned vendor directories
        let candidates = vec![
            get_env_path("PROGRAMDATA"),
            get_env_path("LOCALAPPDATA"),
            get_env_path("APPDATA"),
        ];

        for base_opt in candidates {
            if let Some(base) = base_opt {
                if let Ok(entries) = fs::read_dir(&base) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_dir() {
                            let dir_name = entry.file_name().to_string_lossy().to_string();
                            // Skip standard system folders
                            if is_windows_system_dir(&dir_name) {
                                continue;
                            }

                            // If not matched by any existing app
                            let matched = results.iter().any(|app| {
                                app.name.eq_ignore_ascii_case(&dir_name) ||
                                app.publisher.as_ref().map_or(false, |p| p.eq_ignore_ascii_case(&dir_name)) ||
                                app.install_location.as_ref().map_or(false, |loc| loc.to_string_lossy().contains(&dir_name))
                            });

                            if !matched {
                                let size = calculate_dir_size(&path);
                                if size > 0 {
                                    results.push(InstalledApp {
                                        id: format!("leftover_{}", dir_name),
                                        name: format!("{} [Leftover]", dir_name),
                                        publisher: None,
                                        version: None,
                                        install_location: Some(path.clone()),
                                        main_executable: None,
                                        main_shortcut: None,
                                        uninstall_string: None,
                                        quiet_uninstall_string: None,
                                        registry_key: None,
                                        app_type: AppType::Leftover,
                                        estimated_size_bytes: size,
                                        install_date: None,
                                        artifacts: vec![ResidualArtifact {
                                            kind: ArtifactKind::Directory,
                                            path_or_target: path.to_string_lossy().to_string(),
                                            description: "Orphaned Application / Vendor Directory".to_string(),
                                            size_bytes: Some(size),
                                        }],
                                        is_protected: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn enrich_app_artifacts(&self, app: &mut InstalledApp) {
        let name_lower = app.name.to_lowercase();
        let pub_lower = app.publisher.as_ref().map(|p| p.to_lowercase());

        // 1. Locate shortcuts in Start Menu & Desktop
        let shortcut_dirs = vec![
            get_env_path("PROGRAMDATA").map(|p| p.join("Microsoft\\Windows\\Start Menu\\Programs")),
            get_env_path("APPDATA").map(|p| p.join("Microsoft\\Windows\\Start Menu\\Programs")),
            get_env_path("PUBLIC").map(|p| p.join("Desktop")),
            get_env_path("USERPROFILE").map(|p| p.join("Desktop")),
        ];

        for dir_opt in shortcut_dirs.into_iter().flatten() {
            if dir_opt.exists() {
                for entry in WalkDir::new(&dir_opt).max_depth(3).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext.eq_ignore_ascii_case("lnk") || ext.eq_ignore_ascii_case("url") {
                            let file_stem = path.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
                            if file_stem.contains(&name_lower) || name_lower.contains(&file_stem) {
                                let target_str = path.to_string_lossy().to_string();
                                if !app.artifacts.iter().any(|a| a.path_or_target == target_str) {
                                    if app.main_shortcut.is_none() {
                                        app.main_shortcut = Some(path.to_path_buf());
                                    }
                                    app.artifacts.push(ResidualArtifact {
                                        kind: ArtifactKind::Shortcut,
                                        path_or_target: target_str,
                                        description: "Application Shortcut".to_string(),
                                        size_bytes: Some(entry.metadata().map_or(0, |m| m.len())),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Discover AppData & ProgramData leftover folders
        let appdata_dirs = vec![
            get_env_path("LOCALAPPDATA"),
            get_env_path("APPDATA"),
            get_env_path("PROGRAMDATA"),
        ];

        for base_opt in appdata_dirs.into_iter().flatten() {
            if let Ok(entries) = fs::read_dir(&base_opt) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let item_name = entry.file_name().to_string_lossy().to_lowercase();
                        let is_match = item_name == name_lower || 
                            pub_lower.as_ref().map_or(false, |p| &item_name == p);

                        if is_match {
                            let target_str = path.to_string_lossy().to_string();
                            if !app.artifacts.iter().any(|a| a.path_or_target == target_str) {
                                let size = calculate_dir_size(&path);
                                app.artifacts.push(ResidualArtifact {
                                    kind: ArtifactKind::Directory,
                                    path_or_target: target_str,
                                    description: "Data / Configuration Directory".to_string(),
                                    size_bytes: Some(size),
                                });
                            }
                        }
                    }
                }
            }
        }

        // 3. Discover Prefetch files (*.pf)
        let prefetch_dir = Path::new("C:\\Windows\\Prefetch");
        if prefetch_dir.exists() {
            if let Ok(entries) = fs::read_dir(prefetch_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let file_name = entry.file_name().to_string_lossy().to_lowercase();
                    if file_name.ends_with(".pf") && file_name.contains(&name_lower) {
                        let target_str = entry.path().to_string_lossy().to_string();
                        if !app.artifacts.iter().any(|a| a.path_or_target == target_str) {
                            app.artifacts.push(ResidualArtifact {
                                kind: ArtifactKind::Prefetch,
                                path_or_target: target_str,
                                description: "Windows Prefetch Cache Entry".to_string(),
                                size_bytes: Some(entry.metadata().map_or(0, |m| m.len())),
                            });
                        }
                    }
                }
            }
        }

        // 4. Discover Vendor Registry Keys under HKCU and HKLM
        let vendor_subkeys = vec![
            (HKEY_CURRENT_USER, "Software"),
            (HKEY_LOCAL_MACHINE, "Software"),
            (HKEY_LOCAL_MACHINE, "Software\\Wow6432Node"),
        ];

        for (hive, sub) in vendor_subkeys {
            let root = RegKey::predef(hive);
            if let Ok(key) = root.open_subkey_with_flags(sub, KEY_READ) {
                for subname in key.enum_keys().filter_map(|r| r.ok()) {
                    let sub_lower = subname.to_lowercase();
                    let matches_name = sub_lower == name_lower;
                    let matches_pub = pub_lower.as_ref().map_or(false, |p| &sub_lower == p);

                    if matches_name || matches_pub {
                        let reg_path = format!(
                            "{}\\{}\\{}",
                            if hive == HKEY_CURRENT_USER { "HKCU" } else { "HKLM" },
                            sub,
                            subname
                        );
                        if !app.artifacts.iter().any(|a| a.path_or_target == reg_path) {
                            app.artifacts.push(ResidualArtifact {
                                kind: ArtifactKind::RegistryKey,
                                path_or_target: reg_path,
                                description: "Application / Vendor Registry Subkey".to_string(),
                                size_bytes: None,
                            });
                        }
                    }
                }
            }
        }

        // 5. Discover Crash Dumps in %LOCALAPPDATA%\CrashDumps
        if let Some(local_appdata) = get_env_path("LOCALAPPDATA") {
            let dumps_dir = local_appdata.join("CrashDumps");
            if dumps_dir.exists() {
                if let Ok(entries) = fs::read_dir(&dumps_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let fname = entry.file_name().to_string_lossy().to_lowercase();
                        if fname.ends_with(".dmp") && (fname.contains(&name_lower) || name_lower.contains(&fname.trim_end_matches(".dmp"))) {
                            let target_str = entry.path().to_string_lossy().to_string();
                            if !app.artifacts.iter().any(|a| a.path_or_target == target_str) {
                                let size = entry.metadata().map_or(0, |m| m.len());
                                app.artifacts.push(ResidualArtifact {
                                    kind: ArtifactKind::CrashDump,
                                    path_or_target: target_str,
                                    description: "Application Crash Dump File".to_string(),
                                    size_bytes: Some(size),
                                });
                            }
                        }
                    }
                }
            }
        }

        // 6. Discover Shell Context Menu Handlers (Explorer integration)
        let shellex_roots = [
            (HKEY_CLASSES_ROOT, "*\\shellex\\ContextMenuHandlers"),
            (HKEY_CLASSES_ROOT, "Directory\\shellex\\ContextMenuHandlers"),
            (HKEY_CLASSES_ROOT, "Folder\\shellex\\ContextMenuHandlers"),
        ];

        for (hive, sub) in shellex_roots {
            let root = RegKey::predef(hive);
            if let Ok(key) = root.open_subkey_with_flags(sub, KEY_READ) {
                for subname in key.enum_keys().filter_map(|r| r.ok()) {
                    let sub_lower = subname.to_lowercase();
                    if sub_lower.contains(&name_lower) || name_lower.contains(&sub_lower) {
                        let reg_path = format!("HKCR\\{}\\{}", sub, subname);
                        if !app.artifacts.iter().any(|a| a.path_or_target == reg_path) {
                            app.artifacts.push(ResidualArtifact {
                                kind: ArtifactKind::RegistryKey,
                                path_or_target: reg_path,
                                description: "Explorer Context Menu Shell Extension Handler".to_string(),
                                size_bytes: None,
                            });
                        }
                    }
                }
            }
        }

        // 7. Discover Startup Approved entries in HKLM and HKCU
        let startup_keys = [
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run32"),
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run"),
            (HKEY_CURRENT_USER, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run"),
        ];

        for (hive, sub) in startup_keys {
            let root = RegKey::predef(hive);
            if let Ok(key) = root.open_subkey_with_flags(sub, KEY_READ) {
                for (val_name, _) in key.enum_values().filter_map(|r| r.ok()) {
                    let val_lower = val_name.to_lowercase();
                    if val_lower.contains(&name_lower) || name_lower.contains(&val_lower) {
                        let reg_path = format!("{}\\{} -> {}", if hive == HKEY_LOCAL_MACHINE { "HKLM" } else { "HKCU" }, sub, val_name);
                        if !app.artifacts.iter().any(|a| a.path_or_target == reg_path) {
                            app.artifacts.push(ResidualArtifact {
                                kind: ArtifactKind::RegistryValue,
                                path_or_target: reg_path,
                                description: "Windows Startup Item".to_string(),
                                size_bytes: None,
                            });
                        }
                    }
                }
            }
        }
    }

    fn scan_browser_extensions(&self, results: &mut Vec<InstalledApp>) {
        let local_appdata = match get_env_path("LOCALAPPDATA") {
            Some(p) => p,
            None => return,
        };

        let browser_paths = [
            ("Google Chrome", local_appdata.join("Google\\Chrome\\User Data\\Default\\Extensions")),
            ("Microsoft Edge", local_appdata.join("Microsoft\\Edge\\User Data\\Default\\Extensions")),
            ("Brave Browser", local_appdata.join("BraveSoftware\\Brave-Browser\\User Data\\Default\\Extensions")),
        ];

        for (browser_name, ext_root) in browser_paths {
            if !ext_root.exists() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&ext_root) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let ext_id_path = entry.path();
                    if !ext_id_path.is_dir() {
                        continue;
                    }

                    let ext_id = entry.file_name().to_string_lossy().to_string();
                    if ext_id.len() < 10 {
                        continue;
                    }

                    let mut ext_name = format!("{} Eklentisi ({})", browser_name, &ext_id[..ext_id.len().min(8)]);
                    let mut version = None;
                    let size = calculate_dir_size(&ext_id_path);

                    if let Ok(ver_entries) = fs::read_dir(&ext_id_path) {
                        for ver_entry in ver_entries.filter_map(|e| e.ok()) {
                            let ver_path = ver_entry.path();
                            if ver_path.is_dir() {
                                let manifest_path = ver_path.join("manifest.json");
                                if manifest_path.exists() {
                                    if let Ok(manifest_content) = fs::read_to_string(&manifest_path) {
                                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&manifest_content) {
                                            if let Some(n) = v.get("name").and_then(|n| n.as_str()) {
                                                if !n.starts_with("__MSG_") && !n.is_empty() {
                                                    ext_name = format!("{} ({})", n, browser_name);
                                                }
                                            }
                                            if let Some(ver) = v.get("version").and_then(|v| v.as_str()) {
                                                version = Some(ver.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let target_str = ext_id_path.to_string_lossy().to_string();
                    results.push(InstalledApp {
                        id: format!("ext_{}_{}", browser_name.replace(' ', "_").to_lowercase(), ext_id),
                        name: ext_name,
                        publisher: Some(browser_name.to_string()),
                        version,
                        install_location: Some(ext_id_path.clone()),
                        main_executable: None,
                        main_shortcut: None,
                        uninstall_string: None,
                        quiet_uninstall_string: None,
                        registry_key: None,
                        app_type: AppType::BrowserExtension,
                        estimated_size_bytes: size,
                        install_date: None,
                        artifacts: vec![ResidualArtifact {
                            kind: ArtifactKind::Directory,
                            path_or_target: target_str,
                            description: format!("{} Eklenti Dizini", browser_name),
                            size_bytes: Some(size),
                        }],
                        is_protected: false,
                    });
                }
            }
        }
    }

    pub fn calculate_system_health(&self, apps: &[InstalledApp]) -> SystemHealthStats {
        let mut stats = SystemHealthStats::default();

        for app in apps {
            if app.app_type == AppType::Leftover {
                stats.leftover_count += 1;
                stats.leftover_bytes += app.estimated_size_bytes;
            }
            if app.is_protected {
                stats.protected_count += 1;
            }
            for art in &app.artifacts {
                if art.kind == ArtifactKind::CrashDump {
                    stats.crash_dump_count += 1;
                    stats.crash_dump_bytes += art.size_bytes.unwrap_or(0);
                } else if art.kind == ArtifactKind::Shortcut {
                    let p = Path::new(&art.path_or_target);
                    if !p.exists() {
                        stats.broken_shortcuts_count += 1;
                    }
                }
            }
        }

        let leftover_penalty = (stats.leftover_count as u32 * 2).min(30);
        let crash_penalty = (stats.crash_dump_count as u32 * 3).min(30);
        let shortcut_penalty = (stats.broken_shortcuts_count as u32 * 2).min(20);
        let total_penalty = leftover_penalty + crash_penalty + shortcut_penalty;

        stats.health_score = 100u32.saturating_sub(total_penalty).max(10);
        stats
    }
}

pub fn is_system_protected_component(name: &str, publisher: Option<&str>) -> bool {
    let lower_name = name.to_lowercase();
    let lower_pub = publisher.unwrap_or("").to_lowercase();

    // Critical hardware drivers & system runtimes
    let protected_keywords = [
        "visual c++",
        "redistributable",
        "directx",
        "nvidia graphics driver",
        "nvidia physx",
        "nvidia geforce",
        "amd software",
        "amd radeon",
        "intel(r) graphics",
        "intel graphics",
        "realtek high definition audio",
        "realtek audio",
        "microsoft .net",
        ".net framework",
        "webview2 runtime",
        "windows defender",
        "microsoft edge",
    ];

    if protected_keywords.iter().any(|k| lower_name.contains(k)) {
        return true;
    }

    if lower_pub.contains("microsoft") && (lower_name.contains("framework") || lower_name.contains("runtime") || lower_name.contains("service")) {
        return true;
    }

    false
}

pub fn calculate_dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    if path.is_file() {
        return path.metadata().map_or(0, |m| m.len());
    }
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() {
                total += meta.len();
            }
        }
    }
    total
}

fn get_env_path(name: &str) -> Option<PathBuf> {
    std::env::var(name).ok().map(PathBuf::from)
}

fn is_windows_system_dir(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "microsoft" | "windows" | "system32" | "syswow64" | "packages" | "temp" | "history" | "crypto" | "usoshared" | "usermodule"
    )
}
