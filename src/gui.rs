use eframe::egui;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::debloater::{BloatwareTarget, DebloaterEngine, KNOWN_BLOATWARE};
use crate::i18n::{Language, Locales, Translations};
use crate::model::{AppType, InstalledApp, ScanConfig, SystemHealthStats, UninstallOptions, UninstallReport};
use crate::scanner::SystemScanner;
use crate::ui::format_bytes;
use crate::uninstaller::UninstallationEngine;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveNavTab {
    Dashboard,
    Applications,
    Tracker,
    Debloater,
    BrowserExtensions,
    Leftovers,
    ForceShredder,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterTab {
    All,
    Win32,
    Msi,
    Store,
    Tracked,
}

enum BackgroundMsg {
    ScanFinished {
        apps: Vec<InstalledApp>,
        health: SystemHealthStats,
    },
    UninstallProgress {
        step_number: u8,
        step_name: String,
        detail: String,
        progress: f32,
    },
    UninstallFinished(UninstallReport),
    QuickCleanupFinished(UninstallReport),
    ForceCleanFinished(UninstallReport),
    DebloatFinished(String),
    TrackerProgress(String),
    TrackerFinished(Result<PathBuf, String>),
}

pub struct UninstalrApp {
    apps: Vec<InstalledApp>,
    health_stats: Option<SystemHealthStats>,
    selected_ids: HashSet<String>,
    selected_bloatware: HashSet<String>,
    active_app_index: Option<usize>,
    search_query: String,
    filter_tab: FilterTab,
    active_nav: ActiveNavTab,
    is_scanning: bool,
    show_confirm_modal: bool,
    options: UninstallOptions,
    current_lang: Language,

    // Debloater state
    debloat_status_message: Option<String>,

    // Tracker state
    tracker_installer_input: String,
    tracker_custom_name: String,
    tracker_status_message: Option<String>,
    tracker_is_busy: bool,

    // Force Shredder state
    force_path_input: String,
    force_status_message: Option<String>,

    // Uninstallation live state
    is_uninstalling: bool,
    current_step: u8,
    current_step_name: String,
    current_progress: f32,
    live_logs: Vec<String>,
    last_report: Option<UninstallReport>,

    // Thread communication
    tx: Sender<BackgroundMsg>,
    rx: Receiver<BackgroundMsg>,
}

impl UninstalrApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();

        // Default: select non-critical bloatware
        let mut initial_bloat = HashSet::new();
        for b in KNOWN_BLOATWARE {
            if b.id != "phone_link" && b.id != "teams_personal" {
                initial_bloat.insert(b.id.to_string());
            }
        }

        let mut app = Self {
            apps: Vec::new(),
            health_stats: None,
            selected_ids: HashSet::new(),
            selected_bloatware: initial_bloat,
            active_app_index: None,
            search_query: String::new(),
            filter_tab: FilterTab::All,
            active_nav: ActiveNavTab::Dashboard,
            is_scanning: true,
            show_confirm_modal: false,
            options: UninstallOptions::default(),
            current_lang: Language::Turkish,
            debloat_status_message: None,
            tracker_installer_input: String::new(),
            tracker_custom_name: String::new(),
            tracker_status_message: None,
            tracker_is_busy: false,
            force_path_input: String::new(),
            force_status_message: None,
            is_uninstalling: false,
            current_step: 0,
            current_step_name: String::new(),
            current_progress: 0.0,
            live_logs: Vec::new(),
            last_report: None,
            tx,
            rx,
        };

        app.trigger_scan();
        app
    }

    fn trigger_track_installation(&mut self) {
        let installer_path = PathBuf::from(&self.tracker_installer_input);
        if !installer_path.exists() {
            self.tracker_status_message = Some("Kurulum dosyası bulunamadı / Installer file not found".to_string());
            return;
        }

        let app_name = if !self.tracker_custom_name.trim().is_empty() {
            self.tracker_custom_name.trim().to_string()
        } else {
            installer_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Tracked Application")
                .to_string()
        };

        self.tracker_is_busy = true;
        self.tracker_status_message = Some(self.t().tracker_status_presnap.to_string());

        let tx = self.tx.clone();
        let installer_path_clone = installer_path.clone();
        let installer_str = self.tracker_installer_input.clone();

        thread::spawn(move || {
            let _ = tx.send(BackgroundMsg::TrackerProgress("Pre-snapshot alınıyor...".to_string()));
            let pre_snap = crate::tracker::InstallTracker::take_snapshot();

            let _ = tx.send(BackgroundMsg::TrackerProgress("Kurulum çalıştırılıyor... Kurulum penceresini tamamlayın.".to_string()));
            match crate::tracker::InstallTracker::execute_installer_and_wait(&installer_path_clone) {
                Ok(_) => {
                    let _ = tx.send(BackgroundMsg::TrackerProgress("Post-snapshot taranıyor ve fark (Diff) hesaplanıyor...".to_string()));
                    let post_snap = crate::tracker::InstallTracker::take_snapshot();
                    let profile = crate::tracker::InstallTracker::compute_diff(&pre_snap, &post_snap, app_name, Some(installer_str));
                    let res = crate::tracker::InstallTracker::save_profile(&profile);
                    let _ = tx.send(BackgroundMsg::TrackerFinished(res));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundMsg::TrackerFinished(Err(e)));
                }
            }
        });
    }

    fn t(&self) -> &'static Translations {
        Locales::get(self.current_lang)
    }

    fn trigger_scan(&mut self) {
        self.is_scanning = true;
        self.selected_ids.clear();
        self.active_app_index = None;

        let tx = self.tx.clone();
        thread::spawn(move || {
            let scanner = SystemScanner::new(ScanConfig::default());
            let results = scanner.scan_all();
            let health = scanner.calculate_system_health(&results);
            let _ = tx.send(BackgroundMsg::ScanFinished {
                apps: results,
                health,
            });
        });
    }

    fn trigger_quick_health_cleanup(&mut self) {
        self.is_uninstalling = true;
        self.current_step = 1;
        self.current_step_name = "Hızlı Sağlık Optimizasyonu / Health Optimization".to_string();
        self.current_progress = 0.2;
        self.live_logs.clear();
        self.live_logs.push("Çökme raporları (CrashDumps) taranıyor...".to_string());
        self.last_report = None;

        let tx = self.tx.clone();
        let options = self.options.clone();

        thread::spawn(move || {
            let engine = UninstallationEngine::new(options);
            let report = engine.quick_health_cleanup();
            let _ = tx.send(BackgroundMsg::QuickCleanupFinished(report));
        });
    }

    fn trigger_force_clean(&mut self) {
        let path_str = self.force_path_input.trim().to_string();
        if path_str.is_empty() {
            return;
        }

        self.is_uninstalling = true;
        self.current_step = 1;
        self.current_step_name = format!("Zorla Kaldırılıyor: {}", path_str);
        self.current_progress = 0.1;
        self.live_logs.clear();
        self.last_report = None;

        let tx = self.tx.clone();
        let options = self.options.clone();
        let target_path = PathBuf::from(path_str);

        thread::spawn(move || {
            let engine = UninstallationEngine::new(options);
            let report = engine.force_clean_path(&target_path);
            let _ = tx.send(BackgroundMsg::ForceCleanFinished(report));
        });
    }

    fn trigger_uninstall_confirmed(&mut self) {
        self.show_confirm_modal = false;
        if self.selected_ids.is_empty() {
            return;
        }

        self.is_uninstalling = true;
        self.current_step = 0;
        self.current_step_name = "Kaldırma başlatılıyor / Starting pipeline...".to_string();
        self.current_progress = 0.05;
        self.live_logs.clear();
        self.last_report = None;

        let targets: Vec<InstalledApp> = self
            .apps
            .iter()
            .filter(|a| self.selected_ids.contains(&a.id))
            .cloned()
            .collect();

        let tx = self.tx.clone();
        let options = self.options.clone();

        thread::spawn(move || {
            let engine = UninstallationEngine::new(options);
            let total_targets = targets.len();

            for (idx, app) in targets.into_iter().enumerate() {
                let base_prog = idx as f32 / total_targets as f32;
                let step_prog = 1.0 / total_targets as f32;

                let _ = tx.send(BackgroundMsg::UninstallProgress {
                    step_number: 1,
                    step_name: format!("[{}/{}] {}", idx + 1, total_targets, app.name),
                    detail: format!("Kaldırma başlatıldı: {}", app.name),
                    progress: base_prog + step_prog * 0.1,
                });

                let report = engine.execute_uninstallation(&app);

                for step in &report.steps {
                    for d in &step.details {
                        let _ = tx.send(BackgroundMsg::UninstallProgress {
                            step_number: step.step_number,
                            step_name: step.step_name.clone(),
                            detail: d.clone(),
                            progress: base_prog + step_prog * (step.step_number as f32 / 9.0),
                        });
                    }
                }

                let _ = tx.send(BackgroundMsg::UninstallFinished(report));
            }
        });
    }

    fn trigger_disable_telemetry(&mut self) {
        self.is_uninstalling = true;
        self.current_step = 1;
        self.current_step_name = "Telemetri Servisleri Kapatılıyor...".to_string();
        self.current_progress = 0.5;
        self.live_logs.clear();
        self.last_report = None;

        let tx = self.tx.clone();
        thread::spawn(move || {
            let logs = DebloaterEngine::disable_telemetry();
            for log in logs {
                let _ = tx.send(BackgroundMsg::UninstallProgress {
                    step_number: 1,
                    step_name: "Telemetri Kapatma".to_string(),
                    detail: log,
                    progress: 0.9,
                });
            }
            let _ = tx.send(BackgroundMsg::DebloatFinished("Telemetri servisleri ve tanılama politikaları başarıyla engellendi.".to_string()));
        });
    }

    fn trigger_clean_temp(&mut self) {
        self.is_uninstalling = true;
        self.current_step = 1;
        self.current_step_name = "Geçici Dosyalar ve Önbellekler Temizleniyor...".to_string();
        self.current_progress = 0.3;
        self.live_logs.clear();
        self.last_report = None;

        let tx = self.tx.clone();
        thread::spawn(move || {
            let (count, bytes, logs) = DebloaterEngine::clean_temporary_files();
            for log in logs {
                let _ = tx.send(BackgroundMsg::UninstallProgress {
                    step_number: 1,
                    step_name: "Temp Temizliği".to_string(),
                    detail: log,
                    progress: 0.8,
                });
            }
            let msg = format!("Toplam {} geçici dosya temizlendi ({})", count, format_bytes(bytes));
            let _ = tx.send(BackgroundMsg::DebloatFinished(msg));
        });
    }

    fn trigger_raze_bloatware(&mut self) {
        if self.selected_bloatware.is_empty() {
            return;
        }

        self.is_uninstalling = true;
        self.current_step = 1;
        self.current_step_name = "Bloatware Paketleri Yerle Bir Ediliyor...".to_string();
        self.current_progress = 0.1;
        self.live_logs.clear();
        self.last_report = None;

        let selected = self.selected_bloatware.clone();
        let targets: Vec<BloatwareTarget> = KNOWN_BLOATWARE
            .iter()
            .filter(|b| selected.contains(b.id))
            .cloned()
            .collect();

        let tx = self.tx.clone();
        thread::spawn(move || {
            let total = targets.len();
            for (idx, target) in targets.into_iter().enumerate() {
                let prog = (idx + 1) as f32 / total as f32;
                let _ = tx.send(BackgroundMsg::UninstallProgress {
                    step_number: 1,
                    step_name: format!("[{}/{}] {}", idx + 1, total, target.name),
                    detail: format!("Kaldırılıyor: {}", target.name),
                    progress: prog * 0.9,
                });
                DebloaterEngine::remove_bloatware(&target);
            }
            let _ = tx.send(BackgroundMsg::DebloatFinished("Seçilen tüm bloatware paketleri sistemden temizlendi.".to_string()));
        });
    }

    fn remove_browser_extension(&mut self, app: &InstalledApp) {
        if let Some(ref path) = app.install_location {
            if path.exists() {
                let _ = fs::remove_dir_all(path);
            }
        }
        self.trigger_scan();
    }
}

