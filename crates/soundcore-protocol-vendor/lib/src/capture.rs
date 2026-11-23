use std::sync::{Mutex, OnceLock};

use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct CapturedFrame {
    pub direction: &'static str,
    pub timestamp_ms: u64,
    pub bytes: Vec<u8>,
}

static CAPTURE_SENDER: OnceLock<Mutex<Option<broadcast::Sender<CapturedFrame>>>> = OnceLock::new();

fn capture_sender() -> broadcast::Sender<CapturedFrame> {
    let lock = CAPTURE_SENDER.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().unwrap_or_else(|err| err.into_inner());
    guard
        .get_or_insert_with(|| broadcast::channel(256).0)
        .clone()
}

pub fn subscribe_capture() -> broadcast::Receiver<CapturedFrame> {
    capture_sender().subscribe()
}

pub fn publish_capture(direction: &'static str, timestamp_ms: u64, bytes: &[u8]) {
    let _ = capture_sender().send(CapturedFrame {
        direction,
        timestamp_ms,
        bytes: bytes.to_vec(),
    });
}
