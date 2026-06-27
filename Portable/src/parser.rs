use encoding_rs::{Encoding, UTF_8};

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
    pub encoding: String, // 判定されたエンコード
}

/// RFC 5424 準拠パース (SD/charset 対応)
pub fn parse_syslog(bytes: &[u8]) -> SyslogMessage {
    let mut cursor = 0;
    let mut severity = Severity::Informational;
    let hostname: Option<String> = None;
    let mut tag: Option<String> = None;
    let timestamp = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    // 1. PRI パース (<PRI>)
    if bytes.starts_with(b"<") {
        if let Some(pos) = bytes.iter().position(|&b| b == b'>') {
            if let Ok(pri_str) = std::str::from_utf8(&bytes[1..pos]) {
                if let Ok(pri) = pri_str.parse::<u8>() {
                    severity = Severity::from_pri(pri);
                }
            }
            cursor = pos + 1;
        }
    }

    // 2. RFC 5424 VERSION チェック
    let is_rfc5424 = if cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        if let Some(space_pos) = bytes[cursor..].iter().position(|&b| b == b' ') {
            cursor += space_pos + 1;
            true
        } else {
            false
        }
    } else {
        false
    };

    let mut detected_encoding = None;

    if is_rfc5424 {
        // TIMESTAMP, HOSTNAME, APP-NAME, PROCID, MSGID をスキップ/パース (ASCII前提)
        // ここでは簡易的にスペース区切りで SD 開始位置まで飛ばす
        for _ in 0..5 {
            if let Some(space_pos) = bytes[cursor..].iter().position(|&b| b == b' ') {
                cursor += space_pos + 1;
            } else {
                break;
            }
        }

        // 3. STRUCTURED-DATA (SD) パース
        if cursor < bytes.len() && bytes[cursor] == b'[' {
            if let Some(sd_end) = find_sd_end(&bytes[cursor..]) {
                let sd_bytes = &bytes[cursor..cursor + sd_end];
                // charset="xxx" を探す (ASCII)
                if let Ok(sd_str) = std::str::from_utf8(sd_bytes) {
                    if let Some(charset_start) = sd_str.find("charset=\"") {
                        let inner = &sd_str[charset_start + 9..];
                        if let Some(quote_end) = inner.find('"') {
                            let charset_name = &inner[..quote_end];
                            // 特殊対応: "MSG-UTF8" というラベルを UTF-8 として扱う
                            if charset_name.to_uppercase() == "MSG-UTF8" {
                                detected_encoding = Some(UTF_8);
                            } else {
                                detected_encoding = Encoding::for_label(charset_name.as_bytes());
                            }
                        }
                    }
                }
                cursor += sd_end;
                // SDの後のスペースをスキップ
                if cursor < bytes.len() && bytes[cursor] == b' ' {
                    cursor += 1;
                }
            } else if bytes[cursor..].starts_with(b"- ") {
                cursor += 2;
            }
        } else if bytes[cursor..].starts_with(b"- ") {
            cursor += 2;
        }
    }

    // 4. MSG デコード
    let msg_bytes = &bytes[cursor..];
    let (content, encoding_name) = if !is_rfc5424 {
        // RFC 3164 等は従来通りのスマート判定
        decode_smart(msg_bytes)
    } else {
        /*
           RFC 5424 / 6.4. Message (現実の実装への配慮)
           SD での宣言を最優先し、BOM は「あればスキップする」程度の寛容な扱いにします。
           (規格上 BOM が必須とされるケースでも、それがない「雑な」メッセージを救済します)
        */

        if let Some(enc) = detected_encoding {
            // A. ヘッダーに「意志」がある場合: それを全面的に信じる。
            //    BOM は「UTF-8 であることを示すための付属物」として、あれば外す。
            let is_bom = msg_bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
            let actual_payload = if is_bom { &msg_bytes[3..] } else { msg_bytes };
            let (result, _, _) = enc.decode(actual_payload);

            let label = if is_bom {
                "BOM-Detected"
            } else {
                "BOM-Missing"
            };
            (
                result.into_owned(),
                format!("{} (MSG-SD/{})", enc.name(), label),
            )
        } else if msg_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            // B. ヘッダーが沈黙しているが、BOM だけはある場合: 素直に UTF-8 として扱う。
            let (result, _, _) = UTF_8.decode(&msg_bytes[3..]);
            (result.into_owned(), "UTF-8 (MSG-UTF8/BOM)".to_string())
        } else {
            // C. 何のヒントもない場合: 実態と推測に頼る。
            if std::str::from_utf8(msg_bytes).is_ok() {
                (
                    String::from_utf8_lossy(msg_bytes).into_owned(),
                    "UTF-8 (Implicit)".to_string(),
                )
            } else {
                let (text, enc_name) = decode_smart(msg_bytes);
                (text, format!("{} (Guess)", enc_name))
            }
        }
    };

    // レガシーな TAG パース (RFC 3164 的なやつ)
    let final_content = if !is_rfc5424 {
        if let Some(colon_pos) = content.find(':') {
            let potential_tag = content[..colon_pos].trim();
            if !potential_tag.contains(' ') && !potential_tag.is_empty() {
                tag = Some(potential_tag.to_string());
                content[colon_pos + 1..].trim().to_string()
            } else {
                content
            }
        } else {
            content
        }
    } else {
        content
    };

    SyslogMessage {
        severity,
        timestamp,
        hostname,
        tag,
        content: final_content,
        raw: hex::encode(bytes),
        encoding: encoding_name,
    }
}

