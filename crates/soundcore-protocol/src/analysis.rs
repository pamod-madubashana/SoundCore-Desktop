use std::io::Write;

use anyhow::anyhow;

use crate::transport;

const REPORT_RECIPIENT: &str = include_str!("../keys/report_recipient.pub");
pub const SHARE_URL_MAX: usize = 6000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedFrameDto {
    pub direction: String,
    pub timestamp_ms: u64,
    pub hex: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameAnalysis {
    pub direction: String,
    pub group: u8,
    pub command: u8,
    pub checksum_valid: bool,
    pub bytes: usize,
    pub fields: Vec<(String, String)>,
}

fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    let compact: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.len() % 2 != 0 {
        return None;
    }
    (0..compact.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&compact[i..i + 2], 16).ok())
        .collect()
}

pub fn analyze(hex: &str, model: &str) -> Option<FrameAnalysis> {
    let bytes = decode_hex(hex)?;
    if bytes.starts_with(&transport::TX_PREFIX) {
        return analyze_outbound(&bytes);
    }
    let (frame, consumed) = transport::decode(&bytes).ok()?;
    let checksum_valid =
        consumed >= 2 && transport::checksum(&bytes[..consumed - 1]) == bytes[consumed - 1];
    let mut fields = vec![
        ("length".to_string(), consumed.to_string()),
        ("body bytes".to_string(), frame.body.len().to_string()),
        ("flag".to_string(), frame.flag.to_string()),
    ];
    if (model == "SoundcoreD1202C" || model == "SoundcoreD1202")
        && frame.group == 1
        && frame.command == 1
    {
        if let Ok(state) = crate::native::parse_state_packet(&bytes[..consumed]) {
            for key in [
                "BatteryLevelLeft",
                "BatteryLevelRight",
                "AmbientSoundMode",
                "SpatialAudio",
                "SerialNumber",
                "FirmwareVersion",
            ] {
                if let Some(value) = state.value_for_key(key) {
                    fields.push((key.to_string(), value));
                }
            }
        }
    }
    Some(FrameAnalysis {
        direction: "rx".to_string(),
        group: frame.group,
        command: frame.command,
        checksum_valid,
        bytes: consumed,
        fields,
    })
}

fn analyze_outbound(bytes: &[u8]) -> Option<FrameAnalysis> {
    if bytes.len() < 10 || bytes[..transport::TX_PREFIX.len()] != transport::TX_PREFIX {
        return None;
    }
    let total = u16::from_le_bytes([bytes[7], bytes[8]]) as usize;
    if total != bytes.len() {
        return None;
    }
    let payload = &bytes[9..total - 1];
    let checksum_valid = transport::checksum(&bytes[..total - 1]) == bytes[total - 1];
    let command_name = match (bytes[5], bytes[6]) {
        (1, 1) => "state request",
        (6, 0x81) => "set sound mode",
        (3, 0x8A) => "set equalizer",
        (16, 0x81) => "set spatial audio",
        _ => "command",
    };
    Some(FrameAnalysis {
        direction: "tx".to_string(),
        group: bytes[5],
        command: bytes[6],
        checksum_valid,
        bytes: total,
        fields: vec![
            ("length".to_string(), total.to_string()),
            ("payload bytes".to_string(), payload.len().to_string()),
            ("command".to_string(), command_name.to_string()),
        ],
    })
}

fn frame_meta(hex: &str) -> (String, usize) {
    match decode_hex(hex) {
        Some(bytes) if bytes.len() >= 9 => (format!("{}.{}", bytes[5], bytes[6]), bytes.len()),
        Some(bytes) => ("-".to_string(), bytes.len()),
        None => ("-".to_string(), 0),
    }
}

fn compact_hex(hex: &str) -> String {
    hex.chars().filter(|c| !c.is_whitespace()).collect()
}

fn is_identifier(key: &str) -> bool {
    matches!(
        key,
        "SerialNumber"
            | "FirmwareVersion"
            | "FirmwareVersionLeft"
            | "FirmwareVersionRight"
            | "MacAddress"
    )
}

fn encrypted_payload(frames: &[CapturedFrameDto]) -> String {
    let mut payload = String::new();
    for frame in frames {
        let (group_cmd, _) = frame_meta(&frame.hex);
        payload.push_str(&format!(
            "{}\t{}\t{}\n",
            frame.direction,
            group_cmd,
            compact_hex(&frame.hex)
        ));
    }
    payload
}

