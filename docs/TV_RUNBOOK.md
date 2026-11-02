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
