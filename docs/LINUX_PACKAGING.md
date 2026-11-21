# Linux Packaging and Installer Guide

This guide documents the automated build and packaging pipeline for **BPSL IT Support** (`bpsl-it-support`) across major Linux distributions.

---

## 1. Quick Start

Run the automated build script in the repository root:

```bash
# Build installers for all supported distributions (Fedora, Mint/Ubuntu, Arch, Portable)
./build-linux.sh --all
```

All generated packages are placed in the `dist/` directory.

---

## 2. Supported Distributions & Packages

| Distribution Family | Target OS | Package Format | Direct Installer File | Alias / Symlink |
| :--- | :--- | :--- | :--- | :--- |
| **Red Hat / Fedora** | Fedora 38+, RHEL 8/9, CentOS Stream, Rocky, AlmaLinux | `.rpm` | `dist/bpsl-it-support-1.2.1-1.x86_64.rpm` | `dist/bpsl-it-support_fedora.rpm` |
| **Debian / Ubuntu** | Linux Mint 21/22, Ubuntu 22.04/24.04, Debian 12, Pop!_OS | `.deb` | `dist/bpsl-it-support_1.2.1_amd64.deb` | `dist/bpsl-it-support_linuxmint.deb`<br>`dist/bpsl-it-support_ubuntu.deb` |
| **Arch Linux** | Arch Linux, Manjaro, EndeavourOS, Garuda | `.pkg.tar.zst`<br>`PKGBUILD` | `dist/bpsl-it-support-1.2.1-1-x86_64.pkg.tar.zst`<br>`dist/arch/PKGBUILD` | `dist/bpsl-it-support_arch.pkg.tar.zst` |
| **Universal Linux** | openSUSE, Alpine (glibc), Gentoo, Void, or non-root users | `.tar.gz` | `dist/bpsl-it-support-1.2.1-linux-x86_64.tar.gz` | `dist/bpsl-it-support_portable.tar.gz` |

---

## 3. CLI Build Options

```text
Usage:
  ./build-linux.sh [options]

Options:
  -a, --all              Build packages for all supported Linux distributions (Default)
      --deb, --mint      Build Debian / Linux Mint / Ubuntu package (.deb)
      --rpm, --fedora    Build Fedora / RHEL / CentOS package (.rpm)
      --arch, --pacman   Build Arch Linux package (.pkg.tar.zst) and PKGBUILD
      --tarball, --portable
                         Build Universal Linux portable archive (.tar.gz with installer)
      --skip-build       Skip 'cargo build --release' if binary is already compiled
  -c, --clean            Clean dist/ directory before packaging
  -h, --help             Show help message
```

### Examples

```bash
# Build only Fedora RPM
./build-linux.sh --fedora

# Build only Linux Mint / Ubuntu DEB
./build-linux.sh --mint

# Build only Arch Linux package & AUR files
./build-linux.sh --arch

# Fast rebuild without recompiling release binary
./build-linux.sh --all --skip-build
```

---

## 4. End-User Installation Instructions

### A. Fedora / RHEL / CentOS Stream

Install with `dnf`:

```bash
sudo dnf install ./dist/bpsl-it-support-1.2.1-1.x86_64.rpm
```

To remove:
```bash
sudo dnf remove bpsl-it-support
```

---

### B. Linux Mint / Ubuntu / Debian

Install with `apt`:

```bash
sudo apt install ./dist/bpsl-it-support_1.2.1_amd64.deb
```

*(Alternatively: `sudo dpkg -i ./dist/bpsl-it-support_1.2.1_amd64.deb && sudo apt-get install -f`)*

To remove:
```bash
sudo apt remove bpsl-it-support
```

---

### C. Arch Linux / Manjaro / EndeavourOS

#### Option 1: Install pre-built pacman package directly
```bash
sudo pacman -U ./dist/bpsl-it-support-1.2.1-1-x86_64.pkg.tar.zst
```

#### Option 2: Build using standard PKGBUILD (or AUR)
```bash
cd dist/arch
makepkg -si
```

To remove:
```bash
sudo pacman -R bpsl-it-support
```

---

### D. Universal Linux Portable Tarball (Any Linux Distro)

The portable tarball comes with an intelligent self-contained `install.sh` and `uninstall.sh`.

```bash
tar -xzf ./dist/bpsl-it-support-1.2.1-linux-x86_64.tar.gz
cd bpsl-it-support-1.2.1-linux-x86_64

# Option A: System-wide install (requires sudo)
sudo ./install.sh

# Option B: User-local install into ~/.local (no root/sudo required!)
./install.sh
```

To uninstall:
```bash
./uninstall.sh # or sudo ./uninstall.sh if installed system-wide
```

---

## 5. System Requirements & Runtime Dependencies

The application uses GTK3, WebKit2GTK, and native OpenSSL for high performance and secure Google Sheets HTTPS synchronization.

| Distribution | Required Runtime Packages |
| :--- | :--- |
| **Fedora / RHEL** | `gtk3`, `webkit2gtk4.1` (or `webkit2gtk4.0`), `openssl-libs`, `glib2`, `cairo`, `pango` |
| **Linux Mint / Ubuntu / Debian** | `libgtk-3-0` (or `libgtk-3-0t64`), `libwebkit2gtk-4.1-0` (or `libwebkit2gtk-4.0-37`), `libssl3` (or `libssl1.1`) |
| **Arch Linux / Manjaro** | `gtk3`, `webkit2gtk-4.1`, `openssl` |

---

## 6. Architecture & File Hierarchy

When installed via `.rpm`, `.deb`, `.pkg.tar.zst`, or `install.sh`, files are placed in accordance with standard FreeDesktop and XDG standards:

```text
/usr/bin/
├── bpsl-it-support              # Primary optimized release binary (stripped)
└── management                   # Symlink for backward compatibility

/usr/share/applications/
└── bpsl-it-support.desktop      # FreeDesktop application launcher entry

/usr/share/icons/hicolor/
├── scalable/apps/bpsl-it-support.svg   # High-resolution vector icon
├── 16x16/apps/bpsl-it-support.png
├── 32x32/apps/bpsl-it-support.png
├── 48x48/apps/bpsl-it-support.png
├── 64x64/apps/bpsl-it-support.png
├── 128x128/apps/bpsl-it-support.png
├── 256x256/apps/bpsl-it-support.png
└── 512x512/apps/bpsl-it-support.png

/usr/share/pixmaps/
└── bpsl-it-support.png          # System fallback icon

/usr/share/bpsl-it-support/
└── management.db                # Clean SQLite template with all schema tables and 0 rows
```

### User Database Isolation (XDG Compliance)

On system installs (`/usr/bin`), the application never touches system directories for runtime state. User databases are automatically initialized in:

```text
~/.local/share/bpsl-it-support/management.db
```

This guarantees write permissions, SQLite WAL mode compatibility, and multi-user workstation support.

