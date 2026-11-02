//! Management page descriptors; no execution or platform IO.
use super::*;
impl State {
    pub fn management_rows(&self) -> Vec<(Command, String)> {
        let panel = match &self.subpage {
            Some(Subpage::Management(p)) => p,
            _ => &Panel::Actions,
        };
        let mut rows = vec![];
        let mut add = |c, label: String| rows.push((c, label));
        match panel {
            Panel::Actions => {
                add(Command::QuickActions, "Quick Actions".into());
                add(Command::Open(Panel::SmartActions), "Smart Actions".into());
                add(Command::Open(Panel::Automation), "Automation".into());
            }
            Panel::SmartActions => {
                add(Command::CreateSmart, "Create Smart Action".into());
                for a in &self.preferences.smart_actions {
                    add(
                        Command::EditSmart(a.id.clone()),
                        format!(
                            "{}  |  {}  |  {} steps",
                            a.name,
                            if a.enabled { "Enabled" } else { "Disabled" },
                            a.steps.len()
                        ),
                    );
                }
                add(
                    Command::Back,
                    format!("Back  |  Last execution: {:?}", self.smart.status),
                );
            }
            Panel::SmartEditor => {
                if let Some(a) = &self.smart_draft {
                    add(Command::Rename, format!("Name: {}", a.name));
                    add(
                        Command::ToggleSmart,
                        format!("Enabled: {}", if a.enabled { "On" } else { "Off" }),
                    );
                    for (i, s) in a.steps.iter().enumerate() {
                        add(
                            Command::MoveStep(i, true),
                            format!("{}. {}  |  Move up", i + 1, step_label(s, &self.inventory)),
                        );
                        add(Command::MoveStep(i, false), format!("{}. Move down", i + 1));
                        add(Command::RemoveStep(i), format!("{}. Remove step", i + 1));
                    }
                    add(Command::Open(Panel::AddStep), "Add step".into());
                    add(Command::SaveSmart, "Save".into());
                    if self.preferences.smart_actions.iter().any(|s| s.id == a.id) {
                        add(
                            Command::RunSmart(a.id.clone()),
                            format!("Run saved action  |  {:?}", self.smart.status),
                        );
                        add(
                            Command::PinSmart(a.id.clone()),
                            "Add / remove Quick Action".into(),
                        );
                        add(
                            Command::DeleteSmart(a.id.clone()),
                            "Delete Smart Action".into(),
                        );
                    }
                    add(Command::Back, "Discard draft / Back".into());
                }
            }
            Panel::AddStep => {
                for (s, label) in [
                    (Step::RefreshInventory, "Refresh inventory"),
                    (Step::RefreshInputs, "Refresh inputs"),
                    (Step::OpenSystemSettings, "Open system settings"),
                    (Step::Delay(1000), "Delay 1 second"),
                    (Step::Delay(5000), "Delay 5 seconds"),
                    (Step::Delay(30000), "Delay 30 seconds"),
                ] {
                    add(Command::AddStep(s), label.into());
                }
                for app in self.inventory.apps.iter().filter(|a| {
                    !a.id.starts_with("custom.")
                        && !crate::platform::is_home_id(&a.id)
                        && a.id != crate::platform::APP_ID
                }) {
                    add(
                        Command::AddStep(Step::LaunchApp(app.id.clone())),
                        format!("Launch app: {}", app.title),
                    );
                }
                for input in &self.inventory.inputs {
                    add(
                        Command::AddStep(Step::LaunchInput(input.id.clone())),
                        format!("Launch input: {}", input.port),
                    );
                }
                add(Command::Back, "Back".into());
            }
            Panel::Automation => {
                add(
                    Command::CreateAutomation,
                    "Create automation (requires a saved Smart Action)".into(),
                );
                add(Command::Manual, "Emit Manual event".into());
                for a in &self.preferences.automations {
                    add(
                        Command::Open(Panel::AutomationEditor(a.id.clone())),
                        format!(
                            "{} | {} | {:?}",
                            a.name,
                            if a.enabled { "Enabled" } else { "Disabled" },
                            self.automation.status.get(&a.id).unwrap_or(&Status::Idle)
                        ),
                    );
                }
                add(Command::Back, "Back".into());
            }
            Panel::AutomationEditor(id) => {
                if let Some(a) = self.preferences.automations.iter().find(|a| a.id == *id) {
                    add(
                        Command::ToggleAutomation(id.clone()),
                        format!("Enabled: {}", if a.enabled { "On" } else { "Off" }),
                    );
                    add(
                        Command::Open(Panel::Triggers(id.clone())),
                        format!("Trigger: {}", trigger_label(&a.trigger)),
                    );
                    add(
                        Command::Open(Panel::Conditions(id.clone())),
                        format!("Condition: {}", condition_label(&a.condition)),
                    );
                    add(
                        Command::CycleAction(id.clone()),
                        format!("Smart Action: {} (OK selects next)", a.smart_action),
                    );
                    add(
                        Command::DeleteAutomation(id.clone()),
                        "Delete automation".into(),
                    );
                    add(
                        Command::Back,
                        format!(
                            "Back | cooldown {}s | {:?}",
                            a.cooldown_secs,
                            self.automation.status.get(id).unwrap_or(&Status::Idle)
                        ),
                    );
                }
            }
            Panel::Triggers(id) => {
                add(
                    Command::SetTrigger(id.clone(), Trigger::Manual),
                    "Manual".into(),
                );
                add(
                    Command::SetTrigger(id.clone(), Trigger::Startup),
                    "Startup (Clear TV process)".into(),
                );
                for input in &self.inventory.inputs {
                    add(
                        Command::SetTrigger(id.clone(), Trigger::InputConnected(input.id.clone())),
                        format!("Connected: {}", input.port),
                    );
                    add(
                        Command::SetTrigger(
                            id.clone(),
                            Trigger::InputDisconnected(input.id.clone()),
                        ),
                        format!("Disconnected: {}", input.port),
                    );
                }
                add(Command::Back, "Back".into());
            }
            Panel::Conditions(id) => {
                add(
                    Command::SetCondition(id.clone(), Condition::Always),
                    "Always".into(),
                );
                for input in &self.inventory.inputs {
                    add(
                        Command::SetCondition(
                            id.clone(),
                            Condition::InputIsConnected {
                                id: input.id.clone(),
                            },
                        ),
                        format!("Input connected: {}", input.port),
                    );
                }
                add(
                    Command::SetCondition(
                        id.clone(),
                        Condition::TimeRange {
                            start: 18 * 60,
                            end: 23 * 60,
                        },
                    ),
                    "Local time 18:00-23:00 (unavailable clock fails closed)".into(),
                );
                add(Command::Back, "Back".into());
            }
            Panel::Diagnostics => {
                use crate::system_diagnostics::available as value;
                let d = &self.system;
                add(
                    Command::Root(RootTool::HomeStatus),
                    "Refresh system observations".into(),
                );
                let labels = vec![
                    format!("Clear TV {}", env!("CARGO_PKG_VERSION")),
                    format!(
                        "Apps: {} | Inputs: {}",
                        d.apps
                            .map_or_else(|| "Unavailable".into(), |n| n.to_string()),
                        d.input_count
                            .map_or_else(|| "Unavailable".into(), |n| n.to_string())
                    ),
                    format!(
                        "Inventory success: {} | Loading: {}",
                        value(d.inventory),
                        self.loading
                    ),
                    format!("Input refresh success: {}", value(d.inputs)),
                    format!("Public updater: disabled | Check: {:?}", self.update_status),
                    format!(
                        "Last known update result: {}",
                        match &self.update {
                            crate::update::InstallStatus::Idle => "Unavailable".into(),
                            status => format!("{status:?}"),
                        }
                    ),
                    format!("Home takeover active: {}", value(d.snapshot.takeover)),
                    format!(
                        "Recovery helper: {} | home.sh: {} | self-update.sh: {}",
                        value(d.snapshot.helper),
                        value(d.snapshot.home_script),
                        value(d.snapshot.updater_script)
                    ),
                ];
                for (index, label) in labels.into_iter().enumerate() {
                    add(Command::Info(index), label);
                }
                if d.last_error.is_some() {
                    add(
                        Command::Open(Panel::Diagnostics),
                        "System worker reply failed: Unavailable".into(),
                    );
                }
                add(Command::Back, "Back".into());
            }
            Panel::Recovery => {
                add(
                    Command::Recover(Recovery::RefreshInventory),
                    "Application recovery: refresh inventory".into(),
                );
                add(
                    Command::Recover(Recovery::RefreshInputs),
                    "Application recovery: refresh inputs".into(),
                );
                add(
                    Command::Recover(Recovery::ReopenStockHome),
                    "Home recovery: reopen stock Home (requires confirmed inactive takeover)"
                        .into(),
                );
                add(
                    Command::Open(Panel::Diagnostics),
                    "Update recovery: inspect last known result / packaged recovery components"
                        .into(),
                );
                add(Command::Back, "Back".into());
            }
            Panel::RootTools => {
                add(
                    Command::Root(RootTool::HomeStatus),
                    "HomeStatus (read-only allowlist)".into(),
                );
                add(
                    Command::Open(Panel::Diagnostics),
                    "View observed result".into(),
                );
                add(Command::Back, "Back".into());
            }
        }
        rows
    }
}

