# Maintainer: lazykern
pkgname=syncudio
pkgver=0.20.5
pkgrel=1
pkgdesc="A modern music player with cloud sync capabilities"
arch=('x86_64')
url="https://github.com/lazykern/syncudio-desktop"
license=('MIT')
depends=('fuse2')
optdepends=('libnotify: for desktop notifications')
source=("$pkgname-$pkgver.AppImage::file://src-tauri/target/release/bundle/appimage/Syncudio_${pkgver}_amd64.AppImage")
sha256sums=('SKIP')
options=('!strip')

prepare() {
    chmod +x "$srcdir/$pkgname-$pkgver.AppImage"
    cd "$srcdir"
    "./$pkgname-$pkgver.AppImage" --appimage-extract
}

package() {
    # Create directories
    install -dm755 "$pkgdir/opt/$pkgname"
    install -dm755 "$pkgdir/usr/bin"
    install -dm755 "$pkgdir/usr/share/applications"
    install -dm755 "$pkgdir/usr/share/icons/hicolor/512x512/apps"

    # Install AppImage contents
    cp -r "$srcdir/squashfs-root/"* "$pkgdir/opt/$pkgname/"

    # Create wrapper script
    cat > "$pkgdir/opt/$pkgname/syncudio-wrapper" << EOF
#!/bin/bash
cd /opt/$pkgname
exec ./AppRun "\$@"
EOF
    chmod 755 "$pkgdir/opt/$pkgname/syncudio-wrapper"

    # Create symlink in /usr/bin
    ln -s "/opt/$pkgname/syncudio-wrapper" "$pkgdir/usr/bin/$pkgname"

    # Install desktop file
    cat > "$pkgdir/usr/share/applications/$pkgname.desktop" << EOF
[Desktop Entry]
Name=Syncudio
Comment=A modern music player with cloud sync capabilities
Exec=/usr/bin/syncudio %F
Icon=syncudio
Terminal=false
Type=Application
Categories=Audio;Music;Player;AudioVideo;
MimeType=audio/flac;audio/mp3;audio/wav;audio/ogg;audio/m4a;
EOF

    # Install icon
    install -Dm644 "$srcdir/squashfs-root/syncudio.png" \
        "$pkgdir/usr/share/icons/hicolor/512x512/apps/$pkgname.png"

    # Set permissions
    chmod 755 "$pkgdir/opt/$pkgname/AppRun"
    chmod 755 "$pkgdir/opt/$pkgname/AppRun.wrapped"
} 