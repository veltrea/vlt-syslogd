#!/usr/bin/env bash
# vlt-syslogd: macOS 用アンインストールスクリプト
# install-macos.sh で登録した常駐デーモンを停止・解除し、バイナリを削除する。
# 設定とログ(/usr/local/var/vlt-syslogd)は念のため残す。
#
# 使い方:
#   sudo ./uninstall-macos.sh
#
set -euo pipefail

LABEL="com.veltrea.vlt-syslogd-srv"   # install-macos.sh / Console(service.rs)と一致
INSTALL_BIN="/usr/local/bin/vlt-syslogd-srv"
DATA_DIR="/usr/local/var/vlt-syslogd"
PLIST="/Library/LaunchDaemons/${LABEL}.plist"

if [ "$(id -u)" -ne 0 ]; then
  echo "エラー: 管理者権限が必要です。次のように実行してください:" >&2
  echo "  sudo $0" >&2
  exit 1
fi

echo "==> デーモンを停止・解除"
launchctl bootout "system/${LABEL}" 2>/dev/null || true

echo "==> ${PLIST} を削除"
rm -f "$PLIST"

echo "==> バイナリを削除: ${INSTALL_BIN}"
rm -f "$INSTALL_BIN"

echo
echo "完了しました。"
echo "  - 設定とログは残してあります: ${DATA_DIR}"
echo "  - 不要なら手動で削除してください: sudo rm -rf ${DATA_DIR}"
