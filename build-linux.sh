#!/usr/bin/env bash
# ==============================================================================
# BPSL IT Support - Linux Installer & Package Generation Script
# 
# Supported Distributions:
#   - Fedora, RHEL, CentOS, Rocky Linux (.rpm)
#   - Linux Mint, Ubuntu, Debian, Pop!_OS (.deb)
#   - Arch Linux, Manjaro, EndeavourOS (.pkg.tar.zst + PKGBUILD)
#   - Universal Portable Linux Distribution (.tar.gz with install.sh/uninstall.sh)
# ==============================================================================

set -euo pipefail

# ------------------------------------------------------------------------------
# Metadata & Variables
# ------------------------------------------------------------------------------
APP_BIN_NAME="bpsl-it-support"
LEGACY_BIN_NAME="management"
APP_TITLE="BPSL IT Support"
APP_COMMENT="Internal IT Support Ticket and Task Management desktop application with real-time Google Sheets synchronization"
APP_GENERIC_NAME="IT Support Management"
APP_IDENTIFIER="com.bpsl.itsupport"
MAINTAINER="developershre <developershre0901@gmail.com>"
LICENSE="Proprietary"
URL="https://github.com/developershre/Ticket_management"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="$ROOT_DIR/dist"
BUILD_DIR="$DIST_DIR/build"
RELEASE_BIN="$ROOT_DIR/target/release/$LEGACY_BIN_NAME"

# Parse version from Cargo.toml
VERSION=$(grep '^version = ' "$ROOT_DIR/Cargo.toml" | head -n1 | sed -E 's/version = "([^"]+)"/\1/')
RPM_RELEASE="1"
ARCH_RELEASE="1"

# Target distribution flags
BUILD_DEB=false
BUILD_RPM=false
BUILD_ARCH=false
BUILD_APPIMAGE=false
BUILD_FLATPAK=false
BUILD_TARBALL=false
SKIP_CARGO=false
CLEAN_FIRST=false

# ------------------------------------------------------------------------------
# CLI Argument Parsing
# ------------------------------------------------------------------------------
show_help() {
    cat << EOF
BPSL IT Support - Linux Packaging System (v${VERSION})

Usage:
  ./build-linux.sh [options]

Options:
  -a, --all         Build packages for all supported Linux distributions (Default)
      --deb, --mint Build Debian / Linux Mint / Ubuntu package (.deb)
      --rpm, --fedora
                    Build Fedora / RHEL / CentOS package (.rpm)
      --arch, --pacman
                    Build Arch Linux package (.pkg.tar.zst) and PKGBUILD
      --appimage    Build Universal Standalone AppImage (.AppImage)
      --flatpak     Generate Flatpak manifest and build scripts
      --tarball, --portable
                    Build Universal Linux portable archive (.tar.gz with install script)
      --skip-build  Skip 'cargo build --release' if binary is already compiled
  -c, --clean       Clean dist/ and build staging directory before packaging
  -h, --help        Show this help message

Examples:
  ./build-linux.sh --all             # Build all installers (.deb, .rpm, Arch, AppImage, tarball)
  ./build-linux.sh --fedora          # Build RPM for Fedora only
  ./build-linux.sh --mint            # Build DEB for Linux Mint / Ubuntu only
  ./build-linux.sh --appimage        # Build standalone AppImage only
  ./build-linux.sh --arch            # Build Arch package only
  ./build-linux.sh --all --skip-build # Package existing release binary without recompiling

EOF
}

if [[ $# -eq 0 ]]; then
    BUILD_DEB=true
    BUILD_RPM=true
    BUILD_ARCH=true
    BUILD_APPIMAGE=true
    BUILD_FLATPAK=true
    BUILD_TARBALL=true
fi

while [[ $# -gt 0 ]]; do
    case "$1" in
        -a|--all)
            BUILD_DEB=true
            BUILD_RPM=true
            BUILD_ARCH=true
            BUILD_APPIMAGE=true
            BUILD_FLATPAK=true
            BUILD_TARBALL=true
            shift
            ;;
        --deb|--mint|--ubuntu)
            BUILD_DEB=true
            shift
            ;;
        --rpm|--fedora|--rhel)
            BUILD_RPM=true
            shift
            ;;
        --arch|--pacman)
            BUILD_ARCH=true
            shift
            ;;
        --appimage)
            BUILD_APPIMAGE=true
            shift
            ;;
        --flatpak)
            BUILD_FLATPAK=true
            shift
            ;;
        --tarball|--portable)
            BUILD_TARBALL=true
            shift
            ;;
        --skip-build)
            SKIP_CARGO=true
            shift
            ;;
        -c|--clean)
            CLEAN_FIRST=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# If no specific target chosen (e.g. only --skip-build passed), default to all
if ! $BUILD_DEB && ! $BUILD_RPM && ! $BUILD_ARCH && ! $BUILD_APPIMAGE && ! $BUILD_FLATPAK && ! $BUILD_TARBALL; then
    BUILD_DEB=true
    BUILD_RPM=true
    BUILD_ARCH=true
    BUILD_APPIMAGE=true
    BUILD_FLATPAK=true
    BUILD_TARBALL=true
fi

echo "================================================================================"
echo "  BPSL IT Support - Building Linux Installers (v${VERSION})"
echo "================================================================================"
echo "Targets: "
$BUILD_RPM && echo "  - Fedora / RHEL / CentOS (.rpm)"
$BUILD_DEB && echo "  - Linux Mint / Ubuntu / Debian (.deb)"
$BUILD_ARCH && echo "  - Arch Linux (.pkg.tar.zst + PKGBUILD)"
$BUILD_APPIMAGE && echo "  - Universal Standalone AppImage (.AppImage)"
$BUILD_FLATPAK && echo "  - Flatpak Manifest (com.bpsl.itsupport.yml)"
$BUILD_TARBALL && echo "  - Universal Portable Tarball (.tar.gz)"
echo "--------------------------------------------------------------------------------"

