use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

use crate::ui::format_bytes;

#[derive(Debug, Clone)]
pub struct BloatwareTarget {
    pub id: &'static str,
    pub name: &'static str,
    pub package_pattern: &'static str,
    pub category: &'static str,
    pub description: &'static str,
}

pub const KNOWN_BLOATWARE: &[BloatwareTarget] = &[
    BloatwareTarget {
        id: "cortana",
        name: "Cortana",
        package_pattern: "*Microsoft.549981C3F5F10*",
        category: "Asistan",
        description: "Windows sesli asistanı ve arka plan veri toplayıcısı",
    },
    BloatwareTarget {
        id: "bing_news",
        name: "Microsoft News / Bing Haberler",
        package_pattern: "*Microsoft.BingNews*",
        category: "Haberler",
        description: "Haber ve ilgi alanları arka plan beslemesi",
    },
    BloatwareTarget {
        id: "bing_weather",
        name: "MSN Hava Durumu",
        package_pattern: "*Microsoft.BingWeather*",
        category: "Haberler",
        description: "Konum ve hava durumu takip uygulaması",
    },
    BloatwareTarget {
        id: "solitaire",
        name: "Microsoft Solitaire Collection",
        package_pattern: "*Microsoft.MicrosoftSolitaireCollection*",
        category: "Oyunlar",
        description: "Reklamlı kart oyunu paketi",
    },
    BloatwareTarget {
        id: "feedback_hub",
        name: "Geri Bildirim Merkezi (Feedback Hub)",
        package_pattern: "*Microsoft.WindowsFeedbackHub*",
        category: "Telemetri",
        description: "Kullanıcı geri bildirimi ve tanılama aracı",
    },
    BloatwareTarget {
        id: "get_help",
        name: "Yardım Alın (Get Help)",
        package_pattern: "*Microsoft.GetHelp*",
        category: "Sistem",
        description: "Gereksiz online yardım yardımcısı",
    },
    BloatwareTarget {
        id: "tips",
        name: "Windows İpuçları (Microsoft Tips)",
        package_pattern: "*Microsoft.Getstarted*",
        category: "Sistem",
        description: "Bildirim ve reklam üreten ipucu uygulaması",
    },
    BloatwareTarget {
        id: "3d_viewer",
        name: "3D Görüntüleyici",
        package_pattern: "*Microsoft.Microsoft3DViewer*",
        category: "Medya",
        description: "Gereksiz 3D model görüntüleme paketi",
    },
    BloatwareTarget {
        id: "mixed_reality",
        name: "Karma Gerçeklik Portalı",
        package_pattern: "*Microsoft.MixedReality.Portal*",
        category: "VR/AR",
        description: "Kullanılmayan sanal gerçeklik sistem bileşeni",
    },
    BloatwareTarget {
        id: "phone_link",
        name: "Telefon Bağlantısı (Phone Link)",
        package_pattern: "*Microsoft.YourPhone*",
        category: "Mobil",
        description: "Arka planda çalışan telefon eşleme servisi",
    },
    BloatwareTarget {
        id: "teams_personal",
        name: "Microsoft Teams (Kişisel/Chat)",
        package_pattern: "*MicrosoftTeams*",
        category: "İletişim",
        description: "Windows 11 ile gelen entegre sohbet uygulaması",
    },
];

pub struct DebloaterEngine;

