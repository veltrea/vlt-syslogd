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
    pub fn from_pri(pri: u8) -> Self {
        match pri % 8 {
            0 => Severity::Emergency,
            1 => Severity::Alert,
            2 => Severity::Critical,
            3 => Severity::Error,
            4 => Severity::Warning,
            5 => Severity::Notice,
            6 => Severity::Informational,
            7 => Severity::Debug,
            _ => Severity::Informational,
        }
    }

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
}

pub fn parse_syslog(raw: &str) -> SyslogMessage {
    let mut current = raw;
    let mut severity = Severity::Informational;
    let timestamp = chrono::Local::now().format("%b %d %H:%M:%S").to_string();
    let hostname = None;
    let mut tag = None;

    // PRIパース: <PRI>
    if current.starts_with('<') {
        if let Some(end) = current.find('>') {
            if let Ok(pri) = current[1..end].parse::<u8>() {
                severity = Severity::from_pri(pri);
            }
            current = &current[end + 1..];
        }
    }

    // 簡易パース: 非常に多くのバリエーションがあるため、
    // タグ形式 (TAG: CONTENT) を探す
    if let Some(colon_pos) = current.find(':') {
        let potential_tag = current[..colon_pos].trim();
        // タグにスペースが含まれない場合、タグとして扱う
        if !potential_tag.contains(' ') {
            tag = Some(potential_tag.to_string());
            current = &current[colon_pos + 1..].trim();
        }
    }

    SyslogMessage {
        severity,
        timestamp,
        hostname,
        tag,
        content: current.to_string(),
        raw: raw.to_string(),
    }
}
