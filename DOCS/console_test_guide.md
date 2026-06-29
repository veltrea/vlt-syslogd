# vlt-syslogd Console テスト手順書

- 対象: `Console`（GUI フロントエンド）クレート
- 目的: Console が「Server に TCP 接続して syslog を画面表示」「サービス状態の表示・操作」「syslog 設定の取得・変更」を正しく行えることを、再現可能な手順で確認する。
- 関連: [console_design.md](console_design.md)（設計）/ [console_status_report.md](console_status_report.md)（実装経緯）

通信の全体像（テスト対象）:

```
[syslog機器/送信] --UDP--> [Server] --parse/JSON--> broadcast
                                          | TCP:5141 (JSON Lines, 一方向)
                                          v
                              [Console] 画面表示
                                          | TCP:5142 (JSONL get_config/set_config)
                                          v
                              [Server] config.toml 書換え → 再起動で反映
```

---

## 0. 前提

- Rust ツールチェイン（`cargo`）。リポジトリルートで作業する。
- macOS / Linux / Windows のいずれか。本書のコマンド例は macOS / Linux 前提（Windows は §7 を参照）。
- 結合テストで使うポート: `5514`(テスト用 syslog 受信) / `5141`(配信) / `5142`(制御)。既に使用中でないこと。

```bash
cd /Volumes/2TB_USB/dev/vlt-syslogd-private   # リポジトリルート
```

---

## 1. ビルド & 自動テスト（最初に必ず通す）

```bash
# ワークスペース全体が警告なくチェックできること
cargo check --workspace

# Console のユニットテスト（net.rs: 接続・JSON 復元・切断検知）
cargo test -p vlt-syslogd-console
```

**期待結果**:
- `cargo check --workspace` … `Finished` で終わり、警告ゼロ。
- `cargo test -p vlt-syslogd-console` … 以下 2 件が `ok`。
  - `net::tests::connects_and_decodes_json_lines`（ダミー配信サーバから JSON Lines を受け取り `SyslogMessage` に復元、日本語 UTF-8 も復元）
  - `net::tests::reports_disconnected_when_no_server`（接続先不在で `Disconnected` 通知）

```
test result: ok. 2 passed; 0 failed; ...
```

---

## 2. 結合テスト用に Server を起動する

Server のデータ置き場は既定で `/usr/local/var/vlt-syslogd`（要権限）だが、環境変数 `VLT_SYSLOGD_DATA_DIR` で上書きできる。テストでは権限不要の作業ディレクトリと高ポートを使う。

```bash
# 1) 作業用データディレクトリと設定ファイルを用意
export DD="$(mktemp -d)"
cat > "$DD/config.toml" <<'EOF'
[server]
bind_addr = "127.0.0.1:5514"
stream_addr = "127.0.0.1:5141"
control_addr = "127.0.0.1:5142"

[logging]
level = "info"
max_size_mb = 10
keep_files = 7
EOF

# 2) Server をビルドしてフォアグラウンド常駐モード(run)で起動（バックグラウンド実行）
cargo build -p vlt-syslogd-srv
VLT_SYSLOGD_DATA_DIR="$DD" ./target/debug/vlt-syslogd-srv run > "$DD/srv.log" 2>&1 &
echo $! > "$DD/srv.pid"
sleep 2

# 3) 配信(5141)・制御(5142)が listen していることを確認
lsof -nP -iTCP -sTCP:LISTEN | grep -E "5141|5142"
```

**期待結果**: `127.0.0.1:5141 (LISTEN)` と `127.0.0.1:5142 (LISTEN)` の 2 行が出る。

> 注: `bind_addr` を `0.0.0.0:514`（標準ポート）にする場合、Linux では root / `CAP_NET_BIND_SERVICE` が必要。macOS は 0.0.0.0 への特権ポート bind に root 不要。テストでは高ポート `5514` を使うので権限不要。

---

## 3. 制御プロトコルの検証（get_config / set_config）

GUI を起動せずに、制御ポート(5142)の JSONL プロトコルを独立クライアントで検証する。Console の `control.rs` が送るのと同じ「1 行 JSON 送信 → 1 行 JSON 受信」を再現する。

### 3-A. Python での一括検証（推奨）

