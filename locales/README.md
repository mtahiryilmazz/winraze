# 🌐 WinRaze Localization / Yerelleştirme Kılavuzu

WinRaze, bağımsız geliştiricilerin ve çevirmenlerin projeye kolayca yeni diller ekleyebilmesi için dinamik JSON tabanlı bir yerelleştirme mimarisine sahiptir.

---

## 🛠️ Nasıl Yeni Bir Dil Eklenir?

1. `locales/en.json` veya `locales/tr.json` dosyasını kopyalayın ve eklemek istediğiniz dilin ISO koduyla adlandırın (Örnek: `es.json` [İspanyolca], `fr.json` [Fransızca], `az.json` [Azerbaycanca]).
2. Dosyanın başındaki `locale_name` ve `locale_code` alanlarını güncelleyin:
   ```json
   {
     "locale_name": "Español",
     "locale_code": "es",
     ...
   }
   ```
3. İlgili metinleri hedef dile çevirin.
4. Dosyayı `locales/` klasörüne (veya `%LOCALAPPDATA%\WinRaze\Locales\` dizinine) bırakın.
5. WinRaze'i başlattığınızda yeni diliniz otomatik olarak keşfedilecek ve hem sol alt köşedeki hızlı dil butonunda hem de Ayarlar menüsündeki dil listesinde seçilebilir olacaktır!

Katkılarınızı bir Pull Request olarak GitHub repomuza gönderebilirsiniz. 🦀
