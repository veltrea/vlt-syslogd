#!/usr/bin/env bash
# run-portable.sh - vlt-syslogd-portable.app を起動するスクリプト
#
# このスクリプトは、Gatekeeper の App Translocation (隔離属性による一時ディレクトリ移動)
# を回避しつつ、macOS アプリケーションとして安全に起動します。
#
# 使い方:
#   ./scripts/run-portable.sh
#   VLT_SYSLOGD_BIND=0.0.0.0:5514 ./scripts/run-portable.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_PATH="$ROOT_DIR/dist/vlt-syslogd-portable.app"

# 1. アプリの存在確認
if [ ! -d "$APP_PATH" ]; then
    echo "エラー: $APP_PATH が見つかりません。" >&2
    echo "先に以下のコマンドでビルドを実行してください:" >&2
    echo "  ./Portable/build-macos.sh" >&2
    exit 1
fi

# 2. Gatekeeper による App Translocation 回避のため、quarantine 属性があれば自動で外す
if xattr "$APP_PATH" 2>/dev/null | grep -q "com.apple.quarantine"; then
    echo "Gatekeeper による隔離属性 (quarantine) を解除しています..."
    xattr -dr com.apple.quarantine "$APP_PATH"
fi

# 3. アプリの起動
# 環境変数 VLT_SYSLOGD_BIND が指定されているか、引数がある場合は、
# 環境変数を確実に反映し、標準ログを確認しやすいようにバイナリを直接フォアグラウンドで起動します。
# それ以外の場合は、通常の macOS アプリケーションとして open でバックグラウンド起動します。

if [ -n "${VLT_SYSLOGD_BIND:-}" ] || [ $# -gt 0 ]; then
    echo "引数または環境変数 VLT_SYSLOGD_BIND が指定されたため、フォアグラウンドで直接バイナリを起動します。"
    if [ -n "${VLT_SYSLOGD_BIND:-}" ]; then
        echo "  VLT_SYSLOGD_BIND=$VLT_SYSLOGD_BIND"
    fi
    exec "$APP_PATH/Contents/MacOS/vlt-syslogd-portable" "$@"
else
    echo "vlt-syslogd-portable.app を起動しています..."
    open "$APP_PATH"
fi
