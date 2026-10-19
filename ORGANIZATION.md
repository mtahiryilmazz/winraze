# 🏛️ YILLABS — Kurumsal Organizasyon & Alt Birimler Şeması
*(YILLABS Organization Charter & Subunit Architecture)*

---

## 🌐 1. Genel Bakış & Vizyon (Executive Summary)

**YILLABS**, modern yazılım mühendisliği, düşük seviyeli sistem mimarisi ve yüksek performanslı çözümler üretmek amacıyla **M. Tahir Yılmaz** tarafından kurulan çatı teknoloji organizasyonudur.

* **Kurucu:** M. Tahir Yılmaz ([@mtahiryilmazz](https://github.com/mtahiryilmazz))
* **Çatı Marka:** **YILLABS** (YILmaz + LABS)
* **Temel Felsefe:**
  * **Zero Bloat (Sıfır Şişkinlik):** Hafif, yerel (native) ve bellek dostu çözümler.
  * **Deep Engineering (Derin Mühendislik):** Yüzeysel sarmalayıcılar yerine doğrudan işletim sistemi API'leri ve çekirdek mekanizmalarıyla çalışma.
  * **Privacy First (Önce Gizlilik):** Sıfır telemetri, yerel çalışan bağımsız yazılımlar.

---

## 🏢 2. Organizasyon Yapısı & Alt Birimler (Subunits)

YILLABS, projelerin kapsamını daraltmamak ve her alanda uzmanlaşmış odak grupları oluşturmak için aşağıdaki özerk alt birimlerden oluşur:

```
                          ┌───────────────────────┐
                          │        YILLABS        │
                          │   (Holding / Çatı)    │
                          └───────────┬───────────┘
                                      │
         ┌─────────────────┬──────────┴──────────┬─────────────────┐
         ▼                 ▼                     ▼                 ▼
 ┌───────────────┐ ┌───────────────┐     ┌───────────────┐ ┌───────────────┐
 │YILLABS Systems│ │  YILLABS AI   │     │ YILLABS Cloud │ │YILLABS Creative│
 └───────┬───────┘ └───────┬───────┘     └───────┬───────┘ └───────┬───────┘
         │                 │                     │                 │
   İşletim Sistemi   Yapay Zekâ & Ajan     Dağıtık Mimari    Arayüz (UI/UX)
    Masaüstü & OS      Lokal Modeller      Bulut & DevOps    Tasarım Sistemi
  (DeRust, Cleaners) (Agents, Tooling)   (APIs, Microserv) (Fluent, WinUI)
```

---

### ⚙️ A. YILLABS Systems
* **Odak Alanı:** Düşük seviyeli sistem araçları, Windows/Linux çekirdek mekanizmaları, bellek ve disk optimizasyonu, donanım kaynak yönetimi.
* **Kullanılan Teknolojiler:** Rust, C++, Win32 Native API, NT Kernel, PowerShell.
* **Amiral Gemisi Proje:**
  * 🛡️ **[YILLABS DeRust](https://github.com/mtahiryilmazz/derust)**: Uninstalr mimarisinden ilham alan 9 aşamalı derin toplu program kaldırıcı ve dijital pas sökücü.

---

### 🧠 B. YILLABS AI
* **Odak Alanı:** Otonom yapay zekâ ajanları, yerel (on-device) LLM çıkarım motorları, semantik arama ve akıllı veri analitiği.
* **Kullanılan Teknolojiler:** Python, Rust, ONNX Runtime, CUDA, Llama.cpp, LangChain.
* **Gelecek Projeler:**
  * Akıllı sistem teşhis ve log analiz ajanları.

---

### ☁️ C. YILLABS Cloud & Infra
* **Odak Alanı:** Güvenli veri iletimi, yüksek erişilebilirlikli mikroservisler, otomatik CI/CD hatları ve paket dağıtım ağları (CDN/Winget/Scoop).
* **Kullanılan Teknolojiler:** Go, Rust, Docker, Kubernetes, GitHub Actions, Cloudflare Workers.
* **Gelecek Projeler:**
  * Topluluk tabanlı yazılım artık veritabanı (Cloud Leftover Rules DB) senkronizasyon ağı.

---

### 🎨 D. YILLABS Creative
* **Odak Alanı:** Modern dijital deneyimler, Fluent / WinUI 3 masaüstü tasarım sistemleri, web arayüzleri, ergonomik veri görselleştirme.
* **Kullanılan Teknolojiler:** WinUI 3, egui/eframe, CSS/Web Components, SVG vektör grafikleri.

---

## 📌 3. Proje & İsimlendirme Standartları (Brand Taxonomy)

YILLABS ekosistemindeki tüm projeler aşağıdaki hiyerarşik standartla adlandırılır:

| Kategori | Standart Format | Örnek |
| :--- | :--- | :--- |
| **Tam Ticari / Dağıtım İsmi** | `YILLABS <Ürün Adı>` | `YILLABS DeRust` |
| **Bağlı Birim Referansı** | `YILLABS Systems · <Ürün Adı>` | `YILLABS Systems · DeRust` |
| **Yürütülebilir Dosya (Executable)** | `DeRust.exe` veya `YILLABS-DeRust.exe` | `YILLABS-DeRust.exe` |
| **Paket Dağıtım Kimliği** | `YILLABS.<ÜrünAdı>` | `winget install YILLABS.DeRust` |
| **Kaynak Kod Deposu (Repo)** | `github.com/<org>/<proje-adı>` | `github.com/yillabs/derust` |

---

## 🚀 4. GitHub Organizasyonu Kurulum Rehberi

GitHub platform kuralları gereği, kişisel hesaplar için Organizasyon oluşturma işlemi güvenlik nedeniyle Web arayüzü üzerinden tek tıkla yapılmaktadır:

1. Tarayıcınızda **[github.com/organizations/plan](https://github.com/account/organizations/new?plan=free)** adresini açın.
2. **Organization account name:** `yillabs` yazın.
3. **Contact email:** İletişim e-posta adresinizi girin.
4. **Plan:** "Free" (Ücretsiz) seçeneğiyle oluşturun.
5. Oluşturduktan sonra:
   * Mevcut `https://github.com/mtahiryilmazz/derust` reposunu **Settings -> Danger Zone -> Transfer ownership -> `yillabs`** adımıyla tek tıkla organizasyona aktarabilirsiniz.
   * Tüm commit geçmişi, Issues ve Release'ler otomatik olarak korunacaktır.

---

<div align="center">

*© 2026 **YILLABS**. Tüm hakları saklıdır.*  
*Engineering without limits.*

</div>
