# Maintainer: bennyzen
pkgname=tracelon
pkgver=0.1.0
pkgrel=1
pkgdesc="Desktop app for tracing raster images to clean SVG with optimized curves"
arch=('x86_64')
url="https://github.com/bennyzen/tracelon"
license=('MIT')
depends=(
  'webkit2gtk-4.1'
  'gtk3'
  'glib2'
  'libsoup3'
)
makedepends=(
  'rust'
  'cargo'
  'nodejs'
  'pnpm'
  'pkg-config'
  'openssl'
)
source=()

build() {
  cd "$startdir"
  pnpm install --frozen-lockfile
  pnpm tauri build --bundles deb
}

package() {
  cd "$startdir"

  # Binary
  install -Dm755 "src-tauri/target/release/tracelon" "$pkgdir/usr/bin/tracelon"

  # Icons
  install -Dm644 "src-tauri/icons/32x32.png" "$pkgdir/usr/share/icons/hicolor/32x32/apps/tracelon.png"
  install -Dm644 "src-tauri/icons/128x128.png" "$pkgdir/usr/share/icons/hicolor/128x128/apps/tracelon.png"
  install -Dm644 "src-tauri/icons/icon.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/tracelon.png"

  # Desktop entry
  install -Dm644 /dev/stdin "$pkgdir/usr/share/applications/tracelon.desktop" <<EOF
[Desktop Entry]
Name=Tracelon
Comment=Trace raster images to clean SVG
Exec=tracelon
Icon=tracelon
Terminal=false
Type=Application
Categories=Graphics;VectorGraphics;
MimeType=image/png;image/jpeg;image/webp;
EOF

  # License
  install -Dm644 /dev/stdin "$pkgdir/usr/share/licenses/$pkgname/LICENSE" <<EOF
MIT License

Copyright (c) 2026 bennyzen

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
}