fn step_label(step: &Step, inventory: &crate::model::Inventory) -> String {
    match step {
        Step::LaunchApp(id) => format!(
            "Launch app: {}",
            inventory
                .apps
                .iter()
                .find(|a| &a.id == id)
                .map_or("Unavailable", |a| a.title.as_str())
        ),
        Step::LaunchInput(id) => format!(
            "Launch input: {}",
            inventory
                .inputs
                .iter()
                .find(|a| &a.id == id)
                .map_or("Unavailable", |a| a.port.as_str())
        ),
        Step::RefreshInventory => "Refresh inventory".into(),
        Step::RefreshInputs => "Refresh inputs".into(),
        Step::OpenSystemSettings => "Open system settings".into(),
        Step::Delay(ms) => format!("Delay {} seconds", f64::from(*ms) / 1000.),
    }
}
fn trigger_label(trigger: &Trigger) -> String {
    match trigger {
        Trigger::Manual => "Manual".into(),
        Trigger::Startup => "Clear TV startup".into(),
        Trigger::InputConnected(id) => format!("Input connected: {id}"),
        Trigger::InputDisconnected(id) => format!("Input disconnected: {id}"),
    }
}
fn condition_label(condition: &Condition) -> String {
    match condition {
        Condition::Always => "Always".into(),
        Condition::AppIs { .. } => "Current app: Unavailable".into(),
        Condition::InputIsConnected { id } => format!("Input connected: {id}"),
        Condition::TimeRange { start, end } => format!(
            "Local time {:02}:{:02}-{:02}:{:02}",
            start / 60,
            start % 60,
            end / 60,
            end % 60
        ),
    }
}
