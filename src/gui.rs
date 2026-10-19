use eframe::egui;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::i18n::{Language, Locales, Translations};
use crate::model::{AppType, InstalledApp, ScanConfig, UninstallOptions, UninstallReport};
use crate::scanner::SystemScanner;
use crate::ui::format_bytes;
use crate::uninstaller::UninstallationEngine;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterTab {
    All,
    Win32,
    Msi,
    Store,
    Leftovers,
}

enum BackgroundMsg {
    ScanFinished(Vec<InstalledApp>),
    UninstallProgress {
        step_number: u8,
        step_name: String,
        detail: String,
        progress: f32,
    },
    UninstallFinished(UninstallReport),
}

pub struct UninstalrApp {
    apps: Vec<InstalledApp>,
    selected_ids: HashSet<String>,
    active_app_index: Option<usize>,
    search_query: String,
    filter_tab: FilterTab,
    is_scanning: bool,
    show_options: bool,
    show_review_modal: bool,
    options: UninstallOptions,
    current_lang: Language,

    // Uninstallation state
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

        let mut app = Self {
            apps: Vec::new(),
            selected_ids: HashSet::new(),
            active_app_index: None,
            search_query: String::new(),
            filter_tab: FilterTab::All,
            is_scanning: true,
            show_options: false,
            show_review_modal: false,
            options: UninstallOptions::default(),
            current_lang: Language::Turkish,
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

    fn t(&self) -> &'static Translations {
        Locales::get(self.current_lang)
    }

    fn trigger_scan(&mut self) {
        self.is_scanning = true;
        self.apps.clear();
        self.selected_ids.clear();
        self.active_app_index = None;

        let tx = self.tx.clone();
        thread::spawn(move || {
            let scanner = SystemScanner::new(ScanConfig::default());
            let results = scanner.scan_all();
            let _ = tx.send(BackgroundMsg::ScanFinished(results));
        });
    }

