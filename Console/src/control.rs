//! 常駐サービス(Server 版)の制御ポートへの同期クライアント。
//!
//! サービスの制御ポート(既定 127.0.0.1:5142)へ TCP 接続し、1 行 JSON を送って
//! 1 行 JSON を受け取る(行区切り JSON / JSONL。Content-Length ヘッダーは付けない)。
//! syslog 設定の取得(`get_config`)と変更(`set_config`)に使う。
//!
//! 受信ログのストリーム(`net.rs` / 非同期・常駐)とは責務が違うため別モジュールにする。
//! こちらは「設定画面のボタンを押したときに 1 往復するだけ」なので、同期 TCP で十分。

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

/// サーバ設定の全体像。Server 側 `config::Config` と構造を一致させること。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigDto {
    pub server: ServerSection,
    pub logging: LoggingSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    /// syslog 受信アドレス(例 0.0.0.0:514)。
    pub bind_addr: String,
    /// GUI への配信アドレス(例 127.0.0.1:5141)。
    pub stream_addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSection {
    pub level: String,
    pub max_size_mb: u64,
    pub keep_files: usize,
}

// ---- レスポンス封筒 ----

#[derive(Deserialize)]
struct GetResp {
    ok: bool,
    config: Option<ServerConfigDto>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct SetResp {
    ok: bool,
    restart_required: Option<bool>,
    error: Option<String>,
}

/// 1 行送って 1 行受け取る。接続/読み書きにタイムアウトを設けてフリーズを防ぐ。
fn round_trip(control_addr: &str, request: &str) -> Result<String, String> {
    let addr = control_addr
        .parse::<std::net::SocketAddr>()
        .map_err(|e| format!("制御アドレスが不正です ({addr}): {e}", addr = control_addr, e = e))?;
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
        .map_err(|e| format!("サービスに接続できません ({control_addr}): {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;

    let mut writer = stream.try_clone().map_err(|e| e.to_string())?;
    writer
        .write_all(request.as_bytes())
        .and_then(|_| writer.write_all(b"\n"))
        .and_then(|_| writer.flush())
        .map_err(|e| format!("送信に失敗しました: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("応答の受信に失敗しました: {e}"))?;
    if line.trim().is_empty() {
        return Err("サービスから空の応答が返りました".to_string());
    }
    Ok(line)
}

/// サーバの現在の設定を取得する。
pub fn get_config(control_addr: &str) -> Result<ServerConfigDto, String> {
    let line = round_trip(control_addr, r#"{"cmd":"get_config"}"#)?;
    let resp: GetResp =
        serde_json::from_str(&line).map_err(|e| format!("応答を解釈できません: {e}"))?;
    if !resp.ok {
        return Err(resp.error.unwrap_or_else(|| "サーバがエラーを返しました".to_string()));
    }
    resp.config
        .ok_or_else(|| "応答に config が含まれていません".to_string())
}

/// サーバの設定を変更する。戻り値はサービス再起動が必要かどうか。
pub fn set_config(control_addr: &str, cfg: &ServerConfigDto) -> Result<bool, String> {
    let req = serde_json::json!({ "cmd": "set_config", "config": cfg });
    let line = round_trip(control_addr, &req.to_string())?;
    let resp: SetResp =
        serde_json::from_str(&line).map_err(|e| format!("応答を解釈できません: {e}"))?;
    if !resp.ok {
        return Err(resp.error.unwrap_or_else(|| "サーバがエラーを返しました".to_string()));
    }
    Ok(resp.restart_required.unwrap_or(false))
}
