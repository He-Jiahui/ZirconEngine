# Claude engineering review repairs

Scope: the 2026-10-17 review, on `fix/claude-review-20261017` only.
No push, PR, sibling ZrVM changes, or Jenkins installation changes are authorized.
The preexisting Linux `.codex/config.toml` overlay is preserved.

## Validation limits

This Linux checkout has no verified Jenkins activation, Windows drive-root build
storage, the pinned Rust toolchain, or PowerShell. Python and source checks run locally;
compiled Rust, TLS integration and Windows acceptance remain pending unless
explicit evidence below says otherwise. No build outputs are created here.

## Items

- C1: Separate source contracts from Rust compilation. Use one per-member fmt
  contract, refresh stale source assertions, add path-budget tests, verify
  referenced unittest modules are tracked, and grandfather only review-baseline
  documentation/test-layout debt. The baseline reads Git objects at the reviewed
  revision; CI fetches full history. Existing debt stays visible in reports.
  Validation: 61 focused Python tests passed; docs and test-layout gates report
  zero new violations (36,858 and 63 existing respectively). Nine existing path
  budget violations and 30 existing layering edges are also explicitly baselined.
- H1: Remove managed-storage enforcement from the reusable interface build script;
  retain the generated availability fallback. Move the profile contract workflow
  onto the guarded Windows output lane with Rust 1.94.1. Build-script and workflow
  source checks passed; cross-platform Cargo validation requires approved storage.
- H2: HTTP reuses the rustls handshake verifier (chain, hostname, pin), forbids
  pinned redirects and plaintext TLS material. Replace the old untrusted/mismatched
  hostname fixture with a trusted loopback CA/leaf. Add wrong-pin/no-request
  regression and retain a successful pinned request. Fixture and source checks
  passed; compiled TLS tests are pending the Windows validation lane.