impl eframe::App for UninstalrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle background channel messages
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                BackgroundMsg::ScanFinished { apps, health } => {
                    self.apps = apps;
                    self.health_stats = Some(health);
                    self.is_scanning = false;
                }
                BackgroundMsg::UninstallProgress {
                    step_number,
                    step_name,
                    detail,
                    progress,
                } => {
                    self.current_step = step_number;
                    self.current_step_name = step_name;
                    self.current_progress = progress;
                    self.live_logs.push(detail);
                }
                BackgroundMsg::UninstallFinished(report) => {
                    self.is_uninstalling = false;
                    self.last_report = Some(report);
                    self.selected_ids.clear();
                    self.trigger_scan();
                }
                BackgroundMsg::QuickCleanupFinished(report) => {
                    self.is_uninstalling = false;
                    self.last_report = Some(report);
                    self.trigger_scan();
                }
                BackgroundMsg::ForceCleanFinished(report) => {
                    self.is_uninstalling = false;
                    self.last_report = Some(report);
                    self.force_status_message = Some("Hedef yol ve ilişkili kayıtlar başarıyla silindi.".to_string());
                    self.trigger_scan();
                }
                BackgroundMsg::DebloatFinished(msg) => {
                    self.is_uninstalling = false;
                    self.debloat_status_message = Some(msg);
                    self.trigger_scan();
                }
                BackgroundMsg::TrackerProgress(msg) => {
                    self.tracker_status_message = Some(msg);
                }
                BackgroundMsg::TrackerFinished(res) => {
                    self.tracker_is_busy = false;
                    match res {
                        Ok(p) => {
                            self.tracker_status_message = Some(format!("✔ Kurulum izlendi ve profil kaydedildi: {:?}", p.file_name().unwrap_or_default()));
                            self.tracker_installer_input.clear();
                            self.tracker_custom_name.clear();
                            self.trigger_scan();
                        }
                        Err(e) => {
                            self.tracker_status_message = Some(format!("[-] Kurulum izlenemedi: {}", e));
                        }
                    }
                }
            }
        }

        // ================= WINUI 3 / FLUENT CYBER-DARK VISUALS =================
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(26, 26, 30); // Mica / Acrylic deep dark
        visuals.window_fill = egui::Color32::from_rgb(34, 34, 38);
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(38, 38, 44);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(44, 44, 52);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(58, 58, 68);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 120, 212); // Windows 11 Blue
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);
        visuals.window_rounding = egui::Rounding::same(12.0);
        ctx.set_visuals(visuals);

        let t = self.t();

        // Selection metrics
        let sel_count = self.selected_ids.len();
        let total_sel_bytes: u64 = self
            .apps
            .iter()
            .filter(|a| self.selected_ids.contains(&a.id))
            .map(|a| a.estimated_size_bytes)
            .sum();

        // Counts by category
        let regular_apps_count = self.apps.iter().filter(|a| matches!(a.app_type, AppType::Win32 | AppType::Msi | AppType::AppXStore)).count();
        let ext_count = self.apps.iter().filter(|a| a.app_type == AppType::BrowserExtension).count();
        let leftover_count = self.apps.iter().filter(|a| a.app_type == AppType::Leftover).count();

        // ================= LEFT NAVIGATION RAIL (ORIGINAL WINUI 3 SIDEBAR) =================
        egui::SidePanel::left("left_nav_rail")
            .exact_width(235.0)
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(20, 20, 25))
                    .inner_margin(egui::Margin::symmetric(14.0, 18.0)),
            )
            .show(ctx, |ui| {
                // Brand Header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🚀").size(24.0));
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading(
                                egui::RichText::new("WinRaze")
                                    .color(egui::Color32::from_rgb(244, 244, 245))
                                    .strong()
                                    .size(19.0),
                            );
                            render_pill_badge(ui, "v0.3.0", egui::Color32::from_rgb(14, 165, 233));
                        });
                        ui.label(
                            egui::RichText::new("Raze the bloat · Powered by Rust 🦀")
                                .color(egui::Color32::from_rgb(140, 140, 150))
                                .size(11.0),
                        );
                    });
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                // Sidebar Navigation Items
                render_nav_item(
                    ui,
                    &mut self.active_nav,
                    ActiveNavTab::Dashboard,
                    t.nav_dashboard,
                    self.health_stats.as_ref().map(|h| format!("%{}", h.health_score)),
                    egui::Color32::from_rgb(34, 197, 94),
                );

                render_nav_item(
                    ui,
                    &mut self.active_nav,
                    ActiveNavTab::Applications,
                    t.nav_apps,
                    Some(regular_apps_count.to_string()),
                    egui::Color32::from_rgb(59, 130, 246),
                );

                let tracked_count = self.apps.iter().filter(|a| a.app_type == AppType::Tracked).count();
                render_nav_item(
                    ui,
                    &mut self.active_nav,
                    ActiveNavTab::Tracker,
                    t.nav_tracker,
                    Some(tracked_count.to_string()),
                    egui::Color32::from_rgb(14, 165, 233),
                );

                render_nav_item(
                    ui,
                    &mut self.active_nav,
                    ActiveNavTab::Debloater,
                    t.nav_debloater,
                    Some(KNOWN_BLOATWARE.len().to_string()),
                    egui::Color32::from_rgb(244, 63, 94),
                );

                render_nav_item(
                    ui,
                    &mut self.active_nav,
                    ActiveNavTab::BrowserExtensions,
                    t.nav_extensions,
                    Some(ext_count.to_string()),
                    egui::Color32::from_rgb(168, 85, 247),
                );

                render_nav_item(
                    ui,
                    &mut self.active_nav,
                    ActiveNavTab::Leftovers,
                    t.nav_leftovers,
                    Some(leftover_count.to_string()),
                    egui::Color32::from_rgb(245, 158, 11),
                );

                render_nav_item(
                    ui,
                    &mut self.active_nav,
                    ActiveNavTab::ForceShredder,
                    t.nav_force_clean,
                    None,
                    egui::Color32::from_rgb(239, 68, 68),
                );

                render_nav_item(
                    ui,
                    &mut self.active_nav,
                    ActiveNavTab::Settings,
                    t.nav_settings,
                    None,
                    egui::Color32::from_rgb(161, 161, 170),
                );

                // Bottom of sidebar: Rescan + Language
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(6.0);
                    // Language Switcher Pill
                    let lang_label = match self.current_lang {
                        Language::Turkish => "🇹🇷 Türkçe",
                        Language::English => "🇬🇧 English",
                    };
                    let lang_btn = egui::Button::new(egui::RichText::new(lang_label).size(12.0).strong())
                        .fill(egui::Color32::from_rgb(34, 34, 40))
                        .rounding(egui::Rounding::same(8.0));
                    if ui.add_sized([ui.available_width(), 28.0], lang_btn).clicked() {
                        self.current_lang = match self.current_lang {
                            Language::Turkish => Language::English,
                            Language::English => Language::Turkish,
                        };
                    }

                    ui.add_space(6.0);

                    // Rescan Button
                    let rescan_btn = egui::Button::new(
                        egui::RichText::new(format!("🔄 {}", t.btn_rescan))
                            .size(12.0)
                            .color(egui::Color32::from_rgb(228, 228, 231)),
                    )
                    .fill(egui::Color32::from_rgb(34, 34, 40))
                    .rounding(egui::Rounding::same(8.0));

                    if ui.add_sized([ui.available_width(), 28.0], rescan_btn).clicked() {
                        self.trigger_scan();
                    }

                    ui.add_space(10.0);
                    ui.separator();
                });
            });

        // ================= BOTTOM STATUS FOOTER =================
        egui::TopBottomPanel::bottom("bottom_status_panel")
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(18, 18, 22))
                    .inner_margin(egui::Margin::symmetric(20.0, 9.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self.is_scanning {
                        ui.spinner();
                        ui.label(egui::RichText::new(t.scanning_status).color(egui::Color32::from_rgb(212, 212, 216)));
                    } else if self.is_uninstalling {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(format!("{}: {}", t.modal_uninstalling, self.current_step_name))
                                .color(egui::Color32::from_rgb(239, 68, 68)),
                        );
                        ui.add(egui::ProgressBar::new(self.current_progress).show_percentage());
                    } else {
                        ui.label(egui::RichText::new(format!("✔ {}", t.ready_status)).color(egui::Color32::from_rgb(161, 161, 170)));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(ref h) = self.health_stats {
                            let score_color = if h.health_score >= 80 {
                                egui::Color32::from_rgb(34, 197, 94)
                            } else if h.health_score >= 60 {
                                egui::Color32::from_rgb(234, 179, 8)
                            } else {
                                egui::Color32::from_rgb(239, 68, 68)
                            };
                            ui.label(egui::RichText::new(format!("Sağlık Endeksi: %{}", h.health_score)).strong().color(score_color));
                        }
                    });
                });
            });

        // ================= SIDE DETAILS / INSPECTION DRAWER =================
        if let Some(idx) = self.active_app_index {
            if let Some(app) = self.apps.get(idx) {
                egui::SidePanel::right("app_details_panel")
                    .min_width(340.0)
                    .max_width(460.0)
                    .frame(
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(28, 28, 34))
                            .inner_margin(egui::Margin::same(18.0)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(
                                egui::RichText::new(t.inspect_title)
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(56, 189, 248)),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(t.btn_close).clicked() {
                                    self.active_app_index = None;
                                }
                            });
                        });
                        ui.separator();

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add_space(6.0);
                            ui.label(egui::RichText::new(&app.name).strong().size(16.0).color(egui::Color32::WHITE));

                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(38, 38, 46))
                                .rounding(egui::Rounding::same(8.0))
                                .inner_margin(egui::Margin::same(10.0))
                                .show(ui, |ui| {
                                    if let Some(ref p) = app.publisher {
                                        ui.label(format!("{} {}", t.inspect_publisher, p));
                                    }
                                    if let Some(ref v) = app.version {
                                        ui.label(format!("{} {}", t.inspect_version, v));
                                    }
                                    ui.label(format!("{} {}", t.inspect_size, format_bytes(app.estimated_size_bytes)));

                                    if let Some(ref loc) = app.install_location {
                                        ui.label(format!("{} {}", t.inspect_location, loc.display()));
                                    }
                                    if let Some(ref uninst) = app.uninstall_string {
                                        ui.label(format!("{} {}", t.inspect_command, uninst));
                                    }
                                });

                            ui.add_space(10.0);
                            ui.heading(egui::RichText::new(format!("{} ({})", t.inspect_artifacts, app.artifacts.len())).size(14.0));
                            ui.separator();

                            for (i, art) in app.artifacts.iter().enumerate() {
                                let badge = match art.kind {
                                    crate::model::ArtifactKind::RegistryKey => "Registry",
                                    crate::model::ArtifactKind::Directory => "Directory",
                                    crate::model::ArtifactKind::Shortcut => "Shortcut",
                                    crate::model::ArtifactKind::Prefetch => "Prefetch",
                                    crate::model::ArtifactKind::CrashDump => "CrashDump",
                                    _ => "Artifact",
                                };
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(36, 36, 42))
                                    .rounding(egui::Rounding::same(6.0))
                                    .inner_margin(egui::Margin::same(8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(format!("[{}] {}", i + 1, badge)).color(egui::Color32::from_rgb(251, 191, 36)).strong());
                                            if let Some(s) = art.size_bytes {
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.label(egui::RichText::new(format_bytes(s)).color(egui::Color32::from_rgb(161, 161, 170)));
                                                });
                                            }
                                        });
                                        ui.label(egui::RichText::new(&art.path_or_target).monospace().size(11.0).color(egui::Color32::from_rgb(212, 212, 216)));
                                    });
                                ui.add_space(4.0);
                            }
                        });
                    });
            }
        }

        // ================= CENTRAL CONTENT WORKSPACE =================
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(24, 24, 28))
                    .inner_margin(egui::Margin::symmetric(24.0, 18.0)),
            )
            .show(ctx, |ui| {
                match self.active_nav {
                    ActiveNavTab::Dashboard => {
                        self.render_dashboard_view(ui);
                    }
                    ActiveNavTab::Applications => {
                        self.render_applications_view(ui, sel_count, total_sel_bytes);
                    }
                    ActiveNavTab::Tracker => {
                        self.render_tracker_view(ui);
                    }
                    ActiveNavTab::Debloater => {
                        self.render_debloater_view(ui);
                    }
                    ActiveNavTab::BrowserExtensions => {
                        self.render_extensions_view(ui);
                    }
                    ActiveNavTab::Leftovers => {
                        self.render_leftovers_view(ui);
                    }
                    ActiveNavTab::ForceShredder => {
                        self.render_force_shredder_view(ui);
                    }
                    ActiveNavTab::Settings => {
                        self.render_settings_view(ui);
                    }
                }
            });

        // ================= PRE-UNINSTALL CONFIRMATION MODAL =================
        if self.show_confirm_modal {
            let has_protected = self
                .apps
                .iter()
                .filter(|a| self.selected_ids.contains(&a.id))
                .any(|a| a.is_protected);

            egui::Window::new(t.modal_confirm_title)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .fixed_size(egui::vec2(520.0, 380.0))
                .show(ctx, |ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "Aşağıdaki {} uygulama ve ilişkili tüm kalıntıları kaldırılacak (Toplam: {}):",
                            sel_count,
                            format_bytes(total_sel_bytes)
                        ))
                        .size(13.0)
                        .color(egui::Color32::WHITE),
                    );

                    ui.add_space(8.0);

                    // Selected apps list box
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(32, 32, 38))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                                for app in self.apps.iter().filter(|a| self.selected_ids.contains(&a.id)) {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("•").color(egui::Color32::from_rgb(56, 189, 248)));
                                        ui.label(egui::RichText::new(&app.name).strong());
                                        if app.is_protected {
                                            render_pill_badge(ui, t.badge_protected, egui::Color32::from_rgb(234, 88, 12));
                                        }
                                    });
                                }
                            });
                        });

                    if has_protected {
                        ui.add_space(8.0);
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(60, 30, 20))
                            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(234, 88, 12)))
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(t.warn_protected_modal)
                                        .color(egui::Color32::from_rgb(251, 146, 60))
                                        .size(12.0),
                                );
                            });
                    }

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("🛡️ Güvenlik ve Kurtarma Seçenekleri:").strong());
                    ui.checkbox(&mut self.options.create_restore_point, t.settings_restorepoint);
                    ui.checkbox(&mut self.options.backup_registry, t.settings_regbackup);
                    ui.checkbox(&mut self.options.auto_kill, t.settings_autokill);

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let confirm_btn = egui::Button::new(
                            egui::RichText::new(t.modal_confirm_btn)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(196, 43, 28))
                        .rounding(egui::Rounding::same(8.0));

                        if ui.add(confirm_btn).clicked() {
                            self.trigger_uninstall_confirmed();
                        }

                        if ui.button(t.modal_cancel_btn).clicked() {
                            self.show_confirm_modal = false;
                        }
                    });
                });
        }

        // ================= LIVE UNINSTALL PROGRESS MODAL =================
        if self.is_uninstalling {
            egui::Window::new(t.modal_uninstalling)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .fixed_size(egui::vec2(580.0, 350.0))
                .show(ctx, |ui| {
                    ui.heading(&self.current_step_name);
                    ui.add_space(6.0);
                    ui.add(egui::ProgressBar::new(self.current_progress).show_percentage());
                    ui.separator();
                    ui.label(t.modal_live_log);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for log in self.live_logs.iter().rev().take(30) {
                            ui.label(
                                egui::RichText::new(format!("• {}", log))
                                    .monospace()
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(212, 212, 216)),
                            );
                        }
                    });
                });
        }

        // ================= COMPLETION REPORT MODAL =================
        let mut close_report = false;
        if let Some(ref report) = self.last_report {
            egui::Window::new(t.modal_complete)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .fixed_size(egui::vec2(500.0, 320.0))
                .show(ctx, |ui| {
                    ui.heading(format!("{} {}", t.modal_complete, report.app_name));
                    ui.separator();
                    ui.label(format!("• {} {}", t.metric_freed, format_bytes(report.total_freed_bytes)));
                    ui.label(format!("• {} {}", t.metric_files, report.deleted_files_count));
                    ui.label(format!("• {} {}", t.metric_keys, report.deleted_reg_count));
                    ui.label(format!("• {} {}", t.metric_procs, report.terminated_procs_count));
                    ui.label(format!("• {} {}", t.metric_services, report.stopped_services_count));

                    if report.restore_point_created {
                        ui.label(
                            egui::RichText::new("• 🛡️ Sistem Geri Yükleme Noktası Aktif / Restore Point Saved")
                                .color(egui::Color32::from_rgb(34, 197, 94)),
                        );
                    }
                    if let Some(ref bk) = report.registry_backup_path {
                        ui.label(
                            egui::RichText::new(format!("• 💾 .reg Yedeği / Backup: {}", bk))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(59, 130, 246)),
                        );
                    }

                    if report.requires_reboot {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(t.modal_reboot_notice)
                                .color(egui::Color32::from_rgb(251, 191, 36))
                                .strong(),
                        );
                    }

                    ui.add_space(14.0);
                    if ui.button(t.btn_close).clicked() {
                        close_report = true;
                    }
                });
        }
        if close_report {
            self.last_report = None;
        }

        ctx.request_repaint();
    }
}