fn find_sd_end(bytes: &[u8]) -> Option<usize> {
    let mut depth = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'[' {
            depth += 1;
        } else if b == b']' {
            depth -= 1;
            if depth == 0 {
                return Some(i + 1);
            }
        }
    }
    None
}

fn decode_smart(bytes: &[u8]) -> (String, String) {
    if bytes.is_empty() {
        return (String::new(), "Empty".to_string());
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let (res, _, _) = UTF_8.decode(&bytes[3..]);
        return (res.into_owned(), "UTF-8 (BOM)".to_string());
    }
    if let Ok(s) = std::str::from_utf8(bytes) {
        return (s.to_string(), "UTF-8".to_string());
    }
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let enc = detector.guess(None, true);
    let (res, _, _) = enc.decode(bytes);
    (res.into_owned(), enc.name().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc5424_with_charset() {
        // <13>1 - - - - - [meta charset="Shift_JIS"] (SjisBytes)
        let header = b"<13>1 - - - - - [meta charset=\"Shift_JIS\"] ";
        let sjis_body = [0x82, 0xB1, 0x82, 0xF1, 0x82, 0xC9, 0x82, 0xBF, 0x82, 0xCD];
        let mut full = header.to_vec();
        full.extend_from_slice(&sjis_body);

        let msg = parse_syslog(&full);
        assert!(msg.content.contains("こんにちは"));
        assert_eq!(msg.encoding, "Shift_JIS (MSG-SD/BOM-Missing)");
    }

    #[test]
    fn test_rfc5424_msg_utf8_bom() {
        let header = b"<13>1 - - - - - - ";
        let utf8_body_with_bom = [
            0xEF, 0xBB, 0xBF, 0xE3, 0x81, 0x93, 0xE3, 0x82, 0x93, 0xE3, 0x81, 0xAB, 0xE3, 0x81,
            0xA1, 0xE3, 0x81, 0xAF,
        ];
        let mut full = header.to_vec();
        full.extend_from_slice(&utf8_body_with_bom);

        let msg = parse_syslog(&full);
        assert!(msg.content.contains("こんにちは"));
        assert_eq!(msg.encoding, "UTF-8 (MSG-UTF8/BOM)");
    }

    #[test]
    fn test_rfc5424_with_sd_utf8_label() {
        let header = b"<13>1 - - - - - [meta charset=\"UTF-8\"] ";
        let body = "こんにちは".as_bytes();
        let mut full = header.to_vec();
        full.extend_from_slice(body);

        let msg = parse_syslog(&full);
        assert!(msg.content.contains("こんにちは"));
        assert_eq!(msg.encoding, "UTF-8 (MSG-SD/BOM-Missing)");
    }

    #[test]
    fn test_rfc5424_with_sd_utf8_and_bom() {
        // SD宣言があり、かつBOMもあるケース
        let header = b"<13>1 - - - - - [meta charset=\"UTF-8\"] ";
        let body = [
            0xEF, 0xBB, 0xBF, 0xE3, 0x81, 0x93, 0xE3, 0x82, 0x93, 0xE3, 0x81, 0xAB, 0xE3, 0x81,
            0xA1, 0xE3, 0x81, 0xAF,
        ];
        let mut full = header.to_vec();
        full.extend_from_slice(&body);

        let msg = parse_syslog(&full);
        assert!(msg.content.contains("こんにちは"));
        assert_eq!(msg.encoding, "UTF-8 (MSG-SD/BOM-Detected)");
    }

    #[test]
    fn test_rfc5424_with_msg_utf8_label() {
        let header = b"<13>1 - - - - - [meta charset=\"MSG-UTF8\"] ";
        let body = "こんにちは".as_bytes();
        let mut full = header.to_vec();
        full.extend_from_slice(body);

        let msg = parse_syslog(&full);
        assert!(msg.content.contains("こんにちは"));
        assert_eq!(msg.encoding, "UTF-8 (MSG-SD/BOM-Missing)");
    }

    #[test]
    fn test_rfc3164_sjis_fallback() {
        let sjis_bytes = [0x82, 0xB1, 0x82, 0xF1, 0x82, 0xC9, 0x82, 0xBF, 0x82, 0xCD];
        let msg = parse_syslog(&sjis_bytes);
        assert!(msg.content.contains("こんにちは"));
        assert_eq!(msg.encoding, "Shift_JIS");
    }
}
