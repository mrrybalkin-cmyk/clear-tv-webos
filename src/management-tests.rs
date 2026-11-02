use super::*;
#[test]
fn legacy_preferences_and_quick_action() {
    let mut p: crate::model::Preferences = serde_json::from_str(
        r#"{"quick_actions":[{"kind":"refresh_home"}],"favorites":["netflix"]}"#,
    )
    .unwrap();
    p.sanitize();
    assert!(p.smart_actions.is_empty());
    assert!(p.automations.is_empty());
    assert_eq!(p.favorites, Some(vec!["netflix".into()]));
    let mut s = State::new(p);
    s.management_activate(Command::CreateSmart);
    s.management_activate(Command::AddStep(Step::RefreshInputs));
    assert_eq!(
        s.management_activate(Command::SaveSmart),
        vec![Effect::Save]
    );
    let id = s.preferences.smart_actions[0].id.clone();
    s.management_activate(Command::PinSmart(id.clone()));
    assert!(
        s.quick_action_candidates()
            .contains(&QuickAction::SmartAction(id))
    );
    s.focus = Target::QuickAction(1);
    s.activate();
    assert!(s.smart.busy());
}
#[test]
fn automation_runs_same_executor_and_propagates_failure() {
    let mut s = State::new(Default::default());
    s.management_activate(Command::CreateSmart);
    s.management_activate(Command::AddStep(Step::RefreshInputs));
    s.management_activate(Command::SaveSmart);
    s.management_activate(Command::CreateAutomation);
    let id = s.preferences.automations[0].id.clone();
    s.management_activate(Command::ToggleAutomation(id.clone()));
    s.automation_event(Trigger::Manual, false);
    assert!(s.smart.busy());
    let e = s.management_tick(Instant::now());
    let Effect::SmartStep { token, .. } = e[0] else {
        panic!()
    };
    s.smart.reply(token, Err("denied".into()));
    s.management_tick(Instant::now());
    assert_eq!(s.automation.status[&id], Status::Failed("denied".into()));
}
#[test]
fn damaged_management_entries_preserve_launcher_preferences() {
    for bad in [
        serde_json::json!(null),
        serde_json::json!([{"id":"smart.broken","steps":[{"kind":"shell","value":"reboot"}]}]),
        serde_json::json!(4),
    ] {
        let mut p:crate::model::Preferences=serde_json::from_value(serde_json::json!({"favorites":["netflix"],"greeting_name":"Owner","smart_actions":bad,"automations":bad})).unwrap();
        p.sanitize();
        assert_eq!(p.favorites, Some(vec!["netflix".into()]));
        assert_eq!(p.greeting_name, "Owner");
        assert!(p.smart_actions.is_empty());
    }
}
#[test]
fn editor_order_remove_save_roundtrip_and_keyboard() {
    let mut s = State::new(Default::default());
    s.management_activate(Command::CreateSmart);
    s.management_activate(Command::Rename);
    s.text("Gaming");
    s.finish_text_input();
    for step in [
        Step::RefreshInputs,
        Step::Delay(1000),
        Step::RefreshInventory,
    ] {
        s.management_activate(Command::AddStep(step));
    }
    s.management_activate(Command::MoveStep(2, true));
    s.management_activate(Command::RemoveStep(2));
    s.management_activate(Command::SaveSmart);
    assert_eq!(s.preferences.smart_actions[0].name, "Gaming");
    assert_eq!(
        s.preferences.smart_actions[0].steps,
        vec![Step::RefreshInputs, Step::RefreshInventory]
    );
    let p: crate::model::Preferences =
        serde_json::from_slice(&serde_json::to_vec(&s.preferences).unwrap()).unwrap();
    assert_eq!(p.smart_actions, s.preferences.smart_actions);
}
#[test]
fn startup_once_input_events_and_automation_success() {
    let mut s = State::new(Default::default());
    s.management_activate(Command::CreateSmart);
    s.management_activate(Command::AddStep(Step::RefreshInputs));
    s.management_activate(Command::SaveSmart);
    s.management_activate(Command::CreateAutomation);
    let id = s.preferences.automations[0].id.clone();
    s.management_activate(Command::ToggleAutomation(id.clone()));
    s.management_activate(Command::SetTrigger(id.clone(), Trigger::Startup));
    s.system.inventory = Some(true);
    let pending = s.management_tick(Instant::now());
    assert!(s.startup_fired);
    let Effect::SmartStep { token, .. } = pending[0] else {
        panic!()
    };
    // An extra manual request cannot terminate the automation that already owns execution.
    let action = s.preferences.smart_actions[0].id.clone();
    s.start_smart(&action);
    assert_eq!(s.automation.status[&id], Status::Running);
    s.smart.reply(token, Ok(()));
    s.management_tick(Instant::now());
    assert_eq!(s.automation.status[&id], Status::Succeeded);
    s.management_tick(Instant::now());
    assert!(!s.smart.busy());
    let mut old = crate::model::demo_inventory();
    old.inputs[0].connected = Some(false);
    s.inventory = old.clone();
    let input = old.inputs[0].id.clone();
    s.management_activate(Command::SetTrigger(
        id.clone(),
        Trigger::InputConnected(input.clone()),
    ));
    // A fresh engine has no previous cooldown; actual false -> true observation triggers once.
    s.automation = Default::default();
    old.inputs[0].connected = Some(true);
    s.set_inputs(old.inputs.clone());
    assert!(s.smart.busy());
    let pending = s.management_tick(Instant::now());
    let Effect::SmartStep { token, .. } = pending[0] else {
        panic!()
    };
    s.smart.reply(token, Ok(()));
    s.management_tick(Instant::now());
    s.set_inputs(old.inputs);
    assert!(!s.smart.busy());
}
#[test]
fn remote_actions_sidebar_editor_back_and_step_scrolling() {
    use crate::state::Direction;
    let mut s = State::new(Default::default());
    s.open(2);
    s.focus = Target::Sidebar(ACTIONS_SECTION);
    s.section = ACTIONS_SECTION;
    let assets = crate::view::Assets {
        root: Default::default(),
        version: "test".into(),
        metrics: Default::default(),
        artwork: crate::app_artwork::Service::new(false),
    };
    let controls = |s: &State| {
        crate::view::scene_with_pill(s, &assets, 0., 0., time::OffsetDateTime::UNIX_EPOCH).controls
    };
    s.direction(Direction::Right, &controls(&s));
    assert_eq!(s.focus, Target::Management(Command::QuickActions));
    s.direction(Direction::Down, &controls(&s));
    s.activate();
    assert_eq!(s.subpage, Some(Subpage::Management(Panel::SmartActions)));
    s.activate();
    for _ in 0..16 {
        s.management_activate(Command::AddStep(Step::RefreshInputs));
    }
    for _ in 0..52 {
        s.direction(Direction::Down, &controls(&s));
    }
    assert!(s.scroll > 0.);
    assert_eq!(s.focus, Target::Management(Command::Back));
    s.back();
    assert_eq!(s.subpage, Some(Subpage::Management(Panel::SmartActions)));
    assert!(s.smart_draft.is_none());
}
#[test]
fn missing_disabled_and_deleted_actions_fail_closed() {
    let mut s = State::new(Default::default());
    s.management_activate(Command::CreateSmart);
    s.management_activate(Command::AddStep(Step::RefreshInputs));
    s.management_activate(Command::SaveSmart);
    s.management_activate(Command::CreateAutomation);
    let id = s.preferences.smart_actions[0].id.clone();
    let aid = s.preferences.automations[0].id.clone();
    s.management_activate(Command::ToggleAutomation(aid.clone()));
    s.preferences.smart_actions[0].enabled = false;
    s.automation_event(Trigger::Manual, false);
    assert!(!s.smart.busy());
    assert!(matches!(s.automation.status[&aid], Status::Failed(_)));
    s.management_activate(Command::DeleteSmart(id));
    assert!(!s.preferences.automations[0].enabled);
}
