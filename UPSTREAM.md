# Upstream provenance

This is **Anorak's hardened fork** of [rtk-ai/rtk](https://github.com/rtk-ai/rtk), the Rust CLI proxy that compresses dev-command output before it reaches LLM context.

## Source

Forked from [`Yui-Qi-Tang/rtk:feature/remove_security_issue`](https://github.com/Yui-Qi-Tang/rtk/tree/feature/remove_security_issue), tip commit `2058b32889f69163944cd04962d9b970c1506454`.

That branch sits 26 commits ahead of `rtk-ai/rtk:develop` with a comprehensive security-hardening pass against [issue #640](https://github.com/rtk-ai/rtk/issues/640). Highlights:

- `4b659228` — telemetry network egress hard-disabled by default (`#640 C-1`)
- `88685580` — `rtk err/test/summary` exec'd directly without `sh -c` (`#640 B-1a`)
- `afcfcb1f` — shell-executing subcommands no longer auto-allowed (`#640 B-1b`)
- `09cf2580` — `RTK_TRUST_PROJECT_FILTERS` trust bypass removed (`#640 D-1`)
- `ff3fe073` — global filter trust-gated, user-filter rewrites forbidden (`#640 A-1, D-2`)
- `41398f3a` — sensitive args redacted before tracking storage (`#640 E-1, E-2`)
- `9b8d7f00` — sensitive env vars always masked (`#640 E-3`)
- `669bde64` — `RTK_TEE_DIR` must be an absolute path (`#640 F-1`)
- `785a73ed` — OSC ANSI sequences stripped, including hyperlinks (`#640 G-1`)
- `ead87a7e` — hook-audit log rotated past 5 MB (`#640 H-1`)
- `c5adbe2b` — install.sh verifies SHA-256 checksum (`#640 A-4`)
- `a264ff70` — tracking DB perms hardened, honors `tracking.enabled` (`#640 H-1` + #1790 / #1160 / #1875)
- `43bdb13a` — tee log permissions restricted to `0600` / `0700`
- `ae1539d3` — `rtk rewrite` flag-injection in hooks prevented (#1350)
- `ccddbd63` — AWS `secretsmanager get-secret-value` redacted by default

See `DIFF.md`, `DISCOVERY.md`, and `RED_BLUE.md` (inherited from the source branch) for the full security narrative.

## What Anorak adds on top

1. Binary renamed `rtk` → `anorak-rtk` so it can coexist with upstream rtk on a developer's machine.
2. CI assertion that no network egress regresses (grep of source for `reqwest`/`ureq`/`surf`/`hyper`; binary `strings` check; runtime `lsof` check).
3. GitHub Actions release workflow producing prebuilt darwin-aarch64, darwin-x86_64, and linux-x86_64-musl tarballs with SHA-256 sidecar files.
4. Releases tagged `vX.Y.Z-anorak.N`. First release: `v0.42.0-anorak.1`.

## Divergence policy

- Cherry-pick security or correctness fixes from either `rtk-ai/rtk` or `Yui-Qi-Tang/rtk`. Never auto-merge upstream releases.
- New rtk features (filter changes, new commands) are evaluated for security impact before pulling in.
- If a fix lands upstream that we already have locally, drop ours and prefer upstream's version.

## License

Apache-2.0, inherited from upstream. `NOTICE` is preserved unchanged. Our additions are also Apache-2.0.
