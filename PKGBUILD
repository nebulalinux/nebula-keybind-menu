# Maintainer: Nebula
pkgname=nebula-keybind-menu
pkgver=0.0.1
pkgrel=1
pkgdesc="TUI keybind search menu for Nebula"
arch=('x86_64')
url="https://github.com/nebulalinux"
license=('MIT')
depends=('glibc')
makedepends=('cargo' 'rust')
source=()
sha256sums=()

prepare() {
  rm -rf "$srcdir/nebula-keybind-menu"
  mkdir -p "$srcdir/nebula-keybind-menu"

  # Copy all files from startdir to srcdir/nebula-keybind-menu, including hidden ones,
  # but excluding pkg, target, and the PKGBUILD itself to avoid recursion.
  find "$startdir" -maxdepth 1 \
    -not -path "$startdir" \
    -not -name "pkg" \
    -not -name "target" \
    -not -name "PKGBUILD" \
    -not -name "*.pkg.tar.*" \
    -not -name ".SRCINFO" \
    -exec cp -r {} "$srcdir/nebula-keybind-menu/" \;
}

build() {
  cd "$srcdir/nebula-keybind-menu"
  cargo build --release --locked
}

package() {
  install -Dm755 "$srcdir/nebula-keybind-menu/target/release/nebula-keybind-menu" \
    "$pkgdir/usr/bin/nebula-keybind-menu"

  install -Dm644 "$srcdir/nebula-keybind-menu/config.toml" \
    "$pkgdir/usr/share/nebula-keybind-menu/config.toml"

  install -Dm644 "$srcdir/nebula-keybind-menu/config.toml" \
    "$pkgdir/etc/skel/.config/nebula-keybind-menu/config.toml"

  install -Dm644 "$srcdir/nebula-keybind-menu/README.md" \
    "$pkgdir/usr/share/nebula-keybind-menu/README.md"
  install -Dm644 "$srcdir/nebula-keybind-menu/VERSION" \
    "$pkgdir/usr/share/nebula-keybind-menu/VERSION"
  install -Dm644 "$srcdir/nebula-keybind-menu/LICENSE" \
    "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
