# TV Runbook

## Purpose

Use this runbook for safe manual testing on a rooted LG webOS television.

Never store a real television IP address in this repository.

## SSH

Use a local address only.

Example:

    ssh -i $HOME\.ssh\id_ed25519 root@<TV_IP>

## Luma location

Expected application directory:

    /media/developer/apps/usr/palm/applications/com.ecklf.luma

## Check Home takeover

    sh /media/developer/apps/usr/palm/applications/com.ecklf.luma/home.sh status

## Disable Home takeover

    sh /media/developer/apps/usr/palm/applications/com.ecklf.luma/home.sh disable

## Enable Home takeover

Only enable after a rollback path is confirmed.

    sh /media/developer/apps/usr/palm/applications/com.ecklf.luma/home.sh enable

## Check autostart

Use the packaged Luma autostart script.

Do not modify webOS system boot files manually.

## Hardware verification

After launcher changes, verify:

1. Home opens correctly.
2. Installed applications appear.
3. HDMI inputs appear.
4. Arrow navigation works.
5. OK works.
6. Back works.
7. Home to HDMI to Home works repeatedly.
8. Reboot does not break Home.
9. SSH recovery remains available.

## Emergency rule

Do not overwrite:

- KERNEL
- ROOTFS
- TVSERVICE

## Actions/System MVP verification on OLED42C6RLA

Status: **NOT PERFORMED**. Model OLED42C6RLA, webOS 26 / 11.2.0-3512, firmware 43.21.79 is the
recorded baseline; read the TV's actual values before testing. Installation and any SSH/root
operation require separate authorization. No installation is authorized by this checklist.

After a separately authorized installation of the final verified ARM IPK:

1. Record package filename, SHA-256, commit, TV model/webOS/firmware, and whether stock Home is
   active. Keep existing backups. Do not enable takeover, reset preferences, modify partitions
   or change autostart for this test. Confirm stock LG Home is still the primary Home screen.
2. Open Clear TV as a standalone app. Check existing Favorites order, Apps launch, Recents,
   Inputs, Quick Actions add/remove/reorder and Back/OK/arrow behavior against the prior baseline.
3. Go to Settings → Actions → Smart Actions → Create. Enter `Gaming test` using the remote/native
   keyboard. Add Launch HDMI 1 (use the actual inventory entry), Delay 1 second, Refresh inputs.
   Move the last step up then down; remove and re-add Delay. Save. Reopen and verify name/order.
4. Use Run saved action. Observe HDMI launch. Return to Clear TV and inspect the execution status:
   Succeeded is expected only after all steps completed; record Failed verbatim if TV permissions
   or lifecycle prevent completion. Verify remote UI remains responsive during a 30-second Delay
   scenario using Refresh inputs → Delay 30 seconds → Refresh inputs. A second launch while
   Running must be rejected; verify one sequence completes, with no duplicate execution.
5. Add Gaming test to Quick Actions; launch from Home. Reorder it among existing Quick Actions.
   Close and reopen Clear TV; verify both configuration and order persist. Disable Gaming test,
   try its Quick Action, and confirm controlled rejection without a TV launch. Re-enable after review.
6. Create a saved `Refresh test` Smart Action with Refresh inventory → Delay 1 second → Refresh
   inputs. Create an automation targeting it: Manual, Always, enabled. Emit Manual and verify
   Running then Succeeded, or record the precise failure. Emit again immediately: cooldown must
   prevent a new run. Wait 31 seconds, emit again: one run may start. Disable rule, wait 31 seconds,
   emit: no run. Keep only the rule under test enabled.
7. Configure the rule as InputConnected for the actual HDMI 1 ID, Always. With Clear TV visible
   and focused on Home, unplug/replug the HDMI cable. Allow at least one successful input poll
   before each change. Verify a single run for false → true; a steady cable must not repeat it.
   Repeat with InputDisconnected. Test InputIsConnected using the matching input. If snapshots
   are unknown or polling is suspended, record Unavailable/missed observation, not a fabricated pass.
8. Set Startup + Always, wait for any active run to finish, then close and relaunch Clear TV.
   Verify one execution after the first successful inventory. Do not call this a cold-boot trigger.
   For TimeRange, use the evening preset only when the actual TV local time lies in 18:00–23:00;
   outside the range, verify no launch. If local offset is unavailable, verify fail-closed behavior.
9. Open Service → Diagnostics and refresh observations. Compare Clear TV version and installed
   app/input counts with the UI inventory. Verify read outcomes, disabled public updater, honest
   last-known update result, takeover status and recovery component availability. Failure to read
   a component must show Unavailable. Root Tools → HomeStatus must only read status.
10. Under Recovery, refresh applications and inputs. With takeover confirmed inactive, reopen stock
    Home and verify LG Home appears. With unknown status, reopening must fail safely. Do not enable
    takeover merely to test this gate; active-takeover rejection is covered by deterministic tests.
11. Confirm stock Home and all existing launcher functions remain unchanged. Disable test rules,
    remove test scenarios and their Quick Actions through the UI, and verify existing preferences
    remain. Record PASS/FAIL/Unavailable separately for each item, with screenshots and failures.

No package update, root mutation, takeover enable/disable, SSH session, factory reset, backup
removal or system partition change is part of this procedure. Update installation/recovery and
active-takeover recovery require their own separately authorized hardware test plan.