fn encrypt_payload(payload: &str, recipient: &str) -> anyhow::Result<String> {
    let recipient = recipient
        .trim()
        .parse::<age::x25519::Recipient>()
        .map_err(|err| anyhow!("invalid report recipient: {err}"))?;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(payload.as_bytes())?;
    let compressed = encoder.finish()?;
    let armored = age::encrypt_and_armor(&recipient, &compressed)
        .map_err(|err| anyhow!("failed to encrypt report payload: {err}"))?;
    Ok(armored.replace("\r\n", "\n"))
}

fn build_report_with(
    model: &str,
    name: &str,
    frames: &[CapturedFrameDto],
    recipient: &str,
) -> anyhow::Result<String> {
    let encrypted = encrypt_payload(&encrypted_payload(frames), recipient)?;
    let analyses: Vec<Option<FrameAnalysis>> = frames
        .iter()
        .map(|frame| analyze(&frame.hex, model))
        .collect();
    let redacted = "[redacted]";
    let mut out = String::new();
    out.push_str(&format!("# Device support request: {name} ({model})\n\n"));
    out.push_str("## Environment\n\n");
    out.push_str("- App: SoundCore-Desktop\n");
    out.push_str(&format!("- OS: {}\n", std::env::consts::OS));
    out.push_str(&format!("- Architecture: {}\n", std::env::consts::ARCH));
    out.push_str(&format!("- Captured frames: {}\n\n", frames.len()));
    out.push_str("## Device\n\n");
    out.push_str(&format!("- Name: {name}\n"));
    out.push_str(&format!("- Model: {model}\n\n"));
    out.push_str("## Captured frames\n\n");
    out.push_str(&format!(
        "Captured {} frames from Protocol Lab.\n\n",
        frames.len()
    ));
    out.push_str("| Direction | Group.Cmd | Bytes | Payload |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for frame in frames {
        let (group_cmd, bytes_len) = frame_meta(&frame.hex);
        out.push_str(&format!(
            "| {} | {} | {} | `[encrypted]` |\n",
            frame.direction, group_cmd, bytes_len
        ));
    }
    out.push_str("\n## Encrypted frames\n\n");
    out.push_str("Raw bytes are compressed and encrypted to the maintainer's age key.\n\n");
    out.push_str("```text\n");
    out.push_str(&encrypted);
    out.push_str("```\n\n");
    out.push_str("Save the block as `frames.age`, then run:\n\n");
    out.push_str("`rage -d -i private-key.txt frames.age | gunzip`\n\n");
    out.push_str("## Decoded fields\n\n");
    for (index, (frame, analysis)) in frames.iter().zip(analyses.iter()).enumerate() {
        let (group_cmd, _) = frame_meta(&frame.hex);
        out.push_str(&format!(
            "### Frame {} ({}, {})\n\n",
            index + 1,
            frame.direction,
            group_cmd
        ));
        match analysis {
            Some(decoded) => {
                out.push_str(&format!("- Checksum valid: {}\n", decoded.checksum_valid));
                for (key, value) in &decoded.fields {
                    let shown = if is_identifier(key) {
                        redacted
                    } else {
                        value.as_str()
                    };
                    out.push_str(&format!("- {}: {}\n", key, shown));
                }
            }
            None => out.push_str("- Could not decode frame.\n"),
        }
        out.push('\n');
    }
    out.push_str(
        "> **Note:** Raw frames are encrypted; review the decoded fields before sharing.\n",
    );
    Ok(out)
}