# ------------------------------------------------------------------------------
# Clean directories if requested
# ------------------------------------------------------------------------------
if $CLEAN_FIRST; then
    echo "[Clean] Removing previous build artifacts in dist/..."
    rm -rf "$DIST_DIR"
fi

mkdir -p "$DIST_DIR"
mkdir -p "$BUILD_DIR"

# ------------------------------------------------------------------------------
# Step 1: Compile Rust Release Binary
# ------------------------------------------------------------------------------
if $SKIP_CARGO && [[ -f "$RELEASE_BIN" || -f "${RELEASE_BIN}.exe" ]]; then
    if [[ ! -f "$RELEASE_BIN" && -f "${RELEASE_BIN}.exe" ]]; then
        RELEASE_BIN="${RELEASE_BIN}.exe"
    fi
    echo "[1/6] Using existing release binary: $RELEASE_BIN"
else
    echo "[1/6] Compiling optimized release binary with cargo..."
    cargo build --release
    if [[ ! -f "$RELEASE_BIN" && -f "${RELEASE_BIN}.exe" ]]; then
        RELEASE_BIN="${RELEASE_BIN}.exe"
    fi
fi

if [[ ! -f "$RELEASE_BIN" ]]; then
    echo "Error: Release binary not found at $RELEASE_BIN"
    exit 1
fi

# Prepare stripped binary in build staging to save distribution package size
STRIPPED_BIN="$BUILD_DIR/$APP_BIN_NAME"
cp "$RELEASE_BIN" "$STRIPPED_BIN"
if command -v strip &>/dev/null; then
    strip --strip-unneeded "$STRIPPED_BIN" 2>/dev/null || true
fi
chmod 755 "$STRIPPED_BIN"

# ------------------------------------------------------------------------------
# Step 2: Generate Clean Seed Database (management.db)
# ------------------------------------------------------------------------------
echo "[2/6] Generating pristine empty database seed with all tables..."
SEED_DB="$BUILD_DIR/management.db"
node "$ROOT_DIR/scripts/generate_empty_db.js" "$SEED_DB"
# Also ensure project root management.db is synchronized and empty
node "$ROOT_DIR/scripts/generate_empty_db.js" "$ROOT_DIR/management.db"

# ------------------------------------------------------------------------------
# Step 3: Prepare Icons & Desktop Entry Specification
# ------------------------------------------------------------------------------
echo "[3/6] Generating FreeDesktop icons and application launcher..."
ICON_DIR="$BUILD_DIR/icons"
mkdir -p "$ICON_DIR"

# Copy scalable SVG and base PNG icon
cp "$ROOT_DIR/assets/logo.svg" "$ICON_DIR/$APP_BIN_NAME.svg"
if [[ -f "$ROOT_DIR/assets/logo.png" ]]; then
    cp "$ROOT_DIR/assets/logo.png" "$ICON_DIR/$APP_BIN_NAME.png"
fi

# Generate raster PNG icons using ImageMagick if available
MAGICK_CMD=""
if command -v magick &>/dev/null && magick -version 2>&1 | grep -iq "imagemagick"; then
    MAGICK_CMD="magick"
elif command -v convert &>/dev/null && convert -version 2>&1 | grep -iq "imagemagick"; then
    MAGICK_CMD="convert"
fi

if [[ -n "$MAGICK_CMD" ]]; then
    for size in 16 32 48 64 128 256 512; do
        $MAGICK_CMD -background none "$ROOT_DIR/assets/logo.svg" -resize "${size}x${size}" "$ICON_DIR/${APP_BIN_NAME}_${size}.png" 2>/dev/null || true
    done
    if [[ -f "$ICON_DIR/${APP_BIN_NAME}_256.png" ]]; then
        cp "$ICON_DIR/${APP_BIN_NAME}_256.png" "$ICON_DIR/$APP_BIN_NAME.png"
    fi
else
    echo "Notice: ImageMagick not available. Using native SVG and bundled PNG icons."
fi

# Generate FreeDesktop .desktop file
DESKTOP_FILE="$BUILD_DIR/$APP_BIN_NAME.desktop"
cat << EOF > "$DESKTOP_FILE"
[Desktop Entry]
Version=1.5
Type=Application
Name=${APP_TITLE}
GenericName=${APP_GENERIC_NAME}
Comment=${APP_COMMENT}
Exec=${APP_BIN_NAME} %U
Icon=${APP_BIN_NAME}
Terminal=false
Categories=Office;Development;Utility;Network;
Keywords=Ticket;Support;Task;BPSL;IT;Management;Kanban;Sheets;
StartupNotify=true
StartupWMClass=${APP_BIN_NAME}
X-GNOME-UsesNotifications=true
EOF
chmod 644 "$DESKTOP_FILE"

