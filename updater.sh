#!/bin/sh

# ==========================================
# VibeTorrent Otomatik Guncelleyici (MIPS)
# ==========================================

REPO="spinline/vibetorrent-v3"
BINARY="vibetorrent-mips"
CHECK_INTERVAL=60 # Saniye (1 Dakika)
ARGS="--socket /opt/var/rpc.socket"

echo "--- VibeTorrent Updater Baslatildi ---"
echo "Repo: $REPO"
echo "Binary: $BINARY"
echo "Args: $ARGS"

CURRENT_TAG=""

while true; do
    # GitHub'dan en son yonlendirilen URL'i al (Redirect Takibi)
    LATEST_URL=$(curl -Ls -o /dev/null -w %{url_effective} https://github.com/$REPO/releases/latest)
    
    # URL'in son parcasini (tag ismini) al
    LATEST_TAG=$(basename "$LATEST_URL")

    # Eger tag 'latest' ise veya bos ise hata var demektir.
    if [ -z "$LATEST_TAG" ] || [ "$LATEST_TAG" = "latest" ]; then
        echo "[!] GitHub'a erisilemedi veya release yok. $CHECK_INTERVAL saniye sonra tekrar denenecek."
    elif [ "$LATEST_TAG" != "$CURRENT_TAG" ]; then
        echo "[+] Yeni surum tespit edildi: $LATEST_TAG (Mevcut: ${CURRENT_TAG:-Bilinmiyor})"
        
        # Binary indirme linki
        DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$BINARY"
        TEMP_FILE="${BINARY}_new"

        echo "[*] Indiriliyor: $DOWNLOAD_URL"
        
        # İndirme islemi (wget veya curl)
        rm -f "$TEMP_FILE"
        if command -v wget >/dev/null 2>&1; then
            wget -q -O "$TEMP_FILE" "$DOWNLOAD_URL"
        else
            curl -L -o "$TEMP_FILE" "$DOWNLOAD_URL"
        fi

        # İndirme kontrolu
        if [ -s "$TEMP_FILE" ]; then
            echo "[*] Indirme basarili. Guncelleme uygulaniyor..."

            # 1. Eski sureci bul ve oldur
            PID=$(pidof "$BINARY")
            if [ -n "$PID" ]; then
                echo "[-] Eski surec sonlandiriliyor (PID: $PID)..."
                kill $PID 2>/dev/null
                sleep 2
                kill -9 $PID 2>/dev/null # Zorla kapat
            fi

            # 2. Dosyayi degistir
            mv -f "$TEMP_FILE" "$BINARY"
            chmod +x "$BINARY"

            # 3. Yeni sureci baslat
            echo "[+] Yeni surum baslatiliyor..."
            nohup ./$BINARY $ARGS > vibetorrent.log 2>&1 &
            
            CURRENT_TAG="$LATEST_TAG"
            echo "[OK] Guncelleme tamamlandi. Surum: $CURRENT_TAG"
        else
            echo "[!] Indirme basarisiz! (Dosya boyutu 0 veya erisim hatasi)"
            rm -f "$TEMP_FILE"
        fi
    else
        # Sessiz mod (degisiklik yok)
        :
    fi

    sleep $CHECK_INTERVAL
done
