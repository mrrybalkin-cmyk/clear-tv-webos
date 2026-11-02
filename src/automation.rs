//! Deterministic event matching; one executor, no queues or recursive dispatch.
use crate::{
    model::{Inventory, valid_app_id},
    smart_actions::Status,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
pub const LIMIT: usize = 32;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Trigger {
    Manual,
    Startup,
    InputConnected(String),
    InputDisconnected(String),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    Always,
    AppIs { id: String },
    InputIsConnected { id: String },
    TimeRange { start: u16, end: u16 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Automation {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    pub trigger: Trigger,
    pub condition: Condition,
    pub smart_action: String,
    #[serde(default = "default_cooldown")]
    pub cooldown_secs: u32,
}
fn default_cooldown() -> u32 {
    30
}
impl Automation {
    pub fn valid(&self) -> bool {
        valid_app_id(&self.id)
            && valid_app_id(&self.smart_action)
            && !self.name.is_empty()
            && self.name.chars().count() <= 40
            && (5..=86400).contains(&self.cooldown_secs)
            && match &self.trigger {
                Trigger::InputConnected(id) | Trigger::InputDisconnected(id) => valid_app_id(id),
                _ => true,
            }
            && match &self.condition {
                Condition::AppIs { id } | Condition::InputIsConnected { id } => valid_app_id(id),
                Condition::TimeRange { start, end } => *start < 1440 && *end < 1440 && start != end,
                _ => true,
            }
    }
    pub fn matches(
        &self,
        event: &Trigger,
        inv: &Inventory,
        app: Option<&str>,
        minute: Option<u16>,
    ) -> bool {
        self.enabled
            && self.valid()
            && self.trigger == *event
            && match &self.condition {
                Condition::Always => true,
                Condition::AppIs { id } => app == Some(id.as_str()),
                Condition::InputIsConnected { id } => inv
                    .inputs
                    .iter()
                    .any(|i| i.id == *id && i.connected == Some(true)),
                Condition::TimeRange { start, end } => minute.is_some_and(|m| {
                    if start < end {
                        m >= *start && m < *end
                    } else {
                        m >= *start || m < *end
                    }
                }),
            }
    }
}
pub fn sanitize(items: &mut Vec<Automation>) {
    let mut seen = std::collections::HashSet::new();
    for a in items.iter_mut() {
        a.name = crate::model::clean_input_name(&a.name);
        if !a.valid() {
            a.enabled = false;
        }
    }
    items.retain(|a| valid_app_id(&a.id) && seen.insert(a.id.clone()));
    items.truncate(LIMIT);
}
#[derive(Default)]
pub struct Engine {
    last: BTreeMap<String, Instant>,
    pub status: BTreeMap<String, Status>,
    pub active: Option<String>,
}
impl Engine {
    // Explicit snapshot and event metadata keep selection deterministic and fixture-friendly.
    #[allow(clippy::too_many_arguments)]
    pub fn select(
        &mut self,
        items: &[Automation],
        event: &Trigger,
        inv: &Inventory,
        minute: Option<u16>,
        now: Instant,
        busy: bool,
        derived: bool,
    ) -> Option<String> {
        // Worker results caused by scripts cannot trigger another script (including itself).
        if busy || derived || self.active.is_some() {
            return None;
        }
        let a = items.iter().find(|a| {
            a.matches(event, inv, None, minute)
                && self.last.get(&a.id).is_none_or(|t| {
                    now.saturating_duration_since(*t) >= Duration::from_secs(a.cooldown_secs.into())
                })
        })?;
        self.last.insert(a.id.clone(), now);
        self.active = Some(a.id.clone());
        self.status.insert(a.id.clone(), Status::Running);
        Some(a.smart_action.clone())
    }
    pub fn complete(&mut self, status: &Status) {
        if *status != Status::Running
            && let Some(id) = self.active.take()
        {
            self.status.insert(id, status.clone());
        }
    }
}
/// Only known -> known transitions count. First inventory, missing and unknown values are baselines.
pub fn input_events(old: &Inventory, new: &Inventory) -> Vec<Trigger> {
    new.inputs
        .iter()
        .filter_map(|i| {
            let before = old.inputs.iter().find(|a| a.id == i.id)?.connected?;
            let after = i.connected?;
            (before != after).then(|| {
                if after {
                    Trigger::InputConnected(i.id.clone())
                } else {
                    Trigger::InputDisconnected(i.id.clone())
                }
            })
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    fn item() -> Automation {
        Automation {
            id: "auto.test".into(),
            name: "Test".into(),
            enabled: true,
            trigger: Trigger::Manual,
            condition: Condition::Always,
            smart_action: "smart.test".into(),
            cooldown_secs: 30,
        }
    }
    #[test]
    fn matching_conditions_disabled_serialization() {
        let mut a = item();
        let inv = Inventory::default();
        assert!(a.matches(&Trigger::Manual, &inv, None, None));
        assert!(!a.matches(&Trigger::Startup, &inv, None, None));
        a.enabled = false;
        assert!(!a.matches(&Trigger::Manual, &inv, None, None));
        a.enabled = true;
        a.condition = Condition::AppIs {
            id: "netflix".into(),
        };
        assert!(!a.matches(&Trigger::Manual, &inv, None, None));
        assert!(a.matches(&Trigger::Manual, &inv, Some("netflix"), None));
        a.condition = Condition::InputIsConnected { id: "hdmi1".into() };
        assert!(!a.matches(&Trigger::Manual, &inv, None, None));
        a.condition = Condition::TimeRange {
            start: 1380,
            end: 60,
        };
        assert!(a.matches(&Trigger::Manual, &inv, None, Some(30)));
        assert!(!a.matches(&Trigger::Manual, &inv, None, Some(60)));
        assert!(!a.matches(&Trigger::Manual, &inv, None, None));
        assert_eq!(
            a,
            serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap()
        );
    }
    #[test]
    fn cooldown_cycles_single_flight_and_results() {
        let mut e = Engine::default();
        let a = vec![item()];
        let inv = Inventory::default();
        let now = Instant::now();
        assert!(
            e.select(&a, &Trigger::Manual, &inv, None, now, false, true)
                .is_none()
        );
        assert!(
            e.select(&a, &Trigger::Manual, &inv, None, now, true, false)
                .is_none()
        );
        assert_eq!(
            e.select(&a, &Trigger::Manual, &inv, None, now, false, false),
            Some("smart.test".into())
        );
        assert!(
            e.select(&a, &Trigger::Manual, &inv, None, now, false, false)
                .is_none()
        );
        e.complete(&Status::Succeeded);
        assert_eq!(e.status["auto.test"], Status::Succeeded);
        assert!(
            e.select(&a, &Trigger::Manual, &inv, None, now, false, false)
                .is_none()
        );
        assert!(
            e.select(
                &a,
                &Trigger::Manual,
                &inv,
                None,
                now + Duration::from_secs(30),
                false,
                false
            )
            .is_some()
        );
        e.complete(&Status::Failed("denied".into()));
        assert_eq!(e.status["auto.test"], Status::Failed("denied".into()));
    }
    #[test]
    fn only_observed_input_transitions() {
        let mut old = crate::model::demo_inventory();
        let mut new = old.clone();
        assert!(input_events(&old, &new).is_empty());
        old.inputs[0].connected = Some(false);
        new.inputs[0].connected = Some(true);
        assert_eq!(
            input_events(&old, &new),
            vec![Trigger::InputConnected(new.inputs[0].id.clone())]
        );
        old.inputs[0].connected = None;
        assert!(input_events(&old, &new).is_empty());
    }
}
