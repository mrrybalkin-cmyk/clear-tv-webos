# Hardware Profile

## Known Good Hardware

Model: OLED42C6RLA

Platform: LG webOS 26

webOS version: 11.2.0-3512

Firmware: 43.21.79

## Validation status

| Capability | Status |
|---|---|
| Root access | PASS |
| Root after reboot | PASS |
| SSH key authentication | PASS |
| Homebrew Channel | PASS |
| Luma installation | PASS |
| Native Luma rendering | PASS |
| Application inventory | PASS |
| HDMI inputs | PASS |
| Remote arrows | PASS |
| Remote OK | PASS |
| Remote Back | PASS |
| Autostart | PASS |
| Home takeover | PASS |

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
