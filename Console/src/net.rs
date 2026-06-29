//! 常駐サービス(Server 版)への TCP 接続クライアント。
//!
//! サービスの配信ポート(既定 127.0.0.1:5141)へ接続し、流れてくる JSON Lines を
//! `SyslogMessage` にデシリアライズして GUI へ渡す。サービスが落ちている/再起動中でも
//! GUI を止めないため、切断・接続失敗時は自動で再接続を試みる。
//!
//! GUI とは 3 本のチャネルでやり取りする:
//!   - `msg_tx`  : 受信した SyslogMessage を GUI へ(マネージャ→GUI)
//!   - `state_tx`: 接続状態の変化を GUI へ(マネージャ→GUI)
//!   - `addr_rx` : 接続先アドレスの変更要求を GUI から受ける(GUI→マネージャ)

use crate::parser::SyslogMessage;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

/// 接続状態。GUI 上部のバナー表示に使う。
#[derive(Clone, Debug)]
pub enum ConnState {
    /// 接続を試みている最中。
    Connecting { addr: String },
    /// 接続確立済み(配信受信中)。
    Connected { addr: String },
    /// 未接続(接続失敗 or 切断)。GUI は再試行/接続先変更を促す。
    Disconnected { addr: String, error: String },
}

/// 再接続の待ち時間。失敗時にビジーループにならない程度に短く。
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

/// TCP クライアントの常駐ループ。GUI 起動時に 1 度だけ spawn する。
pub async fn run_client(
    initial_addr: String,
    mut addr_rx: mpsc::Receiver<String>,
    msg_tx: mpsc::Sender<SyslogMessage>,
    state_tx: mpsc::Sender<ConnState>,
) {
    let mut addr = initial_addr;

    loop {
        let _ = state_tx
            .send(ConnState::Connecting { addr: addr.clone() })
            .await;

        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                let _ = state_tx
                    .send(ConnState::Connected { addr: addr.clone() })
                    .await;

                let mut lines = BufReader::new(stream).lines();
                // 受信ループ。切断されたら抜けて再接続。
                // 接続中でもアドレス変更要求が来たら張り直す。
                loop {
                    tokio::select! {
                        line = lines.next_line() => match line {
                            Ok(Some(l)) => {
                                if let Ok(msg) = serde_json::from_str::<SyslogMessage>(&l) {
                                    let _ = msg_tx.send(msg).await;
                                }
                                // パースできない行は無視(将来の互換やノイズに強くする)。
                            }
                            // EOF(サービス側が接続を閉じた)or 読み取りエラー → 再接続へ。
                            Ok(None) | Err(_) => break,
                        },
                        maybe = addr_rx.recv() => match maybe {
                            Some(new_addr) => { addr = new_addr; break; }
                            // GUI 側のチャネルが閉じた = アプリ終了。
                            None => return,
                        },
                    }
                }
            }
            Err(e) => {
                let _ = state_tx
                    .send(ConnState::Disconnected {
                        addr: addr.clone(),
                        error: e.to_string(),
                    })
                    .await;

                // 一定時間待って再試行。待っている間に接続先変更が来たら即それで張り直す。
                tokio::select! {
                    _ = tokio::time::sleep(RECONNECT_DELAY) => {}
                    maybe = addr_rx.recv() => match maybe {
                        Some(new_addr) => addr = new_addr,
                        None => return,
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    /// ダミーの配信サーバを立て、run_client が
    ///   接続成功(Connected) → JSON 行を SyslogMessage にデシリアライズして配送
    /// まで正しく行うことを GUI 抜きで検証する。
    #[tokio::test]
    async fn connects_and_decodes_json_lines() {
        // ポート 0 で OS に空きポートを割り当てさせる。
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        // サーバ側: 1 接続受けて、サービスが送るのと同じ形の JSON 行を 2 本送る。
        // 日本語を含めて UTF-8 のラウンドトリップも確認する。
        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let l1 = r#"{"severity":"Error","timestamp":"2026-06-29T00:00:00.000","hostname":null,"tag":"myapp","content":"こんにちは syslog","raw":"00","encoding":"UTF-8"}"#;
            let l2 = r#"{"severity":"Warning","timestamp":"2026-06-29T00:00:01.000","hostname":null,"tag":"kernel","content":"link down","raw":"01","encoding":"UTF-8"}"#;
            sock.write_all(l1.as_bytes()).await.unwrap();
            sock.write_all(b"\n").await.unwrap();
            sock.write_all(l2.as_bytes()).await.unwrap();
            sock.write_all(b"\n").await.unwrap();
            // クライアントが読み終えるまで接続を保つ。
            tokio::time::sleep(Duration::from_millis(500)).await;
        });

        let (msg_tx, mut msg_rx) = mpsc::channel::<SyslogMessage>(16);
        let (state_tx, mut state_rx) = mpsc::channel::<ConnState>(16);
        let (_addr_tx, addr_rx) = mpsc::channel::<String>(16);

        tokio::spawn(run_client(addr, addr_rx, msg_tx, state_tx));

        // 最初に Connecting、続いて Connected が来るはず。
        let mut saw_connected = false;
        for _ in 0..4 {
            match tokio::time::timeout(Duration::from_secs(2), state_rx.recv()).await {
                Ok(Some(ConnState::Connected { .. })) => {
                    saw_connected = true;
                    break;
                }
                Ok(Some(_)) => continue,
                _ => break,
            }
        }
        assert!(saw_connected, "Connected 状態を受け取れなかった");

        // 1 本目: 日本語がそのまま復元され、tag/severity も正しいこと。
        let m1 = tokio::time::timeout(Duration::from_secs(2), msg_rx.recv())
            .await
            .expect("1本目の受信がタイムアウト")
            .expect("1本目が None");
        assert_eq!(m1.content, "こんにちは syslog");
        assert_eq!(m1.tag.as_deref(), Some("myapp"));
        assert!(matches!(m1.severity, crate::parser::Severity::Error));

        // 2 本目も続けて届くこと。
        let m2 = tokio::time::timeout(Duration::from_secs(2), msg_rx.recv())
            .await
            .expect("2本目の受信がタイムアウト")
            .expect("2本目が None");
        assert_eq!(m2.content, "link down");
        assert_eq!(m2.tag.as_deref(), Some("kernel"));
    }

    /// 接続先が居ない場合は Disconnected 状態を通知すること(自動再接続の前提)。
    #[tokio::test]
    async fn reports_disconnected_when_no_server() {
        // 使われていない可能性が高いアドレス(ポート 1 は通常 listen されない)。
        let addr = "127.0.0.1:1".to_string();

        let (msg_tx, _msg_rx) = mpsc::channel::<SyslogMessage>(16);
        let (state_tx, mut state_rx) = mpsc::channel::<ConnState>(16);
        let (_addr_tx, addr_rx) = mpsc::channel::<String>(16);

        tokio::spawn(run_client(addr, addr_rx, msg_tx, state_tx));

        let mut saw_disconnected = false;
        for _ in 0..4 {
            match tokio::time::timeout(Duration::from_secs(2), state_rx.recv()).await {
                Ok(Some(ConnState::Disconnected { .. })) => {
                    saw_disconnected = true;
                    break;
                }
                Ok(Some(_)) => continue,
                _ => break,
            }
        }
        assert!(saw_disconnected, "Disconnected 状態を受け取れなかった");
    }
}
