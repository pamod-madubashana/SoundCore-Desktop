use std::{fs, path::PathBuf, str::FromStr};

use serde::Deserialize;
use strum::VariantArray;

#[derive(Debug, Deserialize)]
struct Fixture {
    model: String,
    #[serde(default)]
    aliases: Vec<String>,
    source: String,
    coverage: String,
    #[serde(default)]
    state_vectors: Vec<StateVector>,
    #[serde(default)]
    outbound_vectors: Vec<OutboundVector>,
}

#[derive(Debug, Deserialize)]
struct StateVector {
    source: String,
    packet_hex: String,
    expected: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OutboundVector {
    name: String,
    source: String,
    command_hex: String,
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn load_fixtures() -> Vec<(PathBuf, Fixture)> {
    let mut out = Vec::new();
    for entry in fs::read_dir(fixtures_dir()).expect("fixtures directory") {
        let path = entry.expect("fixture entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("json")
            || path.file_name().and_then(|n| n.to_str()) == Some("r60i_capture.sanitized.json")
        {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read fixture");
        let fixture: Fixture = serde_json::from_str(&text).expect("parse fixture");
        out.push((path, fixture));
    }
    out
}

#[test]
fn fixtures_cover_every_real_device_model() {
    let fixtures = load_fixtures();
    assert!(
        fixtures.len() >= 38,
        "expected fixtures for all models, got {}",
        fixtures.len()
    );

    let mut models: Vec<String> = Vec::new();
    for (path, fixture) in &fixtures {
        for model in std::iter::once(&fixture.model).chain(fixture.aliases.iter()) {
            let full = if model.starts_with("Soundcore") {
                model.clone()
            } else {
                format!("Soundcore{model}")
            };
            assert!(
                soundcore_protocol::vendor::DeviceModel::from_str(&full).is_ok(),
                "{}: unknown model {full}",
                path.display()
            );
            models.push(full);
        }
    }
    let real_models: Vec<_> = soundcore_protocol::vendor::DeviceModel::VARIANTS
        .iter()
        .filter(|m| **m != soundcore_protocol::vendor::DeviceModel::SoundcoreDevelopment)
        .filter(|m| !matches!(m, soundcore_protocol::vendor::DeviceModel::SoundcoreD1204))
        .collect();
    for model in real_models {
        let name = model.to_string();
        assert!(
            models.contains(&name),
            "missing fixture coverage for {name}"
        );
    }
}

#[test]
fn fixtures_have_valid_hex_and_documented_sources() {
    for (path, fixture) in load_fixtures() {
        assert!(
            !fixture.source.is_empty(),
            "{}: empty source",
            path.display()
        );
        assert!(
            !fixture.coverage.is_empty(),
            "{}: empty coverage",
            path.display()
        );
        for vector in &fixture.state_vectors {
            assert!(
                decode_hex(&vector.packet_hex).is_some(),
                "{}: invalid state packet hex",
                path.display()
            );
            assert!(
                !vector.source.is_empty(),
                "{}: state vector has no source",
                path.display()
            );
        }
        if fixture.coverage == "real-capture" {
            assert!(
                fixture.state_vectors.iter().any(|v| !v.expected.is_empty()),
                "{}: real capture has no expected values",
                path.display()
            );
        }
        for vector in &fixture.outbound_vectors {
            assert!(
                decode_hex(&vector.command_hex).is_some(),
                "{}: invalid outbound command hex",
                path.display()
            );
            assert!(
                !vector.name.is_empty() && !vector.source.is_empty(),
                "{}: incomplete outbound vector",
                path.display()
            );
        }
    }
}
