# Clear TV WebOS Project

## Objective

Build a clean custom Home experience for rooted LG webOS televisions.

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
- Luma Home takeover: verified

This configuration is the Known Good Hardware baseline.

## Architecture

Keep the upstream native architecture:

- Rust
- SDL2
- OpenGL ES 2
- Luna services
- native webOS integration

Do not replace the launcher with React or a browser application.

The product architecture has two logical layers.

### Launcher layer

Based directly on Luma.

Responsibilities:

- Home interface
- installed applications
- HDMI inputs
- navigation
- favorites
- recent applications
- settings
- visual customization

### Management layer

Add this separately.

Responsibilities:

- diagnostics
- update verification
- SHA-256 verification
- rollback
- health checks
- watchdog
- safe deployment

Do not couple management logic tightly to rendering code.

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

Home takeover is already verified on the primary television.

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
