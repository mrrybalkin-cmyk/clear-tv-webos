//! Remote-first management pages, separate from launcher rendering and platform IO.
use crate::{
    automation::{self, Automation, Condition, Trigger},
    model::{QUICK_ACTION_LIMIT, QuickAction},
    recovery::{Recovery, RootTool},
    smart_actions::{self, SmartAction, Status, Step},
    state::{ACTIONS_SECTION, Effect, Page, State, Subpage, Target, TextField},
};
use serde::Serialize;
use std::time::Instant;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Panel {
    Actions,
    SmartActions,
    SmartEditor,
    AddStep,
    Automation,
    AutomationEditor(String),
    Triggers(String),
    Conditions(String),
    Diagnostics,
    Recovery,
    RootTools,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Command {
    Open(Panel),
    QuickActions,
    CreateSmart,
    EditSmart(String),
    Rename,
    AddStep(Step),
    RemoveStep(usize),
    MoveStep(usize, bool),
    SaveSmart,
    RunSmart(String),
    DeleteSmart(String),
    ToggleSmart,
    PinSmart(String),
    CreateAutomation,
    ToggleAutomation(String),
    SetTrigger(String, Trigger),
    SetCondition(String, Condition),
    CycleAction(String),
    DeleteAutomation(String),
    Manual,
    Root(RootTool),
    Recover(Recovery),
    Info(usize),
    Back,
}
impl Panel {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Actions => "Actions",
            Self::SmartActions => "Smart Actions",
            Self::SmartEditor => "Smart Action",
            Self::AddStep => "Add step",
            Self::Automation => "Automation",
            Self::AutomationEditor(_) => "Automation rule",
            Self::Triggers(_) => "Trigger",
            Self::Conditions(_) => "Condition",
            Self::Diagnostics => "Diagnostics",
            Self::Recovery => "Recovery",
            Self::RootTools => "Root Tools",
        }
    }
}
impl State {
    pub fn management_open(&mut self, panel: Panel) {
        self.overlay = None;
        self.page = Page::Settings;
        self.section = if matches!(
            panel,
            Panel::Diagnostics | Panel::Recovery | Panel::RootTools
        ) {
            crate::state::SYSTEM_SECTION
        } else {
            ACTIONS_SECTION
        };
        self.subpage = Some(Subpage::Management(panel));
        self.scroll = 0.;
        self.panel_started = None;
        self.focus = self
            .management_rows()
            .first()
            .map(|(c, _)| Target::Management(c.clone()))
            .unwrap_or(Target::Sidebar(ACTIONS_SECTION));
        self.dirty = true;
    }
    pub fn management_back(&mut self, panel: Panel) {
        match panel {
            Panel::SmartEditor => {
                self.smart_draft = None;
                self.management_open(Panel::SmartActions);
            }
            Panel::AddStep => self.management_open(Panel::SmartEditor),
            Panel::AutomationEditor(_) => self.management_open(Panel::Automation),
            Panel::Triggers(id) | Panel::Conditions(id) => {
                self.management_open(Panel::AutomationEditor(id))
            }
            Panel::SmartActions | Panel::Automation => self.management_open(Panel::Actions),
            _ => {
                self.subpage = None;
                self.focus = Target::Sidebar(self.section);
                self.scroll = 0.;
                self.dirty = true;
            }
        }
    }
    pub fn start_smart(&mut self, id: &str) -> Vec<Effect> {
        let result = self
            .preferences
            .smart_actions
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| "Smart Action unavailable".to_string())
            .and_then(|a| self.smart.start(a));
        if let Err(e) = result {
            self.notify(e.clone());
            if !self.smart.busy() {
                self.automation.complete(&Status::Failed(e));
            }
        }
        self.dirty = true;
        vec![]
    }
    pub fn automation_event(&mut self, event: Trigger, derived: bool) {
        // AppIs stays unavailable until a reliable foreground-app observer exists.
        let minute = self.automation_offset.map(|offset| {
            let t = time::OffsetDateTime::now_utc().to_offset(offset);
            u16::from(t.hour()) * 60 + u16::from(t.minute())
        });
        if let Some(id) = self.automation.select(
            &self.preferences.automations,
            &event,
            &self.inventory,
            minute,
            Instant::now(),
            self.smart.busy(),
            derived,
        ) {
            self.start_smart(&id);
        }
    }
    pub fn management_observe_inputs(&mut self, next: &crate::model::Inventory) {
        let events = automation::input_events(&self.inventory, next);
        for event in events {
            self.automation_event_with_inventory(event, next);
        }
    }
    fn automation_event_with_inventory(&mut self, event: Trigger, next: &crate::model::Inventory) {
        let minute = self.automation_offset.map(|offset| {
            let t = time::OffsetDateTime::now_utc().to_offset(offset);
            u16::from(t.hour()) * 60 + u16::from(t.minute())
        });
        if let Some(id) = self.automation.select(
            &self.preferences.automations,
            &event,
            next,
            minute,
            Instant::now(),
            self.smart.busy(),
            false,
        ) {
            self.start_smart(&id);
        }
    }
    pub fn management_tick(&mut self, now: Instant) -> Vec<Effect> {
        if !self.startup_fired && self.system.inventory == Some(true) {
            self.startup_fired = true;
            self.automation_event(Trigger::Startup, false);
        }
        let before = self.smart.status.clone();
        let step = self.smart.tick(now, &self.inventory);
        self.automation.complete(&self.smart.status);
        if before != self.smart.status {
            self.dirty = true;
        }
        step.map(|(token, step)| vec![Effect::SmartStep { token, step }])
            .unwrap_or_default()
    }
    pub fn management_activate(&mut self, command: Command) -> Vec<Effect> {
        self.dirty = true;
        match command {
            Command::Open(panel) => {
                let observe = matches!(
                    panel,
                    Panel::Diagnostics | Panel::Recovery | Panel::RootTools
                );
                self.management_open(panel);
                if observe {
                    return vec![Effect::Root(RootTool::HomeStatus)];
                }
            }
            Command::QuickActions => {
                self.management_open(Panel::Actions);
                self.subpage = Some(Subpage::QuickActions);
                self.focus = Target::QuickActionToggle(0);
            }
            Command::CreateSmart => {
                if self.preferences.smart_actions.len() >= smart_actions::ACTION_LIMIT {
                    self.notify("Smart Action limit reached");
                    return vec![];
                }
                let id = (1..)
                    .map(|i| format!("smart.{i}"))
                    .find(|id| !self.preferences.smart_actions.iter().any(|a| &a.id == id))
                    .unwrap();
                self.smart_draft = Some(SmartAction {
                    id,
                    name: "New Smart Action".into(),
                    enabled: true,
                    steps: vec![],
                });
                self.management_open(Panel::SmartEditor);
            }
            Command::EditSmart(id) => {
                self.smart_draft = self
                    .preferences
                    .smart_actions
                    .iter()
                    .find(|a| a.id == id)
                    .cloned();
                self.management_open(Panel::SmartEditor);
            }
            Command::Rename => {
                self.action_name.clear();
                self.open_keyboard(TextField::ActionName);
            }
            Command::ToggleSmart => {
                if let Some(a) = &mut self.smart_draft {
                    a.enabled = !a.enabled;
                }
            }
            Command::AddStep(step) => {
                if let Some(a) = &mut self.smart_draft
                    && a.steps.len() < smart_actions::STEP_LIMIT
                    && step.valid()
                {
                    a.steps.push(step);
                }
                self.management_open(Panel::SmartEditor);
            }
            Command::RemoveStep(i) => {
                if let Some(a) = &mut self.smart_draft
                    && i < a.steps.len()
                {
                    a.steps.remove(i);
                }
                self.management_open(Panel::SmartEditor);
            }
            Command::MoveStep(i, up) => {
                if let Some(a) = &mut self.smart_draft {
                    let next = if up { i.saturating_sub(1) } else { i + 1 };
                    if i < a.steps.len() && next < a.steps.len() {
                        a.steps.swap(i, next);
                    }
                }
            }
            Command::SaveSmart => {
                let Some(a) = self.smart_draft.clone() else {
                    return vec![];
                };
                if !a.valid() {
                    self.notify("Add valid steps before saving");
                    return vec![];
                }
                if let Some(old) = self
                    .preferences
                    .smart_actions
                    .iter_mut()
                    .find(|old| old.id == a.id)
                {
                    *old = a;
                } else if self.preferences.smart_actions.len() < smart_actions::ACTION_LIMIT {
                    self.preferences.smart_actions.push(a);
                }
                self.management_open(Panel::SmartActions);
                return vec![Effect::Save];
            }
            Command::RunSmart(id) => return self.start_smart(&id),
            Command::PinSmart(id) => {
                let q = QuickAction::SmartAction(id);
                if let Some(i) = self.preferences.quick_actions.iter().position(|a| a == &q) {
                    self.preferences.quick_actions.remove(i);
                } else if self.preferences.quick_actions.len() < QUICK_ACTION_LIMIT {
                    self.preferences.quick_actions.push(q);
                }
                return vec![Effect::Save];
            }
            Command::DeleteSmart(id) => {
                self.preferences.smart_actions.retain(|a| a.id != id);
                self.preferences
                    .quick_actions
                    .retain(|a| a != &QuickAction::SmartAction(id.clone()));
                for a in &mut self.preferences.automations {
                    if a.smart_action == id {
                        a.enabled = false;
                    }
                }
                self.smart_draft = None;
                self.management_open(Panel::SmartActions);
                return vec![Effect::Save];
            }
            Command::CreateAutomation => {
                let Some(action) = self.preferences.smart_actions.first() else {
                    self.notify("Save a Smart Action first");
                    return vec![];
                };
                if self.preferences.automations.len() >= automation::LIMIT {
                    return vec![];
                }
                let id = (1..)
                    .map(|i| format!("automation.{i}"))
                    .find(|id| !self.preferences.automations.iter().any(|a| &a.id == id))
                    .unwrap();
                self.preferences.automations.push(Automation {
                    id: id.clone(),
                    name: format!("Automation {}", self.preferences.automations.len() + 1),
                    enabled: false,
                    trigger: Trigger::Manual,
                    condition: Condition::Always,
                    smart_action: action.id.clone(),
                    cooldown_secs: 30,
                });
                self.management_open(Panel::AutomationEditor(id));
                return vec![Effect::Save];
            }
            Command::ToggleAutomation(id) => {
                if let Some(a) = self.preferences.automations.iter_mut().find(|a| a.id == id) {
                    a.enabled = !a.enabled;
                }
                return vec![Effect::Save];
            }
            Command::SetTrigger(id, trigger) => {
                if let Some(a) = self.preferences.automations.iter_mut().find(|a| a.id == id) {
                    a.trigger = trigger;
                }
                self.management_open(Panel::AutomationEditor(id));
                return vec![Effect::Save];
            }
            Command::SetCondition(id, condition) => {
                if let Some(a) = self.preferences.automations.iter_mut().find(|a| a.id == id) {
                    a.condition = condition;
                }
                self.management_open(Panel::AutomationEditor(id));
                return vec![Effect::Save];
            }
            Command::CycleAction(id) => {
                if let Some(a) = self.preferences.automations.iter_mut().find(|a| a.id == id) {
                    let list = &self.preferences.smart_actions;
                    if !list.is_empty() {
                        let index = list
                            .iter()
                            .position(|s| s.id == a.smart_action)
                            .map_or(0, |i| (i + 1) % list.len());
                        a.smart_action = list[index].id.clone();
                    }
                }
                return vec![Effect::Save];
            }
            Command::DeleteAutomation(id) => {
                self.preferences.automations.retain(|a| a.id != id);
                self.management_open(Panel::Automation);
                return vec![Effect::Save];
            }
            Command::Manual => self.automation_event(Trigger::Manual, false),
            Command::Root(tool) => return vec![Effect::Root(tool)],
            Command::Recover(action) => return vec![Effect::Recovery(action)],
            Command::Info(_) => {}
            Command::Back => {
                if let Some(Subpage::Management(panel)) = self.subpage.clone() {
                    self.management_back(panel);
                }
            }
        }
        vec![]
    }
}
#[path = "management_view.rs"]
mod pages;
#[cfg(test)]
#[path = "management-tests.rs"]
mod tests;
