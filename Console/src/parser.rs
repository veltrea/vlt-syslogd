//! GUI フロントエンドが受信する syslog メッセージの型定義。
//!
//! このクレート自身は syslog をパースしない(パースは常駐サービス側の責務)。
//! サービス(Server 版)が JSON Lines で送ってくる `SyslogMessage` を
//! デシリアライズするだけなので、送信側(`Server/src/parser.rs`)と
//! **フィールド構成・enum 定義を必ず一致させること**。

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Emergency = 0,
    Alert = 1,
    Critical = 2,
    Error = 3,
    Warning = 4,
    Notice = 5,
    Informational = 6,
    Debug = 7,
}

impl Severity {
    /// 重大度ごとの表示色(RGB)。Portable 版ビューアと同じ配色に揃える。
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Severity::Emergency | Severity::Alert | Severity::Critical => (255, 100, 100), // Red
            Severity::Error => (255, 120, 120),   // Light Red
            Severity::Warning => (255, 200, 100), // Orange/Yellow
            Severity::Notice | Severity::Informational => (150, 255, 150), // Green
            Severity::Debug => (180, 180, 255),   // Light Blue
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyslogMessage {
    pub severity: Severity,
    pub timestamp: String,
    pub hostname: Option<String>,
    pub tag: Option<String>,
    pub content: String,
    pub raw: String,
    pub encoding: String,
}
