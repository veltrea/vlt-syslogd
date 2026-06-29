#!/usr/bin/env bash
# vlt-syslogd: Linux 用アンインストールスクリプト(systemd)
# install-linux.sh で登録した常駐サービスを停止・解除し、バイナリを削除する。
# 設定とログ(/var/lib/vlt-syslogd)は念のため残す。
#
# 使い方:
#   sudo ./uninstall-linux.sh
#
set -euo pipefail

UNIT_NAME="vlt-syslogd-srv.service"
INSTALL_BIN="/usr/local/bin/vlt-syslogd-srv"
DATA_DIR="/var/lib/vlt-syslogd"
UNIT_PATH="/etc/systemd/system/${UNIT_NAME}"

if [ "$(id -u)" -ne 0 ]; then
  echo "エラー: 管理者権限が必要です。次のように実行してください:" >&2
  echo "  sudo $0" >&2
  exit 1
fi

echo "==> サービスを停止・解除"
systemctl stop "${UNIT_NAME}" 2>/dev/null || true
systemctl disable "${UNIT_NAME}" 2>/dev/null || true

echo "==> ${UNIT_PATH} を削除"
rm -f "$UNIT_PATH"
systemctl daemon-reload

echo "==> バイナリを削除: ${INSTALL_BIN}"
rm -f "$INSTALL_BIN"

echo
echo "完了しました。"
echo "  - 設定とログは残してあります: ${DATA_DIR}"
echo "  - 不要なら手動で削除してください: sudo rm -rf ${DATA_DIR}"
