# Changelog

All notable changes to **YILLABS DeRust** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0-beta] - 2026-10-19

### 🚀 Public Beta Release

This marks the first public beta release of **YILLABS DeRust**, an open-source, ultra-fast Windows batch uninstaller and deep residual cleaner inspired by the internal architecture of Uninstalr, developed under **YILLABS**.

### ✨ Added
* **9-Step Deep Uninstallation Engine**:
  * **Step 1 & 2 (AutoKill)**: Enumerates processes locking application files using Windows `ToolHelp32` API and shuts down related Windows Services via `sc stop`.
  * **Step 3 (Shortcuts)**: Cleans orphaned `.lnk` and `.url` shortcuts from Start Menu (`%ProgramData%`, `%AppData%`) and Desktop (`%Public%`, `%UserProfile%`).
  * **Step 4 (Silent MSI)**: Detects MSI installer product codes and executes silent uninstallation (`msiexec /X {GUID} /quiet /norestart /passive`).
  * **Step 5 (Native Uninstaller)**: Executes native setup uninstallers with injected silent switches for Inno Setup (`/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`), NSIS (`/S`), and generic installers.
  * **Step 6 (AppX / UWP)**: Full removal of modern Microsoft Store packages via PowerShell `Remove-AppxPackage`.
  * **Step 7 (Deep Registry Purge)**: Cleans orphaned keys across `HKLM`, `HKCU`, `Wow6432Node`, `AppCompatFlags`, and shell history.
  * **Step 8 (Filesystem Cleanup & Reboot Queue)**: Purges application files and `AppData` directories. In case of locked system files, queues them for deletion on next reboot using Win32 `MoveFileExW(..., MOVEFILE_DELAY_UNTIL_REBOOT)`.
  * **Step 9 (Post-Processing & Pruning)**: Prunes empty parent directories, dead shortcuts, and cleans Windows Prefetch (`C:\Windows\Prefetch\*.pf`) cache records.
* **WinUI 3 & WPF-UI Desktop GUI**:
  * Modern Mica/Acrylic dark aesthetic (`#202020` & `#27272a`).
  * Top KPI Metric Cards (Total Apps, Total Size, Selected Space, Orphaned Leftovers).
  * Fluent Card Tiles for application list with real-time size chips, publisher labels, and type pill badges.
  * Segmented filter control (`All`, `Win32`, `MSI`, `Store`, `Leftovers`).
  * Real-time search and filter text box.
  * Side Inspection Drawer previewing all detected residual registry keys, directories, shortcuts, and prefetch files.
  * Live uninstallation modal displaying real-time step names, percentage progress bar, and streaming log.
* **Full Localization (i18n)**:
  * 🇹🇷 **Türkçe** (Default)
  * 🇬🇧 **English**
  * Instant runtime switching via top command bar pill or settings window.
* **Command Line Interface (CLI)**:
  * `derust list`: Lists installed Win32, MSI, Store apps and leftovers with size calculations.
  * `derust inspect <query>`: Inspects residual artifacts for a given application.
  * `derust uninstall <targets...>`: Unattended batch removal.
  * `--dry-run`: Simulation mode without touching the filesystem or registry.
  * `derust leftovers`: Scans for orphaned residual folders.
