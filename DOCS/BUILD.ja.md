# vlt-syslogd — ビルドガイド

English version: [BUILD.md](BUILD.md)

ソースから実行ファイルを作る手順です。**出来上がったものを入れて使うだけなら、このガイドは不要です** — 配布バイナリを使い、[INSTALL.ja.md](INSTALL.ja.md) に進んでください。

---

## 1. 前提

[Rust ツールチェイン](https://rustup.rs)（`cargo`）が必要です。導入は `--profile minimal` で十分です。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal
```

---

## 2. まとめてビルド

リポジトリを展開したフォルダの**ルートで**実行します。

```bash
cargo build --release --workspace
```

`--workspace` は3つの部品すべてを一度にビルドします。成果物は `target/release/` に出ます。

- `target/release/vlt-syslogd-srv`（Server）
- `target/release/vlt-syslogd-console`（Console）
- `target/release/vlt-syslogd-portable`（Portable）

> ファイル名の `vlt-syslogd-srv` などは**クレート名**です。cargo に「どの部品を扱うか」を伝える正式名で、`-p`（package）の後ろに書きます。1つだけビルドしたいなら `cargo build --release -p vlt-syslogd-portable` のように指定します。

---

## 3. ビルドせずに直接動かす（開発時）

サービス登録せず、その場で起動して確認したいときは `cargo run` を使います。

```bash
# Portable（単体で UDP 514 を待ち受ける）
cargo run --release -p vlt-syslogd-portable

# Console（Server に接続して表示する GUI）
cargo run --release -p vlt-syslogd-console

# Server（画面なしの常駐エンジンを、サービス登録せず前面で起動）
cargo run --release -p vlt-syslogd-srv
```

`cargo run` は内部でビルドしてから起動するので、§2 を別途実行する必要はありません。

---

## 4. ビルドしたものを配布・常駐させるには

- **常駐サービスとして入れる** → [INSTALL.ja.md](INSTALL.ja.md) §3。インストールスクリプトは `target/release/` の実行ファイルを自動で見つけます。
- **macOS で配布する** → `.app` にまとめて ad-hoc 署名します（プロジェクトの署名手順を参照）。

---

## 5. プラットフォーム別の注意

- **Linux**: GUI（Console / Portable）の表示に CJK フォントが要ります。無いと日本語が □（豆腐）になります。Noto Sans CJK / IPA 等を導入してください。
- **Windows**: ターゲットによっては MSVC ツールチェイン（Visual Studio Build Tools）が必要です。
- **クロスビルド**: 各 OS 向けのバイナリは、基本的にその OS 上でビルドするのが確実です。