// ================= VIEW RENDER IMPLEMENTATIONS =================
impl UninstalrApp {
    fn render_dashboard_view(&mut self, ui: &mut egui::Ui) {
        let t = self.t();

        ui.heading(
            egui::RichText::new(t.health_title)
                .size(22.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new(t.health_score_desc)
                .size(12.5)
                .color(egui::Color32::from_rgb(161, 161, 170)),
        );

        ui.add_space(16.0);

        let health_opt = self.health_stats.clone();
        if let Some(health) = health_opt {
            let score_color = if health.health_score >= 80 {
                egui::Color32::from_rgb(34, 197, 94)
            } else if health.health_score >= 60 {
                egui::Color32::from_rgb(234, 179, 8)
            } else {
                egui::Color32::from_rgb(239, 68, 68)
            };

            let score_status = if health.health_score >= 80 {
                "Sistem Kararlı & Temiz (Optimal)"
            } else if health.health_score >= 60 {
                "İyileştirme Tavsiye Edilir (Moderate)"
            } else {
                "Kritik Artık Yükü (Needs Cleanup)"
            };

            // Health Score Hero Banner
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(32, 32, 38))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(48, 48, 56)))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::symmetric(20.0, 16.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Big Score Number
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(format!("%{}", health.health_score))
                                    .size(42.0)
                                    .strong()
                                    .color(score_color),
                            );
                            ui.label(
                                egui::RichText::new(score_status)
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(212, 212, 216)),
                            );
                        });

                        ui.add_space(30.0);

                        // 1-Click Quick Optimize Action Button
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let opt_btn = egui::Button::new(
                                egui::RichText::new(t.health_opt_btn)
                                    .size(14.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(16, 185, 129))
                            .rounding(egui::Rounding::same(10.0));

                            if ui.add_sized([260.0, 42.0], opt_btn).clicked() {
                                self.trigger_quick_health_cleanup();
                            }
                        });
                    });

                    ui.add_space(10.0);
                    // Health Progress Bar
                    ui.add(
                        egui::ProgressBar::new(health.health_score as f32 / 100.0)
                            .fill(score_color)
                            .animate(false),
                    );
                });

            ui.add_space(16.0);

            // 4 KPI Metric Cards
            ui.horizontal(|ui| {
                render_kpi_card(
                    ui,
                    "💥",
                    t.health_crash_dumps,
                    &format!("{} ({})", health.crash_dump_count, format_bytes(health.crash_dump_bytes)),
                    egui::Color32::from_rgb(239, 68, 68),
                );
                ui.add_space(10.0);
                render_kpi_card(
                    ui,
                    "⚡",
                    t.stat_leftovers,
                    &format!("{} ({})", health.leftover_count, format_bytes(health.leftover_bytes)),
                    egui::Color32::from_rgb(245, 158, 11),
                );
                ui.add_space(10.0);
                render_kpi_card(
                    ui,
                    "🔗",
                    t.health_shortcuts,
                    &health.broken_shortcuts_count.to_string(),
                    egui::Color32::from_rgb(56, 189, 248),
                );
                ui.add_space(10.0);
                render_kpi_card(
                    ui,
                    "🛡️",
                    t.health_protected,
                    &health.protected_count.to_string(),
                    egui::Color32::from_rgb(34, 197, 94),
                );
            });

            ui.add_space(20.0);

            // Safety Information Card
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 32, 40))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(44, 48, 60)))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::symmetric(16.0, 14.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🛡️").size(22.0));
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("DeRust Güvenli Kaldırma Mimarisi")
                                    .strong()
                                    .size(14.0)
                                    .color(egui::Color32::WHITE),
                            );
                            ui.label(
                                egui::RichText::new(
                                    "Her kaldırma işleminde otomatik Windows Sistem Geri Yükleme Noktası (Restore Point) oluşturulur ve silinen kayıt defteri anahtarları %LOCALAPPDATA%\\DeRust\\Backups\\Registry\\ klasörüne .reg formatında yedeklenir.",
                                )
                                .size(12.0)
                                .color(egui::Color32::from_rgb(161, 161, 170)),
                            );
                        });
                    });
                });
        } else {
            ui.spinner();
            ui.label("Sistem sağlık verileri taranıyor...");
        }
    }

    fn render_applications_view(&mut self, ui: &mut egui::Ui, sel_count: usize, total_sel_bytes: u64) {
        let t = self.t();

        // Row 1: Search & Filter Tabs & Action Buttons
        ui.horizontal(|ui| {
            // Search Input
            ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .desired_width(260.0)
                    .hint_text(t.search_hint)
                    .margin(egui::Margin::symmetric(10.0, 7.0)),
            );

            ui.add_space(10.0);

            // Segmented Filter Tabs
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(36, 36, 42))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        render_filter_tab(ui, &mut self.filter_tab, FilterTab::All, t.tab_all);
                        render_filter_tab(ui, &mut self.filter_tab, FilterTab::Win32, t.tab_win32);
                        render_filter_tab(ui, &mut self.filter_tab, FilterTab::Msi, t.tab_msi);
                        render_filter_tab(ui, &mut self.filter_tab, FilterTab::Store, t.tab_store);
                        render_filter_tab(ui, &mut self.filter_tab, FilterTab::Tracked, t.tab_tracked);
                    });
                });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Batch Uninstall Button
                let uninstall_text = if sel_count > 0 {
                    format!("🚀 {} ({} · {})", t.btn_uninstall, sel_count, format_bytes(total_sel_bytes))
                } else {
                    format!("🚀 {} (0)", t.btn_uninstall)
                };

                let uninstall_btn = egui::Button::new(
                    egui::RichText::new(uninstall_text)
                        .color(egui::Color32::WHITE)
                        .strong()
                        .size(13.5),
                )
                .fill(if sel_count > 0 {
                    egui::Color32::from_rgb(196, 43, 28)
                } else {
                    egui::Color32::from_rgb(52, 52, 60)
                })
                .rounding(egui::Rounding::same(8.0));

                if ui.add_enabled(sel_count > 0 && !self.is_uninstalling, uninstall_btn).clicked() {
                    self.show_confirm_modal = true;
                }
            });
        });

        ui.add_space(10.0);

        // Filter apps list
        let search_lower = self.search_query.to_lowercase();
        let filtered_apps: Vec<(usize, &InstalledApp)> = self
            .apps
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                // Exclude extension and leftover from this view
                if matches!(a.app_type, AppType::BrowserExtension | AppType::Leftover) {
                    return false;
                }

                let tab_match = match self.filter_tab {
                    FilterTab::All => true,
                    FilterTab::Win32 => a.app_type == AppType::Win32,
                    FilterTab::Msi => a.app_type == AppType::Msi,
                    FilterTab::Store => a.app_type == AppType::AppXStore,
                    FilterTab::Tracked => a.app_type == AppType::Tracked,
                };
                if !tab_match {
                    return false;
                }

                if search_lower.is_empty() {
                    true
                } else {
                    a.name.to_lowercase().contains(&search_lower)
                        || a.publisher
                            .as_ref()
                            .map_or(false, |p| p.to_lowercase().contains(&search_lower))
                }
            })
            .collect();

        // Command Bar: Select All / Deselect All
        ui.horizontal(|ui| {
            let all_checked = !filtered_apps.is_empty()
                && filtered_apps.iter().all(|(_, a)| self.selected_ids.contains(&a.id));

            let mut check_state = all_checked;
            if ui.checkbox(&mut check_state, if all_checked { t.btn_deselect_all } else { t.btn_select_all }).changed() {
                if check_state {
                    for (_, a) in &filtered_apps {
                        self.selected_ids.insert(a.id.clone());
                    }
                } else {
                    for (_, a) in &filtered_apps {
                        self.selected_ids.remove(&a.id);
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{} {}", filtered_apps.len(), t.stat_apps))
                        .color(egui::Color32::from_rgb(161, 161, 170))
                        .size(12.0),
                );
            });
        });

        ui.add_space(6.0);

        // Scrollable Modern App Cards
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (real_idx, app) in filtered_apps {
                let mut is_checked = self.selected_ids.contains(&app.id);
                let is_active = self.active_app_index == Some(real_idx);

                let card_bg = if is_active {
                    egui::Color32::from_rgb(26, 48, 80)
                } else if is_checked {
                    egui::Color32::from_rgb(44, 44, 54)
                } else {
                    egui::Color32::from_rgb(32, 32, 38)
                };

                let card_border = if is_active {
                    egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 120, 212))
                } else if is_checked {
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(99, 102, 241))
                } else {
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(44, 44, 52))
                };

                egui::Frame::none()
                    .fill(card_bg)
                    .stroke(card_border)
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut is_checked, "").changed() {
                                if is_checked {
                                    self.selected_ids.insert(app.id.clone());
                                } else {
                                    self.selected_ids.remove(&app.id);
                                }
                            }

                            let (badge_text, badge_color) = match app.app_type {
                                AppType::Win32 => ("Win32", egui::Color32::from_rgb(34, 197, 94)),
                                AppType::Msi => ("MSI", egui::Color32::from_rgb(59, 130, 246)),
                                AppType::AppXStore => ("Store", egui::Color32::from_rgb(168, 85, 247)),
                                AppType::Portable => ("Portable", egui::Color32::from_rgb(234, 179, 8)),
                                AppType::Leftover => ("Leftover", egui::Color32::from_rgb(239, 68, 68)),
                                AppType::BrowserExtension => ("Extension", egui::Color32::from_rgb(6, 182, 212)),
                                AppType::Tracked => ("Tracked", egui::Color32::from_rgb(14, 165, 233)),
                            };
                            render_pill_badge(ui, badge_text, badge_color);

                            if app.is_protected {
                                ui.add_space(2.0);
                                render_pill_badge(ui, t.badge_protected, egui::Color32::from_rgb(234, 88, 12));
                            }

                            ui.add_space(4.0);

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let inspect_btn = ui.add(
                                    egui::Button::new(egui::RichText::new("🔍").size(13.5))
                                        .fill(egui::Color32::from_rgb(46, 46, 56))
                                        .rounding(egui::Rounding::same(6.0)),
                                );
                                if inspect_btn.clicked() {
                                    self.active_app_index = Some(real_idx);
                                }

                                ui.add_space(8.0);

                                let size_str = format_bytes(app.estimated_size_bytes);
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(40, 40, 50))
                                    .rounding(egui::Rounding::same(6.0))
                                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(size_str)
                                                .color(egui::Color32::from_rgb(251, 191, 36))
                                                .strong()
                                                .size(12.0),
                                        );
                                    });

                                ui.add_space(10.0);

                                // Middle Title & Publisher info with safe truncation
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.vertical(|ui| {
                                        let display_name = if app.name.chars().count() > 42 {
                                            format!("{}...", app.name.chars().take(39).collect::<String>())
                                        } else {
                                            app.name.clone()
                                        };

                                        let name_btn = ui.add(
                                            egui::Button::new(
                                                egui::RichText::new(display_name)
                                                    .strong()
                                                    .size(13.5)
                                                    .color(egui::Color32::from_rgb(244, 244, 245)),
                                            )
                                            .frame(false),
                                        );

                                        if name_btn.clicked() {
                                            self.active_app_index = Some(real_idx);
                                        }

                                        let pub_str = app.publisher.as_deref().unwrap_or("Bilinmeyen Yayımcı");
                                        let ver_str = app.version.as_deref().unwrap_or("-");
                                        let info_str = format!("{} · v{}", pub_str, ver_str);
                                        let display_info = if info_str.chars().count() > 50 {
                                            format!("{}...", info_str.chars().take(47).collect::<String>())
                                        } else {
                                            info_str
                                        };

                                        ui.label(
                                            egui::RichText::new(display_info)
                                                .size(11.0)
                                                .color(egui::Color32::from_rgb(155, 155, 165)),
                                        );
                                    });
                                });
                            });
                        });
                    });
                ui.add_space(6.0);
            }
        });
    }

    fn render_extensions_view(&mut self, ui: &mut egui::Ui) {
        let t = self.t();

        ui.heading(
            egui::RichText::new(t.ext_title)
                .size(20.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new(t.ext_desc)
                .size(12.5)
                .color(egui::Color32::from_rgb(161, 161, 170)),
        );

        ui.add_space(14.0);

        let extensions: Vec<InstalledApp> = self
            .apps
            .iter()
            .filter(|a| a.app_type == AppType::BrowserExtension)
            .cloned()
            .collect();

        if extensions.is_empty() {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(32, 32, 38))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(20.0))
                .show(ui, |ui| {
                    ui.label("Tarayıcı eklentisi bulunamadı veya henüz yüklenmedi.");
                });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for app in extensions {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(32, 32, 38))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(44, 44, 52)))
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let pub_str = app.publisher.as_deref().unwrap_or("Browser");
                            let browser_color = match pub_str {
                                "Google Chrome" => egui::Color32::from_rgb(59, 130, 246),
                                "Microsoft Edge" => egui::Color32::from_rgb(16, 185, 129),
                                "Brave Browser" => egui::Color32::from_rgb(249, 115, 22),
                                _ => egui::Color32::from_rgb(168, 85, 247),
                            };
                            render_pill_badge(ui, pub_str, browser_color);

                            ui.add_space(6.0);

                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(&app.name)
                                        .strong()
                                        .size(14.0)
                                        .color(egui::Color32::WHITE),
                                );
                                let ver_str = app.version.as_deref().unwrap_or("-");
                                let loc_str = app.install_location.as_ref().map_or("-".to_string(), |p| p.display().to_string());
                                ui.label(
                                    egui::RichText::new(format!("Sürüm: {} · Dizin: {}", ver_str, loc_str))
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(161, 161, 170)),
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let remove_btn = egui::Button::new(
                                    egui::RichText::new(format!("🗑 {}", t.ext_btn_remove))
                                        .color(egui::Color32::WHITE)
                                        .size(12.0),
                                )
                                .fill(egui::Color32::from_rgb(196, 43, 28))
                                .rounding(egui::Rounding::same(6.0));

                                if ui.add(remove_btn).clicked() {
                                    self.remove_browser_extension(&app);
                                }
                            });
                        });
                    });
                ui.add_space(6.0);
            }
        });
    }

    fn render_leftovers_view(&mut self, ui: &mut egui::Ui) {
        let t = self.t();

        ui.heading(
            egui::RichText::new(t.nav_leftovers)
                .size(20.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new("Daha önce kaldırılmış programlardan kalan sahipsiz AppData, ProgramFiles ve Kayıt Defteri artıkları.")
                .size(12.5)
                .color(egui::Color32::from_rgb(161, 161, 170)),
        );

        ui.add_space(14.0);

        let leftovers: Vec<(usize, &InstalledApp)> = self
            .apps
            .iter()
            .enumerate()
            .filter(|(_, a)| a.app_type == AppType::Leftover)
            .collect();

        if leftovers.is_empty() {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(32, 32, 38))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(20.0))
                .show(ui, |ui| {
                    ui.label("Tebrikler! Sistemde sahipsiz program kalıntısı bulunamadı.");
                });
            return;
        }

        ui.horizontal(|ui| {
            ui.label(format!("Toplam {} adet sahipsiz artık tespit edildi.", leftovers.len()));
        });

        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (real_idx, app) in leftovers {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(32, 32, 38))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(52, 44, 44)))
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            render_pill_badge(ui, "Artık / Leftover", egui::Color32::from_rgb(239, 68, 68));
                            ui.add_space(6.0);

                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&app.name).strong().size(14.0).color(egui::Color32::WHITE));
                                if let Some(ref loc) = app.install_location {
                                    ui.label(egui::RichText::new(loc.display().to_string()).size(11.0).color(egui::Color32::from_rgb(161, 161, 170)));
                                }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let inspect_btn = ui.add(
                                    egui::Button::new(egui::RichText::new("🔍").size(14.0))
                                        .fill(egui::Color32::from_rgb(46, 46, 56))
                                        .rounding(egui::Rounding::same(6.0)),
                                );
                                if inspect_btn.clicked() {
                                    self.active_app_index = Some(real_idx);
                                }

                                ui.add_space(6.0);
                                let size_str = format_bytes(app.estimated_size_bytes);
                                ui.label(egui::RichText::new(size_str).color(egui::Color32::from_rgb(251, 191, 36)).strong());
                            });
                        });
                    });
                ui.add_space(6.0);
            }
        });
    }

    fn render_force_shredder_view(&mut self, ui: &mut egui::Ui) {
        let t = self.t();

        ui.heading(
            egui::RichText::new(t.force_title)
                .size(20.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new(t.force_desc)
                .size(12.5)
                .color(egui::Color32::from_rgb(161, 161, 170)),
        );

        ui.add_space(16.0);

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(32, 32, 38))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(46, 46, 54)))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(20.0, 18.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Hedef Program / Klasör Yolu:").strong().size(13.0));
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.force_path_input)
                            .desired_width(ui.available_width() - 110.0)
                            .hint_text(t.force_path_hint)
                            .margin(egui::Margin::symmetric(10.0, 8.0)),
                    );

                    let browse_btn = egui::Button::new(t.force_btn_browse)
                        .fill(egui::Color32::from_rgb(48, 48, 58))
                        .rounding(egui::Rounding::same(8.0));

                    if ui.add_sized([100.0, 32.0], browse_btn).clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.force_path_input = folder.display().to_string();
                        }
                    }
                });

                ui.add_space(14.0);

                let target_exists = Path::new(&self.force_path_input).exists() && !self.force_path_input.trim().is_empty();

                if target_exists {
                    ui.label(
                        egui::RichText::new("✔ Belirtilen yol doğrulandı. İşlem başlatıldığında kilitli süreçler sonlandırılacak, kayıt defteri temizlenecek ve klasör imha edilecektir.")
                            .color(egui::Color32::from_rgb(34, 197, 94))
                            .size(12.0),
                    );
                } else if !self.force_path_input.trim().is_empty() {
                    ui.label(
                        egui::RichText::new("⚠ Belirtilen yol bulunamadı.")
                            .color(egui::Color32::from_rgb(239, 68, 68))
                            .size(12.0),
                    );
                }

                ui.add_space(16.0);

                let shred_btn = egui::Button::new(
                    egui::RichText::new(t.force_btn_shred)
                        .size(14.0)
                        .strong()
                        .color(egui::Color32::WHITE),
                )
                .fill(if target_exists {
                    egui::Color32::from_rgb(196, 43, 28)
                } else {
                    egui::Color32::from_rgb(60, 60, 68)
                })
                .rounding(egui::Rounding::same(8.0));

                if ui.add_sized([220.0, 38.0], shred_btn).clicked() && target_exists {
                    self.trigger_force_clean();
                }

                if let Some(ref status) = self.force_status_message {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(status).color(egui::Color32::from_rgb(34, 197, 94)));
                }
            });
    }

    fn render_tracker_view(&mut self, ui: &mut egui::Ui) {
        let t = self.t();

        ui.heading(
            egui::RichText::new(t.tracker_title)
                .size(20.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new(t.tracker_desc)
                .size(12.5)
                .color(egui::Color32::from_rgb(161, 161, 170)),
        );

        ui.add_space(14.0);

        // Tracking Setup Wizard Card
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(32, 32, 40))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(50, 50, 64)))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(20.0, 18.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("1. Kurulum Dosyasını Seçin (.exe veya .msi)").strong().size(13.5).color(egui::Color32::from_rgb(56, 189, 248)));
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.tracker_installer_input)
                            .desired_width(ui.available_width() - 165.0)
                            .hint_text(t.tracker_path_hint)
                            .margin(egui::Margin::symmetric(10.0, 8.0)),
                    );

                    let browse_btn = egui::Button::new(t.tracker_btn_browse)
                        .fill(egui::Color32::from_rgb(48, 52, 64))
                        .rounding(egui::Rounding::same(8.0));

                    if ui.add_sized([150.0, 34.0], browse_btn).clicked() {
                        if let Some(file) = rfd::FileDialog::new()
                            .add_filter("Kurulum Dosyası (Installer)", &["exe", "msi"])
                            .pick_file()
                        {
                            self.tracker_installer_input = file.display().to_string();
                            if self.tracker_custom_name.trim().is_empty() {
                                if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
                                    self.tracker_custom_name = stem.to_string();
                                }
                            }
                        }
                    }
                });

                ui.add_space(10.0);
                ui.label(egui::RichText::new("2. Program Adı (İsteğe Bağlı):").size(12.5).color(egui::Color32::from_rgb(212, 212, 216)));
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.tracker_custom_name)
                        .desired_width(ui.available_width() - 165.0)
                        .hint_text("Örn: Google Chrome, Notepad++...")
                        .margin(egui::Margin::symmetric(10.0, 7.0)),
                );

                ui.add_space(16.0);

                let file_exists = Path::new(&self.tracker_installer_input).exists() && !self.tracker_installer_input.trim().is_empty();

                ui.horizontal(|ui| {
                    let install_btn = egui::Button::new(
                        egui::RichText::new(t.tracker_btn_install)
                            .size(13.5)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(if file_exists && !self.tracker_is_busy {
                        egui::Color32::from_rgb(14, 165, 233)
                    } else {
                        egui::Color32::from_rgb(46, 48, 58)
                    })
                    .rounding(egui::Rounding::same(8.0));

                    if ui.add_sized([250.0, 38.0], install_btn).clicked() && file_exists && !self.tracker_is_busy {
                        self.trigger_track_installation();
                    }

                    if self.tracker_is_busy {
                        ui.add_space(12.0);
                        ui.spinner();
                        if let Some(ref msg) = self.tracker_status_message {
                            ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(56, 189, 248)).strong().size(12.5));
                        }
                    } else if let Some(ref status) = self.tracker_status_message {
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new(status).color(egui::Color32::from_rgb(34, 197, 94)).strong().size(12.5));
                    }
                });
            });

        ui.add_space(18.0);

        // Section: Tracked Applications List
        let tracked_apps: Vec<&InstalledApp> = self.apps.iter().filter(|a| a.app_type == AppType::Tracked).collect();

        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new(format!("Kayıtlı ve İzlenmiş Kurulumlar ({})", tracked_apps.len()))
                    .size(15.0)
                    .color(egui::Color32::WHITE),
            );
            render_pill_badge(ui, "Sıfır Kalıntı Garantisi", egui::Color32::from_rgb(14, 165, 233));
        });

        ui.add_space(8.0);

        if tracked_apps.is_empty() {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(28, 28, 34))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::symmetric(20.0, 30.0))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("📷").size(32.0));
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("Henüz izlenerek kurulmuş bir program bulunmuyor.")
                                .strong()
                                .size(14.0)
                                .color(egui::Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new("Yukarıdaki alandan bir kurulum dosyası (.exe/.msi) seçip 'Kurulumu Başlat & Canlı İzle' butonuna tıklayarak ilk programınızı izleyebilirsiniz.")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(161, 161, 170)),
                        );
                    });
                });
        } else {
            let mut to_uninstall_id: Option<String> = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                for app in tracked_apps {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(32, 32, 40))
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(44, 44, 56)))
                        .rounding(egui::Rounding::same(10.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                render_pill_badge(ui, "İzlenmiş", egui::Color32::from_rgb(14, 165, 233));
                                ui.label(egui::RichText::new(&app.name).strong().size(14.0).color(egui::Color32::WHITE));
                                if let Some(ref date) = app.install_date {
                                    render_pill_badge(ui, date, egui::Color32::from_rgb(113, 113, 122));
                                }
                                render_pill_badge(ui, &format_bytes(app.estimated_size_bytes), egui::Color32::from_rgb(234, 179, 8));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let remove_btn = egui::Button::new(
                                        egui::RichText::new("🗑 Sıfır Artıkla Kaldır")
                                            .size(12.0)
                                            .strong()
                                            .color(egui::Color32::WHITE),
                                    )
                                    .fill(egui::Color32::from_rgb(196, 43, 28))
                                    .rounding(egui::Rounding::same(6.0));

                                    if ui.add(remove_btn).clicked() {
                                        to_uninstall_id = Some(app.id.clone());
                                    }
                                });
                            });
                        });
                    ui.add_space(6.0);
                }
            });

            if let Some(id) = to_uninstall_id {
                if !self.is_uninstalling {
                    self.selected_ids.clear();
                    self.selected_ids.insert(id);
                    self.trigger_uninstall_confirmed();
                }
            }
        }
    }

    fn render_debloater_view(&mut self, ui: &mut egui::Ui) {
        let t = self.t();

        ui.heading(
            egui::RichText::new(t.debloat_title)
                .size(20.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new(t.debloat_desc)
                .size(12.5)
                .color(egui::Color32::from_rgb(161, 161, 170)),
        );

        ui.add_space(14.0);

        // Action Toolbar
        ui.horizontal(|ui| {
            let telem_btn = egui::Button::new(
                egui::RichText::new(t.debloat_telemetry_btn)
                    .color(egui::Color32::WHITE)
                    .strong()
                    .size(12.5),
            )
            .fill(egui::Color32::from_rgb(16, 185, 129))
            .rounding(egui::Rounding::same(8.0));

            if ui.add(telem_btn).clicked() {
                self.trigger_disable_telemetry();
            }

            ui.add_space(8.0);

            let temp_btn = egui::Button::new(
                egui::RichText::new(t.debloat_temp_btn)
                    .color(egui::Color32::WHITE)
                    .strong()
                    .size(12.5),
            )
            .fill(egui::Color32::from_rgb(59, 130, 246))
            .rounding(egui::Rounding::same(8.0));

            if ui.add(temp_btn).clicked() {
                self.trigger_clean_temp();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let raze_count = self.selected_bloatware.len();
                let raze_btn = egui::Button::new(
                    egui::RichText::new(format!("{} ({})", t.debloat_raze_all_btn, raze_count))
                        .color(egui::Color32::WHITE)
                        .strong()
                        .size(13.0),
                )
                .fill(if raze_count > 0 {
                    egui::Color32::from_rgb(196, 43, 28)
                } else {
                    egui::Color32::from_rgb(52, 52, 60)
                })
                .rounding(egui::Rounding::same(8.0));

                if ui.add_enabled(raze_count > 0 && !self.is_uninstalling, raze_btn).clicked() {
                    self.trigger_raze_bloatware();
                }
            });
        });

        if let Some(ref msg) = self.debloat_status_message {
            ui.add_space(10.0);
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(26, 48, 38))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(34, 197, 94)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(format!("✔ {}", msg)).color(egui::Color32::from_rgb(134, 239, 172)).strong());
                });
        }

        ui.add_space(12.0);

        // Bloatware Catalog Cards
        egui::ScrollArea::vertical().show(ui, |ui| {
            for target in KNOWN_BLOATWARE {
                let mut is_checked = self.selected_bloatware.contains(target.id);

                egui::Frame::none()
                    .fill(if is_checked {
                        egui::Color32::from_rgb(44, 38, 44)
                    } else {
                        egui::Color32::from_rgb(32, 32, 38)
                    })
                    .stroke(egui::Stroke::new(
                        1.0_f32,
                        if is_checked {
                            egui::Color32::from_rgb(244, 63, 94)
                        } else {
                            egui::Color32::from_rgb(44, 44, 52)
                        },
                    ))
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut is_checked, "").changed() {
                                if is_checked {
                                    self.selected_bloatware.insert(target.id.to_string());
                                } else {
                                    self.selected_bloatware.remove(target.id);
                                }
                            }

                            render_pill_badge(ui, target.category, egui::Color32::from_rgb(244, 63, 94));
                            ui.add_space(6.0);

                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(target.name).strong().size(14.0).color(egui::Color32::WHITE));
                                ui.label(egui::RichText::new(target.description).size(11.5).color(egui::Color32::from_rgb(161, 161, 170)));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(target.package_pattern).monospace().size(11.0).color(egui::Color32::from_rgb(113, 113, 122)));
                            });
                        });
                    });
                ui.add_space(6.0);
            }
        });
    }

    fn render_settings_view(&mut self, ui: &mut egui::Ui) {
        let t = self.t();

        ui.heading(
            egui::RichText::new(t.settings_title)
                .size(20.0)
                .strong()
                .color(egui::Color32::WHITE),
        );
        ui.add_space(14.0);

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(32, 32, 38))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(46, 46, 54)))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(20.0, 18.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(t.settings_lang).strong());
                    ui.selectable_value(&mut self.current_lang, Language::Turkish, "🇹🇷 Türkçe");
                    ui.selectable_value(&mut self.current_lang, Language::English, "🇬🇧 English");
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.checkbox(&mut self.options.create_restore_point, t.settings_restorepoint);
                ui.add_space(4.0);
                ui.checkbox(&mut self.options.backup_registry, t.settings_regbackup);
                ui.add_space(4.0);
                ui.checkbox(&mut self.options.auto_kill, t.settings_autokill);
                ui.add_space(4.0);
                ui.checkbox(&mut self.options.clean_empty_dirs, t.settings_emptydirs);
                ui.add_space(4.0);
                ui.checkbox(&mut self.options.clean_broken_shortcuts, t.settings_shortcuts);
                ui.add_space(4.0);
                ui.checkbox(&mut self.options.clean_prefetch, t.settings_prefetch);
                ui.add_space(4.0);
                ui.checkbox(&mut self.options.auto_reboot, t.settings_autoreboot);
                ui.add_space(4.0);
                ui.checkbox(&mut self.options.dry_run, t.settings_dryrun);
            });
    }
}

