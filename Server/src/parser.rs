use encoding_rs::{Encoding, UTF_8};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Emergency = 0, Alert = 1, Critical = 2, Error = 3,
    Warning = 4, Notice = 5, Informational = 6, Debug = 7,
}

impl Severity {
    pub fn from_pri(pri: u8) -> Self {
        match pri % 8 {
            0 => Severity::Emergency, 1 => Severity::Alert, 2 => Severity::Critical, 3 => Severity::Error,
            4 => Severity::Warning, 5 => Severity::Notice, 6 => Severity::Informational, 7 => Severity::Debug,
            _ => Severity::Informational,
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

pub fn parse_syslog(bytes: &[u8]) -> SyslogMessage {
    let mut cursor = 0;
    let mut severity = Severity::Informational;
    let mut tag: Option<String> = None;
    let timestamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string();

    if bytes.starts_with(b"<") {
        if let Some(pos) = bytes.iter().position(|&b| b == b'>') {
            if let Ok(pri_str) = std::str::from_utf8(&bytes[1..pos]) {
                if let Ok(pri) = pri_str.parse::<u8>() { severity = Severity::from_pri(pri); }
            }
            cursor = pos + 1;
        }
    }

    let is_rfc5424 = if cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        if let Some(space_pos) = bytes[cursor..].iter().position(|&b| b == b' ') {
            cursor += space_pos + 1;
            true
        } else { false }
    } else { false };

    let mut detected_encoding = None;

    if is_rfc5424 {
        for _ in 0..5 {
            if let Some(space_pos) = bytes[cursor..].iter().position(|&b| b == b' ') { cursor += space_pos + 1; }
            else { break; }
        }
        if cursor < bytes.len() && bytes[cursor] == b'[' {
            if let Some(sd_end) = find_sd_end(&bytes[cursor..]) {
                let sd_bytes = &bytes[cursor..cursor + sd_end];
                if let Ok(sd_str) = std::str::from_utf8(sd_bytes) {
                    if let Some(charset_start) = sd_str.find("charset=\"") {
                        let inner = &sd_str[charset_start + 9..];
                        if let Some(quote_end) = inner.find('"') {
                            let charset_name = &inner[..quote_end];
                            if charset_name.to_uppercase() == "MSG-UTF8" { detected_encoding = Some(UTF_8); }
                            else { detected_encoding = Encoding::for_label(charset_name.as_bytes()); }
                        }
                    }
                }
                cursor += sd_end;
                if cursor < bytes.len() && bytes[cursor] == b' ' { cursor += 1; }
            }
        }
    }

    let msg_bytes = &bytes[cursor..];
    let (content, encoding_name) = if !is_rfc5424 {
        decode_smart(msg_bytes)
    } else {
        if let Some(enc) = detected_encoding {
            let is_bom = msg_bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
            let actual_payload = if is_bom { &msg_bytes[3..] } else { msg_bytes };
            let (result, _, _) = enc.decode(actual_payload);
            (result.into_owned(), format!("{} (SD/{})", enc.name(), if is_bom { "BOM" } else { "NoBOM" }))
        } else if msg_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            let (result, _, _) = UTF_8.decode(&msg_bytes[3..]);
            (result.into_owned(), "UTF-8 (BOM)".to_string())
        } else {
            if std::str::from_utf8(msg_bytes).is_ok() { (String::from_utf8_lossy(msg_bytes).into_owned(), "UTF-8".to_string()) }
            else { decode_smart(msg_bytes) }
        }
    };

    let final_content = if !is_rfc5424 {
        if let Some(colon_pos) = content.find(':') {
            let potential_tag = content[..colon_pos].trim();
            if !potential_tag.contains(' ') && !potential_tag.is_empty() {
                tag = Some(potential_tag.to_string());
                content[colon_pos + 1..].trim().to_string()
            } else { content }
        } else { content }
    } else { content };

    SyslogMessage { severity, timestamp, hostname: None, tag, content: final_content, raw: hex::encode(bytes), encoding: encoding_name }
}

fn find_sd_end(bytes: &[u8]) -> Option<usize> {
    let mut depth = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'[' { depth += 1; } else if b == b']' { depth -= 1; if depth == 0 { return Some(i + 1); } }
    }
    None
}

fn decode_smart(bytes: &[u8]) -> (String, String) {
    if bytes.is_empty() { return (String::new(), "Empty".to_string()); }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let (res, _, _) = UTF_8.decode(&bytes[3..]);
        return (res.into_owned(), "UTF-8 (BOM)".to_string());
    }
    if let Ok(s) = std::str::from_utf8(bytes) { return (s.to_string(), "UTF-8".to_string()); }
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let enc = detector.guess(None, true);
    let (res, _, _) = enc.decode(bytes);
    (res.into_owned(), enc.name().to_string())
}
