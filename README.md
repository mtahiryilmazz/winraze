# 🛡️ YILABS Uninstaller

<div align="center">

**Modern, Yüksek Performanslı Toplu Yazılım Kaldırıcı & Derin Kalıntı Temizleyici**  
*Ultra-Fast Native Windows Batch Uninstaller & Deep Cleaner*

[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078d4.svg?style=flat-square&logo=windows)](https://github.com/mtahiryilmazz)
[![UI](https://img.shields.io/badge/Design-WinUI%203%20%7C%20Fluent-blue.svg?style=flat-square)](https://github.com/mtahiryilmazz)
[![Engine](https://img.shields.io/badge/Architecture-9--Step%20Pipeline-success.svg?style=flat-square)](https://github.com/mtahiryilmazz)
[![Language](https://img.shields.io/badge/Language-TR%20%7C%20EN-purple.svg?style=flat-square)](https://github.com/mtahiryilmazz)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)

*Crafted with precision by **[YILABS Studio](https://github.com/mtahiryilmazz)** (M. Tahir Yılmaz)*

</div>

---

## 🌟 YILABS Uninstaller Nedir?

**YILABS Uninstaller**, popüler Windows kaldırıcı **Uninstalr**'ın çalışma mantığı tersine mühendislikle çözümlenerek, **YILABS Studio** çatısı altında modern sistem standartlarına uygun olarak sıfırdan geliştirilmiş yeni nesil bir masaüstü uygulamasıdır.

Windows'un yerel "Program Ekle/Kaldır" arayüzünün geride bıraktığı yüzlerce yetim kayıt defteri anahtarını, veri klasörlerini (`AppData`, `ProgramData`), önbellek dosyalarını (`Prefetch`) ve kilitli servisleri **9 aşamalı tescilli boru hattı (9-step pipeline)** ile tamamen ortadan kaldırır.

---

## 🪟 Öne Çıkan Özellikler

### 1. 🎨 Modern WinUI 3 / Fluent Tasarım Dili
* **Mica & Acrylic Karanlık Arayüz:** Gözü yormayan modern koyu tema paleti ve yumuşatılmış köşeler.
* **Üst Panel KPI İstatistik Kartları:** Toplam yazılım sayısı, diskte kaplanan gerçek alan, seçilen alan ve sahipsiz kalıntılar için anlık bilgi kartları.
* **Fluent Kart-Karo (Card Tiles) Mimarisi:** Düzensiz tablolar yerine her yazılım için hover efektli, tip etiketli (`Win32`, `MSI`, `Store`, `Leftover`) ve boyut çipli modern kartlar.
* **Segmented Filter Bar:** `Tümü`, `Win32`, `MSI`, `Store / UWP` ve `Kalıntılar` arasında tek tıkla akıcı geçiş.
* **Canlı Arama Çubuğu:** Yazılım veya yayımcı adını yazarken anında sonuç veren hızlı filtreleme.

### 2. 🌐 Tam Yerelleştirme (Localization - TR / EN)
* **Çift Dil Desteği:** 🇹🇷 **Türkçe** ve 🇬🇧 **English**.
* Üst çubuktaki tek bir butona basarak arayüzün dilini (tüm menüler, kaldırma adımları, durum mesajları ve ayarlar dahil) anında değiştirebilme.

### 3. ⚙️ 9 Aşamalı Derin Kaldırma Boru Hattı
1. **Adım 1 & 2 (AutoKill):** İlgili programın çalışan süreçlerini (`TerminateProcess`) ve Windows servislerini (`sc stop`) otomatik kapatır.
2. **Adım 3 (Kısayollar):** Başlat Menüsü ve Masaüstü `.lnk`/`.url` dosyalarını temizler.
3. **Adım 4 (Sessiz MSI):** MSI paketlerini sessiz modda (`msiexec /X {GUID} /quiet /norestart`) tetikler.
4. **Adım 5 (Yerel Kaldırıcı):** InnoSetup (`/VERYSILENT /NORESTART`), NSIS (`/S`) sessiz anahtarlarıyla kaldırır.
5. **Adım 6 (AppX Store):** Modern paketleri `Remove-AppxPackage` ile temizler.
6. **Adım 7 (Kayıt Defteri Temizliği):** `HKCU`, `HKLM`, `Wow6432Node`, `AppCompatFlags` altındaki tüm sahipsiz anahtarları siler.
7. **Adım 8 (Dosya Sistemi & Kilitli Dosyalar):** Kurulum ve `AppData` klasörlerini siler; kilitli kalanları `MoveFileExW(..., MOVEFILE_DELAY_UNTIL_REBOOT)` ile yeniden başlatmada temizlenmek üzere kuyruğa alır.
8. **Adım 9 (Post-Processing):** Boş kalan ana klasörleri ve Windows Prefetch (`.pf`) kayıtlarını temizler.

---

## 🚀 Çalıştırma ve Kullanım

Proje doğrudan çift tıklanarak çalışmaya hazır tek bir bağımsız `.exe` olarak derlenmiştir:

### 🖥️ Grafik Arayüzü (GUI) Başlatma
Doğrudan [YILABS-Uninstaller.exe](file:///C:/Users/BT/.gemini/antigravity-ide/scratch/uninstalr-rs/YILABS-Uninstaller.exe) dosyasına çift tıklayarak masaüstü arayüzünü açabilirsiniz.

PowerShell'den başlatmak için:
```powershell
.\YILABS-Uninstaller.exe
```

### ⌨️ Komut Satırı (CLI) Modu
Aynı ikili dosya terminal üzerinden arka plan otomasyonları için de kullanılabilir:

```powershell
# Kurulu tüm yazılımları listeleme
.\YILABS-Uninstaller.exe list

# Belirli bir uygulamayı filtreleme
.\YILABS-Uninstaller.exe list --filter "Chrome"

# Uygulamanın tüm kalıntılarını ve kayıt anahtarlarını inceleme (Review)
.\YILABS-Uninstaller.exe inspect "Program Adı"

# Simülasyon modunda test etme (Sisteme hiçbir müdahale yapmaz)
.\YILABS-Uninstaller.exe uninstall "Program Adı" --dry-run

# Gerçek toplu kaldırma
.\YILABS-Uninstaller.exe uninstall "Program 1" "Program 2"

# Yalnızca sahipsiz kalıntıları tarama
.\YILABS-Uninstaller.exe leftovers
```

---

## 🛠️ Kaynak Koddan Derleme (Building from Source)

Eğer projeyi sıfırdan kendiniz derlemek isterseniz:

```powershell
# 1. Repoyu klonlayın
git clone https://github.com/mtahiryilmazz/yilabs-uninstaller.git
cd yilabs-uninstaller

# 2. Release modunda derleyin
cargo build --release

# 3. Çalıştırın
.\target\release\Uninstalr.exe
```

---

## 🏢 YILABS Studio Hakkında

**YILABS Studio**, modern, yüksek performanslı ve tavizsiz mühendislik standartlarına sahip yazılım araçları geliştiren bağımsız bir teknoloji stüdyosudur.

* **Geliştirici:** M. Tahir Yılmaz ([@mtahiryilmazz](https://github.com/mtahiryilmazz))
* **Stüdyo:** YILABS Studio
* **Lisans:** [MIT License](LICENSE)
