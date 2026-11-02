# Automation

Configure the small deterministic engine under **Settings → Actions → Automation**. Create a
rule, enable it explicitly, choose trigger/condition and select a saved Smart Action. New rules
are disabled, use Manual + Always and a 30-second cooldown. The editor has no workflow graph.

```text
Observed event → enabled rule + condition + cooldown → existing Smart Action executor
```

## Observed triggers

| Trigger           | Actual source                                                   | Limit                                 |
| ----------------- | --------------------------------------------------------------- | ------------------------------------- |
| Manual            | Explicit **Emit Manual event** UI action                        | Evaluates Manual rules in saved order |
| Startup           | First successful app inventory in this Clear TV process         | Once per process, not TV power-on     |
| InputConnected    | Known false → true cable state between accepted input snapshots | Sampled while existing refresh runs   |
| InputDisconnected | Known true → false cable state between accepted input snapshots | Sampled while existing refresh runs   |

Initial snapshots, missing inputs and `None` connection states do not create transitions. Signal
changes are not cable transitions. Input polling retains the existing focus/visibility/input-page
rules: this is not a persistent background TV service, and events while Clear TV is hidden,
terminated or between samples can be missed. AppStarted is **not implemented**: a successful launch
reply is not proof of an observed start or foreground application.

## Conditions

- Always.
- InputIsConnected: the input exists and its observed cable state is `Some(true)`.
- TimeRange: half-open local minute range, supports crossing midnight; both ends are 0–1439 and
  must differ. A missing local offset fails closed. The offset is captured before workers start,
  like the existing launcher clock; restart Clear TV after timezone/DST changes. No UTC fallback
  is substituted for an unavailable local offset in condition evaluation.
- AppIs: typed model and evaluator accept an observed app ID. There is currently no reliable
  foreground observer connected to the engine; runtime context is unavailable, so this condition
  never passes and is not offered in the MVP picker.

The UI offers Always, observed inputs and an evening TimeRange preset (18:00–23:00). Other valid
time ranges can be persisted but do not yet have a custom-time editor.

## Containment and outcomes

At most 32 rules. IDs/names follow Smart Action limits; cooldown must be 5–86,400 seconds.
Invalid rules are disabled. Unknown serialized variants are rejected; no user string is executed.

Rules are evaluated in preference order. The first eligible rule owns the single executor; other
matches and events arriving during execution are dropped, not queued. Cooldown uses a monotonic
clock from attempt start, including failed attempts. Cooldowns and run status are process-local;
they reset when Clear TV restarts. Startup is consequently evaluated once in each new process.

There is no action to emit triggers or recursively run scenarios. Script-produced inventory
replies arrive while the executor is busy and cannot start automation, preventing feedback loops.
Repeated identical input snapshots generate no events. A disabled, missing or invalid Smart
Action produces Failed; a failed step ends the rule's run; success is reported only after the
executor completes every step. No retries are performed. The UI shows Idle/Running/Succeeded/Failed.

Future, **not implemented**: reliable application lifecycle observer, additional triggers, arbitrary
time editing, persistent cooldown history and a visual editor. All automations and platform steps
introduced in this change still require physical testing on OLED42C6RLA.
