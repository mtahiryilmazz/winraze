use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::model::{AppType, InstalledApp, UninstallReport};

pub struct TerminalUI;

impl TerminalUI {
    pub fn print_banner() {
        println!("{}", r#"
 __   _____ _        _    ____ ____  
 \ \ / /_ _| |      / \  | __ ) ___| 
  \ V / | || |     / _ \ |  _ \___ \ 
   | |  | || |___ / ___ \| |_) |__) |
   |_| |___|_____/_/   \_\____/____/ 
                                     
      YILLABS DeRust · v0.1.0-beta
"#.cyan().bold());
    }

    pub fn create_spinner(msg: &'static str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(80));
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(msg);
        pb
    }

    pub fn print_app_list(apps: &[InstalledApp]) {
        println!(
            "{:<4} {:<36} {:<15} {:<12} {:<10} {}",
            "ID".bold(),
            "Application Name".bold(),
            "Version".bold(),
            "Size".bold(),
            "Type".bold(),
            "Publisher".bold()
        );
        println!("{}", "-".repeat(100).dimmed());

        for (idx, app) in apps.iter().enumerate() {
            let size_str = format_bytes(app.estimated_size_bytes);
            let version_str = app.version.as_deref().unwrap_or("-");
            let pub_str = app.publisher.as_deref().unwrap_or("-");

            let type_str = match app.app_type {
                AppType::Win32 => "Win32".green(),
                AppType::Msi => "MSI".blue(),
                AppType::AppXStore => "Store".magenta(),
                AppType::Portable => "Portable".yellow(),
                AppType::Leftover => "Leftover".red(),
                AppType::BrowserExtension => "Extension".cyan(),
            };

            let truncated_name = if app.name.chars().count() > 34 {
                format!("{}...", app.name.chars().take(31).collect::<String>())
            } else {
                app.name.clone()
            };

            let truncated_pub = if pub_str.chars().count() > 25 {
                format!("{}...", pub_str.chars().take(22).collect::<String>())
            } else {
                pub_str.to_string()
            };

            println!(
                "{:<4} {:<36} {:<15} {:<12} {:<10} {}",
                idx + 1,
                truncated_name.white().bold(),
                version_str.dimmed(),
                size_str.yellow(),
                type_str,
                truncated_pub.dimmed()
            );
        }
        println!("{}", "-".repeat(100).dimmed());
        println!("Total discovered: {} software items", apps.len().to_string().green().bold());
    }

    pub fn print_app_review(app: &InstalledApp) {
        println!("\n{}", "========================================================".cyan());
        println!("{} {}", "Inspection Review:".bold(), app.name.green().bold());
        println!("{}", "========================================================".cyan());

        if let Some(ref publ) = app.publisher {
            println!("  Publisher:        {}", publ);
        }
        if let Some(ref ver) = app.version {
            println!("  Version:          {}", ver);
        }
        if let Some(ref loc) = app.install_location {
            println!("  Install Location: {}", loc.display());
        }
        if let Some(ref uninst) = app.uninstall_string {
            println!("  Uninstall String: {}", uninst);
        }
        println!("  Calculated Size:  {}", format_bytes(app.estimated_size_bytes).yellow().bold());
        println!("  Total Artifacts:  {}", app.artifacts.len().to_string().cyan().bold());

        println!("\n{}", "--- Tracked Residual Artifacts ---".dimmed());
        for (i, art) in app.artifacts.iter().enumerate() {
            let size_badge = art.size_bytes.map(|s| format!(" ({})", format_bytes(s))).unwrap_or_default();
            println!("  [{:>2}] {:<14} : {}{}", 
                i + 1, 
                format!("{:?}", art.kind).yellow(), 
                art.path_or_target, 
                size_badge.dimmed()
            );
        }
    }

    pub fn print_uninstall_report(report: &UninstallReport) {
        println!("\n{}", "========================================================".green());
        println!("{} {}", "Uninstallation Report for:".bold(), report.app_name.white().bold());
        println!("{}", "========================================================".green());

        for step in &report.steps {
            let status = if step.success { "✔".green() } else { "✘".red() };
            println!(" {} Step {}: {}", status, step.step_number, step.step_name.bold());
            for detail in &step.details {
                println!("    • {}", detail.dimmed());
            }
        }

        println!("\n{}", "--- Summary Statistics ---".bold());
        println!("  Files/Dirs Cleaned:       {}", report.deleted_files_count.to_string().green());
        println!("  Registry Keys Removed:    {}", report.deleted_reg_count.to_string().green());
        println!("  Processes Terminated:     {}", report.terminated_procs_count.to_string().yellow());
        println!("  Services Stopped:         {}", report.stopped_services_count.to_string().yellow());
        println!("  Total Disk Space Freed:   {}", format_bytes(report.total_freed_bytes).green().bold());

        if report.requires_reboot {
            println!("\n{}", "⚠ NOTICE: Locked files were scheduled for deletion on next system reboot.".yellow().bold());
        } else {
            println!("\n{}", "✔ Clean removal complete. No system reboot required.".green().bold());
        }
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
