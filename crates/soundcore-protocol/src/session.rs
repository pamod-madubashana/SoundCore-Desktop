use std::path::PathBuf;

use macaddr::MacAddr6;
use soundcore_protocol_core::{DeviceModel, OpenSCQ30Session, storage::PairedDevice};

use crate::device::ConnectedDevice;

pub struct SoundcoreSession {
    inner: OpenSCQ30Session,
}

impl SoundcoreSession {
    pub async fn new() -> Result<Self, String> {
        let db_path = match std::env::var_os("OPENSCQ30_DATABASE_PATH") {
            Some(path) => PathBuf::from(path),
            None => dirs::config_dir()
                .ok_or_else(|| "no config directory".to_owned())?
                .join("openscq30")
                .join("database.sqlite"),
        };
        let inner = OpenSCQ30Session::new(db_path)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Self { inner })
    }

    pub async fn pair(&self, mac: MacAddr6, model: DeviceModel) -> Result<(), String> {
        self.inner
            .pair(PairedDevice {
                mac_address: mac,
                model,
                is_demo: false,
            })
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_devices(&self, model: DeviceModel) -> Result<Vec<String>, String> {
        let devices = self
            .inner
            .list_devices(model)
            .await
            .map_err(|e| e.to_string())?;
        Ok(devices
            .into_iter()
            .map(|d| format!("{} {}", d.name, d.mac_address))
            .collect())
    }

    pub async fn connect(&self, mac: MacAddr6) -> Result<ConnectedDevice, String> {
        let device = self.inner.connect(mac).await.map_err(|e| e.to_string())?;
        Ok(ConnectedDevice::new(device))
    }

    pub fn inner(&self) -> &OpenSCQ30Session {
        &self.inner
    }
}
