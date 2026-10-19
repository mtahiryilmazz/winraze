# 🗺️ YILLABS DeRust — Product Roadmap

This document outlines the strategic development plan and upcoming feature milestones for **YILLABS DeRust**.

---

## 📍 Phase 1: Core Engine & Public Beta `[v0.1.0-beta]` — *(Completed)*

- [x] **9-Step Deep Uninstallation Engine**:
  - [x] AutoKill: Process termination via `ToolHelp32` & Service shutdown via `sc stop`.
  - [x] Desktop & Start Menu shortcut cleanup.
  - [x] MSI silent switch injection (`/quiet /norestart /passive`).
  - [x] Inno Setup, NSIS, and native installer silent flag injection.
  - [x] Modern AppX / MSIX package removal via PowerShell runtime.
  - [x] Deep registry subkey purge (`HKCU`, `HKLM`, `Wow6432Node`, `AppCompatFlags`).
  - [x] Win32 `MoveFileExW` (`MOVEFILE_DELAY_UNTIL_REBOOT`) reboot deletion queue for locked files.
  - [x] Empty parent directory pruning and Windows Prefetch (`.pf`) cleanup.
- [x] **Modern WinUI 3 / WPF-UI Desktop GUI**:
  - [x] Acrylic & Mica dark theme palette.
  - [x] Top KPI Metric Cards (Installed Apps, Total Size, Selected Space, Leftovers).
  - [x] Fluent Card Tile list view with type pill badges and size chips.
  - [x] Segmented filter control & real-time search.
  - [x] Side Inspection Drawer previewing all detected residual files, registry keys, and shortcuts.
  - [x] Live 9-step progress modal with streaming log.
- [x] **Localization**:
  - [x] 🇹🇷 Turkish & 🇬🇧 English complete translations with on-the-fly switching.
- [x] **Command Line Interface (CLI)**:
  - [x] `list`, `inspect`, `uninstall`, `leftovers`, and `--dry-run` simulation modes.

---

## 📍 Phase 2: Safety, Snapshots & Rollback `[v0.2.0-beta]` — *(In Progress)*

- [ ] **Windows System Restore Point Integration**:
  - Automatic creation of a System Restore point prior to initiating batch uninstallation.
- [ ] **Registry & File Backup (Rollback Capability)**:
  - Export deleted registry keys to `.reg` backup files before deletion.
  - One-click restore button in case of accidental removal.
- [ ] **Smart Protection / Whitelist System**:
  - Built-in protection rules for mission-critical system components:
    - Microsoft Visual C++ Redistributables
    - DirectX Runtime
    - Graphics drivers (NVIDIA, AMD, Intel display drivers)
    - Windows Defender & system runtimes
- [ ] **Custom Exclusion Rules**:
  - Allow users to mark apps as "Protected / Never Uninstall".

---

## 📍 Phase 3: Deep System Cleaning & Shell Extensions `[v0.3.0]`

- [ ] **Context Menu Cleaner**:
  - Detect and remove orphaned right-click menu items left behind in Windows Explorer (`HKEY_CLASSES_ROOT\*\shellex`).
- [ ] **Windows Services & Scheduled Tasks Purger**:
  - Clean abandoned scheduled tasks (`schtasks`) and orphaned driver services.
- [ ] **Browser Extensions & Cache Cleaner**:
  - Detect leftover extensions, plugins, and cache data left behind by uninstalled browsers.
- [ ] **Startup Manager**:
  - View and disable startup entries associated with removed or unnecessary applications.

---

## 📍 Phase 4: Community Cloud Rules & Crowdsourced Signatures `[v0.4.0]`

- [ ] **Crowdsourced Leftover Rules Database**:
  - Cloud-synced definition files for notoriously stubborn software suites (Adobe Creative Cloud, Autodesk, Antivirus software, OEM bloatware).
- [ ] **Installation Monitor (Real-time Snapshot)**:
  - "Install with DeRust": Record system changes during an installer run to ensure 100.0% clean removal later.
- [ ] **Telemetry-Free Crash Reporting & Diagnostic Logs**:
  - Optional localized crash logs for user bug reports.

---

## 📍 Phase 5: v1.0 General Availability (GA Release)

- [ ] **Official Package Distribution**:
  - Windows Package Manager (`winget install YILLABS.DeRust`).
  - Chocolatey and Scoop manifest submissions.
- [ ] **Code Signing & Integrity**:
  - Digitally signed Windows executable and MSIX installer.
- [ ] **Auto-Updater**:
  - Seamless, background update notifications for new DeRust releases.
- [ ] **Light / Dark Mode Dynamic Theme Matching**:
  - Respect Windows 11 system accent colors and light/dark theme preference.

---

*Roadmap subject to community feedback and contributor priorities. Submit feature requests via [GitHub Issues](https://github.com/mtahiryilmazz).*
