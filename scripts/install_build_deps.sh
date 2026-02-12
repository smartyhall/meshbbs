#!/usr/bin/env bash
set -euo pipefail

# Install MeshBBS build dependencies across common Linux distros.
# This focuses on the libudev + pkg-config requirement needed by serialport/libudev-sys.

if [[ "${EUID}" -eq 0 ]]; then
  SUDO=""
else
  SUDO="sudo"
fi

install_debian() {
  ${SUDO} apt-get update
  ${SUDO} apt-get install -y --no-install-recommends pkg-config libudev-dev
}

install_fedora() {
  ${SUDO} dnf install -y pkgconf-pkg-config systemd-devel
}

install_arch() {
  ${SUDO} pacman -Sy --noconfirm pkgconf systemd
}

if command -v apt-get >/dev/null 2>&1; then
  echo "Detected apt-get (Debian/Ubuntu). Installing dependencies..."
  install_debian
elif command -v dnf >/dev/null 2>&1; then
  echo "Detected dnf (Fedora/RHEL-like). Installing dependencies..."
  install_fedora
elif command -v pacman >/dev/null 2>&1; then
  echo "Detected pacman (Arch). Installing dependencies..."
  install_arch
else
  cat <<'MSG'
Unsupported package manager.
Please install manually:
  - pkg-config (or pkgconf)
  - libudev development headers (.pc file)
Examples:
  Debian/Ubuntu: libudev-dev pkg-config
  Fedora/RHEL:   systemd-devel pkgconf-pkg-config
  Arch:          systemd pkgconf
MSG
  exit 1
fi

echo
echo "Dependency installation complete."
echo "Sanity check:"
echo "  pkg-config --libs --cflags libudev"