```bash
python3 - "$DD" <<'PY'
import socket, json, time, sys
DD=sys.argv[1]
def ctl(req):
    s=socket.create_connection(("127.0.0.1",5142),timeout=3)
    s.sendall((json.dumps(req)+"\n").encode())
    line=s.makefile("r").readline(); s.close()
    return line.strip()

# 1) get_config: 現在設定が返る
r=ctl({"cmd":"get_config"}); print("get_config:",r)
cfg=json.loads(r)["config"]
assert cfg["server"]["control_addr"]=="127.0.0.1:5142"

# 2) set_config: 値を変えて保存。restart_required が返る
cfg["logging"]["keep_files"]=99
r=ctl({"cmd":"set_config","config":cfg}); print("set_config:",r)
resp=json.loads(r); assert resp["ok"] and resp["restart_required"]
time.sleep(0.3)
assert "keep_files = 99" in open(DD+"/config.toml").read()

# 3) 異常系
print("unknown:", ctl({"cmd":"bogus"}))
s=socket.create_connection(("127.0.0.1",5142),timeout=3); s.sendall(b"not json\n")
print("badjson:", s.makefile("r").readline().strip()); s.close()
print("CONTROL OK")
PY
```

**期待結果**:
- `get_config` … `{"config":{...},"ok":true}`（`server.bind_addr/stream_addr/control_addr` と `logging.level/max_size_mb/keep_files` を含む）。
- `set_config` … `{"ok":true,"restart_required":true}`。直後に `config.toml` へ `keep_files = 99` が永続化されている。
- 異常系 … unknown cmd は `{"ok":false,"error":"unknown cmd: ..."}`、不正 JSON は `{"ok":false,"error":"invalid json: ..."}`。
- 最後に `CONTROL OK` が出る。

### 3-B. nc（netcat）での単発確認（任意）

```bash
printf '{"cmd":"get_config"}\n' | nc 127.0.0.1 5142
```
→ 1 行の JSON（`...,"ok":true}`）が返れば OK。

---

## 4. ストリーム配信の検証（UDP → TCP 5141）

Server に syslog を 1 通投げ、配信ポート(5141)へ JSON Lines が流れることを確認する。Console の受信経路（`net.rs::run_client`）と同じものをクライアント側で再現する。

```bash
python3 <<'PY'
import socket, time, json
# 配信ポートに先に接続しておく
st=socket.create_connection(("127.0.0.1",5141),timeout=3); time.sleep(0.3)
# UDP で syslog を 1 通送る（PRI=34, 日本語入り）
u=socket.socket(socket.AF_INET,socket.SOCK_DGRAM)
u.sendto("<34>Oct 11 22:14:15 myhost myapp: 日本語 test".encode(),("127.0.0.1",5514)); u.close()
st.settimeout(3)
line=st.makefile("r").readline().strip(); st.close()
print("stream:", line)
m=json.loads(line)
assert "日本語" in m["content"]
print("STREAM OK severity=%s" % m["severity"])
PY
```

**期待結果**: `{"severity":...,"timestamp":...,"content":"... 日本語 test",...}` が 1 行流れ、`日本語` が UTF-8 で正しく復元され、`STREAM OK` が出る。
（PRI=34 は facility=4/severity=2 → `Critical` と判定される。tag 抽出は Server 側パーサの仕様に依存し `null` のことがある。これは Console 側の問題ではない。）

---

## 5. GUI 手動テスト（実機での目視確認）

> ⚠ 以下は**画面を目視する必要がある**項目。自動化やビルド成功では代替できない。表示環境のある実機で行うこと。

§2 で Server を起動した状態のまま、別ターミナルで Console を起動する。Console の接続先・制御先を Server に合わせるため、初回は環境設定で `127.0.0.1:5141` / `127.0.0.1:5142` を指定する（既定値どおりなら変更不要）。

```bash
cargo run -p vlt-syslogd-console
```

### チェックリスト

