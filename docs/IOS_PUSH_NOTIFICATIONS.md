# iOS Push Notification Desteği

VibeTorrent artık iOS 16.4+ cihazlarda push notification destekliyor! 🎉

## Gereksinimler

- **iOS 16.4 veya üzeri** (Mart 2023 ve sonrası)
- Safari tarayıcısı
- HTTPS bağlantısı (production ortamında)

## Nasıl Kullanılır?

### 1. Ana Ekrana Ekle

Push notification'lar **sadece Home Screen'e eklenmiş PWA'larda çalışır**. Safari'den doğrudan çalışmaz!

#### Adımlar:
1. Safari'de uygulamayı açın
2. Alt kısımdaki **Paylaş** butonuna tıklayın (⬆️ ikonu)
3. **"Ana Ekrana Ekle"** seçeneğini seçin
4. İsmi onaylayıp **"Ekle"** butonuna basın
5. Ana ekrandaki ikona tıklayarak uygulamayı açın

### 2. Notification İzni Verin

İlk açılışta uygulama notification izni isteyecektir:
- **"İzin Ver"** seçeneğini seçin
- Eğer atlarsanız, daha sonra Safari ayarlarından izin verebilirsiniz

### 3. Push Notification Otomatik Aktif Olur

Ana ekrandan açılan PWA'da:
- Uygulama otomatik olarak push notification'a abone olur
- Torrent tamamlandığında bildirim alırsınız
- **Uygulama kapalı olsa bile bildirim gelir!**

## Teknik Detaylar

### iOS Safari Kısıtlamaları:

✅ **Çalışır:**
- Home Screen'e eklenmiş PWA
- iOS 16.4+ Safari

❌ **Çalışmaz:**
- Safari browser mode (standalone olmayan)
- iOS 16.4 altı sürümler
- Chrome veya diğer tarayıcılar iOS'ta (WebKit kısıtlaması)

### Test Etme:

1. iOS cihazınızdan production URL'e gidin (HTTPS gerekli)
2. Ana ekrana ekleyin
3. Bir torrent indirin ve tamamlanmasını bekleyin
4. Uygulamayı kapatın
5. Torrent tamamlandığında notification alacaksınız!

### Sorun Giderme:

**"Push notification desteklenmiyor" mesajı görüyorum:**
- Ana ekrana eklediniz mi? Safari'den değil, ana ekrandaki ikondan açmalısınız
- iOS 16.4+ sürümü mü kullanıyorsunuz?

**Notification gelmiyor:**
- Settings → VibeTorrent → Notifications → izinlerin açık olduğundan emin olun
- Ana ekrandaki PWA'dan açtığınızdan emin olun (Safari'den değil)
- Developer Console'da "Push subscription" log'unu kontrol edin

**Notification izni reddettim, nasıl yeniden açarım?**
- Settings → Safari → Advanced → Website Data → VibeTorrent'i silin
- Uygulamayı ana ekrandan silin ve yeniden ekleyin

## Platform Karşılaştırması

| Platform | Push Support | Gereksinim |
|----------|--------------|------------|
| **Android Chrome** | ✅ Tam destek | Browser veya PWA |
| **iOS 16.4+ Safari** | ✅ PWA destekli | Ana ekrana eklenmiş olmalı |
| **iOS 16.3 ve altı** | ❌ Desteklenmez | - |
| **Desktop (Chrome/Edge)** | ✅ Tam destek | Browser veya PWA |
| **Desktop Safari** | ⚠️ Sınırlı | macOS 13+ (Ventura) |

## Güvenlik

- VAPID anahtarları kullanılıyor
- End-to-end şifreli push notification
- Subscription'lar backend'de güvenli saklanıyor
- iOS Safari security model tam uyumlu

## Geliştiriciler İçin

```rust
// iOS detection
if crate::utils::platform::is_ios() {
    // iOS-specific kod
}

// Standalone mode kontrolü
if crate::utils::platform::is_standalone() {
    // PWA mode
}

// Push notification desteği
if crate::utils::platform::supports_push_notifications() {
    // Subscribe
}
```

## Kaynaklar

- [iOS Safari Web Push](https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/)
- [PWA on iOS Guide](https://developer.apple.com/wwdc23/10120)
- [Web Push API](https://developer.mozilla.org/en-US/docs/Web/API/Push_API)
