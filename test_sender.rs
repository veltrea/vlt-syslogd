use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to address");
    let target = "127.0.0.1:514";

    let messages = [
        "<14>Jan 29 10:50:00 test-host tag: 🟢 システム正常稼働中。UTF-8 日本語テスト。",
        "<11>Jan 29 10:50:01 test-host tag: 🔴 エラー発生！データベース接続に失敗しました。",
        "<12>Jan 29 10:50:02 test-host tag: ⚠️ 警告：メモリ消費量が 80% を超えました。",
        "<13>Jan 29 10:50:03 test-host tag: ℹ️ 情報：バックアップ処理が完了しました（成功）。",
        "<15>Jan 29 10:50:04 test-host tag: 🦀 Rust からの Syslog メッセージです。こんにちは！",
    ];

    println!(
        "Starting stress test: Sending 1000 messages to {}...",
        target
    );

    for i in 0..1000 {
        let msg = messages[i % messages.len()];
        let full_msg = format!("{} [No. {}]", msg, i);
        let _ = socket.send_to(full_msg.as_bytes(), target);

        if i % 100 == 0 {
            println!("Sent {} messages...", i);
            thread::sleep(Duration::from_millis(10)); // 少しだけ待機してバーストを調整
        }
    }

    println!("Stress test completed.");
}