| # | 操作 | 期待する見た目・挙動 |
|---|---|---|
| 1 | 起動直後 | ウィンドウタイトル「vlt-syslogd Console」。ヘッダ右に接続状態。Server 起動済みなら緑「● 受信中 (127.0.0.1:5141)」。 |
| 2 | §4 の UDP 送信を実行 | ログ表 に 1 行追加され、Severity に応じて色が付く。**日本語が文字化けしない**（□ にならない）。 |
| 3 | 大量送信して Auto-scroll | チェック ON で最下部に追従、OFF でスクロール位置が固定。 |
| 4 | Filter 欄に文字入力 | content / tag / hostname に部分一致する行だけ表示。「x」で解除。 |
| 5 | 「🗑 Clear」 | 表示中のログが消える。 |
| 6 | Server を停止（§6 の kill） | 数秒で赤「○ 切断」になり、切断バナーが出る。バナーの「再接続」欄が使える。 |
| 7 | Server を再度起動 | 自動再接続し、緑「● 受信中」に戻る（手動操作不要）。 |
| 8 | ヘッダの「サービス: …」表示 | OS のサービス状態（🟢稼働中/⚪停止中/❌未インストール/❓不明）。未インストール環境では「❌ 未インストール」。 |
| 9 | 「⚙ 設定」→ 環境設定 | 「接続設定」と「サーバ設定 (syslog)」の 2 セクションが出る。 |
| 10 | サーバ設定の「現在値を取得」 | 制御ポート経由で bind_addr/stream_addr/ログレベル/最大サイズ/保持数が埋まる。失敗時は赤字メッセージ。 |
| 11 | 値を変更して「サーバへ適用(再起動)」 | `set_config` 後にサービス再起動を試みる。サービス未インストール時は「再起動に失敗」と赤字（= 設定保存自体は成功、これは想定挙動）。 |
| 12 | 編集メニュー（コピー/ペースト等） | macOS は画面最上部メニュー、Windows/Linux はアプリ内メニューバーから操作でき、テキスト欄に効く。 |
| 13 | ログ行を右クリック | 「Copy Message」「Copy as Hex」が出てクリップボードへコピーできる。 |

> サービス操作（開始/停止/再起動）の実挙動は、Server を**実際に launchd/systemd/Windows サービスとして登録**している環境でのみ完全確認できる。未登録の開発環境では「未インストール」表示・操作失敗が正常。

---

## 6. 後片付け

```bash
kill "$(cat "$DD/srv.pid")" 2>/dev/null      # テスト用 Server を停止
lsof -nP -iTCP -sTCP:LISTEN | grep -E "5141|5142" || echo "ポート解放OK"
rm -rf "$DD"                                  # 一時データを削除
```

---

## 7. OS 別の補足

- **macOS**: `cargo run -p vlt-syslogd-console` で起動可。配布時は ad-hoc 署名した `.app` を使う（CLAUDE.md の署名手順参照）。0.0.0.0 への特権ポート bind は root 不要。
- **Linux**: Noto CJK / IPA フォントが入っていないと日本語が □ になる（`load_cjk_font` の候補参照）。標準ポート 514 を使うなら root か `CAP_NET_BIND_SERVICE`。サービス状態は `systemctl` で判定（unit 名 `vlt-syslogd-srv.service`）。
- **Windows**: ビルドは `cargo build -p vlt-syslogd-console`。サービス状態は `sc query vlt-syslogd-srv`。`set_config` 適用時のサービス再起動は UAC 昇格（PowerShell `Start-Process -Verb RunAs`）が走る。日本語コンソール出力の文字化けに注意（cmd は CP932）。
- リモート VM（ubuntu/alma/debian/work1 等）でのクロス確認は `mssh`/`wake` ラッパー経由で行う。

---

## 8. トラブルシュート

| 症状 | 原因・対処 |
|---|---|
| `cargo test` が `Address already in use` | 5141/5142/5514 を別プロセスが使用中。§6 で停止、またはポートを変える。 |
| Console が常に「○ 切断」 | Server 未起動 / 配信アドレス不一致。環境設定の「配信アドレス」を Server の `stream_addr` に合わせる。 |
| サーバ設定の「現在値を取得」が失敗 | 制御アドレス不一致 / Server が古い（control ポート未対応のビルド）。`control_addr` を確認し Server を再ビルド。 |
| 日本語が □（豆腐） | CJK フォント未検出。Linux は Noto CJK 等を導入。 |
| 「適用(再起動)」で再起動失敗 | サービス未登録の開発環境では正常（設定保存は成功）。実運用は launchd/systemd/Windows サービス登録後に確認。 |
