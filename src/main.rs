mod gui;
mod i18n;
mod model;
mod process_killer;
mod scanner;
mod ui;
mod uninstaller;

use clap::{Parser, Subcommand};
use colored::*;

use model::{ScanConfig, UninstallOptions};
use scanner::SystemScanner;
use ui::TerminalUI;
use uninstaller::UninstallationEngine;

#[derive(Parser)]
#[command(name = "derust")]
#[command(author = "M. Tahir Yılmaz <YILLABS>")]
#[command(version = "0.1.0-beta")]
#[command(about = "YILLABS DeRust: Modern, high-performance batch uninstaller & deep residual cleaner for Windows")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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
    },

    /// Scan system exclusively for orphaned leftover files and directories
    Leftovers,

    /// Launch the interactive GUI Desktop Application
    Gui,
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
        Some(Commands::List { filter, no_store, json }) => {
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
    }
}
