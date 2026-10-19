# 🛡️ YILLABS DeRust

<div align="center">

**The High-Performance Native Windows Batch Uninstaller & Deep Cleaner**  
*Sistemin Dijital Pasını Söken Modern Toplu Kaldırıcı*

[![Version](https://img.shields.io/badge/Version-v0.1.0--beta-yellow.svg?style=flat-square)](CHANGELOG.md)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078d4.svg?style=flat-square&logo=windows)](https://github.com/mtahiryilmazz)
[![UI](https://img.shields.io/badge/Design-WinUI%203%20%7C%20Fluent-blue.svg?style=flat-square)](https://github.com/mtahiryilmazz)
[![Engine](https://img.shields.io/badge/Architecture-9--Step%20Pipeline-success.svg?style=flat-square)](https://github.com/mtahiryilmazz)
[![Language](https://img.shields.io/badge/Language-TR%20%7C%20EN-purple.svg?style=flat-square)](https://github.com/mtahiryilmazz)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)

*Developed by **[YILLABS](ORGANIZATION.md)** · Authored by **M. Tahir Yılmaz***

[Features](#-features--özellikler) • [Roadmap](ROADMAP.md) • [Changelog](CHANGELOG.md) • [Organization](ORGANIZATION.md) • [Quick Start](#-quick-start--hızlı-başlangıç) • [Architecture](#-9-step-pipeline-architecture)

</div>

---

> [!NOTE]
> **Beta Aşamasındadır (Public Beta):**  
> YILLABS DeRust şu an aktif geliştirme aşamasındadır (**v0.1.0-beta**). Karşılaştığınız hataları veya önerilerinizi [GitHub Issues](https://github.com/mtahiryilmazz/derust/issues) üzerinden paylaşabilirsiniz. Gelecek özellik planlaması için **[ROADMAP.md](ROADMAP.md)** dosyasını inceleyin.

---

## 🌟 What is YILLABS DeRust?

Windows'un dahili "Program Ekle veya Kaldır" arayüzü çoğu zaman arkasında gigabaytlarca artık veri, yüzlerce yetim kayıt defteri anahtarı ve arka planda çalışmaya devam eden kilitli servisler bırakır.

**YILLABS DeRust**, popüler Windows kaldırıcı *Uninstalr* mimarisinden ilham alarak **YILLABS** çatısı altında sıfırdan geliştirilmiş, tek tıklamayla birden çok yazılımı sessizce ve kökten temizleyen **yüksek performanslı bir açık kaynak masaüstü güç aracıdır**.

---

## ✨ Features / Özellikler

### 1. 🪟 WinUI 3 & Fluent Desktop GUI
* **Mica & Acrylic Karanlık Arayüz:** Windows 11 Fluent tasarım standartlarına uygun, yuvarlatılmış köşeler ve modern renk paleti.
* **Üst Panel KPI İstatistik Kartları:** Toplam yazılım adedi, kaplanan gerçek disk alanı, kaldırılmak üzere seçilen alan ve sahipsiz kalıntılar için anlık sayaçlar.
* **Modern Kart-Karo (Card Tiles) Mimarisi:** Düzensiz tablolar yerine tip etiketi (`Win32`, `MSI`, `Store`, `Leftover`), yayımcı bilgisi ve altın sarısı disk boyutu çipine sahip akıcı kartlar.
* **Segmented Filter Bar:** `Tümü`, `Win32`, `MSI`, `Store / UWP` ve `Kalıntılar` arasında tek tıkla anında filtreleme.
* **Sağ Açılır Derin İnceleme Çekmecesi:** Herhangi bir karta tıklandığında kurulum dizini, kaldırıcı komutu, kısayollar ve tespit edilen tüm kayıt defteri anahtarlarını önizleme imkânı.

### 2. 🌐 Tam Yerelleştirme (Localization - TR / EN)
* **Çift Dil:** 🇹🇷 **Türkçe** (Varsayılan) ve 🇬🇧 **English**.
* Üst çubuktaki tek tıkla `🇹🇷 Türkçe` / `🇬🇧 English` butonu veya ayarlar menüsü üzerinden tüm arayüzü anında tercüme edebilme.

### 3. ⚙️ 9 Aşamalı Derin Kaldırma Boru Hattı (The 9-Step Pipeline)
1. **Adım 1 & 2 (AutoKill):** Kaldırılacak yazılımın dosyalarını kilitleyen süreçleri (`ToolHelp32` API) ve Windows servislerini (`sc stop`) otomatik tespit edip sonlandırır.
2. **Adım 3 (Kısayollar):** Başlat Menüsü ve Masaüstü üzerindeki `.lnk`/`.url` dosyalarını temizler.
3. **Adım 4 (Sessiz MSI):** MSI paketlerini otomatik tespit edip sessiz modda (`msiexec /X {GUID} /quiet /norestart /passive`) çalıştırır.
4. **Adım 5 (Yerel Kaldırıcı):** InnoSetup (`/VERYSILENT /NORESTART`), NSIS (`/S`) sessiz anahtarlarını enjekte ederek çalıştırır.
5. **Adım 6 (AppX Store):** UWP / Microsoft Store paketlerini `Remove-AppxPackage` ile temizler.
6. **Adım 7 (Derin Kayıt Defteri Temizliği):** `HKCU`, `HKLM`, `Wow6432Node`, `AppCompatFlags` altındaki tüm ilişkili anahtarları siler.
7. **Adım 8 (Dosya Sistemi & Kilitli Dosyalar):** Kurulum dizinini ve `AppData` klasörlerini siler; işletim sistemi tarafından kilitlenen dosyaları Windows API'si `MoveFileExW(..., MOVEFILE_DELAY_UNTIL_REBOOT)` ile yeniden başlatmada temizlenmek üzere kuyruğa alır.
8. **Adım 9 (Post-Processing & Pruning):** Boş kalan üst klasörleri, kırık kısayolları ve Windows Prefetch (`.pf`) önbellek kayıtlarını budar.

---

## 🚀 Quick Start / Hızlı Başlangıç

Proje doğrudan çift tıklanarak çalışmaya hazır tek bir bağımsız Windows çalıştırılabilir dosyası (`.exe`) olarak üretilmiştir:

### 🖥️ Grafik Arayüzü (GUI) Başlatma
Doğrudan [DeRust.exe](file:///C:/Users/BT/.gemini/antigravity-ide/scratch/uninstalr-rs/DeRust.exe) (veya `YILLABS-DeRust.exe`) dosyasına **çift tıklayarak** masaüstü uygulamasını başlatabilirsiniz.

PowerShell üzerinden çalıştırmak için:
```powershell
cd "C:\Users\BT\.gemini\antigravity-ide\scratch\uninstalr-rs"
.\DeRust.exe
```

### ⌨️ Komut Satırı (CLI) Modu
Aynı ikili dosya arka plan otomasyonları ve sunucular için zengin parametre desteğine sahiptir:

```powershell
# 1. Kurulu tüm uygulamaları ve disk alanlarını listeleme
.\DeRust.exe list

# 2. Belirli bir uygulamayı arama
.\DeRust.exe list --filter "Chrome"

# 3. Bir uygulamanın kayıt anahtarlarını ve kalıntılarını derinlemesine inceleme
.\DeRust.exe inspect "Uygulama Adı"

# 4. Simülasyon modu (Sistemde hiçbir dosya silmeden adımları önizler)
.\DeRust.exe uninstall "Uygulama Adı" --dry-run

# 5. Gerçek ve sessiz toplu kaldırma
.\DeRust.exe uninstall "Uygulama 1" "Uygulama 2"

# 6. Yalnızca sahipsiz kalıntıları (leftovers) tarama
.\DeRust.exe leftovers
```

---

## 🗺️ Roadmap & Changelog

* 🧭 Gelecek sürümler ve geliştirme takvimi: **[ROADMAP.md](ROADMAP.md)**
* 📝 Sürüm değişiklik geçmişi: **[CHANGELOG.md](CHANGELOG.md)**
* 🏛️ Organizasyon ve Alt Birimler: **[ORGANIZATION.md](ORGANIZATION.md)**

---

## 🛠️ Kaynak Koddan Derleme (Building from Source)

```powershell
# Repoyu klonlayın
git clone https://github.com/mtahiryilmazz/derust.git
cd derust

# Release modunda derleyin
cargo build --release

# Derlenen binary: target/release/DeRust.exe
.\target\release\DeRust.exe
```

---

## 🏢 Hakkında & Lisans

* **Çatı Organizasyon:** **[YILLABS](ORGANIZATION.md)**
* **Bağlı Birim:** **YILLABS Systems** (İşletim sistemi araçları, performans ve sistem optimizasyonu)
* **Geliştirici & Kurucu:** M. Tahir Yılmaz ([@mtahiryilmazz](https://github.com/mtahiryilmazz))
* **Lisans:** [MIT License](LICENSE) — Telif Hakkı (c) 2026 YILLABS. Açık kaynak ve özgürce kullanılabilir.