// ================= UI HELPER COMPONENTS =================

fn render_nav_item(
    ui: &mut egui::Ui,
    current_tab: &mut ActiveNavTab,
    target: ActiveNavTab,
    label: &str,
    badge: Option<String>,
    accent_color: egui::Color32,
) {
    let is_active = *current_tab == target;
    let width = ui.available_width();
    let desired_size = egui::vec2(width, 36.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let is_hovered = response.hovered();

    let bg_color = if is_active {
        egui::Color32::from_rgb(34, 40, 56)
    } else if is_hovered {
        egui::Color32::from_rgb(28, 30, 40)
    } else {
        egui::Color32::TRANSPARENT
    };

    let border_stroke = if is_active {
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(50, 60, 85))
    } else {
        egui::Stroke::NONE
    };

    let text_color = if is_active {
        egui::Color32::WHITE
    } else if is_hovered {
        egui::Color32::from_rgb(240, 240, 245)
    } else {
        egui::Color32::from_rgb(161, 161, 170)
    };

    // Draw background
    ui.painter().rect(rect, egui::Rounding::same(8.0), bg_color, border_stroke);

    // Left indicator bar for active item
    if is_active {
        let indicator_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 3.0, rect.min.y + 8.0),
            egui::vec2(3.5, 20.0),
        );
        ui.painter().rect_filled(indicator_rect, egui::Rounding::same(2.0), accent_color);
    }

    // Inner content layout
    let content_rect = rect.shrink2(egui::vec2(12.0, 0.0));
    ui.allocate_new_ui(
        egui::UiBuilder::new()
            .max_rect(content_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |child_ui| {
            child_ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .color(text_color)
                        .strong()
                        .size(13.0),
                );

                if let Some(b) = badge {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        render_pill_badge(ui, &b, accent_color);
                    });
                }
            });
        },
    );

    if response.clicked() {
        *current_tab = target;
    }

    ui.add_space(2.0);
}

