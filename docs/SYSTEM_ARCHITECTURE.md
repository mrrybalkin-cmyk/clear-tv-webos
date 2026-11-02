# Clear TV system architecture

```text
Clear TV
├── Launcher: Apps / Inputs / Recents / Favorites
├── Actions: Quick Actions / Smart Actions / Automation
└── System: Diagnostics / Update / Recovery / Root Tools
```

The normal operating mode is a standalone application on stock LG Home. Home takeover remains
optional. Native Rust/SDL2/GLES2, package identity and the Luma platform architecture are retained.
No TV installation, SSH command or system mutation is part of local development validation.

## Data and execution

```text
Remote / pointer → State → typed Effect → platform worker → existing webOS/Luna
                      ↑                       ↓
                      └──────── typed Reply ──┘
State → view::Scene → native renderer
```

`smart_actions.rs` owns bounded typed steps and the nonblocking sequential executor;
`automation.rs` owns matching, conditions, cooldown and single-flight rule ownership;
`management.rs` owns interaction transitions, with page descriptors in `management_view.rs`;
`system_diagnostics.rs` owns optional system observations; `recovery.rs` owns recovery types and
Home reopening validation. Existing `diagnostics.rs` remains the renderer profiler.

The launcher inventory remains its existing type. System observations and runtime execution status
are separate State fields. Preferences gain defaulted collections without replacing existing
favorites, inputs, custom tiles or Quick Actions. Persistence uses existing atomic storage.

TV work stays on the existing platform worker. Delay uses scheduler deadlines. Success/failure
replies advance or terminate a scenario; tokens reject stale completions. Automation always calls
this executor, with no second execution path. No unconfirmed system APIs are implemented.

## System pages

Open **Service → Diagnostics / Recovery / Root Tools**. These pages live outside Home content.
Diagnostics show the running version, last observed installed-app/input counts, inventory outcome,
input refresh outcome, updater state, last known update result, observed Home takeover status and
presence of packaged recovery helper/scripts. `Idle` updater state is shown as an unavailable last
result, not a successful update. Desktop/failed/unsupported system observations show Unavailable.
Desktop demo inventory is not reported as observed TV counts. Counts exclude custom and preview app tiles. File presence establishes availability, not health
or proof that recovery was tested. Home enabled/mounted disagreement stays Unavailable.

Root Tools currently allow only `HomeStatus`. It calls the existing native helper's fixed
`home status` operation through the existing Homebrew privilege boundary, validates the service
and helper replies, then parses typed observations. The helper status response now carries
`returnValue: true`, matching the existing strict platform JSON envelope parser. There is no
user-provided operation, argument list, script path or command string.

## Recovery and safety

| Area                 | Implemented operation                                                                                 | Category                           |
| -------------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------- |
| Application recovery | Refresh installed inventory; refresh inputs                                                           | Safe                               |
| Home recovery        | Reopen the observed stock Home target, only after a fresh worker status read proves takeover inactive | Safe                               |
| Update recovery      | Inspect existing updater result and packaged recovery components                                      | Safe observation                   |
| Root Tools           | Native helper HomeStatus read                                                                         | Privileged, explicit fixed handler |

Home recovery rejects active, inconsistent, unknown takeover and unknown Home identities. It does
not disable takeover or modify mounts. Disabled takeover does not prove every unrelated third-party
Home modification is absent. Destructive operations, arbitrary shell/root execution, factory reset,
backup deletion and KERNEL/ROOTFS/TVSERVICE changes are not implemented. Preference reset and
in-app takeover disable are deferred; existing manual recovery instructions remain authoritative.

## Update boundary

Existing SHA-256 verification, staging, version/installer validation, previous-takeover-aware
Home reapply, failure recovery and packaged `self-update.sh` remain unchanged. Public upstream
discovery and installation remain disabled. No Luma feed is connected or re-enabled. A fresh
standalone run without a known update result reports Unavailable. Existing recovery scripts do
not imply restoration of a previous IPK; Home recovery and application-version rollback differ.

## Validation boundary

Unit tests cover model/serde bounds, legacy/damaged preferences, step sequencing/errors/timeout,
missing targets, Quick Actions, trigger conditions/cooldown/cycle/single-flight outcomes, system
reply handling and the root/recovery allowlist. Native smoke exercises remote creation, naming,
adding steps, saving/reloading and diagnostics with isolated preferences. Desktop never claims
TV steps executed. ARM packaging verifies the actual IPK and includes `self-update.sh`.

See [Smart Actions](SMART_ACTIONS.md), [Automation](AUTOMATION.md) and the exact manual test
procedure in [TV runbook](TV_RUNBOOK.md). Future picture/sound/HA/system-write capabilities must
have explicit typed implementations and independent hardware evidence before being advertised.
