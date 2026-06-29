#!/usr/bin/env bash
# build-macos.sh — vlt-syslogd GUI の macOS 配布物を作る。
#
# 2つの形態を、同じソースから feature だけ変えてビルドし、それぞれ
# ad-hoc 署名済みの .app + zip にする(dist/ に出力):
#
#   - App 版      … cargo build --release             (データ = ~/Library/Application Support)
#   - Portable 版 … cargo build --release --features portable (データ = .app の隣)
#
# 署名は ad-hoc(Apple Developer 不要)。同梱物を全部置いた「後」に deep sign する
# (順序を誤ると署名シールが壊れ、TCC が要求を登録しない)。
#
# 使い方:  ./Portable/build-macos.sh   (リポジトリのどこから実行してもよい)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # = .../Portable
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"                          # = リポジトリルート
ICNS="$ROOT/icons/vlt-syslogd.icns"
DIST="$ROOT/dist"
EXE="vlt-syslogd-portable"   # CFBundleExecutable(両形態で同じバイナリ名)

# バージョンは Portable/Cargo.toml から取得
VER="$(grep -m1 '^version' "$SCRIPT_DIR/Cargo.toml" | sed -E 's/.*"(.*)".*/\1/')"

if [ ! -f "$ICNS" ]; then
  echo "エラー: $ICNS が見つかりません(先にアイコンを生成してください)" >&2
  exit 1
fi

mkdir -p "$DIST"

# --- .app を組み立てて ad-hoc 署名し zip 化する関数 ---
# 引数: 1=表示名  2=bundle id  3=ビルドした実行ファイルのパス  4=出力ベース名
package_app() {
  local name="$1" bundle_id="$2" bin="$3" outbase="$4"
  local app="$DIST/${name}.app"
  local contents="$app/Contents"

  echo "==> パッケージ: ${name}.app  (id=$bundle_id)"
  rm -rf "$app"
  mkdir -p "$contents/MacOS" "$contents/Resources"

  install -m 755 "$bin" "$contents/MacOS/$EXE"
  cp "$ICNS" "$contents/Resources/vlt-syslogd.icns"

  cat > "$contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>${name}</string>
    <key>CFBundleDisplayName</key><string>${name}</string>
    <key>CFBundleExecutable</key><string>${EXE}</string>
    <key>CFBundleIdentifier</key><string>${bundle_id}</string>
    <key>CFBundleVersion</key><string>${VER}</string>
    <key>CFBundleShortVersionString</key><string>${VER}</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleIconFile</key><string>vlt-syslogd</string>
    <key>LSMinimumSystemVersion</key><string>10.14</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST

  # 同梱物を全部置いた後に deep sign(ad-hoc 定石。タイムスタンプ鯖には行かない)
  codesign --sign - --deep --force --timestamp=none "$app" >/dev/null 2>&1
  # 浅い verify で判定(framework symlink の警告は TCC に無関係)
  codesign --verify "$app" && echo "    署名 OK"

  local zip="$DIST/${outbase}-v${VER}.zip"
  rm -f "$zip"
  ( cd "$DIST" && ditto -c -k --keepParent "${name}.app" "$zip" )
  echo "    → $(basename "$zip")"
}

echo "==> [1/2] App 版をビルド(データは ~/Library/Application Support)"
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"
APP_BIN="$DIST/.bin-app"; cp "$ROOT/target/release/$EXE" "$APP_BIN"

echo "==> [2/2] Portable 版をビルド(データは .app の隣)"
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml" --features portable
PORT_BIN="$DIST/.bin-portable"; cp "$ROOT/target/release/$EXE" "$PORT_BIN"

package_app "vlt-syslogd"          "com.veltrea.vlt-syslogd"          "$APP_BIN"  "vlt-syslogd-macos-app"
package_app "vlt-syslogd-portable" "com.veltrea.vlt-syslogd.portable" "$PORT_BIN" "vlt-syslogd-macos-portable"

rm -f "$APP_BIN" "$PORT_BIN"

echo
echo "完了。dist/ に2形態の .app と zip を出力しました:"
ls -1 "$DIST"/*.zip | sed 's/^/  /'
echo
echo "配布時の注意(Portable 版を USB 等で配る場合、受け取った人は初回に1回):"
echo "  xattr -dr com.apple.quarantine vlt-syslogd-portable.app"
