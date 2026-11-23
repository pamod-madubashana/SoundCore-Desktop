use serde::{Deserialize, Serialize};

pub const TX_PREFIX: [u8; 5] = [0x08, 0xEE, 0x00, 0x00, 0x00];
pub const RX_PREFIX: [u8; 4] = [0x09, 0xFF, 0x00, 0x00];
pub const HEADER_LEN: usize = 7;
pub const RX_HEADER_LEN: usize = 9;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub flag: u8,
    pub group: u8,
    pub command: u8,
    pub body: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    Incomplete,
    BadPrefix,
    BadLength,
    BadChecksum,
}

pub fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte))
}

pub fn encode(group: u8, command: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(HEADER_LEN + 3 + payload.len());
    frame.extend_from_slice(&TX_PREFIX);
    frame.push(group);
    frame.push(command);
    let total = (HEADER_LEN + 3 + payload.len()) as u16;
    frame.extend_from_slice(&total.to_le_bytes());
    frame.extend_from_slice(payload);
    frame.push(checksum(&frame));
    frame
}

pub fn decode(input: &[u8]) -> Result<(Frame, usize), DecodeError> {
    let prefix_len = input.len().min(RX_PREFIX.len());
    if input[..prefix_len] != RX_PREFIX[..prefix_len] {
        return Err(DecodeError::BadPrefix);
    }
    if input.len() < RX_HEADER_LEN {
        return Err(DecodeError::Incomplete);
    }
    let total = u16::from_le_bytes([input[7], input[8]]) as usize;
    if total < RX_HEADER_LEN + 1 || total > input.len() {
        return Err(DecodeError::Incomplete);
    }
    let frame = &input[..total];
    if checksum(&frame[..total - 1]) != frame[total - 1] {
        return Err(DecodeError::BadChecksum);
    }
    let frame = Frame {
        flag: input[4],
        group: input[5],
        command: input[6],
        body: input[RX_HEADER_LEN..total - 1].to_vec(),
    };
    Ok((frame, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Fixture {
        state_vectors: Vec<StateVector>,
    }
    #[derive(Deserialize)]
    struct StateVector {
        packet_hex: String,
    }

    fn unhex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn encodes_known_commands() {
        let anc = encode(0x06, 0x81, &[2, 0, 1, 0, 0, 0, 0]);
        assert_eq!(&anc[..7], &[0x08, 0xEE, 0, 0, 0, 0x06, 0x81]);
        assert_eq!(u16::from_le_bytes([anc[7], anc[8]]) as usize, anc.len());
        assert_eq!(anc.last(), Some(&checksum(&anc[..anc.len() - 1])));

        let eq = encode(0x03, 0x8A, &[1, 2, 3]);
        assert_eq!(&eq[..7], &[0x08, 0xEE, 0, 0, 0, 0x03, 0x8A]);
        assert_eq!(eq.len(), 13);
    }

    #[test]
    fn decodes_captured_frames_from_fixtures() {
        let json = include_str!("../fixtures/a3028.json");
        let fixture: Fixture = serde_json::from_str(json).unwrap();
        let mut decoded = 0;
        for vector in fixture.state_vectors {
            let bytes = unhex(&vector.packet_hex);
            if bytes.len() < RX_HEADER_LEN || bytes[..4] != RX_PREFIX {
                continue;
            }
            let (_frame, consumed) = decode(&bytes).expect("captured frame must decode");
            assert_eq!(consumed, bytes.len());
            decoded += 1;
        }
        assert!(decoded > 0, "no captured frames were decoded");
    }

    #[test]
    fn rejects_invalid_frames() {
        assert_eq!(decode(&[0x09, 0xFF]), Err(DecodeError::Incomplete));
        assert_eq!(
            decode(&[0x00, 0x00, 0, 0, 0, 0, 0, 0, 0]),
            Err(DecodeError::BadPrefix)
        );
        let mut bad: Vec<u8> = Vec::new();
        bad.extend_from_slice(&RX_PREFIX);
        bad.extend_from_slice(&[1, 1, 1, 13, 0]);
        bad.extend_from_slice(&[1, 2, 3, 0]);
        let last = bad.len() - 1;
        bad[last] = checksum(&bad[..last]).wrapping_add(1);
        assert_eq!(decode(&bad), Err(DecodeError::BadChecksum));
    }
}
