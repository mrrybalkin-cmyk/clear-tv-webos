# Clear TV WebOS Project

## Objective

Build a standalone launcher, actions and controlled system-management platform for rooted LG webOS televisions.

The project uses `ecklf/lg-luma` as its technical base.

Preserve compatibility with upstream Luma where practical.

## Verified hardware

Primary validated device:

- Model: OLED42C6RLA
- Platform: webOS 26
- webOS version: 11.2.0-3512
- Firmware: 43.21.79
- Root access: verified
- Root persistence: verified
- SSH key access: verified
- Homebrew Channel: verified
- Luma native UI: verified
- Installed app inventory: verified
- HDMI inputs: verified
- Remote navigation: verified
- Luma autostart: verified
- Luma Home takeover capability: verified
- Current operating mode: standalone application on stock LG Home

This configuration is the Known Good Hardware baseline.

## Architecture

Keep the upstream native architecture:

- Rust
- SDL2
- OpenGL ES 2
- Luna services
- native webOS integration

Do not replace the launcher with React or a browser application.

The implemented product has three logical levels:

- **Launcher:** Apps, Inputs, Recents and Favorites, retaining the Luma native UI.
- **Actions:** Quick Actions, bounded sequential Smart Actions and deterministic Automation.
- **System:** Diagnostics, existing safe Update mechanism, controlled Recovery and allowlisted Root Tools.

Home remains the launcher. Configuration is under Settings → Actions; system pages open through
Service. Standalone operation on stock LG Home remains the recommended mode. Public upstream
update checks remain disabled; no public Luma feed is connected.

See [System architecture](SYSTEM_ARCHITECTURE.md), [Smart Actions](SMART_ACTIONS.md) and
[Automation](AUTOMATION.md) for implemented behavior and explicitly deferred capabilities.
New platform features have local unit/render/package validation only until the runbook is
completed on physical hardware. Earlier hardware results below apply to their recorded baseline.

## Safety rules

Never overwrite these system partitions:

- KERNEL
- ROOTFS
- TVSERVICE

Do not modify signed system files permanently.

Prefer runtime mounts and reversible changes.

Do not enable destructive operations automatically.

Create a rollback path before every privileged change.

Do not commit television IP addresses.

Do not commit device identifiers.

Do not commit passwords.

Do not commit SSH private keys.

Do not commit API tokens.

Use placeholders for local network configuration.

## Home takeover

Home takeover capability is verified on the primary television.

The current recommended operating mode keeps stock LG Home active.

Clear TV runs as a standalone application.

Home takeover remains optional and disabled by default during current development.

Do not change takeover logic without explicit project need.

The primary rollback command is:

    sh /media/developer/apps/usr/palm/applications/com.ecklf.luma/home.sh disable

Reboot after rollback when required.

## Development rules

Make small changes.

Test one behavior at a time.

Keep upstream changes easy to merge.

Avoid large rewrites without technical justification.

Preserve upstream tests.

Add tests for new behavior where practical.

Do not claim television behavior from desktop tests.

Hardware behavior requires manual television verification.

## Upstream

`origin` is the Clear TV WebOS fork.

`upstream` is `ecklf/lg-luma`.

Upstream push must remain disabled.

Fetch upstream changes before integration work.

Never force-push upstream.

## Initial product scope

Phase 1:

- preserve stable Luma behavior
- define Clear TV visual direction
- simplify Home
- improve navigation
- define management architecture

Phase 2:

- diagnostics
- safe updater
- checksum verification
- rollback
- watchdog

Phase 3:

- optional integrations
- Home Assistant
- notifications
- selected system cleanup

Debloat is not an MVP requirement.
