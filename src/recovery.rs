//! Allowlisted recovery. No command text or user arguments cross the privilege boundary.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Safety {
    Safe,
    Privileged,
    /// No action in this MVP implements this category.
    #[allow(dead_code)]
    Destructive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RootTool {
    HomeStatus,
}
impl RootTool {
    pub fn safety(self) -> Safety {
        Safety::Privileged
    }
    pub fn arguments(self) -> [String; 1] {
        match self {
            Self::HomeStatus => ["status".into()],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Recovery {
    RefreshInventory,
    RefreshInputs,
    ReopenStockHome,
}
impl Recovery {
    pub fn safety(self) -> Safety {
        Safety::Safe
    }
}
/// Re-read immediately before reopening Home; cached UI status is never authorization.
pub fn stock_home(snapshot: &crate::system_diagnostics::SystemSnapshot) -> Result<&str, String> {
    if snapshot.takeover != Some(false) {
        return Err("Home takeover is active or unavailable".into());
    }
    snapshot
        .stock_home
        .as_deref()
        .filter(|id| crate::platform::is_home_id(id))
        .ok_or_else(|| "Stock Home identity unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allowlist_rejects_commands_and_arguments() {
        assert_eq!(RootTool::HomeStatus.arguments(), ["status"]);
        assert_eq!(RootTool::HomeStatus.safety(), Safety::Privileged);
        for value in [
            r#""shell""#,
            r#"{"home_status":";reboot"}"#,
            r#""disable_home_takeover""#,
        ] {
            assert!(serde_json::from_str::<RootTool>(value).is_err());
        }
        assert_eq!(Recovery::RefreshInventory.safety(), Safety::Safe);
    }
}
#[cfg(test)]
mod gate_tests {
    use super::*;
    #[test]
    fn recovery_requires_current_known_status_and_allowlisted_home() {
        let mut s = crate::system_diagnostics::SystemSnapshot::default();
        assert!(stock_home(&s).is_err());
        s.takeover = Some(true);
        s.stock_home = Some(crate::platform::HOME_ID.into());
        assert!(stock_home(&s).is_err());
        s.takeover = Some(false);
        assert_eq!(stock_home(&s).unwrap(), crate::platform::HOME_ID);
        s.stock_home = Some("x;reboot".into());
        assert!(stock_home(&s).is_err());
    }
}