pub fn build_report(
    model: &str,
    name: &str,
    frames: &[CapturedFrameDto],
    _include_identifiers: bool,
) -> anyhow::Result<String> {
    build_report_with(model, name, frames, REPORT_RECIPIENT)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportShare {
    pub report: String,
    pub github_url: String,
    pub email_url: String,
    pub github_title_url: String,
    pub email_title_url: String,
}

const ISSUE_BASE_URL: &str = "https://github.com/pamod-madubashana/SoundCore-Desktop/issues/new";
const SUPPORT_EMAIL: &str = "contact+soundcore@pamod.is-a.dev";

fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn report_title(model: &str, name: &str) -> String {
    format!("Device support request: {name} ({model})")
}

fn issue_url(title: &str, body: &str) -> String {
    format!(
        "{ISSUE_BASE_URL}?title={}&body={}",
        url_encode(title),
        url_encode(body)
    )
}

fn issue_title_url(title: &str) -> String {
    format!("{ISSUE_BASE_URL}?title={}", url_encode(title))
}

fn email_url(subject: &str, body: &str) -> String {
    format!(
        "mailto:{SUPPORT_EMAIL}?subject={}&body={}",
        url_encode(subject),
        url_encode(body)
    )
}

fn email_title_url(subject: &str) -> String {
    format!("mailto:{SUPPORT_EMAIL}?subject={}", url_encode(subject))
}

pub fn build_share_bundle(
    model: &str,
    name: &str,
    frames: &[CapturedFrameDto],
) -> anyhow::Result<ReportShare> {
    let report = build_report(model, name, frames, false)?;
    let title = report_title(model, name);
    Ok(ReportShare {
        github_url: issue_url(&title, &report),
        email_url: email_url(&title, &report),
        github_title_url: issue_title_url(&title),
        email_title_url: email_title_url(&title),
        report,
    })
}

pub fn github_issue_url(
    model: &str,
    name: &str,
    frames: &[CapturedFrameDto],
) -> anyhow::Result<String> {
    Ok(build_share_bundle(model, name, frames)?.github_url)
}

pub fn email_report_url(
    model: &str,
    name: &str,
    frames: &[CapturedFrameDto],
) -> anyhow::Result<String> {
    Ok(build_share_bundle(model, name, frames)?.email_url)
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use serde::Deserialize;

    fn armored_block(report: &str) -> &str {
        report
            .split("```text\n")
            .nth(1)
            .expect("armor block")
            .split("\n```")
            .next()
            .expect("armor end")
    }

    fn decrypt_payload(identity: &age::x25519::Identity, armored: &str) -> String {
        let compressed = age::decrypt(identity, armored.as_bytes()).expect("decrypt report");
        let mut payload = String::new();
        flate2::read::GzDecoder::new(&compressed[..])
            .read_to_string(&mut payload)
            .expect("gunzip payload");
        payload
    }

    #[derive(Deserialize)]
    struct CaptureFixture {
        model: String,
        rx_frames: Vec<String>,
        tx_frames: Vec<String>,
    }

    fn fixture() -> CaptureFixture {
        serde_json::from_str(include_str!("../fixtures/r60i_capture.sanitized.json"))
            .expect("capture fixture")
    }

    fn capture_frames(capture: &CaptureFixture) -> Vec<CapturedFrameDto> {
        capture
            .rx_frames
            .iter()
            .enumerate()
            .map(|(index, hex)| CapturedFrameDto {
                direction: "rx".to_string(),
                timestamp_ms: index as u64 * 10,
                hex: hex.clone(),
            })
            .collect()
    }

    #[test]
    fn analyzes_real_capture_frames() {
        let capture = fixture();
        let mut states = 0;
        for hex in &capture.rx_frames {
            let decoded = analyze(hex, &capture.model).expect("real RX frame must analyze");
            assert_eq!(decoded.direction, "rx");
            assert!(decoded.checksum_valid);
            assert!(decoded.bytes > 0);
            assert!(decoded.fields.iter().any(|(key, _)| key == "length"));
            assert!(decoded.fields.iter().any(|(key, _)| key == "body bytes"));
            assert!(decoded.fields.iter().any(|(key, _)| key == "flag"));
            if decoded.group == 1 && decoded.command == 1 {
                assert!(
                    decoded
                        .fields
                        .iter()
                        .any(|(key, _)| key == "BatteryLevelLeft")
                );
                assert!(
                    decoded
                        .fields
                        .iter()
                        .any(|(key, _)| key == "AmbientSoundMode")
                );
                assert!(decoded.fields.iter().any(|(key, _)| key == "SpatialAudio"));
                assert!(decoded.fields.iter().any(|(key, _)| key == "SerialNumber"));
                assert!(
                    decoded
                        .fields
                        .iter()
                        .any(|(key, _)| key == "FirmwareVersion")
                );
                states += 1;
            }
        }
        assert!(states > 0, "no state frames decoded");
        let outbound =
            analyze(&capture.tx_frames[0], &capture.model).expect("real TX frame must analyze");
        assert_eq!(outbound.direction, "tx");
        assert_eq!(outbound.group, 1);
        assert_eq!(outbound.command, 1);
        assert!(outbound.checksum_valid);
        assert!(
            outbound
                .fields
                .iter()
                .any(|(key, value)| key == "command" && value == "state request")
        );
    }

    #[test]
    fn never_includes_device_identifiers() {
        let capture = fixture();
        let frames = capture_frames(&capture);
        let serial = analyze(&capture.rx_frames[0], &capture.model)
            .expect("state frame")
            .fields
            .into_iter()
            .find(|(key, _)| key == "SerialNumber")
            .map(|(_, value)| value)
            .expect("serial field");
        assert!(!serial.is_empty());
        let report = build_report(&capture.model, "R60i", &frames, true).expect("report");
        assert!(!report.contains(&serial));
        assert!(!report.contains("MAC address"));
        assert!(!report.contains("Serial number"));
        assert!(!report.contains("Firmware:"));
        assert!(report.contains("[redacted]"));
    }

    #[test]
    fn embedded_recipient_is_valid() {
        assert!(
            REPORT_RECIPIENT
                .trim()
                .parse::<age::x25519::Recipient>()
                .is_ok()
        );
    }

    #[test]
    fn encrypted_payload_round_trips_and_hides_raw_hex() {
        let capture = fixture();
        let frames = capture_frames(&capture);
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public().to_string();
        let report =
            build_report_with(&capture.model, "R60i", &frames, &recipient).expect("report");
        assert!(report.contains("-----BEGIN AGE ENCRYPTED FILE-----"));
        assert!(report.contains("rage -d"));
        for frame in &frames {
            assert!(!report.contains(&compact_hex(&frame.hex)));
        }
        let payload = decrypt_payload(&identity, armored_block(&report));
        for frame in &frames {
            let expected = format!(
                "rx\t{}\t{}",
                frame_meta(&frame.hex).0,
                compact_hex(&frame.hex)
            );
            assert!(payload.contains(&expected));
        }
        assert!(!payload.contains("SerialNumber"));
    }

    #[test]
    fn share_links_are_prefilled_and_bounded() {
        let capture = fixture();
        let frames = capture_frames(&capture);
        let share = build_share_bundle(&capture.model, "R60i", &frames).expect("share bundle");
        assert!(share.github_url.starts_with(
            "https://github.com/pamod-madubashana/SoundCore-Desktop/issues/new?title="
        ));
        assert!(
            share
                .github_url
                .contains("Device%20support%20request%3A%20R60i")
        );
        assert!(share.github_url.contains("&body="));
        assert!(share.github_url.len() < 30000);
        assert!(
            share
                .email_url
                .starts_with("mailto:contact+soundcore@pamod.is-a.dev?subject=")
        );
        assert!(share.email_url.contains("&body="));
        assert!(share.email_url.len() < 30000);
        assert!(!share.github_title_url.contains("&body="));
        assert!(!share.email_title_url.contains("&body="));
        assert!(share.report.contains("-----BEGIN AGE ENCRYPTED FILE-----"));
        assert!(
            github_issue_url(&capture.model, "R60i", &frames)
                .expect("issue url")
                .starts_with(
                    "https://github.com/pamod-madubashana/SoundCore-Desktop/issues/new?title="
                )
        );
        assert!(
            email_report_url(&capture.model, "R60i", &frames)
                .expect("email url")
                .starts_with("mailto:contact+soundcore@pamod.is-a.dev?subject=")
        );
    }

    #[test]
    fn report_has_title_frame_count_and_sections() {
        let capture = fixture();
        let frames = capture_frames(&capture);
        let report = build_report(&capture.model, "R60i", &frames, false).expect("report");
        assert!(report.starts_with("# Device support request: R60i (SoundcoreD1202C)"));
        assert!(report.contains(&frames.len().to_string()));
        assert!(report.contains("## Captured frames"));
        assert!(report.contains("## Encrypted frames"));
        assert!(report.contains("## Decoded fields"));
        assert!(!report.contains("## Steps to reproduce"));
        assert!(!report.contains("Full hex:"));
        assert!(report.contains("`[encrypted]`"));
        assert!(report.contains("encrypted"));
    }
}
