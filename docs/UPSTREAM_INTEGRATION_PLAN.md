# Upstream Integration Plan

## Repositories

Origin:

    mrrybalkin-cmyk/clear-tv-webos

Upstream:

    ecklf/lg-luma

## Policy

Keep upstream as a read-only integration source.

Do not push to upstream.

Prefer feature branches for Clear TV development.

## Update procedure

Fetch upstream:

    git fetch upstream

Inspect divergence:

    git log --oneline --graph --decorate --all

Review upstream changes before integration.

Do not merge blindly.

Test launcher behavior after integration.

Test Home takeover on hardware after relevant changes.

## Conflict priority

Preserve these upstream properties first:

1. native Rust launcher architecture
2. webOS compatibility
3. input handling
4. packaging
5. Home integration
6. recovery behavior
7. CI tests

Clear TV customization must not weaken these properties.

## Fork strategy

Prefer additive changes.

Avoid replacing stable upstream subsystems.

Keep management services separate from rendering.

Document intentional divergence.