fn render_kpi_card(ui: &mut egui::Ui, icon: &str, title: &str, value: &str, accent_color: egui::Color32) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(32, 32, 40))
        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(46, 48, 60)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(20.0));
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(title)
                            .size(11.5)
                            .color(egui::Color32::from_rgb(161, 161, 170)),
                    );
                    ui.label(
                        egui::RichText::new(value)
                            .size(15.5)
                            .strong()
                            .color(accent_color),
                    );
                });
            });
        });
}

fn render_filter_tab(ui: &mut egui::Ui, current: &mut FilterTab, target: FilterTab, label: &str) {
    let is_selected = *current == target;
    let bg_color = if is_selected {
        egui::Color32::from_rgb(0, 120, 212)
    } else {
        egui::Color32::TRANSPARENT
    };
    let text_color = if is_selected {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_rgb(175, 175, 185)
    };

    let btn = egui::Button::new(egui::RichText::new(label).color(text_color).strong().size(12.5))
        .fill(bg_color)
        .rounding(egui::Rounding::same(6.0));

    if ui.add(btn).clicked() {
        *current = target;
    }
}

fn render_pill_badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(color.linear_multiply(0.18))
        .stroke(egui::Stroke::new(1.0_f32, color))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(7.0, 2.5))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).color(color).strong().size(11.0));
        });
}

pub fn run_gui() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 800.0])
            .with_min_inner_size([960.0, 600.0])
            .with_title("WinRaze v0.3.0 · Windows Deep Uninstaller & System Debloater"),
        ..Default::default()
    };

    eframe::run_native(
        "WinRaze",
        native_options,
        Box::new(|cc| Ok(Box::new(UninstalrApp::new(cc)))),
    )
}

