use serde_json::{json, Value};
use std::{fs, io::Write, path::PathBuf};

use crate::{Result, K380_PID, K380_VID};

pub struct Release {
    path: PathBuf,
    backup: PathBuf,
    original: Vec<u8>,
    temporary: Value,
    active: bool,
}

impl Release {
    pub fn new() -> Result<Self> {
        let home = std::env::var_os("HOME").ok_or("HOME 未设置")?;
        let path = PathBuf::from(home).join(".config/karabiner/karabiner.json");
        let original = fs::read(&path)?;
        let mut temporary: Value = serde_json::from_slice(&original)?;
        release_k380(&mut temporary)?;
        let backup = path.with_extension("json.k380-backup");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup)
            .map_err(|e| format!("无法创建备份 {}：{e}", backup.display()))?;
        file.set_permissions(fs::metadata(&path)?.permissions())?;
        file.write_all(&original)?;
        file.sync_all()?;
        let mut guard = Self {
            path,
            backup,
            original,
            temporary,
            active: false,
        };
        // Refuse to overwrite changes made while preparing the backup.
        if fs::read(&guard.path)? != guard.original {
            return Err("Karabiner 配置已变更，已保留备份，未修改配置".into());
        }
        guard.replace(&serde_json::to_vec_pretty(&guard.temporary)?)?;
        guard.active = true;
        Ok(guard)
    }

    fn replace(&self, contents: &[u8]) -> Result<()> {
        let temp = self.path.with_extension("json.k380-tmp");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        let result = (|| {
            file.set_permissions(fs::metadata(&self.path)?.permissions())?;
            file.write_all(contents)?;
            file.sync_all()?;
            fs::rename(&temp, &self.path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        Ok(result?)
    }

    pub fn restore(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let current: Value = serde_json::from_slice(&fs::read(&self.path)?)?;
        if current != self.temporary {
            return Err(format!(
                "Karabiner 配置被其他程序修改，未覆盖；原配置备份：{}",
                self.backup.display()
            )
            .into());
        }
        self.replace(&self.original)?;
        self.active = false;
        fs::remove_file(&self.backup)?;
        Ok(())
    }
}

impl Drop for Release {
    fn drop(&mut self) {
        if let Err(e) = self.restore() {
            eprintln!("恢复 Karabiner 配置失败：{e}");
        }
    }
}

fn release_k380(config: &mut Value) -> Result<()> {
    let profile = config["profiles"]
        .as_array_mut()
        .and_then(|profiles| profiles.iter_mut().find(|p| p["selected"] == true))
        .ok_or("Karabiner 没有选中的配置")?;
    if profile.get("devices").is_none() {
        profile["devices"] = json!([]);
    }
    let devices = profile["devices"]
        .as_array_mut()
        .ok_or("无效的 devices 配置")?;
    let mut found = false;
    for device in devices.iter_mut() {
        let ids = &device["identifiers"];
        if ids["vendor_id"] == K380_VID && ids["product_id"] == K380_PID {
            device["ignore"] = json!(true);
            found = true;
        }
    }
    if !found {
        devices.push(json!({
            "identifiers": {
                "vendor_id": K380_VID,
                "product_id": K380_PID,
                "is_keyboard": true
            },
            "ignore": true
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn releases_only_k380_in_selected_profile() {
        let device = json!({
            "identifiers": {"vendor_id": K380_VID, "product_id": K380_PID},
            "ignore": false,
            "simple_modifications": []
        });
        let other = json!({"identifiers": {"vendor_id": 1}, "ignore": false});
        let mut config = json!({"profiles": [
            {"selected": false, "devices": [device.clone()]},
            {"selected": true, "devices": [device.clone(), other.clone()]}
        ]});
        release_k380(&mut config).unwrap();
        assert_eq!(config["profiles"][0]["devices"][0], device);
        assert_eq!(config["profiles"][1]["devices"][0]["ignore"], true);
        assert_eq!(config["profiles"][1]["devices"][1], other);
    }

    #[test]
    fn adds_missing_device_and_rejects_missing_profile() {
        let mut config = json!({"profiles": [{"selected": true}]});
        release_k380(&mut config).unwrap();
        assert_eq!(config["profiles"][0]["devices"][0]["ignore"], true);
        assert!(release_k380(&mut json!({})).is_err());
    }
}
