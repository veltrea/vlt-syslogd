#!/usr/bin/env bash
# vlt-syslogd: macOS 用インストールスクリプト
#
# サーバーエンジン(vlt-syslogd-srv)を root の常駐デーモンとして登録し、
# 標準ポート 514(UDP)で、Mac の起動時から自動で待ち受けるようにする。
# 管理者パスワードを聞かれるのはこのインストール時の 1 回だけ。
# 以後はユーザーが何かを起動する必要はない(常に動いている)。
#
# 使い方:
#   sudo ./install-macos.sh [/path/to/vlt-syslogd-srv]
#
set -euo pipefail

LABEL="com.veltrea.vlt-syslogd-srv"   # Console(service.rs)の launchd ラベルと一致させること
BIN_NAME="vlt-syslogd-srv"
INSTALL_BIN="/usr/local/bin/${BIN_NAME}"
DATA_DIR="/usr/local/var/vlt-syslogd"   # config.toml と logs の置き場(= 作業フォルダ)
PLIST="/Library/LaunchDaemons/${LABEL}.plist"

# 514 番(1024 未満)を開く + /Library/LaunchDaemons へ書き込むため root が要る
if [ "$(id -u)" -ne 0 ]; then
  echo "エラー: 管理者権限が必要です。次のように実行してください:" >&2
  echo "  sudo $0" >&2
  exit 1
fi

# インストールするバイナリを決める(優先順):
#   1) 引数で渡されたパス
#   2) このスクリプトと同じフォルダの vlt-syslogd-srv
#   3) release ビルド(./target/release/vlt-syslogd-srv)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ "${1:-}" != "" ]; then
  SRC_BIN="$1"
elif [ -x "${SCRIPT_DIR}/${BIN_NAME}" ]; then
  SRC_BIN="${SCRIPT_DIR}/${BIN_NAME}"
elif [ -x "${SCRIPT_DIR}/../target/release/${BIN_NAME}" ]; then
  # ワークスペースの release ビルド(Server/ の 1 つ上の target/)
  SRC_BIN="${SCRIPT_DIR}/../target/release/${BIN_NAME}"
elif [ -x "${SCRIPT_DIR}/target/release/${BIN_NAME}" ]; then
  SRC_BIN="${SCRIPT_DIR}/target/release/${BIN_NAME}"
else
  echo "エラー: ${BIN_NAME} が見つかりません。" >&2
  echo "  先に 'cargo build --release' でビルドするか、バイナリのパスを引数で渡してください:" >&2
  echo "  sudo $0 /path/to/${BIN_NAME}" >&2
  exit 1
fi

echo "==> バイナリを配置: ${INSTALL_BIN}"
install -d -m 755 "$(dirname "$INSTALL_BIN")"
install -m 755 "$SRC_BIN" "$INSTALL_BIN"
# ad-hoc 署名/ダウンロード由来の quarantine 属性があれば外す
xattr -d com.apple.quarantine "$INSTALL_BIN" 2>/dev/null || true

echo "==> データ/ログ用フォルダを作成: ${DATA_DIR}"
install -d -m 755 "$DATA_DIR"
install -d -m 755 "${DATA_DIR}/logs"

echo "==> LaunchDaemon を作成: ${PLIST}"
cat > "$PLIST" <<PLIST_EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>${INSTALL_BIN}</string>
        <string>run</string>
    </array>
    <!-- config.toml と logs はカレントフォルダ基準なので作業フォルダを固定する -->
    <key>WorkingDirectory</key>
    <string>${DATA_DIR}</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>${DATA_DIR}/logs/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>${DATA_DIR}/logs/stderr.log</string>
</dict>
</plist>
PLIST_EOF
chown root:wheel "$PLIST"
chmod 644 "$PLIST"

echo "==> デーモンを登録・起動"
# 既に読み込まれていれば一旦外してから入れ直す
launchctl bootout "system/${LABEL}" 2>/dev/null || true
launchctl bootstrap system "$PLIST"
launchctl enable "system/${LABEL}" 2>/dev/null || true

echo
echo "完了しました。"
echo "  - 標準ポート 514(UDP)で待ち受ける常駐デーモンとして登録しました(再起動後も自動で動きます)。"
echo "  - 設定ファイル: ${DATA_DIR}/config.toml (初回起動時に自動生成。ポート等を変えたいときに編集)"
echo "  - ログ: ${DATA_DIR}/logs/"
echo "  - 状態確認: sudo launchctl print system/${LABEL}"
echo "  - アンインストール: sudo ./uninstall-macos.sh"
