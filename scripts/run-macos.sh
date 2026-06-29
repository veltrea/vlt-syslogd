#!/usr/bin/env bash
#
# ビルド済みの vlt-syslogd（Server / Console）を macOS で起動する開発用スクリプト。
#
#   - Server  : syslog を 514 で待ち受ける常駐デーモン。特権ポートのため sudo で起動する。
#   - Console : Server に TCP 接続してログを表示する GUI フロントエンド。通常権限で起動する。
#
# 使い方:
#   bash scripts/run-macos.sh
#
# 事前に `cargo build`（または `cargo build --release`）でバイナリを用意しておくこと。
# 停止方法はスクリプト末尾のメッセージに表示する。

set -euo pipefail

# リポジトリルート（このスクリプトの 1 つ上）を基準にする。
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# release を優先し、無ければ debug を使う。
if [ -x "$ROOT/target/release/vlt-syslogd-srv" ] && [ -x "$ROOT/target/release/vlt-syslogd-console" ]; then
  BIN_DIR="$ROOT/target/release"
elif [ -x "$ROOT/target/debug/vlt-syslogd-srv" ] && [ -x "$ROOT/target/debug/vlt-syslogd-console" ]; then
  BIN_DIR="$ROOT/target/debug"
else
  echo "エラー: ビルド済みバイナリが見つかりません。" >&2
  echo "  先に次を実行してください: cargo build -p vlt-syslogd-srv -p vlt-syslogd-console" >&2
  exit 1
fi

SRV_BIN="$BIN_DIR/vlt-syslogd-srv"
CON_BIN="$BIN_DIR/vlt-syslogd-console"
SRV_LOG="/tmp/vlt-syslogd-srv.log"
CON_LOG="/tmp/vlt-syslogd-console.log"

echo "使用するバイナリ: $BIN_DIR"
echo

# ---- Server（514 は特権ポートなので sudo で起動）----
echo "Server を起動します。514 番のバインドに管理者権限が必要です。"
echo "パスワードを求められたら入力してください。"
# 先にフォアグラウンドで認証してキャッシュを取得（バックグラウンドの sudo はプロンプトが見えないため）。
sudo -v
sudo "$SRV_BIN" > "$SRV_LOG" 2>&1 &
SRV_PID=$!
echo "  Server 起動  PID=$SRV_PID  ログ=$SRV_LOG"

# Server が待ち受けを開始するまで少し待つ。
sleep 1

# ---- Console（GUI・通常権限）----
echo "Console を起動します。"
"$CON_BIN" > "$CON_LOG" 2>&1 &
CON_PID=$!
echo "  Console 起動  PID=$CON_PID  ログ=$CON_LOG"

# ターミナルから直接起動した GUI は前面化されず、ウィンドウが画面外/背面に出ることがある。
# 前面に出し、ウィンドウを見える位置（左上）へ移動する。
sleep 1
osascript <<OSA 2>/dev/null || true
tell application "System Events"
  set theProc to (first process whose unix id is $CON_PID)
  set frontmost of theProc to true
  if (count of windows of theProc) > 0 then
    set position of window 1 of theProc to {100, 100}
  end if
end tell
OSA

echo
echo "起動完了。"
echo "  停止: sudo kill $SRV_PID   # Server（管理者権限で動作中）"
echo "        kill $CON_PID        # Console"
echo "  ログ: tail -f $SRV_LOG"
echo "        tail -f $CON_LOG"
