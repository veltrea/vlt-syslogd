#!/usr/bin/env bash
# vlt-syslogd: Linux 用インストールスクリプト(systemd)
#
# サーバーエンジン(vlt-syslogd-srv)を systemd の常駐サービスとして登録し、
# 標準ポート 514(UDP)で、OS の起動時から自動で待ち受けるようにする。
# 管理者(root)権限が要るのはこのインストール時だけ。以後は常に動いている。
#
# 使い方:
#   sudo ./install-linux.sh [/path/to/vlt-syslogd-srv]
#
set -euo pipefail

UNIT_NAME="vlt-syslogd-srv.service"      # Console(service.rs)の LINUX_UNIT と一致させること
BIN_NAME="vlt-syslogd-srv"
INSTALL_BIN="/usr/local/bin/${BIN_NAME}"
DATA_DIR="/var/lib/vlt-syslogd"          # platform.rs の Linux data_dir() と一致(config.toml と logs の置き場)
UNIT_PATH="/etc/systemd/system/${UNIT_NAME}"

# 514 番(1024 未満)を開く + /etc/systemd へ書き込むため root が要る
if [ "$(id -u)" -ne 0 ]; then
  echo "エラー: 管理者権限が必要です。次のように実行してください:" >&2
  echo "  sudo $0" >&2
  exit 1
fi

if ! command -v systemctl >/dev/null 2>&1; then
  echo "エラー: systemctl が見つかりません(systemd 環境が必要です)。" >&2
  exit 1
fi

# インストールするバイナリを決める(優先順):
#   1) 引数で渡されたパス
#   2) このスクリプトと同じフォルダの vlt-syslogd-srv
#   3) ワークスペースの release ビルド(../target/release)
#   4) Server 単体の release ビルド(./target/release)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ "${1:-}" != "" ]; then
  SRC_BIN="$1"
elif [ -x "${SCRIPT_DIR}/${BIN_NAME}" ]; then
  SRC_BIN="${SCRIPT_DIR}/${BIN_NAME}"
elif [ -x "${SCRIPT_DIR}/../target/release/${BIN_NAME}" ]; then
  SRC_BIN="${SCRIPT_DIR}/../target/release/${BIN_NAME}"
elif [ -x "${SCRIPT_DIR}/target/release/${BIN_NAME}" ]; then
  SRC_BIN="${SCRIPT_DIR}/target/release/${BIN_NAME}"
else
  echo "エラー: ${BIN_NAME} が見つかりません。" >&2
  echo "  先に 'cargo build --release -p vlt-syslogd-srv' でビルドするか、パスを引数で渡してください:" >&2
  echo "  sudo $0 /path/to/${BIN_NAME}" >&2
  exit 1
fi

echo "==> バイナリを配置: ${INSTALL_BIN}"
install -d -m 755 "$(dirname "$INSTALL_BIN")"
install -m 755 "$SRC_BIN" "$INSTALL_BIN"

echo "==> データ/ログ用フォルダを作成: ${DATA_DIR}"
install -d -m 755 "$DATA_DIR"
install -d -m 755 "${DATA_DIR}/logs"

echo "==> systemd ユニットを作成: ${UNIT_PATH}"
cat > "$UNIT_PATH" <<UNIT_EOF
[Unit]
Description=vlt-syslogd syslog server (vlt-syslogd-srv)
Documentation=https://github.com/veltrea/vlt-syslogd
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
# config.toml と logs はデータフォルダ基準なので作業フォルダを固定する
WorkingDirectory=${DATA_DIR}
ExecStart=${INSTALL_BIN} run
Restart=on-failure
RestartSec=2
# root 実行(標準ポート 514 を開くため)。非 root で動かしたい場合は下記コメントを参照。
#   User=vlt-syslogd
#   AmbientCapabilities=CAP_NET_BIND_SERVICE
#   (専用ユーザー作成: useradd --system --no-create-home --shell /usr/sbin/nologin vlt-syslogd
#    と ${DATA_DIR} の chown が必要)

[Install]
WantedBy=multi-user.target
UNIT_EOF
chmod 644 "$UNIT_PATH"

echo "==> サービスを登録・起動"
systemctl daemon-reload
systemctl enable "${UNIT_NAME}"
systemctl restart "${UNIT_NAME}"

echo
echo "完了しました。"
echo "  - 標準ポート 514(UDP)で待ち受ける常駐サービスとして登録しました(再起動後も自動で動きます)。"
echo "  - 設定ファイル: ${DATA_DIR}/config.toml (初回起動時に自動生成。ポート等を変えたいときに編集)"
echo "  - ログ: ${DATA_DIR}/logs/"
echo "  - 状態確認: systemctl status ${UNIT_NAME}"
echo "  - ログ確認: journalctl -u ${UNIT_NAME} -f"
echo "  - アンインストール: sudo ./uninstall-linux.sh"
