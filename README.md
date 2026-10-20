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

### 1. 🪟 Özgün WinUI 3 & Fluent Cyber-Dark Arayüzü (Kopya Değil, Özel Tasarım)
* **Sol Navigasyon Rayı (Navigation Rail):** Klasik eski tablolar veya reklam dolu pencereler yerine; sol tarafta dikey olarak konumlandırılmış, modern rozet sayaçlarına (`Badge Counters`) ve renk kodlu göstergelere sahip akıcı menü çubuğu:
  * 📊 **Sistem Sağlığı & Dashboard:** Windows sistem sağlık endeksi (%0-100), çökme raporu sayaçları ve 1-Tık hızlı temizlik.
  * 📦 **Kurulu Yazılımlar:** Win32, MSI ve Microsoft Store uygulamaları, arama filtresi ve toplu seçim.
  * 🧩 **Tarayıcı Eklentileri:** Chrome, Edge ve Brave eklentilerini doğrudan diskten listeleme ve kaldırma.
  * ⚡ **Kalıntılar & Artıklar:** Sahipsiz AppData, ProgramFiles ve Kayıt Defteri artıklarının analizi.
  * 🔥 **Zorla Kaldırıcı (Force Shredder):** Kaldırıcısı bozuk programları dosya sistemi ve kilitli süreç seviyesinde yok etme.
  * ⚙️ **Güvenlik & Tercihler:** Geri yükleme noktası, kayıt defteri yedeği ve kaldırma motoru ayarları.
* **Sağ Açılır Derin İnceleme Çekmecesi:** Herhangi bir programa tıklandığında kurulum dizini, kaldırıcı komutu, kısayollar, crash dump dosyaları ve kayıt defteri bağlantılarını önizleme imkânı.
* **Kaldırma Öncesi Güvenlik Doğrulaması:** Kaldırma başlatılmadan önce seçilen programları, sistem geri yükleme noktası ve .reg yedekleme tercihlerini özetleyen modern onay penceresi.

### 2. 🛡️ REA Analizleriyle Entegre Edilen Gelişmiş Güvenlik ve Temizlik Motoru
* **Adım 0 (Sistem Geri Yükleme Noktası):** Her kaldırma işlemi öncesinde otomatik olarak Windows Sistem Geri Yükleme Noktası (*Checkpoint-Computer*) oluşturulur.
* **Adım 7 (Kayıt Defteri Geri Alma Yedeği):** Silinen tüm anahtarlar `%LOCALAPPDATA%\DeRust\Backups\Registry\` altına `.reg` formatında yedeklenir. İstediğiniz an çift tıklayarak geri yükleyebilirsiniz.
* **Çökme Dökümleri & Prefetch Temizliği:** `%LOCALAPPDATA%\CrashDumps` ve Windows Prefetch (`*.pf`) önbelleklerini temizleyerek sistem stabilitesini artırır.
* **Genişletilmiş Sessiz Kaldırıcı Algılama:** Inno Setup, NSIS, InstallShield (`/s /v"/qn"`), Wise, MSI ve Squirrel paketlerini otomatik tanır ve kullanıcı müdahalesi olmadan sessizce kaldırır.
* **Kritik Sistem Bileşenleri Koruması (Protected Badge):** Visual C++, DirectX, GPU sürücüleri, .NET Framework gibi kritik bileşenleri turuncu `🛡️ Sistem Koruması` rozetiyle işaretler ve yanlışlıkla silinmelerini önler.

### 3. 🌐 Tam Yerelleştirme (Localization - TR / EN)
* **Çift Dil:** 🇹🇷 **Türkçe** (Varsayılan) ve 🇬🇧 **English**.
* Sol alt menüdeki tek tıkla `🇹🇷 Türkçe` / `🇬🇧 English` butonu veya ayarlar menüsü üzerinden tüm arayüzü anında tercüme edebilme.

---

## 🚀 Quick Start / Hızlı Başlangıç

### 🖥️ Grafik Arayüzü (GUI) Başlatma
Doğrudan `DeRust.exe` dosyasına çift tıklayarak veya komut satırından parametresiz çağırarak modern WinUI 3 arayüzünü başlatabilirsiniz:
```powershell
cargo run --release
# veya doğrudan ikili dosya:
.\target\release\DeRust.exe
```

### ⌨️ Komut Satırı (CLI) Modu
Aynı ikili dosya otomasyonlar, sistem yöneticileri ve sunucular için gelişmiş parametre desteğine sahiptir:

```powershell
# 1. Windows Sistem Sağlık Endeksini Değerlendirme
.\DeRust.exe health

# 2. 1-Tık Hızlı Sağlık Temizliği (CrashDumps, Prefetch, Kırık Kısayollar)
.\DeRust.exe optimize
.\DeRust.exe optimize --dry-run   # Simülasyon modu

# 3. Web Tarayıcı Eklentilerini Listeleme (Chrome, Edge, Brave)
.\DeRust.exe extensions

# 4. Zorla Kaldırıcı & Dosya Öğütücü (Bozuk/kilitli klasörleri temizleme)
.\DeRust.exe force-clean "C:\Program Files\InatciKlasor"
.\DeRust.exe force-clean "C:\Program Files\InatciKlasor" --dry-run

# 5. Kurulu tüm uygulamaları ve disk alanlarını listeleme
.\DeRust.exe list
.\DeRust.exe list --filter "Chrome"

# 6. Uygulamanın kayıt anahtarlarını ve kalıntılarını derinlemesine inceleme
.\DeRust.exe inspect "Uygulama Adı"

# 7. Geri Yükleme Noktası ve Kayıt Defteri Yedeğiyle Güvenli Kaldırma
.\DeRust.exe uninstall "Uygulama 1" "Uygulama 2"
.\DeRust.exe uninstall "Uygulama 1" --dry-run
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
