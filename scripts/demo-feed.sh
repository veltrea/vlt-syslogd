#!/usr/bin/env bash
#
# demo-feed.sh — Feed vlt-syslogd with a curated stream of realistic syslog
# messages, designed for taking a clean screenshot of the Console UI.
#
# It deliberately mixes severities so the color-coded Severity column shows a
# pleasant spread (green / orange / red / blue), and includes a few UTF-8
# (Japanese) lines to demonstrate the encoding-detection (Enc) column.
#
# Usage:
#   ./demo-feed.sh [host] [port]
#
#   host   target address (default: 127.0.0.1)
#   port   target UDP port (default: 514)
#
#   DELAY=0.25 ./demo-feed.sh      # seconds between messages (default 0.15)
#   LOOP=1     ./demo-feed.sh      # keep looping until Ctrl-C
#
# Requires: nc (netcat, UDP mode).
#
# Wire format is RFC 3164:  <PRI>TAG: MESSAGE
#   PRI = facility * 8 + severity
#   severity: 0 Emerg 1 Alert 2 Crit 3 Err 4 Warn 5 Notice 6 Info 7 Debug
# vlt-syslogd extracts the Tag (text before the first colon) and severity,
# then renders the message in the severity color.

set -euo pipefail

HOST="${1:-127.0.0.1}"
PORT="${2:-514}"
DELAY="${DELAY:-0.15}"
LOOP="${LOOP:-0}"

if ! command -v nc >/dev/null 2>&1; then
  echo "error: 'nc' (netcat) not found in PATH" >&2
  exit 1
fi

# One entry per line: "<PRI>TAG: MESSAGE"
# Ordered so the stream reads like a live feed, mostly green with red/orange
# accents, a debug line, and two Japanese lines for the Enc column.
messages=(
  # severity 6 (Info, green)
  '<86>sshd[20451]: Accepted publickey for admin from 192.0.2.42 port 54021 ssh2'
  '<30>systemd[1]: Started Session 412 of user admin.'
  '<134>backup-agent: バックアップが正常に完了しました (45.2 GB, 経過 00:12:08)'
  # severity 5 (Notice, green)
  '<29>nginx: 192.0.2.10 - "GET /api/v1/status HTTP/1.1" 200 1534 "Mozilla/5.0"'
  '<46>CRON[4412]: (root) CMD (/usr/local/sbin/logrotate /etc/logrotate.conf)'
  # severity 4 (Warning, orange)
  '<84>sshd[20460]: Failed password for invalid user test from 203.0.113.55 port 41922 ssh2'
  '<4>kernel: [UFW BLOCK] IN=eth0 OUT= SRC=203.0.113.99 DST=198.51.100.5 PROTO=TCP DPT=23'
  '<140>監視エージェント: CPU使用率が閾値を超過しました (92%, host=web-03)'
  # severity 3 (Error, light red)
  '<11>nginx: 198.51.100.8 - "POST /login HTTP/1.1" 502 0 upstream connect timed out'
  '<19>postfix/smtpd[7781]: NOQUEUE: reject: RCPT from unknown[198.51.100.23]: 554 5.7.1'
  # severity 6 (Info, green) — keep the green flowing
  '<134>dhcpd: DHCPACK on 192.0.2.87 (client laptop-09) via eth0'
  '<30>systemd[1]: Reloaded nginx.service.'
  # severity 2 (Critical, red)
  '<2>kernel: Out of memory: Killed process 8123 (java) total-vm:4194304kB'
  # severity 1 (Alert, red)
  '<1>systemd[1]: Failed to start postgresql.service - PostgreSQL database server.'
  # severity 7 (Debug, blue)
  '<31>dockerd: level=debug msg="container health check passed" id=7f3a9c name=api'
  # severity 6 (Info, green)
  '<86>sshd[20480]: pam_unix(sshd:session): session opened for user admin by (uid=0)'
)

send_one() {
  # printf keeps it to a single UDP datagram; -w0 = no wait, return immediately.
  printf '%s' "$1" | nc -u -w0 "$HOST" "$PORT"
}

feed_once() {
  for m in "${messages[@]}"; do
    send_one "$m"
    sleep "$DELAY"
  done
}

echo "Feeding ${#messages[@]} messages to ${HOST}:${PORT} (UDP, RFC 3164)..."
if [[ "$LOOP" == "1" ]]; then
  echo "Looping — press Ctrl-C to stop."
  while true; do feed_once; done
else
  feed_once
  echo "Done."
fi