    fn trigger_uninstall(&mut self) {
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
}

impl eframe::App for UninstalrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll background messages
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                BackgroundMsg::ScanFinished(results) => {
                    self.apps = results;
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
            }
        }

        // ================= WINUI 3 / WPF-UI FLUENT VISUALS =================
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(32, 32, 32); // Mica / Acrylic dark
        visuals.window_fill = egui::Color32::from_rgb(39, 39, 42); // WinUI 3 Window
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(44, 44, 48);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(49, 49, 54);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(63, 63, 70);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 120, 212); // Windows 11 Blue
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);
        visuals.window_rounding = egui::Rounding::same(12.0);
        ctx.set_visuals(visuals);

        let t = self.t();

        // Compute selection metrics upfront
        let sel_count = self.selected_ids.len();
        let total_sel_bytes: u64 = self
            .apps
            .iter()
            .filter(|a| self.selected_ids.contains(&a.id))
            .map(|a| a.estimated_size_bytes)
            .sum();

        // ================= TOP HEADER & COMMAND BAR =================
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(26, 26, 29)).inner_margin(egui::Margin::symmetric(20.0, 14.0)))
            .show(ctx, |ui| {
                // Row 1: Logo, Title, Language Switcher and Actions
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🛡️").size(26.0));
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading(
                                egui::RichText::new(t.app_title)
                                    .color(egui::Color32::from_rgb(244, 244, 245))
                                    .strong()
                                    .size(20.0),
                            );
                            render_pill_badge(ui, "BETA", egui::Color32::from_rgb(234, 179, 8));
                        });
                        ui.label(
                            egui::RichText::new(t.app_subtitle)
                                .color(egui::Color32::from_rgb(161, 161, 170))
                                .size(12.0),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Language Switcher Pill Button
                        let lang_label = match self.current_lang {
                            Language::Turkish => "🇹🇷 Türkçe",
                            Language::English => "🇬🇧 English",
                        };
                        let lang_btn = egui::Button::new(egui::RichText::new(lang_label).size(12.5).strong())
                            .fill(egui::Color32::from_rgb(45, 45, 52))
                            .rounding(egui::Rounding::same(14.0));

                        if ui.add(lang_btn).clicked() {
                            self.current_lang = match self.current_lang {
                                Language::Turkish => Language::English,
                                Language::English => Language::Turkish,
                            };
                        }

                        // Settings Button
                        let settings_btn = egui::Button::new(egui::RichText::new(format!("⚙ {}", t.btn_settings)).size(13.0))
                            .fill(egui::Color32::from_rgb(45, 45, 52))
                            .rounding(egui::Rounding::same(8.0));
                        if ui.add(settings_btn).clicked() {
                            self.show_options = !self.show_options;
                        }

                        // Rescan Button
                        let rescan_btn = egui::Button::new(egui::RichText::new(format!("🔄 {}", t.btn_rescan)).size(13.0))
                            .fill(egui::Color32::from_rgb(45, 45, 52))
                            .rounding(egui::Rounding::same(8.0));
                        if ui.add(rescan_btn).clicked() {
                            self.trigger_scan();
                        }

                        // Batch Uninstall Button (WinUI 3 Accent / Critical Button)
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
                            egui::Color32::from_rgb(196, 43, 28) // WinUI 3 Critical Accent Red
                        } else {
                            egui::Color32::from_rgb(60, 60, 68)
                        })
                        .rounding(egui::Rounding::same(8.0));

                        if ui.add_enabled(sel_count > 0 && !self.is_uninstalling, uninstall_btn).clicked() {
                            self.trigger_uninstall();
                        }

                        // Review Button
                        if sel_count > 0 {
                            let rev_btn = egui::Button::new(egui::RichText::new(format!("🔍 {}", t.btn_review)).size(13.0))
                                .fill(egui::Color32::from_rgb(0, 120, 212))
                                .rounding(egui::Rounding::same(8.0));
                            if ui.add(rev_btn).clicked() {
                                self.show_review_modal = true;
                            }
                        }
                    });
                });

                ui.add_space(12.0);

                // Row 2: WinUI 3 Modern KPI Metric Cards
                let total_inventory_size: u64 = self.apps.iter().map(|a| a.estimated_size_bytes).sum();
                let leftover_count = self.apps.iter().filter(|a| a.app_type == AppType::Leftover).count();

                ui.horizontal(|ui| {
                    render_kpi_card(ui, "📦", t.stat_apps, &self.apps.len().to_string(), egui::Color32::from_rgb(56, 189, 248));
                    ui.add_space(10.0);
                    render_kpi_card(ui, "💾", t.stat_size, &format_bytes(total_inventory_size), egui::Color32::from_rgb(168, 85, 247));
                    ui.add_space(10.0);
                    render_kpi_card(ui, "🧹", t.stat_selected, &format!("{} ({})", self.selected_ids.len(), format_bytes(total_sel_bytes)), egui::Color32::from_rgb(239, 68, 68));
                    ui.add_space(10.0);
                    render_kpi_card(ui, "⚡", t.stat_leftovers, &leftover_count.to_string(), egui::Color32::from_rgb(245, 158, 11));
                });

                ui.add_space(12.0);

                // Row 3: Modern Search Input & Segmented Filter Tabs
                ui.horizontal(|ui| {
                    // WinUI 3 Search Input
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .desired_width(260.0)
                            .hint_text(t.search_hint)
                            .margin(egui::Margin::symmetric(10.0, 7.0)),
                    );

                    ui.add_space(12.0);

                    // WinUI 3 Segmented Filter Bar
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(40, 40, 45))
                        .rounding(egui::Rounding::same(10.0))
                        .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                render_filter_tab(ui, &mut self.filter_tab, FilterTab::All, t.tab_all);
                                render_filter_tab(ui, &mut self.filter_tab, FilterTab::Win32, t.tab_win32);
                                render_filter_tab(ui, &mut self.filter_tab, FilterTab::Msi, t.tab_msi);
                                render_filter_tab(ui, &mut self.filter_tab, FilterTab::Store, t.tab_store);
                                render_filter_tab(ui, &mut self.filter_tab, FilterTab::Leftovers, t.tab_leftovers);
                            });
                        });
                });
            });

        // ================= BOTTOM STATUS FOOTER =================
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(24, 24, 27)).inner_margin(egui::Margin::symmetric(16.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self.is_scanning {
                        ui.spinner();
                        ui.label(egui::RichText::new(t.scanning_status).color(egui::Color32::from_rgb(212, 212, 216)));
                    } else if self.is_uninstalling {
                        ui.spinner();
                        ui.label(egui::RichText::new(format!("{}: {}", t.modal_uninstalling, self.current_step_name)).color(egui::Color32::from_rgb(239, 68, 68)));
                        ui.add(egui::ProgressBar::new(self.current_progress).show_percentage());
                    } else {
                        ui.label(egui::RichText::new(format!("✔ {}", t.ready_status)).color(egui::Color32::from_rgb(161, 161, 170)));
                    }
                });
            });

        // ================= SIDE DETAILS / INSPECTION DRAWER =================
        if let Some(idx) = self.active_app_index {
            if let Some(app) = self.apps.get(idx) {
                egui::SidePanel::right("app_details_panel")
                    .min_width(340.0)
                    .max_width(460.0)
                    .frame(egui::Frame::none().fill(egui::Color32::from_rgb(28, 28, 32)).inner_margin(egui::Margin::same(16.0)))
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
                                .fill(egui::Color32::from_rgb(38, 38, 44))
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

        // ================= MAIN CENTRAL CARDS LIST =================
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(24, 24, 27)).inner_margin(egui::Margin::symmetric(20.0, 10.0)))
            .show(ctx, |ui| {
                let search_lower = self.search_query.to_lowercase();

                let filtered_apps: Vec<(usize, &InstalledApp)> = self
                    .apps
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| {
                        let tab_match = match self.filter_tab {
                            FilterTab::All => true,
                            FilterTab::Win32 => a.app_type == AppType::Win32,
                            FilterTab::Msi => a.app_type == AppType::Msi,
                            FilterTab::Store => a.app_type == AppType::AppXStore,
                            FilterTab::Leftovers => a.app_type == AppType::Leftover,
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

                // WinUI 3 Command Bar over List: Select All / Deselect All
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

                // Scrollable Modern Card Tiles
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (real_idx, app) in filtered_apps {
                        let mut is_checked = self.selected_ids.contains(&app.id);
                        let is_active = self.active_app_index == Some(real_idx);

                        let card_bg = if is_active {
                            egui::Color32::from_rgb(24, 49, 83) // Focused item
                        } else if is_checked {
                            egui::Color32::from_rgb(45, 45, 54)
                        } else {
                            egui::Color32::from_rgb(34, 34, 38) // WinUI 3 Default Card
                        };

                        let card_border = if is_active {
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 120, 212))
                        } else if is_checked {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(99, 102, 241))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(46, 46, 52))
                        };

                        egui::Frame::none()
                            .fill(card_bg)
                            .stroke(card_border)
                            .rounding(egui::Rounding::same(10.0))
                            .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Checkbox
                                    if ui.checkbox(&mut is_checked, "").changed() {
                                        if is_checked {
                                            self.selected_ids.insert(app.id.clone());
                                        } else {
                                            self.selected_ids.remove(&app.id);
                                        }
                                    }

                                    // Type Pill Badge
                                    let (badge_text, badge_color) = match app.app_type {
                                        AppType::Win32 => ("Win32", egui::Color32::from_rgb(34, 197, 94)),
                                        AppType::Msi => ("MSI", egui::Color32::from_rgb(59, 130, 246)),
                                        AppType::AppXStore => ("Store", egui::Color32::from_rgb(168, 85, 247)),
                                        AppType::Portable => ("Portable", egui::Color32::from_rgb(234, 179, 8)),
                                        AppType::Leftover => ("Leftover", egui::Color32::from_rgb(239, 68, 68)),
                                    };

                                    render_pill_badge(ui, badge_text, badge_color);

                                    ui.add_space(4.0);

                                    // App Name & Publisher
                                    ui.vertical(|ui| {
                                        let name_btn = ui.add(
                                            egui::Button::new(
                                                egui::RichText::new(&app.name)
                                                    .strong()
                                                    .size(14.5)
                                                    .color(egui::Color32::from_rgb(244, 244, 245)),
                                            )
                                            .frame(false),
                                        );

                                        if name_btn.clicked() {
                                            self.active_app_index = Some(real_idx);
                                        }

                                        let pub_str = app.publisher.as_deref().unwrap_or("Bilinmeyen Yayımcı");
                                        let ver_str = app.version.as_deref().unwrap_or("-");
                                        ui.label(
                                            egui::RichText::new(format!("{} · v{}", pub_str, ver_str))
                                                .size(11.5)
                                                .color(egui::Color32::from_rgb(161, 161, 170)),
                                        );
                                    });

                                    // Right-aligned Size Badge & Inspect Action
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let inspect_btn = ui.add(
                                            egui::Button::new(egui::RichText::new("🔍").size(14.0))
                                                .fill(egui::Color32::from_rgb(48, 48, 56))
                                                .rounding(egui::Rounding::same(6.0)),
                                        );
                                        if inspect_btn.clicked() {
                                            self.active_app_index = Some(real_idx);
                                        }

                                        ui.add_space(8.0);

                                        // Size Pill Chip
                                        let size_str = format_bytes(app.estimated_size_bytes);
                                        egui::Frame::none()
                                            .fill(egui::Color32::from_rgb(42, 42, 50))
                                            .rounding(egui::Rounding::same(6.0))
                                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(size_str)
                                                        .color(egui::Color32::from_rgb(251, 191, 36))
                                                        .strong()
                                                        .size(12.5),
                                                );
                                            });
                                    });
                                });
                            });
                        ui.add_space(6.0);
                    }
                });
            });

        // ================= SETTINGS MODAL =================
        if self.show_options {
            egui::Window::new(t.settings_title)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .fixed_size(egui::vec2(480.0, 360.0))
                .show(ctx, |ui| {
                    ui.add_space(6.0);

                    // Language Selector
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(t.settings_lang).strong());
                        ui.selectable_value(&mut self.current_lang, Language::Turkish, "🇹🇷 Türkçe");
                        ui.selectable_value(&mut self.current_lang, Language::English, "🇬🇧 English");
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.checkbox(&mut self.options.auto_kill, t.settings_autokill);
                    ui.checkbox(&mut self.options.clean_empty_dirs, t.settings_emptydirs);
                    ui.checkbox(&mut self.options.clean_broken_shortcuts, t.settings_shortcuts);
                    ui.checkbox(&mut self.options.clean_prefetch, t.settings_prefetch);
                    ui.checkbox(&mut self.options.auto_reboot, t.settings_autoreboot);
                    ui.checkbox(&mut self.options.dry_run, t.settings_dryrun);

                    ui.add_space(16.0);
                    if ui.button(t.btn_save).clicked() {
                        self.show_options = false;
                    }
                });
        }

        // ================= LIVE UNINSTALL PROGRESS MODAL =================
        if self.is_uninstalling {
            egui::Window::new(t.modal_uninstalling)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .fixed_size(egui::vec2(560.0, 340.0))
                .show(ctx, |ui| {
                    ui.heading(&self.current_step_name);
                    ui.add_space(6.0);
                    ui.add(egui::ProgressBar::new(self.current_progress).show_percentage());
                    ui.separator();
                    ui.label(t.modal_live_log);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for log in self.live_logs.iter().rev().take(25) {
                            ui.label(egui::RichText::new(format!("• {}", log)).monospace().size(11.0).color(egui::Color32::from_rgb(212, 212, 216)));
                        }
                    });
                });
        }

        // ================= UNINSTALL COMPLETION REPORT MODAL =================
        let mut close_report = false;
        if let Some(ref report) = self.last_report {
            egui::Window::new(t.modal_complete)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false)
                .resizable(false)
                .fixed_size(egui::vec2(440.0, 280.0))
                .show(ctx, |ui| {
                    ui.heading(format!("{} {}", t.modal_complete, report.app_name));
                    ui.separator();
                    ui.label(format!("• {} {}", t.metric_freed, format_bytes(report.total_freed_bytes)));
                    ui.label(format!("• {} {}", t.metric_files, report.deleted_files_count));
                    ui.label(format!("• {} {}", t.metric_keys, report.deleted_reg_count));
                    ui.label(format!("• {} {}", t.metric_procs, report.terminated_procs_count));
                    ui.label(format!("• {} {}", t.metric_services, report.stopped_services_count));

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

// Helper: Modern KPI Metric Cards
fn render_kpi_card(ui: &mut egui::Ui, icon: &str, title: &str, value: &str, accent_color: egui::Color32) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(34, 34, 38))
        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(46, 46, 52)))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(18.0));
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(title).size(11.0).color(egui::Color32::from_rgb(161, 161, 170)));
                    ui.label(egui::RichText::new(value).size(15.0).strong().color(accent_color));
                });
            });
        });
}

// Helper: Filter Tab Pill
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
        egui::Color32::from_rgb(161, 161, 170)
    };

    let btn = egui::Button::new(egui::RichText::new(label).color(text_color).strong().size(12.0))
        .fill(bg_color)
        .rounding(egui::Rounding::same(8.0));

    if ui.add(btn).clicked() {
        *current = target;
    }
}

// Helper: Colored Pill Badge
fn render_pill_badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(color.linear_multiply(0.2))
        .stroke(egui::Stroke::new(1.0_f32, color))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).color(color).strong().size(10.5));
        });
}

pub fn run_gui() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([880.0, 540.0])
            .with_title("YILLABS DeRust (v0.1.0-beta) - Batch Uninstaller & Deep Cleaner"),
        ..Default::default()
    };

    eframe::run_native(
        "YILLABS_DeRust",
        native_options,
        Box::new(|cc| Ok(Box::new(UninstalrApp::new(cc)))),
    )
}
