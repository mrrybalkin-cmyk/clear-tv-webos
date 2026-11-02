# Hardware Profile

## Known Good Hardware

Model: OLED42C6RLA

Platform: LG webOS 26

webOS version: 11.2.0-3512

Firmware: 43.21.79

## Validation status

| Capability               | Status |
| ------------------------ | ------ |
| Root access              | PASS   |
| Root after reboot        | PASS   |
| SSH key authentication   | PASS   |
| Homebrew Channel         | PASS   |
| Luma installation        | PASS   |
| Native Luma rendering    | PASS   |
| Application inventory    | PASS   |
| HDMI inputs              | PASS   |
| Remote arrows            | PASS   |
| Remote OK                | PASS   |
| Remote Back              | PASS   |
| Autostart                | PASS   |
| Home takeover capability | PASS   |
| Standalone app mode      | PASS   |

## Notes

The exact OLED42C6RLA model was validated manually.

Do not generalize this result to other LG models.

Each additional television model requires its own hardware validation.

## Clear TV hardware validation

Date: 2026-11-02

Branch: feat/clear-tv-branding

Result: PASS

Validated on OLED42C6RLA:

- custom Clear TV IPK installed successfully
- Clear TV branding rendered correctly
- application inventory works
- HDMI inputs work
- remote arrows work
- OK works
- Back works
- Home takeover works
- HDMI to Home works
- cold boot restores Clear TV Home
- SSH recovery remains available
- stock Luma backup verified by SHA-256

Rollback remains available through the saved stock backup and `home.sh disable`.

## Current operating mode

Date: 2026-11-02

Branch: feat/clear-tv-home-ui

Current configuration:

- stock LG Home remains active
- Clear TV runs as a standalone application
- Home takeover is disabled
- remote control works normally inside Clear TV
- configurable Quick Actions passed hardware validation
- Quick Actions navigation and reordering passed hardware validation
- Home takeover remains available as an optional reversible mode

A Magic Remote pairing loss occurred during testing.

Remote operation was restored by re-registering the Magic Remote.

No evidence currently links the pairing loss to Quick Actions.

## Actions/System MVP hardware gate (not yet performed)

The Smart Actions executor, input-transition/Startup/Manual automation, Diagnostics, Recovery UI
and HomeStatus Root Tool added after the recorded baseline have **not** been physically tested.
Local native renders, unit tests and ARM IPK verification cannot change this status.

Use the exact [OLED42C6RLA procedure](TV_RUNBOOK.md#actionssystem-mvp-verification-on-oled42c6rla).
Retain standalone mode and stock LG Home throughout. Do not infer that input events are delivered
while Clear TV is hidden or terminated. New management labels are currently English; existing
launcher localization remains unchanged.
