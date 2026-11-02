//! System observations are optional: failed or unsupported reads stay Unavailable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemSnapshot {
    pub takeover: Option<bool>,
    pub stock_home: Option<String>,
    pub helper: Option<bool>,
    pub home_script: Option<bool>,
    pub updater_script: Option<bool>,
}
impl SystemSnapshot {
    pub fn from_home(value: &serde_json::Value) -> Result<Option<bool>, String> {
        let enabled = value
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .ok_or("Invalid Home status reply")?;
        let mounted = value
            .get("mounted")
            .and_then(serde_json::Value::as_bool)
            .ok_or("Invalid Home mount reply")?;
        // Enabled configuration alone does not prove an active takeover.
        if enabled != mounted {
            return Ok(None);
        }
        Ok(Some(enabled))
    }
}
#[derive(Default)]
pub struct Diagnostics {
    pub snapshot: SystemSnapshot,
    pub inventory: Option<bool>,
    pub apps: Option<usize>,
    pub input_count: Option<usize>,
    pub inputs: Option<bool>,
    pub last_error: Option<String>,
}
impl Diagnostics {
    pub fn reply(&mut self, result: Result<SystemSnapshot, String>) {
        match result {
            Ok(value) => {
                self.snapshot = value;
                self.last_error = None;
            }
            Err(error) => {
                self.snapshot = SystemSnapshot::default();
                self.last_error = Some(error);
            }
        }
    }
}
pub fn available(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "Yes",
        Some(false) => "No",
        None => "Unavailable",
    }
}
/// A failed stat is not evidence that a component is absent.
pub fn component(path: impl AsRef<std::path::Path>) -> Option<bool> {
    match std::fs::metadata(path) {
        Ok(m) => Some(m.is_file()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Some(false),
        Err(_) => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_valid_failed_reply() {
        let mut d = Diagnostics::default();
        assert_eq!(d.apps, None);
        assert_eq!(d.input_count, None);
        assert_eq!(
            component(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")),
            Some(true)
        );
        assert_eq!(component("invalid\0component"), None);
        assert_eq!(available(d.snapshot.takeover), "Unavailable");
        d.reply(Ok(SystemSnapshot {
            takeover: Some(false),
            ..Default::default()
        }));
        assert_eq!(available(d.snapshot.takeover), "No");
        d.reply(Err("denied".into()));
        assert_eq!(d.snapshot, SystemSnapshot::default());
        assert_eq!(d.last_error.as_deref(), Some("denied"));
        assert_eq!(
            SystemSnapshot::from_home(&serde_json::json!({"enabled":true,"mounted":true})).unwrap(),
            Some(true)
        );
        assert_eq!(
            SystemSnapshot::from_home(&serde_json::json!({"enabled":true,"mounted":false}))
                .unwrap(),
            None
        );
        assert!(SystemSnapshot::from_home(&serde_json::json!({})).is_err());
    }
}
