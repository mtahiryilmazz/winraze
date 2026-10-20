# 🗺️ WinRaze — Ürün Yol Haritası (Product Roadmap)

Bu belge, **WinRaze (Windows Kalıntı ve Bloatware Temizleme Motoru)** projesinin stratejik geliştirme planını, tamamlanan aşamalarını ve canlıya geçiş adımlarını açıklar.

---

## 📍 Aşama 1: Çekirdek Motor ve İlk Sürüm `[v0.1.0-beta]` — *(Tamamlandı)*

- [x] **9 Adımlı Derinlemesine Kaldırma Motoru**:
  - [x] Otomatik Sonlandırma: Kilitli süreçleri `ToolHelp32` ile kapatma ve servisleri durdurma.
  - [x] Masaüstü ve Başlat Menüsü kısayollarını temizleme.
  - [x] Sessiz kaldırma bayrakları enjeksiyonu (MSI `/quiet`, Inno Setup, NSIS sessiz parametreleri).
  - [x] Modern Windows Mağazası (AppX / MSIX) paketlerini PowerShell ile kaldırma.
  - [x] Derin Kayıt Defteri (Registry) temizliği (`HKCU`, `HKLM`, `Wow6432Node`, `AppCompatFlags`).
  - [x] Kilitli dosyalar için Win32 `MoveFileExW` (`MOVEFILE_DELAY_UNTIL_REBOOT`) yeniden başlatmada silme kuyruğu.
  - [x] Boş kalan ebeveyn klasörleri ve Windows Prefetch (`.pf`) artıklarını budama.
- [x] **Modern WinUI / Fluent Masaüstü Arayüzü (GUI)**:
  - [x] Akrilik ve koyu tema renk paleti.
  - [x] Üst KPI metrik kartları (Yüklü Programlar, Toplam Boyut, Seçilen Alan, Artıklar).
  - [x] Arama, filtreleme ve detaylı artık önizleme çekmecesi.
  - [x] Canlı 9 adımlı işlem ilerleme penceresi.
- [x] **Çift Dil Desteği (i18n)**:
  - [x] Türkçe (Varsayılan) ve İngilizce tam dil desteği.
- [x] **Gelişmiş Komut Satırı Arayüzü (CLI)**:
  - [x] `scan`, `list`, `inspect`, `uninstall`, `purge`, `clean`, `leftovers`, `health` ve `--dry-run` simülasyon modları.

---

## 📍 Aşama 2: Güvenlik, Yedekleme ve Geri Alma `[v0.2.0-beta]` — *(Tamamlandı)*

- [x] **Windows Sistem Geri Yükleme Noktası Entegrasyonu**:
  - Kaldırma işlemi başlamadan önce otomatik sistem geri yükleme noktası oluşturma.
- [x] **Kayıt Defteri Yedeği (Geri Alma İmkânı)**:
  - Silinen registry anahtarlarını `.reg` dosyası olarak `%LOCALAPPDATA%\WinRaze\Backups\Registry\` altına yedekleme.
- [x] **Akıllı Sistem Koruması / Beyaz Liste**:
  - Kritik sistem bileşenlerini (Visual C++, DirectX, GPU Sürücüleri, Windows Güvenliği) kazara silinmeye karşı koruma rozeti `[🛡️ Sistem Koruması]`.
- [x] **Hızlı Sistem Sağlığı İyileştirmesi (1-Tık Optimizasyon)**:
  - Çökme raporları (`CrashDumps`), Prefetch önbelleği ve kırık kısayolları tek tıkla süpürme.

---

## 📍 Aşama 3: Kurulum İzleyici & Sistem Anlık Görüntüsü (Snapshot Tracker) `[v0.3.0 - AKTİF GELİŞTİRME]`

- [x] **Kurulum Öncesi & Sonrası Sistem Fotoğrafı (Pre/Post-Install Snapshot)**:
  - Program kurulmadan önce kayıt defteri (`HKLM`, `HKCU`, `HKCR`) ve dosya sistemini (`Program Files`, `AppData`, `ProgramData`) tarayarak hafızaya kaydetme.
  - Kurulum bittikten sonra tekrar tarayarak farkı (Diff) anında hesaplama.
- [x] **Akıllı Gürültü Filtresi (Noise Filtering)**:
  - Windows Update, Defender, Prefetch ve geçici log dosyalarını kurulum kaydına dahil etmeyip yalnızca ilgili programın kayıtlarını ayıklama.
- [x] **İzleme Günlüğü ile Kusursuz Kaldırma (Zero-Leftover Uninstall)**:
  - İzlenerek kurulan programları tek bir dosya veya kayıt defteri kalıntısı bırakmadan tamamen kaldırma.
- [x] **GUI ve CLI Entegrasyonu**:
  - Komut satırından `winraze track <setup.exe>` ile doğrudan kurulum izleme.
  - GUI üzerinden "Kurulumu İzle" sihirbazı.

---

## 📍 Aşama 4: Derin Sistem Temizliği & Kabuk Uzantıları `[v0.4.0]`

- [ ] **Sağ Tık Bağlam Menüsü Temizleyici (Context Menu Cleaner)**:
  - Silinmiş programlardan kalan öksüz sağ tık menü girdilerini temizleme (`HKCR\*\shellex`).
- [ ] **Öksüz Windows Servisleri ve Zamanlanmış Görevler Temizleyici**:
  - Artık mevcut olmayan uygulamaların arkada bıraktığı arka plan servis ve zamanlanmış görevlerini temizleme.
- [ ] **Başlangıç Yöneticisi (Startup Manager)**:
  - Windows açılışını yavaşlatan gereksiz başlangıç girdilerini listeleme ve devre dışı bırakma.

---

## 📍 Aşama 5: Canlıya Geçiş (v1.0 Genel Dağıtım & Release)**

- [ ] **Optimize Edilmiş Canlı Sürüm (Release Binary)**:
  - Minimum boyut, maksimum hız ve LTO optimizasyonu ile derlenmiş `winraze.exe`.
- [ ] **Resmi Windows Paket Dağıtımı**:
  - `winget install WinRaze` manifestosu hazırlığı.
- [ ] **Taşınabilir (Portable) ve Kurulumlu Sürümler**:
  - Kurulum gerektirmeyen tek parça bağımsız `.exe` ve kurulum sihirbazı.

---

*Geri bildirim ve önerileriniz için [GitHub Issues](https://github.com/mtahiryilmazz) üzerinden katkıda bulunabilirsiniz.*