impl DebloaterEngine {
    /// Write action entry to %APPDATA%\WinRaze\raze.log
    pub fn log_action(entry: &str) {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let log_dir = PathBuf::from(appdata).join("WinRaze");
            let _ = fs::create_dir_all(&log_dir);
            let log_path = log_dir.join("raze.log");

            let timestamp = chrono_lite_timestamp();
            let line = format!("[{}] {}\n", timestamp, entry);

            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
                let _ = file.write_all(line.as_bytes());
            }
        }
    }

    /// Disable Windows Diagnostic Data, Telemetry Services, and Tracking
    pub fn disable_telemetry() -> Vec<String> {
        let mut logs = Vec::new();
        logs.push("Telemetri ve tanılama engelleme protokolü başlatıldı...".to_string());

        // 1. Registry DataCollection Policies
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok((key, _)) = hklm.create_subkey("SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection") {
            let _ = key.set_value("AllowTelemetry", &0u32);
            let _ = key.set_value("DoNotShowFeedbackNotifications", &1u32);
            logs.push("Kayıt Defteri: AllowTelemetry = 0 ve Geri bildirim bildirimleri kapatıldı.".to_string());
        }

        // 2. Advertising ID
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok((key, _)) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\AdvertisingInfo") {
            let _ = key.set_value("Enabled", &0u32);
            logs.push("Kayıt Defteri: Windows Reklam Kimliği (Advertising ID) devre dışı bırakıldı.".to_string());
        }

        // 3. Tailored Experiences
        if let Ok((key, _)) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Privacy") {
            let _ = key.set_value("TailoredExperiencesWithDiagnosticDataEnabled", &0u32);
            logs.push("Kayıt Defteri: Kişiselleştirilmiş deneyimler ve veri toplama engellendi.".to_string());
        }

        // 4. Disable Cortana Search
        if let Ok((key, _)) = hklm.create_subkey("SOFTWARE\\Policies\\Microsoft\\Windows\\Windows Search") {
            let _ = key.set_value("AllowCortana", &0u32);
            logs.push("Kayıt Defteri: Cortana arama entegrasyonu kapatıldı.".to_string());
        }

        // 5. Stop and Disable DiagTrack service
        let s1 = Command::new("sc").args(&["stop", "DiagTrack"]).output();
        let s2 = Command::new("sc").args(&["config", "DiagTrack", "start=disabled"]).output();
        if s1.is_ok() || s2.is_ok() {
            logs.push("Windows Servisi: DiagTrack (Bağlı Kullanıcı Deneyimleri ve Telemetri) durduruldu ve devre dışı bırakıldı.".to_string());
        }

        // 6. Stop and Disable dmwappushservice
        let _ = Command::new("sc").args(&["stop", "dmwappushservice"]).output();
        let _ = Command::new("sc").args(&["config", "dmwappushservice", "start=disabled"]).output();
        logs.push("Windows Servisi: dmwappushservice (WAP Push Yönlendirme Servisi) devre dışı bırakıldı.".to_string());

        Self::log_action("Telemetri ve tanılama servisleri engellendi.");
        logs
    }

    /// Remove a known bloatware package pattern
    pub fn remove_bloatware(target: &BloatwareTarget) -> bool {
        let ps_cmd = format!(
            "Get-AppxPackage -AllUsers {} | Remove-AppxPackage -AllUsers -ErrorAction SilentlyContinue",
            target.package_pattern
        );

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
            .output();

        let success = output.map_or(false, |o| o.status.success());
        Self::log_action(&format!("Bloatware Kaldırıldı: {} ({})", target.name, target.package_pattern));
        success
    }

    /// Clean Temporary Files, Windows Temp, and Browser Caches (`clean --temp`)
    pub fn clean_temporary_files() -> (usize, u64, Vec<String>) {
        let mut deleted_count = 0;
        let mut freed_bytes = 0u64;
        let mut logs = Vec::new();

        logs.push("Geçici dosya ve önbellek temizleme protokolü başlatıldı...".to_string());

        // 1. User Temp Directory (%TEMP%)
        if let Ok(temp_env) = std::env::var("TEMP") {
            let (count, bytes) = clean_directory_contents(Path::new(&temp_env));
            deleted_count += count;
            freed_bytes += bytes;
            logs.push(format!("Kullanıcı Temp Dizini: {} dosya temizlendi ({})", count, format_bytes(bytes)));
        }

        // 2. Windows Temp Directory (C:\Windows\Temp)
        let win_temp = Path::new("C:\\Windows\\Temp");
        if win_temp.exists() {
            let (count, bytes) = clean_directory_contents(win_temp);
            deleted_count += count;
            freed_bytes += bytes;
            logs.push(format!("Windows Sistem Temp: {} dosya temizlendi ({})", count, format_bytes(bytes)));
        }

        // 3. Browser Cache Folders (Chrome, Edge, Brave)
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let local_path = PathBuf::from(&local_appdata);
            let cache_targets = [
                local_path.join("Google\\Chrome\\User Data\\Default\\Cache"),
                local_path.join("Microsoft\\Edge\\User Data\\Default\\Cache"),
                local_path.join("BraveSoftware\\Brave-Browser\\User Data\\Default\\Cache"),
                local_path.join("Microsoft\\Windows\\Explorer"), // Thumbnail caches
            ];

            for cache_dir in &cache_targets {
                if cache_dir.exists() {
                    let (count, bytes) = clean_directory_contents(cache_dir);
                    if count > 0 {
                        deleted_count += count;
                        freed_bytes += bytes;
                        logs.push(format!("Önbellek Temizlendi ({}): {} dosya ({})", cache_dir.file_name().unwrap_or_default().to_string_lossy(), count, format_bytes(bytes)));
                    }
                }
            }
        }

        Self::log_action(&format!("Geçici Dosyalar Temizlendi: {} dosya, {}", deleted_count, format_bytes(freed_bytes)));
        (deleted_count, freed_bytes, logs)
    }
}

fn clean_directory_contents(dir: &Path) -> (usize, u64) {
    let mut count = 0;
    let mut bytes = 0u64;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Ok(meta) = path.metadata() {
                let size = meta.len();
                if meta.is_file() {
                    if fs::remove_file(&path).is_ok() {
                        count += 1;
                        bytes += size;
                    }
                } else if meta.is_dir() {
                    if fs::remove_dir_all(&path).is_ok() {
                        count += 1;
                        bytes += size;
                    }
                }
            }
        }
    }

    (count, bytes)
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    format!("{now}")
}
