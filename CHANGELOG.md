# Changelog

All notable changes to **YILLABS DeRust** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.3.0] - 2026-10-20
### 📷 Kurulum İzleyici & Sistem Anlık Görüntüsü (Snapshot & Install Tracker Engine)

- **Kurulum Öncesi ve Sonrası Sistem Fotoğrafı (Pre/Post-Install Snapshot)**:
  - Hedef kurulum çalıştırılmadan önce `ProgramFiles`, `AppData`, `ProgramData`, Masaüstü/Başlat kısayolları ve kritik kayıt defteri kovanlarını (`HKLM\Software`, `HKCU\Software`, `HKLM\Software\WOW6432Node`) tarayarak belleğe kaydeder.
  - Kurulum tamamlandıktan sonra aynı alanlar tekrar taranır ve iki durum arasındaki fark (Diff Engine) otomatik hesaplanır.
- **Akıllı Gürültü Filtresi (Noise Filtering)**:
  - Windows Update, Defender, Prefetch, tarayıcı önbellekleri ve geçici sistem logları filtrelenerek sadece programa ait yeni dosyalar ve kayıt defteri anahtarları ayıklanır.
- **Kayıt Depolama & Profillendirme (`%LOCALAPPDATA%\WinRaze\Tracked\<id>.json`)**:
  - Tespit edilen tüm oluşturulmuş dosyalar, klasörler ve kayıt defteri anahtarları izleme profili olarak diske kaydedilir.
- **Sıfır Artık Kaldırma (Zero-Leftover Uninstallation)**:
  - İzlenerek kurulan uygulamalar normal kaldırıcıya ihtiyaç kalmadan doğrudan izleme günlüğündeki tüm girdiler silinerek %100 artık kalıntısız temizlenir.
- **Yeni GUI ve CLI Komutları**:
  - **Masaüstü GUI**: Sol navigasyon rayına `🔍 Kurulum İzleyici` sekmesi, program kurma sihirbazı ve kayıtlı izlenen programlar listesi eklendi.
  - **CLI**: `winraze track <setup.exe>`, `winraze tracked-list` ve `winraze tracked-remove <id>` komutları kullanıma sunuldu.

---

## [0.2.0-beta] - 2026-10-20
### 🛡️ Safety, Health Dashboard, Browser Extensions & Force Shredder (Distinctive UI Overhaul)

- **Distinctive WinUI 3 Navigation Rail & Dashboard UI**:
  - Replaced the single-window top header with an original, modern **Fluent Left Navigation Rail** (avoiding generic copycat table designs of Uninstalr or spammy promotional UI of IObit).
  - Categorized views: 📊 **Sistem Sağlığı (System Health)**, 📦 **Kurulu Yazılımlar (Apps)**, 🧩 **Tarayıcı Eklentileri (Extensions)**, ⚡ **Kalıntılar & Artıklar (Residuals)**, 🔥 **Zorla Kaldırıcı (Force Shredder)**, and ⚙️ **Güvenlik & Tercihler (Settings)**.
- **Windows System Health Index & 1-Click Optimization**:
  - Calculates a real-time system health score (%0-100) by analyzing registry residues, crash dumps in `%LOCALAPPDATA%\CrashDumps`, broken shortcuts, and prefetch bloat.
  - Added **"⚡ 1-Tık Hızlı Sağlık Temizliği"** button and CLI command (`DeRust.exe optimize`) that automatically purges crash dumps, prefetch cache, and broken shortcuts with 1 click.
- **Multi-Browser Extensions Manager**:
  - Automatically discovers extensions across Google Chrome, Microsoft Edge, and Brave browsers from their respective User Data directories.
  - Allows 1-click removal of unwanted browser extensions and displays extension IDs, version, and footprint.
- **Force Uninstaller & File/Directory Shredder (`DeRust.exe force-clean`)**:
  - Eliminates stubborn software directories, terminating any processes locking files, unregistering background services, purging related registry keys, and scheduling boot deletions if needed.
  - Native Windows folder browsing dialog integration via `rfd`.
- **Pre-Uninstall Security Verification Modal**:
  - Added a dedicated safety confirmation sheet before executing uninstallation, allowing users to toggle Restore Point creation, `.reg` backup, and AutoKill, with automatic warnings if protected system components are selected.
- **Windows System Restore Point Integration (Step 0)**:
  - Automates system restore point creation (`Checkpoint-Computer`) prior to running batch uninstallation pipelines, ensuring users can roll back system state.
- **Pre-Deletion Registry Backup (`.reg` Rollback Export)**:
  - Automatically exports all targeted registry subkeys to `%LOCALAPPDATA%\DeRust\Backups\Registry\` before deletion using native Windows registry export.
- **Smart Protection Shield**:
  - Automatically detects critical system runtimes and drivers (Microsoft Visual C++, DirectX, NVIDIA/AMD/Intel drivers, .NET runtimes) and displays a `[🛡️ Sistem Koruması]` badge in the GUI to prevent accidental deletion.
- **Shell Extensions & Context Menu Handlers Purge**:
  - Detects and cleans orphaned shell context menu handlers in `HKEY_CLASSES_ROOT\*\shellex\ContextMenuHandlers` and `Directory\shellex\ContextMenuHandlers`.
- **Application Crash Dump Leftover Cleaner**:
  - Scans `%LOCALAPPDATA%\CrashDumps` for abandoned `.dmp` files left behind by uninstalled applications.
- **New CLI Subcommands**:
  - `DeRust.exe health`: Evaluates and displays the Windows System Health Index and telemetry breakdown.
  - `DeRust.exe optimize`: Executes 1-click quick health optimization.
  - `DeRust.exe force-clean <path>`: Forcefully eradicates any folder and related registry keys.
  - `DeRust.exe extensions`: Discovers and lists all browser extensions.

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
