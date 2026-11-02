# Smart Actions

Smart Actions are saved, typed sequences of existing safe launcher operations. Configure them in
**Settings → Actions → Smart Actions**. Home remains a launcher; pin a saved scenario through
**Add / remove Quick Action**, or use **Settings → Actions → Quick Actions**. Existing Quick
Action entries keep their `kind`/`id` representation and ordering.

## Editor

Create a scenario, enter its name with the existing native/remote keyboard, add steps, move each
step up/down, remove steps, then **Save**. Back discards the draft. Editing does not replace the
saved scenario until Save. **Run saved action** deliberately runs the saved version. The editor
supports enabled/disabled state. Deleting a scenario removes its Quick Action and disables
referencing automation rules without deleting those rules.

## Implemented steps

| Rust variant          | Serialized kind        | Behavior                                                          |
| --------------------- | ---------------------- | ----------------------------------------------------------------- |
| `LaunchApp(String)`   | `launch_app`           | Check real inventory, send existing app launch request            |
| `LaunchInput(String)` | `launch_input`         | Check real input inventory, send existing input launch request    |
| `RefreshInventory`    | `refresh_inventory`    | Read apps and inputs; fail if either inventory read fails         |
| `RefreshInputs`       | `refresh_inputs`       | Read inputs using the existing platform adapter                   |
| `OpenSystemSettings`  | `open_system_settings` | Launch the existing `com.palm.app.settings` target                |
| `Delay(u32)`          | `delay`                | Wait milliseconds using a UI scheduler deadline, without blocking |

Example preference fragment:

```json
{
  "smart_actions": [
    {
      "id": "smart.gaming",
      "name": "Gaming",
      "enabled": true,
      "steps": [
        { "kind": "launch_input", "value": "com.webos.app.hdmi1" },
        { "kind": "delay", "value": 1000 },
        { "kind": "refresh_inputs" }
      ]
    }
  ],
  "quick_actions": [{ "kind": "smart_action", "id": "smart.gaming" }]
}
```

Use an input ID actually reported by the device. This example is configuration syntax, not a
claim of hardware validation. Launch success means the existing Luna request was accepted; it
does not prove foreground visibility or video signal.

## Bounds and execution

- Maximum 32 scenarios, 16 steps per scenario, 40 characters per name, 30,000 ms per Delay.
- IDs use the existing 128-byte alphanumeric component allowlist, with `.`, `_`, `-` separators.
- Custom tiles, Clear TV itself, stock Home and scroll-preview placeholders cannot be LaunchApp
  targets. Steps cannot run another scenario, emit automation events, or execute commands.
- Invalid scenarios are disabled during sanitization; excess steps and invalid steps are removed
  from the disabled record. Re-enable only after reviewing and saving the repaired scenario.
- Unknown/malformed new preference objects are rejected individually. Other valid entries and
  legacy launcher preferences survive. Missing collections default to empty.
- One global executor; a second request is rejected. No scenario queue, retries, or parallel steps.
- State is `Idle`, `Running`, `Succeeded`, or `Failed(error)`. Failed or missing targets stop the
  remaining sequence. Completed side effects are not reversed. Worker replies carry tokens;
  stale replies cannot advance another run. Reply timeout is 45 seconds; a timed-out platform
  operation is not automatically retried and may have completed on the device.
- Desktop deliberately rejects TV steps. Local Delay and deterministic executor tests do not
  establish hardware execution. The app must remain alive for subsequent steps to run.

## Extension point

`Step` is a serializable Rust enum; the scheduler emits typed `Effect::SmartStep`, and the platform
worker returns a typed result. A new capability requires a new enum variant, validation, explicit
worker implementation, tests and a safety classification. Unknown variants fail closed.

Future, **not implemented**: picture/sound/volume/energy profiles, notifications, HTTP and Home
Assistant steps. There are no guessed Luna endpoints, generic shell steps or extension plug-ins.
The editor currently offers Delay presets of 1, 5 and 30 seconds; persisted values within the bound
are supported by the executor.
