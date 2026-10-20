mod debloater;
mod gui;
mod i18n;
mod model;
mod process_killer;
mod scanner;
mod ui;
mod uninstaller;

use clap::{Parser, Subcommand};
use colored::*;

use debloater::{DebloaterEngine, KNOWN_BLOATWARE};
use model::{ScanConfig, UninstallOptions};
use scanner::SystemScanner;
use ui::TerminalUI;
use uninstaller::UninstallationEngine;

#[derive(Parser)]
#[command(name = "winraze")]
#[command(version = "0.1.0")]
#[command(about = "WinRaze: Raze the bloat. Reclaim your Windows. Powered by Rust. 🦀")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan system and report bloatware, leftovers, and reclaimable space (Read-Only safe mode)
    Scan {
        /// Filter apps by name substring
        #[arg(short, long)]
        filter: Option<String>,

        /// Exclude Microsoft Store / AppX packages
        #[arg(long)]
        no_store: bool,

        /// Output as raw JSON
        #[arg(long)]
        json: bool,
    },

    /// Launch the interactive GUI Desktop Application
    Gui,

    /// Unattended purge: raze preinstalled bloatware packages and disable telemetry
    Purge {
        /// Automatic yes to prompts; run unassisted
        #[arg(short, long)]
        yes: bool,
    },

    /// Clean temporary files, Windows temp, and browser caches
    Clean {
        /// Clean temporary files and caches
        #[arg(long)]
        temp: bool,
    },

    /// List all installed applications, Store apps and leftovers
    List {
        /// Filter apps by name substring
        #[arg(short, long)]
        filter: Option<String>,

        /// Exclude Microsoft Store / AppX packages
        #[arg(long)]
        no_store: bool,

        /// Output as raw JSON
        #[arg(long)]
        json: bool,
    },

    /// Deeply inspect an application and preview all residual artifacts
    Inspect {
        /// Application name, ID, or substring
        target: String,

        /// Output preview as JSON
        #[arg(long)]
        json: bool,
    },

    /// Uninstall one or multiple applications in an unattended batch
    Uninstall {
        /// One or more application names or indices to remove
        targets: Vec<String>,

        /// Simulation only: preview deletions without modifying system
        #[arg(long)]
        dry_run: bool,

        /// Automatically terminate locking processes & services
        #[arg(long, default_value_t = true)]
        auto_kill: bool,

        /// Clean empty parent directories afterwards
        #[arg(long, default_value_t = true)]
        clean_empty_dirs: bool,

        /// Automatically reboot if locked files are scheduled for boot removal
        #[arg(long)]
        auto_reboot: bool,

        /// Skip creating a Windows System Restore point before uninstall
        #[arg(long)]
        no_restore_point: bool,

        /// Skip backing up registry keys to .reg files before deletion
        #[arg(long)]
        no_backup: bool,
    },

    /// Scan system exclusively for orphaned leftover files and directories
    Leftovers,

    /// Calculate Windows System Health Index and telemetry/crash breakdown
    Health,

    /// 1-Click quick health optimization (CrashDumps, Prefetch cache, broken shortcuts)
    Optimize {
        /// Simulation only: preview deletions without modifying system
        #[arg(long)]
        dry_run: bool,
    },

    /// Forcefully eradicate a stubborn software folder, terminating locking processes and removing registry keys
    ForceClean {
        /// Absolute folder or file path to eradicate
        path: String,

        /// Simulation only: preview deletions without modifying system
        #[arg(long)]
        dry_run: bool,
    },

    /// List all discovered browser extensions (Chrome, Edge, Brave)
    Extensions,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Gui) | None => {
            // Launch the native Desktop GUI by default!
            if let Err(e) = gui::run_gui() {
                eprintln!("Failed to launch GUI: {}. Falling back to CLI...", e);
                // Fallback to CLI listing
                TerminalUI::print_banner();
                let scanner = SystemScanner::new(ScanConfig::default());
                let spinner = TerminalUI::create_spinner("Scanning system registries, AppX catalog and data locations...");
                let apps = scanner.scan_all();
                spinner.finish_and_clear();
                TerminalUI::print_app_list(&apps);
            }
        }

        Some(Commands::Scan { filter, no_store, json }) | Some(Commands::List { filter, no_store, json }) => {
            let config = ScanConfig {
                include_store_apps: !no_store,
                include_portable_apps: true,
                detect_leftovers: true,
                deep_scan_paths: Vec::new(),
            };

            let scanner = SystemScanner::new(config);
            let mut apps = scanner.scan_all();

            if let Some(f) = filter {
                let f_lower = f.to_lowercase();
                apps.retain(|a| a.name.to_lowercase().contains(&f_lower));
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&apps).unwrap());
            } else {
                TerminalUI::print_banner();
                TerminalUI::print_app_list(&apps);
            }
        }

        Some(Commands::Inspect { target, json }) => {
            let scanner = SystemScanner::new(ScanConfig::default());
            let apps = scanner.scan_all();

            let target_lower = target.to_lowercase();
            let matched = apps.into_iter().find(|a| {
                a.name.to_lowercase().contains(&target_lower) || a.id.to_lowercase() == target_lower
            });

            if let Some(app) = matched {
                if json {
                    println!("{}", serde_json::to_string_pretty(&app).unwrap());
                } else {
                    TerminalUI::print_banner();
                    TerminalUI::print_app_review(&app);
                }
            } else {
                eprintln!("{}", format!("[-] No application found matching query: '{}'", target).red());
            }
        }

        Some(Commands::Uninstall {
            targets,
            dry_run,
            auto_kill,
            clean_empty_dirs,
            auto_reboot,
            no_restore_point,
            no_backup,
        }) => {
            TerminalUI::print_banner();

            if targets.is_empty() {
                eprintln!("{}", "[-] Please specify at least one target application to uninstall.".red());
                return;
            }

            let scanner = SystemScanner::new(ScanConfig::default());
            let spinner = TerminalUI::create_spinner("Scanning system inventory for selected targets...");
            let apps = scanner.scan_all();
            spinner.finish_and_clear();

            let options = UninstallOptions {
                auto_kill,
                auto_reboot,
                dry_run,
                create_restore_point: !no_restore_point,
                backup_registry: !no_backup,
                clean_empty_dirs,
                clean_broken_shortcuts: true,
                clean_prefetch: true,
                timeout_seconds: 120,
            };

            let engine = UninstallationEngine::new(options);

            for target in targets {
                let target_lower = target.to_lowercase();
                if let Some(app) = apps.iter().find(|a| a.name.to_lowercase().contains(&target_lower) || a.id.to_lowercase() == target_lower) {
                    let report = engine.execute_uninstallation(app);
                    TerminalUI::print_uninstall_report(&report);
                } else {
                    eprintln!("{}", format!("[-] Target app not found: '{}'", target).red());
                }
            }
        }

        Some(Commands::Leftovers) => {
            TerminalUI::print_banner();
            let scanner = SystemScanner::new(ScanConfig::default());
            let apps = scanner.scan_all();
            let leftovers: Vec<_> = apps.into_iter().filter(|a| a.app_type == model::AppType::Leftover).collect();

            println!("{}", "=== Discovered Orphaned Software Leftovers ===".yellow().bold());
            TerminalUI::print_app_list(&leftovers);
        }

        Some(Commands::Health) => {
            TerminalUI::print_banner();
            let scanner = SystemScanner::new(ScanConfig::default());
            let spinner = TerminalUI::create_spinner("Evaluating Windows System Health Index...");
            let apps = scanner.scan_all();
            let health = scanner.calculate_system_health(&apps);
            spinner.finish_and_clear();

            println!("\n{}", "================ Windows System Health Index ================".cyan().bold());
            let score_colored = if health.health_score >= 80 {
                format!("%{}", health.health_score).green().bold()
            } else if health.health_score >= 60 {
                format!("%{}", health.health_score).yellow().bold()
            } else {
                format!("%{}", health.health_score).red().bold()
            };
            println!("  Overall Health Score:       {}", score_colored);
            println!("  Orphaned Leftovers:         {} (Total: {})", health.leftover_count, ui::format_bytes(health.leftover_bytes));
            println!("  Crash Reports (CrashDumps): {} (Total: {})", health.crash_dump_count, ui::format_bytes(health.crash_dump_bytes));
            println!("  Broken Shortcuts:           {}", health.broken_shortcuts_count);
            println!("  Protected System Apps:      {}", health.protected_count);
            println!("{}", "=============================================================\n".cyan());
        }

        Some(Commands::Optimize { dry_run }) => {
            TerminalUI::print_banner();
            println!("{}", "[*] Running 1-Click Quick System Health Cleanup...".yellow().bold());
            let mut options = UninstallOptions::default();
            options.dry_run = dry_run;
            let engine = UninstallationEngine::new(options);
            let report = engine.quick_health_cleanup();
            TerminalUI::print_uninstall_report(&report);
        }

        Some(Commands::ForceClean { path, dry_run }) => {
            TerminalUI::print_banner();
            println!("{}", format!("[*] Force Clean / Shredding Target Path: {}", path).red().bold());
            let target_path = std::path::PathBuf::from(&path);
            let mut options = UninstallOptions::default();
            options.dry_run = dry_run;
            let engine = UninstallationEngine::new(options);
            let report = engine.force_clean_path(&target_path);
            TerminalUI::print_uninstall_report(&report);
        }

        Some(Commands::Extensions) => {
            TerminalUI::print_banner();
            let scanner = SystemScanner::new(ScanConfig::default());
            let spinner = TerminalUI::create_spinner("Scanning browser extensions (Chrome, Edge, Brave)...");
            let apps = scanner.scan_all();
            spinner.finish_and_clear();

            let extensions: Vec<_> = apps.into_iter().filter(|a| a.app_type == model::AppType::BrowserExtension).collect();
            println!("{}", format!("=== Discovered Browser Extensions ({}) ===", extensions.len()).magenta().bold());
            TerminalUI::print_app_list(&extensions);
        }

        Some(Commands::Purge { yes }) => {
            TerminalUI::print_banner();
            println!("{}", "🔥 WinRaze: Unattended Bloatware & Telemetry Purge Protocol Initiated".red().bold());
            if !yes {
                println!("{}", "[-] Use --yes flag to confirm unattended eradication: winraze purge --yes".yellow());
                return;
            }

            println!("{}", "[*] Disabling Windows Telemetry & Diagnostic Data Tracking...".cyan());
            let telem_logs = DebloaterEngine::disable_telemetry();
            for l in telem_logs {
                println!("  • {}", l.green());
            }

            println!("\n{}", "[*] Razing Preinstalled Windows Bloatware Packages...".magenta());
            for target in KNOWN_BLOATWARE {
                let success = DebloaterEngine::remove_bloatware(target);
                let mark = if success { "✔".green() } else { "•".dimmed() };
                println!("  {} {} ({})", mark, target.name.bold(), target.category.dimmed());
            }

            println!("\n{}", "[*] Purging System Temp & Browser Caches...".yellow());
            let (count, bytes, _) = DebloaterEngine::clean_temporary_files();
            println!("  ✔ Cleaned {} temporary cache files ({})", count, ui::format_bytes(bytes).green().bold());

            println!("\n{}", "✔ WinRaze Purge Completed. System debloated and privacy secured.".green().bold());
        }

        Some(Commands::Clean { temp }) => {
            TerminalUI::print_banner();
            if temp {
                println!("{}", "[*] Cleaning Temporary Files, Windows Temp, and Browser Caches...".cyan().bold());
                let (count, bytes, logs) = DebloaterEngine::clean_temporary_files();
                for l in logs {
                    println!("  • {}", l);
                }
                println!("\n{}", format!("✔ Successfully freed {} across {} files.", ui::format_bytes(bytes), count).green().bold());
            } else {
                println!("{}", "[-] Specify what to clean, for example: winraze clean --temp".yellow());
            }
        }
    }
}