# Generate FreeDesktop AppStream / AppData Metainfo XML file
METAINFO_FILE="$BUILD_DIR/$APP_IDENTIFIER.metainfo.xml"
cat << EOF > "$METAINFO_FILE"
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>${APP_IDENTIFIER}</id>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>${LICENSE}</project_license>
  <name>${APP_TITLE}</name>
  <summary>${APP_GENERIC_NAME} with Google Sheets Synchronization</summary>
  <description>
    <p>
      ${APP_COMMENT}
    </p>
    <p>
      Key capabilities include:
    </p>
    <ul>
      <li>Interactive Kanban board with drag-and-drop workflow stages</li>
      <li>Advanced IT data table with multi-column filtering, sorting, and export</li>
      <li>Bi-directional real-time synchronization with Google Sheets</li>
      <li>Offline-ready SQLite database caching with automatic background synchronization</li>
      <li>Native system notifications for tickets, reminders, and employee assignments</li>
      <li>Smooth Dark and Light theme switching</li>
    </ul>
  </description>
  <launchable type="desktop-id">${APP_BIN_NAME}.desktop</launchable>
  <icon type="stock">${APP_BIN_NAME}</icon>
  <url type="homepage">${URL}</url>
  <url type="bugtracker">${URL}/issues</url>
  <developer_name>Bhagwati Power and Steel Limited</developer_name>
  <provides>
    <binary>${APP_BIN_NAME}</binary>
    <binary>${LEGACY_BIN_NAME}</binary>
  </provides>
  <categories>
    <category>Office</category>
    <category>Development</category>
    <category>Utility</category>
    <category>Network</category>
  </categories>
  <keywords>
    <keyword>ticket</keyword>
    <keyword>support</keyword>
    <keyword>task</keyword>
    <keyword>bpsl</keyword>
    <keyword>it</keyword>
    <keyword>management</keyword>
    <keyword>kanban</keyword>
    <keyword>sheets</keyword>
  </keywords>
  <releases>
    <release version="${VERSION}" date="2026-11-23">
      <description>
        <p>Release v${VERSION}: Native notifications, compact context menu, fast theme switcher, and multi-distro packaging.</p>
      </description>
    </release>
  </releases>
  <content_rating type="oars-1.1" />
</component>
EOF
chmod 644 "$METAINFO_FILE"

# Helper function to populate standard Linux root hierarchy for an installer
populate_linux_tree() {
    local target_root="$1"
    
    mkdir -p "$target_root/usr/bin"
    mkdir -p "$target_root/usr/share/applications"
    mkdir -p "$target_root/usr/share/metainfo"
    mkdir -p "$target_root/usr/share/bpsl-it-support"
    mkdir -p "$target_root/usr/share/icons/hicolor/scalable/apps"
    mkdir -p "$target_root/usr/share/pixmaps"

    # Install binaries
    cp "$STRIPPED_BIN" "$target_root/usr/bin/$APP_BIN_NAME"
    chmod 755 "$target_root/usr/bin/$APP_BIN_NAME"
    ln -sf "$APP_BIN_NAME" "$target_root/usr/bin/$LEGACY_BIN_NAME"

    # Install desktop entry
    cp "$DESKTOP_FILE" "$target_root/usr/share/applications/$APP_BIN_NAME.desktop"
    chmod 644 "$target_root/usr/share/applications/$APP_BIN_NAME.desktop"

    # Install AppStream metainfo
    cp "$METAINFO_FILE" "$target_root/usr/share/metainfo/$APP_IDENTIFIER.metainfo.xml"
    chmod 644 "$target_root/usr/share/metainfo/$APP_IDENTIFIER.metainfo.xml"

    # Install icons
    cp "$ICON_DIR/$APP_BIN_NAME.svg" "$target_root/usr/share/icons/hicolor/scalable/apps/$APP_BIN_NAME.svg"
    if [[ -f "$ICON_DIR/$APP_BIN_NAME.png" ]]; then
        cp "$ICON_DIR/$APP_BIN_NAME.png" "$target_root/usr/share/pixmaps/$APP_BIN_NAME.png"
    fi
    for size in 16 32 48 64 128 256 512; do
        if [[ -f "$ICON_DIR/${APP_BIN_NAME}_${size}.png" ]]; then
            mkdir -p "$target_root/usr/share/icons/hicolor/${size}x${size}/apps"
            cp "$ICON_DIR/${APP_BIN_NAME}_${size}.png" "$target_root/usr/share/icons/hicolor/${size}x${size}/apps/$APP_BIN_NAME.png"
            chmod 644 "$target_root/usr/share/icons/hicolor/${size}x${size}/apps/$APP_BIN_NAME.png"
        fi
    done

    # Install seed database
    cp "$SEED_DB" "$target_root/usr/share/bpsl-it-support/management.db"
    chmod 644 "$target_root/usr/share/bpsl-it-support/management.db"
}

# ------------------------------------------------------------------------------
# Step 4A: Build Linux Mint / Ubuntu / Debian (.deb)
# ------------------------------------------------------------------------------
if $BUILD_DEB; then
    echo "[4/6] Building Linux Mint / Ubuntu / Debian package (.deb)..."
    DEB_STAGE="$BUILD_DIR/deb/bpsl-it-support_${VERSION}_amd64"
    rm -rf "$DEB_STAGE"
    mkdir -p "$DEB_STAGE/DEBIAN"
    populate_linux_tree "$DEB_STAGE"

    # Calculate installed size in KB
    INSTALLED_SIZE=$(du -sk "$DEB_STAGE" | cut -f1)

    # Control file
    cat << EOF > "$DEB_STAGE/DEBIAN/control"
