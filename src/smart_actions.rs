//! Typed, bounded scenarios; sequencing stays on the UI thread, execution on workers.
use crate::model::{Inventory, valid_app_id};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
pub const ACTION_LIMIT: usize = 32;
pub const STEP_LIMIT: usize = 16;
pub const DELAY_LIMIT_MS: u32 = 30_000;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Step {
    LaunchApp(String),
    LaunchInput(String),
    RefreshInventory,
    RefreshInputs,
    OpenSystemSettings,
    Delay(u32),
}
impl Step {
    pub fn safety(&self) -> crate::recovery::Safety {
        crate::recovery::Safety::Safe
    }
    pub fn valid(&self) -> bool {
        match self {
            Self::LaunchApp(id) => {
                valid_app_id(id)
                    && !crate::scroll_preview::is_placeholder(id)
                    && !id.starts_with("custom.")
                    && !crate::platform::is_home_id(id)
                    && id != crate::platform::APP_ID
            }
            Self::LaunchInput(id) => valid_app_id(id),
            Self::Delay(ms) => *ms <= DELAY_LIMIT_MS,
            _ => true,
        }
    }
    pub fn validate_target(&self, inventory: &Inventory) -> Result<(), String> {
        if !self.valid() {
            return Err("Invalid Smart Action step".into());
        }
        match self {
            Self::LaunchApp(id) if !inventory.apps.iter().any(|a| a.id == *id) => {
                Err("Application unavailable".into())
            }
            Self::LaunchInput(id) if !inventory.inputs.iter().any(|a| a.id == *id) => {
                Err("Input unavailable".into())
            }
            _ => Ok(()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmartAction {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub steps: Vec<Step>,
}
impl SmartAction {
    pub fn valid(&self) -> bool {
        valid_app_id(&self.id)
            && !self.name.trim().is_empty()
            && self.name.chars().count() <= 40
            && !self.steps.is_empty()
            && self.steps.len() <= STEP_LIMIT
            && self.steps.iter().all(Step::valid)
    }
}
pub fn sanitize(actions: &mut Vec<SmartAction>) {
    let mut seen = std::collections::HashSet::new();
    for a in actions.iter_mut() {
        a.name = crate::model::clean_input_name(&a.name);
        if !a.valid() {
            a.enabled = false;
            a.steps.truncate(STEP_LIMIT);
            a.steps.retain(Step::valid);
        }
    }
    actions.retain(|a| valid_app_id(&a.id) && seen.insert(a.id.clone()));
    actions.truncate(ACTION_LIMIT);
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub enum Status {
    #[default]
    Idle,
    Running,
    Succeeded,
    Failed(String),
}
#[derive(Default)]
pub struct Executor {
    pub status: Status,
    pub action_id: Option<String>,
    steps: Vec<Step>,
    index: usize,
    waiting: Option<(u64, Instant)>,
    delay: Option<Instant>,
    generation: u64,
}
impl Executor {
    pub fn busy(&self) -> bool {
        self.status == Status::Running
    }
    pub fn start(&mut self, a: &SmartAction) -> Result<(), String> {
        if self.busy() {
            return Err("A Smart Action is already running".into());
        }
        if !a.enabled || !a.valid() {
            return Err("Smart Action disabled or invalid".into());
        }
        self.steps = a.steps.clone();
        self.index = 0;
        self.waiting = None;
        self.delay = None;
        self.action_id = Some(a.id.clone());
        self.status = Status::Running;
        Ok(())
    }
    pub fn tick(&mut self, now: Instant, inventory: &Inventory) -> Option<(u64, Step)> {
        if !self.busy() {
            return None;
        }
        if let Some((_, deadline)) = self.waiting {
            if now >= deadline {
                self.fail("Platform reply timed out".into());
            }
            return None;
        }
        if let Some(until) = self.delay {
            if now < until {
                return None;
            }
            self.delay = None;
            self.index += 1;
        }
        if self.index == self.steps.len() {
            self.status = Status::Succeeded;
            return None;
        }
        let step = self.steps[self.index].clone();
        if let Err(e) = step.validate_target(inventory) {
            self.fail(e);
            return None;
        }
        if let Step::Delay(ms) = step {
            self.delay = Some(now + Duration::from_millis(ms.into()));
            return None;
        }
        self.generation = self.generation.wrapping_add(1);
        self.waiting = Some((self.generation, now + Duration::from_secs(45)));
        Some((self.generation, step))
    }
    pub fn accepts(&self, token: u64) -> bool {
        self.busy() && self.waiting.map(|w| w.0) == Some(token)
    }
    pub fn current_step(&self) -> Option<&Step> {
        self.steps.get(self.index)
    }
    pub fn reply(&mut self, token: u64, result: Result<(), String>) {
        if !self.busy() || self.waiting.map(|w| w.0) != Some(token) {
            return;
        }
        self.waiting = None;
        match result {
            Ok(()) => self.index += 1,
            Err(e) => self.fail(e),
        }
    }
    fn fail(&mut self, e: String) {
        self.status = Status::Failed(e);
        self.waiting = None;
        self.delay = None;
    }
}
// Extensions require a new typed Step plus a reviewed platform handler. Unknown variants fail closed.
#[cfg(test)]
mod tests {
    use super::*;
    fn action(steps: Vec<Step>) -> SmartAction {
        SmartAction {
            id: "smart.test".into(),
            name: "Test".into(),
            enabled: true,
            steps,
        }
    }
    #[test]
    fn serialization() {
        let a = action(vec![
            Step::LaunchApp("netflix".into()),
            Step::LaunchInput("hdmi1".into()),
            Step::Delay(10),
            Step::RefreshInventory,
            Step::RefreshInputs,
            Step::OpenSystemSettings,
        ]);
        assert_eq!(
            a,
            serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap()
        );
        assert!(serde_json::from_str::<Step>(r#"{"kind":"shell","value":"id"}"#).is_err());
    }
    #[test]
    fn limits_and_sanitization() {
        for steps in [
            vec![Step::Delay(DELAY_LIMIT_MS + 1)],
            vec![Step::RefreshInputs; STEP_LIMIT + 1],
            vec![Step::LaunchApp("x;id".into())],
            vec![],
        ] {
            let mut v = vec![action(steps)];
            sanitize(&mut v);
            assert!(!v[0].enabled);
        }
        let mut v = vec![action(vec![Step::RefreshInputs]); 40];
        sanitize(&mut v);
        assert_eq!(v.len(), 1);
        let mut v: Vec<_> = (0..40)
            .map(|i| SmartAction {
                id: format!("smart.{i}"),
                ..action(vec![Step::RefreshInputs])
            })
            .collect();
        sanitize(&mut v);
        assert_eq!(v.len(), ACTION_LIMIT);
    }
    #[test]
    fn order_delay_failure_single_flight() {
        let mut e = Executor::default();
        let now = Instant::now();
        let inv = Inventory::default();
        e.start(&action(vec![
            Step::RefreshInputs,
            Step::Delay(100),
            Step::RefreshInventory,
        ]))
        .unwrap();
        assert!(e.start(&action(vec![Step::RefreshInputs])).is_err());
        let (id, s) = e.tick(now, &inv).unwrap();
        assert_eq!(s, Step::RefreshInputs);
        assert!(e.tick(now, &inv).is_none());
        e.reply(id + 1, Ok(()));
        assert!(e.tick(now, &inv).is_none());
        e.reply(id, Ok(()));
        assert!(e.tick(now, &inv).is_none());
        assert!(e.tick(now + Duration::from_millis(99), &inv).is_none());
        let (id, s) = e.tick(now + Duration::from_millis(100), &inv).unwrap();
        assert_eq!(s, Step::RefreshInventory);
        e.reply(id, Err("denied".into()));
        assert_eq!(e.status, Status::Failed("denied".into()));
    }
    #[test]
    fn missing_targets_success_timeout() {
        let inv = Inventory::default();
        let now = Instant::now();
        for s in [
            Step::LaunchApp("missing".into()),
            Step::LaunchInput("missing".into()),
        ] {
            let mut e = Executor::default();
            e.start(&action(vec![s])).unwrap();
            assert!(e.tick(now, &inv).is_none());
            assert!(matches!(e.status, Status::Failed(_)));
        }
        let mut e = Executor::default();
        e.start(&action(vec![Step::RefreshInputs])).unwrap();
        let (id, _) = e.tick(now, &inv).unwrap();
        e.reply(id, Ok(()));
        e.tick(now, &inv);
        assert_eq!(e.status, Status::Succeeded);
        e.start(&action(vec![Step::RefreshInputs])).unwrap();
        e.tick(now, &inv);
        e.tick(now + Duration::from_secs(46), &inv);
        assert!(matches!(e.status, Status::Failed(_)));
    }
}
