#!/usr/bin/env bash
# ==============================================================================
# Package and install dual KatanA and KatanB applications for macOS.
# Both apps run completely independently and simultaneously with isolated configs,
# isolated caches, and distinctive icons (Original Deep Navy vs Dark Gold).
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

INSTALL_DIR="${INSTALL_DIR:-/Applications}"
BINARY="${ROOT_DIR}/target/release/KatanA"
ICON_A="${ROOT_DIR}/assets/icon.icns"
ICON_B="${ROOT_DIR}/target/icon_b.icns"

VERSION="$(grep '^version' "${ROOT_DIR}/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')"
BUILD_NUMBER="${BUILD_NUMBER:-1}"

if [ ! -f "${BINARY}" ]; then
    echo "Error: Binary not found at ${BINARY}. Run 'cargo build --release --package katana-ui --bin KatanA' first." >&2
    exit 1
fi

if [ ! -f "${ICON_B}" ]; then
    echo "Generating KatanB dark gold icon..."
    python3 "${SCRIPT_DIR}/generate_gold_icon.py"
fi

echo "============================================================"
echo " Packaging KatanA & KatanB for macOS"
echo " Destination: ${INSTALL_DIR}"
echo " Version:     v${VERSION} (Build ${BUILD_NUMBER})"
echo "============================================================"

for app_name in KatanA KatanB; do
    APP_PATH="${INSTALL_DIR}/${app_name}.app"
    CONTENTS="${APP_PATH}/Contents"
    MACOS_DIR="${CONTENTS}/MacOS"
    RESOURCES_DIR="${CONTENTS}/Resources"

    echo "==> Installing ${app_name}.app..."
    rm -rf "${APP_PATH}"
    mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

    # 1. Copy binary with distinct executable name (enables config/cache isolation)
    cp "${BINARY}" "${MACOS_DIR}/${app_name}"
    chmod +x "${MACOS_DIR}/${app_name}"

    # 2. Copy distinctive icon
    if [ "${app_name}" = "KatanA" ]; then
        cp "${ICON_A}" "${RESOURCES_DIR}/icon.icns"
        BUNDLE_ID="com.github.katana.app"
    else
        cp "${ICON_B}" "${RESOURCES_DIR}/icon.icns"
        BUNDLE_ID="com.github.katana-b.app"
    fi

    # 3. Create Info.plist
    cat > "${CONTENTS}/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleExecutable</key>
    <string>${app_name}</string>
    <key>CFBundleName</key>
    <string>${app_name}</string>
    <key>CFBundleDisplayName</key>
    <string>${app_name}</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>v${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${BUILD_NUMBER}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

    # 4. Ad-hoc codesign
    codesign --force --deep --sign - "${APP_PATH}" > /dev/null 2>&1
    touch "${APP_PATH}"
    echo "  ✅ Installed and signed: ${APP_PATH}"
done

echo ""
echo "✨ Dual apps successfully installed to ${INSTALL_DIR}!"
echo "   - KatanA: ${INSTALL_DIR}/KatanA.app (Original Deep Navy)"
echo "   - KatanB: ${INSTALL_DIR}/KatanB.app (Dark Gold / 暗金色)"
