# 🚀 WinRaze

<p align="center">
  <strong>Raze the bloat. Reclaim your Windows. Powered by Rust. 🦀</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078d4.svg?style=flat-square&logo=windows" alt="Platform">
  <img src="https://img.shields.io/badge/Language-Rust-orange.svg?style=flat-square&logo=rust" alt="Language">
  <img src="https://img.shields.io/badge/Version-v0.1.0-blue.svg?style=flat-square" alt="Version">
  <img src="https://img.shields.io/badge/License-MIT-green.svg?style=flat-square" alt="License">
</p>

---

**WinRaze**, Windows işletim sistemindeki inatçı gereksiz uygulamaları (bloatware), sistem kalıntılarını, önbellekleri ve arkada gizlice çalışan veri toplama (telemetri) servislerini **yerle bir etmek (raze)** için Rust diliyle geliştirilmiş, şimşek hızında ve açık kaynaklı bir uninstaller & debloater aracıdır.

Sisteminizin akıcılığını bozan tüm dijital pası söküp atar, Windows'u ilk günkü saf ve hafif haline geri döndürür.

---

## ✨ Öne Çıkan Özellikler

- **🦀 Rust Gücü ve Hızı:** Sıfır bağımlılık (zero-dependencies), ultra düşük RAM kullanımı ve milisaniyeler içinde tamamlanan derin sistem taraması.
- **🗑️ Akıllı Uninstaller:** Windows'un normal yollarla kaldırılmasına izin vermediği inatçı sistem uygulamalarını (Cortana, OneDrive vb.) ve arkalarında bıraktığı kayıt defteri (Registry) çöplerini kökünden kazır.
- **🔒 Güvenli Temizlik:** Varsayılan olarak "salt okunur" (Read-Only) modda başlar. Siz açıkça onay vermeden hiçbir sistem dosyasını zorla silmez.
- **🚀 Telemetri Engelleme:** Arka planda kaynak tüketen ve verilerinizi izleyen gizli Windows servislerini (`DiagTrack`, `dmwappushservice`) tek tıkla devre dışı bırakır.
- **📦 Taşınabilir (Portable):** Kurulum gerektirmez, kayıt defterini kirletmez. Sadece tek bir `.exe` dosyasını indirin ve çalıştırın.
- **🛡️ Sistem Geri Yükleme ve .reg Yedeği:** Kaldırma öncesinde otomatik Windows Sistem Geri Yükleme Noktası (`Checkpoint-Computer`) oluşturur ve silinen anahtarları `%LOCALAPPDATA%\DeRust\Backups\Registry\` altına `.reg` formatında yedekler.
- **🧩 Tarayıcı Eklenti Yöneticisi:** Google Chrome, Microsoft Edge ve Brave tarayıcılarına kurulu eklentileri doğrudan diskten tespit edip kaldırır.
- **🔥 Zorla Kaldırıcı & Öğütücü:** Kaldırıcısı bozuk veya Denetim Masasında görünmeyen inatçı klasörleri kilitleyen işlemleri sonlandırıp (`AutoKill`) imha eder.

---

## 🛠️ Kullanım ve CLI Komutları

WinRaze hem modern bir Fluent masaüstü arayüzüne (GUI) sahiptir hem de doğrudan komut satırından (CLI) yönetilebilir.

```bash
# Sistemi tarar ve temizlenebilecek alanların raporunu çıkarır (Güvenli Mod)
winraze scan

# İnteraktif modern masaüstü arayüzünü (GUI) açar
winraze gui

# Onay istemeden sistemdeki tüm bloatware ve kalıntıları temizler, telemetriyi kapatır
winraze purge --yes

# Sadece tarayıcı önbelleklerini ve geçici sistem dosyalarını temizler
winraze clean --temp

# Windows Sistem Sağlık Endeksini (%0-100) görüntüler
winraze health

# 1-Tık hızlı sağlık optimizasyonu (CrashDumps, Prefetch, kırık kısayollar)
winraze optimize

# Web tarayıcı eklentilerini (Chrome, Edge, Brave) listeler
winraze extensions

# Bozuk bir yazılım klasörünü zorla imha eder
winraze force-clean "C:\Program Files\InatciKlasor"
```

---

## 🛡️ Güvenlik Protokolü

WinRaze sistemi "yıkarken" güvenliğinizi ön planda tutar:
- Tarayıcı geçmişinize ve kayıtlı şifrelerinize asla dokunmaz.
- O an sistem tarafından kullanılan (kilitli) kritik dosyaları zorla silmez; bir sonraki sistem yeniden başlatmasında silinmek üzere (`MoveFileExW`) kuyruğa alır.
- Yapılan tüm işlemlerin kaydını `%APPDATA%\WinRaze\raze.log` dosyasına kaydeder, böylece neyin silindiğini her zaman görebilirsiniz.
- Visual C++, DirectX, GPU ve ses sürücüleri gibi kritik donanım bileşenleri `🛡️ Sistem Koruması` kalkanıyla korunur.

---

## 🚀 Hızlı Başlangıç (Geliştiriciler İçin)

Projeyi yerelinizde derlemek için sisteminizde **Rust toolchain** kurulu olmalıdır.

```bash
# Projeyi klonlayın
git clone https://github.com/mtahiryilmazz/winraze.git
cd winraze

# Release modunda optimize ederek derleyin
cargo build --release

# Derlenen çalıştırılabilir dosyayı başlatın
./target/release/winraze.exe scan
```

---

## 📄 Lisans

Bu proje **MIT Lisansı** ile korunmaktadır. Detaylar için [LICENSE](LICENSE) dosyasına göz atabilirsiniz.
