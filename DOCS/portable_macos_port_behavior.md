# Portable 版の macOS ポート挙動（調査メモ）

Portable 版（GUI ビューア）が macOS で syslog 受信ポートをどう扱うかの調査結果と、
それを踏まえて入れた変更の記録。Server 版とは別系統のメモ。

## 背景・きっかけ

「macOS では 514 が使えないとき、自動的に 5514 が選択されるはず」という想定があったため、
実際のコード挙動と実機での bind 可否を確認した。

## 実機で確認した事実（macOS, 非 root / uid 501）

最小再現（UDP ソケットを `0.0.0.0` に bind するだけ）で計測した結果:

| bind 先          | 結果                                   |
| ---------------- | -------------------------------------- |
| `0.0.0.0:514`    | **FAIL — Permission denied (EACCES)**  |
| `0.0.0.0:5514`   | **OK**                                 |

- macOS でも **1024 未満の特権ポートは root が必要**。非 root では 514 を bind できない。
- コード内コメントにあった「macOS では 0.0.0.0 への特権ポート bind は root 不要」は
  **事実誤認**だった（今回修正済み）。

## コードが実際に行う挙動

関係するのは `Portable/src/main.rs` の `socket_manager` / `poll_bind_status` /
`show_bind_banner` と、`Portable/src/settings.rs` の既定値。

1. 既定ポートは **514**（`settings::Settings::default()`）。
2. 起動時に `try_bind(514)`（= `UdpSocket::bind("0.0.0.0:514")`）を試す。
3. **成功すれば 514 のまま待ち受ける**（5514 へは切り替わらない）。
   - 非 root の macOS では 514 は失敗するため、通常起動はこの分岐に入らない。
4. **失敗時**は `BindState::Failed` を GUI に送り、上部に失敗バナーを表示する。
   - バナーのポート入力欄には **`5514` が自動でプリフィル**される。
   - `socket_manager` は自動リトライ・自動フォールバックをせず、
     `bind_rx.recv()` でブロックして GUI からの新しいポート指定を待つ。
5. ユーザーが **「待ち受け開始」ボタンを 1 回押す**と、入力されたポート（既定で 5514）で
   bind し直す。5514 は非 root でも bind 可能。

### 「自動で 5514」は半分だけ正しい

- ✅ macOS では 514 が必ず失敗し、**5514 の候補が自動で提示される**。
- ❌ ただし **5514 で実際に待ち受けるにはボタン 1 クリックが必要**で、完全自動ではない。
  設計意図としては「手動でポートを確定させる」前提だった。

## 入れた変更

### 1. 誤ったコメント・ヒント文言の修正

`Portable/src/main.rs` の 3 箇所（失敗バナーの原因説明、`socket_manager` の OS 別コメント、
macOS 用の失敗ヒント文字列）から「macOS は特権ポート bind に root 不要」という誤記を削除し、
「macOS / Linux いずれも非 root では 514 は Permission denied」という実態に合わせた。

### 2. 復旧したポートの永続化（毎回クリックの解消）

それまで、環境設定ウィンドウ経由（`apply_preferences`）はポートを `config.toml` に保存するのに、
失敗バナーのボタン経由は保存していなかった。このため macOS では
「起動するたび 514 で失敗 → 毎回 5514 をクリック」という摩擦があった。

`poll_bind_status` で **bind 成功（`BindState::Bound`）を受信したとき、その実効ポートを
`config.toml` に保存**するようにした。経路（バナー / 環境設定）を問わず永続化される。

```rust
if let BindState::Bound(addr) = &state {
    if let Some(port) = addr.rsplit(':').next().and_then(|p| p.parse::<u16>().ok()) {
        let current = settings::load();
        if current.bind_port != port {            // 変化時だけ書く
            let cfg = settings::Settings { bind_port: port, ..current }; // log_dir は維持
            let _ = settings::save(&cfg);
            self.pref_port = port.to_string();
        }
    }
}
```

効果（macOS）:

| 起動         | Before                                   | After                                      |
| ------------ | ---------------------------------------- | ------------------------------------------ |
| 初回         | 514 失敗 → バナー → 5514 入力 → クリック | 同じ（一度きり）                           |
| **2 回目以降** | **毎回** 514 失敗 → 毎回クリック         | config の **5514 で直接 bind**、クリック不要 |

挙動は隠していない（514 失敗を黙って 5514 にすり替える「自動フォールバック」はしていない）。
514 が使える環境では従来どおり 514 が保存される。

### 3. 未使用 import の削除

`use std::env::cfg;` は未使用（ポート判定は `cfg!(target_os = ...)` マクロで import 不要）。
削除してビルド警告ゼロにした。

## 検討したが入れなかった案

- **起動時に 5514 を自動 bind（無言フォールバック）**: 反対。syslog 送信側の機器・ルータは
  送信先 514 固定のものが多く、黙って 5514 に移ると「アプリは動くのにログが来ない」状態を
  隠してしまう。やるなら「⚠ 標準の 514 ではなく 5514 で待ち受け中」という消えない
  ステータス表示とセットにすべき（未実装、要望次第）。
- **既定ポートを 5514 に変更**: 反対。514 は syslog 標準であり既定として正しい。
  5514 を既定にすると 514 送信機器を取りこぼす。

## 確実に 514 で受けたいとき

- `sudo` で Portable を起動する、または
- 常駐用途は **Server 版**（macOS では launchd LaunchDaemon = root で動くため 514 を bind 可能）を使う。

## 関連ファイル

- `Portable/src/main.rs` — `socket_manager` / `try_bind` / `poll_bind_status` / `show_bind_banner`
- `Portable/src/settings.rs` — 既定ポート 514、`config.toml` の load/save
- `DOCS/REFERENCE.md` / `DOCS/REFERENCE.ja.md` — 利用者向けの「Portable で 514 が使えないとき」節