Package: ${APP_BIN_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Installed-Size: ${INSTALLED_SIZE}
Maintainer: ${MAINTAINER}
Homepage: ${URL}
Depends: libc6 (>= 2.31), libgtk-3-0 (>= 3.24) | libgtk-3-0t64, libwebkit2gtk-4.1-0 (>= 2.36) | libwebkit2gtk-4.0-37, libssl3 | libssl1.1
Recommends: libayatana-appindicator3-1 | libappindicator3-1
Description: ${APP_TITLE}
 ${APP_COMMENT}.
 Provides offline-ready SQLite cache, background Google Sheets sync,
 push notifications, and employee-ticket management.
EOF
    chmod 0755 "$DEB_STAGE/DEBIAN"
    chmod 0644 "$DEB_STAGE/DEBIAN/control"

    # postinst script
    cat << 'EOF' > "$DEB_STAGE/DEBIAN/postinst"
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi
if command -v appstreamcli >/dev/null 2>&1; then
    appstreamcli refresh-cache >/dev/null 2>&1 || true
fi
exit 0
EOF
    chmod 0755 "$DEB_STAGE/DEBIAN/postinst"

    # postrm script
    cat << 'EOF' > "$DEB_STAGE/DEBIAN/postrm"
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi
if command -v appstreamcli >/dev/null 2>&1; then
    appstreamcli refresh-cache >/dev/null 2>&1 || true
fi
exit 0
EOF
    chmod 0755 "$DEB_STAGE/DEBIAN/postrm"

    DEB_OUTPUT="$DIST_DIR/${APP_BIN_NAME}_${VERSION}_amd64.deb"
    if command -v dpkg-deb &>/dev/null; then
        dpkg-deb --build --root-owner-group "$DEB_STAGE" "$DEB_OUTPUT" > /dev/null
    elif command -v node &>/dev/null && [[ -f "$ROOT_DIR/scripts/create_deb.js" ]]; then
        node "$ROOT_DIR/scripts/create_deb.js" "$DEB_STAGE" "$DEB_OUTPUT"
    else
        echo "Notice: Neither dpkg-deb nor Node.js found. Staged deb directory preserved in: $DEB_STAGE"
    fi

    if [[ -f "$DEB_OUTPUT" ]]; then
        # Convenience aliases for quick identification
        ln -sf "${APP_BIN_NAME}_${VERSION}_amd64.deb" "$DIST_DIR/${APP_BIN_NAME}_linuxmint.deb" 2>/dev/null || cp "$DEB_OUTPUT" "$DIST_DIR/${APP_BIN_NAME}_linuxmint.deb" 2>/dev/null || true
        ln -sf "${APP_BIN_NAME}_${VERSION}_amd64.deb" "$DIST_DIR/${APP_BIN_NAME}_ubuntu.deb" 2>/dev/null || cp "$DEB_OUTPUT" "$DIST_DIR/${APP_BIN_NAME}_ubuntu.deb" 2>/dev/null || true
        echo "  -> Created: $DEB_OUTPUT"
    fi
fi

# ------------------------------------------------------------------------------
# Step 4B: Build Fedora / RHEL / CentOS (.rpm)
# ------------------------------------------------------------------------------
if $BUILD_RPM; then
    echo "[4/6] Building Fedora / RHEL package (.rpm)..."
    RPM_TOP="$BUILD_DIR/rpm"
    rm -rf "$RPM_TOP"
    mkdir -p "$RPM_TOP"/{BUILD,RPMS,SOURCES/tree,SPECS,SRPMS,BUILDROOT,tmp}

    # Populate filesystem tree in SOURCES/tree
    populate_linux_tree "$RPM_TOP/SOURCES/tree"

    RPM_SPEC="$RPM_TOP/SPECS/${APP_BIN_NAME}.spec"
    cat << EOF > "$RPM_SPEC"
%define _topdir ${RPM_TOP}
%define _tmppath ${RPM_TOP}/tmp

Name:           ${APP_BIN_NAME}
Version:        ${VERSION}
Release:        ${RPM_RELEASE}%{?dist}
Summary:        ${APP_TITLE}
License:        ${LICENSE}
URL:            ${URL}
Vendor:         Bhagwati Power and Steel Limited
Group:          Applications/Productivity
BuildArch:      x86_64

# Dependencies for GTK3 / WebKit2GTK on Fedora & Enterprise Linux
Requires:       gtk3 >= 3.24
Requires:       (webkit2gtk4.1 or webkit2gtk4.0 or webkit2gtk3)
Requires:       openssl-libs
Requires:       glib2
Requires:       cairo
Requires:       pango

%description
${APP_COMMENT}

%prep

%build

%install
mkdir -p %{buildroot}
cp -a %{_sourcedir}/tree/usr %{buildroot}/

%files
/usr/bin/${APP_BIN_NAME}
/usr/bin/${LEGACY_BIN_NAME}
/usr/share/applications/${APP_BIN_NAME}.desktop
/usr/share/metainfo/${APP_IDENTIFIER}.metainfo.xml
/usr/share/icons/hicolor/*/apps/${APP_BIN_NAME}.*
/usr/share/pixmaps/${APP_BIN_NAME}.png
/usr/share/bpsl-it-support/management.db

%post
/bin/touch --no-create /usr/share/icons/hicolor &>/dev/null || :
/usr/bin/update-desktop-database &>/dev/null || :

%postun
if [ \$1 -eq 0 ] ; then
    /bin/touch --no-create /usr/share/icons/hicolor &>/dev/null
    /usr/bin/gtk-update-icon-cache /usr/share/icons/hicolor &>/dev/null || :
    /usr/bin/update-desktop-database &>/dev/null || :
fi

%posttrans
/usr/bin/gtk-update-icon-cache /usr/share/icons/hicolor &>/dev/null || :
/usr/bin/update-desktop-database &>/dev/null || :

%changelog
* Sat Nov 21 2026 ${MAINTAINER} - ${VERSION}-${RPM_RELEASE}
- Linux installer release with native Google Sheets sync and notification support
EOF

    if command -v rpmbuild &>/dev/null; then
        rpmbuild --quiet --define "_topdir ${RPM_TOP}" --define "_tmppath ${RPM_TOP}/tmp" -bb "$RPM_SPEC"

        BUILT_RPM=$(find "$RPM_TOP/RPMS" -type f -name "*.rpm" 2>/dev/null | head -n1)
        if [[ -n "$BUILT_RPM" ]]; then
            RPM_OUTPUT="$DIST_DIR/${APP_BIN_NAME}-${VERSION}-${RPM_RELEASE}.x86_64.rpm"
            cp "$BUILT_RPM" "$RPM_OUTPUT"
            ln -sf "${APP_BIN_NAME}-${VERSION}-${RPM_RELEASE}.x86_64.rpm" "$DIST_DIR/${APP_BIN_NAME}_fedora.rpm" 2>/dev/null || true
            echo "  -> Created: $RPM_OUTPUT"
        else
            echo "Error: RPM file was not produced."
        fi
    else
        echo "  Notice: rpmbuild not found on host (requires Fedora/RHEL or Docker/WSL to produce .rpm)."
        echo "  -> Staged RPM spec and filesystem tree ready at: $RPM_SPEC"
    fi
fi

# ------------------------------------------------------------------------------
# Step 4C: Build Arch Linux (.pkg.tar.zst) and PKGBUILD
# ------------------------------------------------------------------------------
if $BUILD_ARCH; then
    echo "[4/6] Building Arch Linux / Manjaro package (.pkg.tar.zst) & PKGBUILD..."
    ARCH_DIR="$DIST_DIR/arch"
    rm -rf "$ARCH_DIR"
    mkdir -p "$ARCH_DIR"

    # 1. Output standard PKGBUILD and .SRCINFO for AUR or makepkg
    cat << EOF > "$ARCH_DIR/PKGBUILD"
# Maintainer: ${MAINTAINER}
pkgname=${APP_BIN_NAME}
pkgver=${VERSION}
pkgrel=${ARCH_RELEASE}
pkgdesc="${APP_TITLE} - ${APP_GENERIC_NAME}"
arch=('x86_64')
url="${URL}"
license=('custom:proprietary')
depends=('gtk3' 'webkit2gtk-4.1' 'openssl')
optdepends=('libappindicator-gtk3: System tray and indicator support')
provides=('${APP_BIN_NAME}' '${LEGACY_BIN_NAME}')
conflicts=('${APP_BIN_NAME}' '${LEGACY_BIN_NAME}')

package() {
    install -Dm755 "\$srcdir/bpsl-it-support" "\$pkgdir/usr/bin/bpsl-it-support"
    ln -s "bpsl-it-support" "\$pkgdir/usr/bin/management"
    install -Dm644 "\$srcdir/bpsl-it-support.desktop" "\$pkgdir/usr/share/applications/bpsl-it-support.desktop"
    install -Dm644 "\$srcdir/bpsl-it-support.svg" "\$pkgdir/usr/share/icons/hicolor/scalable/apps/bpsl-it-support.svg"
    install -Dm644 "\$srcdir/${APP_IDENTIFIER}.metainfo.xml" "\$pkgdir/usr/share/metainfo/${APP_IDENTIFIER}.metainfo.xml"
    install -Dm644 "\$srcdir/management.db" "\$pkgdir/usr/share/bpsl-it-support/management.db"
}
EOF

    cat << EOF > "$ARCH_DIR/.SRCINFO"
pkgbase = ${APP_BIN_NAME}
	pkgdesc = ${APP_TITLE} - ${APP_GENERIC_NAME}
	pkgver = ${VERSION}
	pkgrel = ${ARCH_RELEASE}
	url = ${URL}
	arch = x86_64
	license = custom:proprietary
	depends = gtk3
	depends = webkit2gtk-4.1
	depends = openssl
	optdepends = libappindicator-gtk3: System tray and indicator support
	provides = ${APP_BIN_NAME}
	provides = ${LEGACY_BIN_NAME}
	conflicts = ${APP_BIN_NAME}
	conflicts = ${LEGACY_BIN_NAME}
	source = bpsl-it-support
	source = bpsl-it-support.desktop
	source = bpsl-it-support.svg
	source = ${APP_IDENTIFIER}.metainfo.xml
	source = management.db

pkgname = ${APP_BIN_NAME}
EOF

    # 2. Build installable .pkg.tar.zst package archive
    ARCH_STAGE="$BUILD_DIR/arch_stage"
    rm -rf "$ARCH_STAGE"
    mkdir -p "$ARCH_STAGE"
    populate_linux_tree "$ARCH_STAGE"

    ARCH_SIZE=$(du -sb "$ARCH_STAGE" | cut -f1)
    BUILD_TIMESTAMP=$(date +%s)

    cat << EOF > "$ARCH_STAGE/.PKGINFO"
format = 2
pkgname = ${APP_BIN_NAME}
pkgbase = ${APP_BIN_NAME}
pkgver = ${VERSION}-${ARCH_RELEASE}
pkgdesc = ${APP_TITLE} - ${APP_GENERIC_NAME}
url = ${URL}
builddate = ${BUILD_TIMESTAMP}
packager = ${MAINTAINER}
size = ${ARCH_SIZE}
arch = x86_64
license = proprietary
depend = gtk3
depend = webkit2gtk-4.1
depend = openssl
optdepend = libappindicator-gtk3: System tray and indicator support
provides = ${APP_BIN_NAME}
provides = ${LEGACY_BIN_NAME}
conflict = ${APP_BIN_NAME}
conflict = ${LEGACY_BIN_NAME}
makepkgopt = strip
EOF

    ARCH_OUTPUT="$DIST_DIR/${APP_BIN_NAME}-${VERSION}-${ARCH_RELEASE}-x86_64.pkg.tar.zst"
    if (cd "$ARCH_STAGE" && tar --zstd --owner=0 --group=0 -cf "$ARCH_OUTPUT" .PKGINFO usr 2>/dev/null); then
        ln -sf "${APP_BIN_NAME}-${VERSION}-${ARCH_RELEASE}-x86_64.pkg.tar.zst" "$DIST_DIR/${APP_BIN_NAME}_arch.pkg.tar.zst" 2>/dev/null || true
        echo "  -> Created: $ARCH_OUTPUT"
    else
        echo "  Notice: zstd compression not available on host. Staged Arch package ready in: $ARCH_STAGE"
    fi
    echo "  -> Created: $ARCH_DIR/PKGBUILD and .SRCINFO"
fi

# ------------------------------------------------------------------------------
# Step 4D: Build Universal Portable Linux Tarball (.tar.gz)
# ------------------------------------------------------------------------------
if $BUILD_TARBALL; then
    echo "[4/6] Building Universal Portable Linux package (.tar.gz)..."
    TARBALL_STAGE="$BUILD_DIR/portable/bpsl-it-support-${VERSION}-linux-x86_64"
    rm -rf "$TARBALL_STAGE"
    mkdir -p "$TARBALL_STAGE"

    cp "$STRIPPED_BIN" "$TARBALL_STAGE/$APP_BIN_NAME"
    chmod 755 "$TARBALL_STAGE/$APP_BIN_NAME"
    ln -sf "$APP_BIN_NAME" "$TARBALL_STAGE/$LEGACY_BIN_NAME"
    cp "$SEED_DB" "$TARBALL_STAGE/management.db"
    cp "$DESKTOP_FILE" "$TARBALL_STAGE/$APP_BIN_NAME.desktop"
    cp "$METAINFO_FILE" "$TARBALL_STAGE/$APP_IDENTIFIER.metainfo.xml"
    cp "$ICON_DIR/$APP_BIN_NAME.svg" "$TARBALL_STAGE/logo.svg"
    if [[ -f "$ICON_DIR/$APP_BIN_NAME.png" ]]; then
        cp "$ICON_DIR/$APP_BIN_NAME.png" "$TARBALL_STAGE/logo.png"
    fi

    # Self-contained smart installer script for any distro (system or user local)
    cat << 'EOF' > "$TARBALL_STAGE/install.sh"
#!/usr/bin/env bash
set -e

APP_NAME="bpsl-it-support"
APP_IDENTIFIER="com.bpsl.itsupport"
TITLE="BPSL IT Support"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Installing $TITLE ==="

if [ "$(id -u)" -eq 0 ]; then
    INSTALL_PREFIX="/usr/local"
    BIN_DIR="$INSTALL_PREFIX/bin"
    APP_DIR="$INSTALL_PREFIX/share/applications"
    META_DIR="$INSTALL_PREFIX/share/metainfo"
    ICON_DIR="$INSTALL_PREFIX/share/icons/hicolor/scalable/apps"
    DATA_DIR="$INSTALL_PREFIX/share/$APP_NAME"
else
    INSTALL_PREFIX="${XDG_DATA_HOME:-$HOME/.local}"
    BIN_DIR="$HOME/.local/bin"
    APP_DIR="$HOME/.local/share/applications"
    META_DIR="$HOME/.local/share/metainfo"
    ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
    DATA_DIR="$HOME/.local/share/$APP_NAME"
    mkdir -p "$BIN_DIR"
fi

mkdir -p "$BIN_DIR" "$APP_DIR" "$META_DIR" "$ICON_DIR" "$DATA_DIR"

echo "Installing binary to $BIN_DIR..."
cp "$SCRIPT_DIR/$APP_NAME" "$BIN_DIR/$APP_NAME"
chmod 755 "$BIN_DIR/$APP_NAME"
ln -sf "$APP_NAME" "$BIN_DIR/management"

if [ -f "$DATA_DIR/management.db" ]; then
    echo "Existing database detected at $DATA_DIR/management.db."
    echo "Preserving user data and updating database tables if required..."
    "$BIN_DIR/$APP_NAME" --migrate-db 2>/dev/null || true
else
    echo "Fresh install: initializing fresh database with all tables at $DATA_DIR/management.db..."
    cp "$SCRIPT_DIR/management.db" "$DATA_DIR/management.db"
    chmod 644 "$DATA_DIR/management.db"
fi
rm -f "$DATA_DIR/empty.db"

echo "Installing icons to $ICON_DIR..."
cp "$SCRIPT_DIR/logo.svg" "$ICON_DIR/$APP_NAME.svg"

echo "Installing AppStream metadata to $META_DIR..."
if [ -f "$SCRIPT_DIR/$APP_IDENTIFIER.metainfo.xml" ]; then
    cp "$SCRIPT_DIR/$APP_IDENTIFIER.metainfo.xml" "$META_DIR/$APP_IDENTIFIER.metainfo.xml"
    chmod 644 "$META_DIR/$APP_IDENTIFIER.metainfo.xml"
fi

echo "Installing desktop entry to $APP_DIR..."
sed -e "s|^Exec=.*|Exec=$BIN_DIR/$APP_NAME %U|" \
    -e "s|^Icon=.*|Icon=$ICON_DIR/$APP_NAME.svg|" \
    "$SCRIPT_DIR/$APP_NAME.desktop" > "$APP_DIR/$APP_NAME.desktop"
chmod 644 "$APP_DIR/$APP_NAME.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" || true
fi

echo "================================================="
echo "  $TITLE installed successfully!"
echo "  Run via terminal: $APP_NAME"
echo "  Or launch from your Applications menu."
echo "================================================="
EOF
    chmod 755 "$TARBALL_STAGE/install.sh"

    # Uninstaller script
    cat << 'EOF' > "$TARBALL_STAGE/uninstall.sh"
#!/usr/bin/env bash
set -e

APP_NAME="bpsl-it-support"
APP_IDENTIFIER="com.bpsl.itsupport"
TITLE="BPSL IT Support"

echo "=== Uninstalling $TITLE ==="

if [ "$(id -u)" -eq 0 ]; then
    rm -f "/usr/local/bin/$APP_NAME"
    rm -f "/usr/local/bin/management"
    rm -f "/usr/local/share/applications/$APP_NAME.desktop"
    rm -f "/usr/local/share/metainfo/$APP_IDENTIFIER.metainfo.xml"
    rm -f "/usr/local/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
    rm -rf "/usr/local/share/$APP_NAME"
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "/usr/local/share/applications" || true
    fi
else
    rm -f "$HOME/.local/bin/$APP_NAME"
    rm -f "$HOME/.local/bin/management"
    rm -f "$HOME/.local/share/applications/$APP_NAME.desktop"
    rm -f "$HOME/.local/share/metainfo/$APP_IDENTIFIER.metainfo.xml"
    rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
    rm -rf "$HOME/.local/share/$APP_NAME"
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$HOME/.local/share/applications" || true
    fi
fi

echo "$TITLE uninstalled successfully."
EOF
    chmod 755 "$TARBALL_STAGE/uninstall.sh"

    # README for the portable bundle
    cat << EOF > "$TARBALL_STAGE/README.txt"
================================================================================
  ${APP_TITLE} (v${VERSION}) - Universal Linux Standalone Package
================================================================================

QUICK START:
1. Run directly without installation:
   ./${APP_BIN_NAME}

2. Install system-wide (requires sudo):
   sudo ./install.sh

3. Install for current user only (no sudo required):
   ./install.sh

4. Uninstall:
   ./uninstall.sh (or sudo ./uninstall.sh if installed system-wide)

SYSTEM REQUIREMENTS:
- GTK 3 (gtk3 / libgtk-3-0)
- WebKit2GTK (webkit2gtk4.1 / libwebkit2gtk-4.1-0)
- OpenSSL (openssl-libs / libssl3)
EOF

    TARBALL_OUTPUT="$DIST_DIR/${APP_BIN_NAME}-${VERSION}-linux-x86_64.tar.gz"
    (cd "$BUILD_DIR/portable" && tar -czf "$TARBALL_OUTPUT" "bpsl-it-support-${VERSION}-linux-x86_64")
    ln -sf "${APP_BIN_NAME}-${VERSION}-linux-x86_64.tar.gz" "$DIST_DIR/${APP_BIN_NAME}_portable.tar.gz"
    echo "  -> Created: $TARBALL_OUTPUT"
fi

# ------------------------------------------------------------------------------
# Step 4E: Build Universal Standalone AppImage (.AppImage)
# ------------------------------------------------------------------------------
if $BUILD_APPIMAGE; then
    echo "[4/6] Building Universal Standalone AppImage (.AppImage)..."
    APPDIR="$BUILD_DIR/appimage/AppDir"
    rm -rf "$APPDIR"
    mkdir -p "$APPDIR"
    populate_linux_tree "$APPDIR"

    # AppImage root launcher and icons
    cp "$DESKTOP_FILE" "$APPDIR/$APP_BIN_NAME.desktop"
    cp "$ICON_DIR/$APP_BIN_NAME.svg" "$APPDIR/$APP_BIN_NAME.svg"
    if [[ -f "$ICON_DIR/$APP_BIN_NAME.png" ]]; then
        cp "$ICON_DIR/$APP_BIN_NAME.png" "$APPDIR/$APP_BIN_NAME.png"
        cp "$ICON_DIR/$APP_BIN_NAME.png" "$APPDIR/.DirIcon"
    else
        cp "$ICON_DIR/$APP_BIN_NAME.svg" "$APPDIR/.DirIcon"
    fi

    # Create standard AppRun entrypoint
    cat << 'EOF' > "$APPDIR/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${HERE}/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "${HERE}/usr/bin/bpsl-it-support" "$@"
EOF
    chmod 755 "$APPDIR/AppRun"

    APPIMAGE_OUTPUT="$DIST_DIR/${APP_BIN_NAME}-${VERSION}-x86_64.AppImage"
    APPIMAGETOOL_CMD=""
    # Only download and execute appimagetool on native Linux environments
    if [[ "$OSTYPE" != "msys"* && "$OSTYPE" != "win32"* && "$OSTYPE" != "cygwin"* ]]; then
        if command -v appimagetool &>/dev/null; then
            APPIMAGETOOL_CMD="appimagetool"
        elif [[ -f "$BUILD_DIR/appimagetool-x86_64.AppImage" ]]; then
            APPIMAGETOOL_CMD="$BUILD_DIR/appimagetool-x86_64.AppImage"
        elif command -v wget &>/dev/null || command -v curl &>/dev/null; then
            echo "  [AppImage] Downloading appimagetool..."
            TOOL_PATH="$BUILD_DIR/appimagetool-x86_64.AppImage"
            TOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
            if command -v curl &>/dev/null; then
                curl -sSL -o "$TOOL_PATH" "$TOOL_URL" || true
            else
                wget -q -O "$TOOL_PATH" "$TOOL_URL" || true
            fi
            if [[ -f "$TOOL_PATH" && -s "$TOOL_PATH" ]]; then
                chmod +x "$TOOL_PATH"
                APPIMAGETOOL_CMD="$TOOL_PATH"
            fi
        fi

        if [[ -n "$APPIMAGETOOL_CMD" ]]; then
            ARCH=x86_64 "$APPIMAGETOOL_CMD" --appimage-extract-and-run "$APPDIR" "$APPIMAGE_OUTPUT" 2>/dev/null || \
            ARCH=x86_64 "$APPIMAGETOOL_CMD" "$APPDIR" "$APPIMAGE_OUTPUT" || true
        fi
    fi

    if [[ -f "$APPIMAGE_OUTPUT" ]]; then
        chmod +x "$APPIMAGE_OUTPUT"
        ln -sf "${APP_BIN_NAME}-${VERSION}-x86_64.AppImage" "$DIST_DIR/${APP_BIN_NAME}.AppImage"
        echo "  -> Created: $APPIMAGE_OUTPUT"
    else
        echo "  -> Assembled ready AppDir at: $APPDIR"
        echo "  -> Note: Run 'appimagetool $APPDIR $APPIMAGE_OUTPUT' on a Linux host to finalize .AppImage."
    fi
fi

# ------------------------------------------------------------------------------
# Step 4F: Generate Flatpak Manifest (com.bpsl.itsupport.yml)
# ------------------------------------------------------------------------------
if $BUILD_FLATPAK; then
    echo "[4/6] Generating Flatpak manifest (com.bpsl.itsupport.yml)..."
    FLATPAK_DIR="$DIST_DIR/flatpak"
    rm -rf "$FLATPAK_DIR"
    mkdir -p "$FLATPAK_DIR"

    cat << EOF > "$FLATPAK_DIR/${APP_IDENTIFIER}.yml"
app-id: ${APP_IDENTIFIER}
runtime: org.gnome.Platform
runtime-version: '47'
sdk: org.gnome.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
command: ${APP_BIN_NAME}

finish-args:
  - --share=ipc
  - --socket=fallback-x11
  - --socket=wayland
  - --share=network
  - --device=dri
  - --talk-name=org.freedesktop.Notifications
  - --filesystem=xdg-config
  - --filesystem=xdg-data

modules:
  - name: ${APP_BIN_NAME}
    buildsystem: simple
    build-commands:
      - cargo --offline fetch --manifest-path Cargo.toml || true
      - cargo build --release --locked
      - install -Dm755 target/release/${LEGACY_BIN_NAME} /app/bin/${APP_BIN_NAME}
      - ln -sf ${APP_BIN_NAME} /app/bin/${LEGACY_BIN_NAME}
      - install -Dm644 assets/logo.svg /app/share/icons/hicolor/scalable/apps/${APP_BIN_NAME}.svg
      - install -Dm644 ${APP_BIN_NAME}.desktop /app/share/applications/${APP_BIN_NAME}.desktop
      - install -Dm644 ${APP_IDENTIFIER}.metainfo.xml /app/share/metainfo/${APP_IDENTIFIER}.metainfo.xml
      - install -Dm644 management.db /app/share/${APP_BIN_NAME}/management.db
    sources:
      - type: dir
        path: ../..
EOF
    cp "$DESKTOP_FILE" "$FLATPAK_DIR/${APP_BIN_NAME}.desktop"
    cp "$METAINFO_FILE" "$FLATPAK_DIR/${APP_IDENTIFIER}.metainfo.xml"
    echo "  -> Created Flatpak manifest: $FLATPAK_DIR/${APP_IDENTIFIER}.yml"
fi

# ------------------------------------------------------------------------------
# Step 5: Summary Report & Installation Instructions
# ------------------------------------------------------------------------------
echo ""
echo "================================================================================"
echo "  BUILD COMPLETE - Ready-to-Distribute Linux Installers"
echo "================================================================================"
echo ""
echo "Generated installer packages in $DIST_DIR:"
ls -lh "$DIST_DIR"/*.{deb,rpm,pkg.tar.zst,tar.gz,AppImage} 2>/dev/null || true

echo ""
echo "--------------------------------------------------------------------------------"
echo "DISTRIBUTION INSTALLATION COMMANDS:"
echo "--------------------------------------------------------------------------------"
if $BUILD_RPM; then
    echo "● Fedora / RHEL / CentOS / Rocky Linux:"
    echo "    sudo dnf install ./dist/${APP_BIN_NAME}-${VERSION}-${RPM_RELEASE}.x86_64.rpm"
    echo ""
fi
if $BUILD_DEB; then
    echo "● Linux Mint / Ubuntu / Debian / Pop!_OS:"
    echo "    sudo apt install ./dist/${APP_BIN_NAME}_${VERSION}_amd64.deb"
    echo ""
fi
if $BUILD_ARCH; then
    echo "● Arch Linux / Manjaro / EndeavourOS:"
    echo "    sudo pacman -U ./dist/${APP_BIN_NAME}-${VERSION}-${ARCH_RELEASE}-x86_64.pkg.tar.zst"
    echo "    # Or build/install via PKGBUILD: cd dist/arch && makepkg -si"
    echo ""
fi
if $BUILD_APPIMAGE; then
    echo "● Universal Standalone AppImage (Zero Installation, any Linux distro):"
    echo "    chmod +x ./dist/${APP_BIN_NAME}-${VERSION}-x86_64.AppImage"
    echo "    ./dist/${APP_BIN_NAME}-${VERSION}-x86_64.AppImage"
    echo ""
fi
if $BUILD_FLATPAK; then
    echo "● Flatpak Builder (Sandboxed multi-distro):"
    echo "    flatpak-builder --user --install --force-clean build-dir ./dist/flatpak/${APP_IDENTIFIER}.yml"
    echo ""
fi
if $BUILD_TARBALL; then
    echo "● Universal Linux Portable Tarball:"
    echo "    tar -xzf ./dist/${APP_BIN_NAME}-${VERSION}-linux-x86_64.tar.gz"
    echo "    cd bpsl-it-support-${VERSION}-linux-x86_64 && ./install.sh"
    echo ""
fi
echo "================================================================================"
