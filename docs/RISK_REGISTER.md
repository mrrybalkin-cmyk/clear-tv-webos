# Risk Register

| Risk | Impact | Control |
|---|---|---|
| Home launcher fails | High | Keep SSH recovery and home.sh disable |
| Root persistence fails | High | Verify root after reboot |
| Upstream conflict | Medium | Keep Clear TV changes isolated |
| System partition damage | Critical | Never overwrite system partitions |
| Bad update package | High | Add SHA-256 verification |
| Broken configuration | Medium | Keep rollback copies |
| Watchdog restart loop | High | Add bounded retries |
| Private data committed | High | Use placeholders and ignored local config |
| TV-specific behavior generalized | Medium | Validate each model separately |
| Large fork divergence | High | Prefer small upstream-compatible changes |

## Current position

No critical blocker is known on OLED42C6RLA.

The current Luma baseline works correctly on the validated television.
