---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-11
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: W5 NET-P0-005 WebSocket security policy fail-close
session: astra-w5-network-wss-verifier-20260918-r1
---

# W5 WebSocket certificate policy enforcement

The WebSocket backend now admits certificate pinning and custom-root
configuration only through a rustls connector that enforces both policies
during the TLS handshake. `validate_websocket_security_policy` still rejects
missing host pins before request creation; a configured pin or root set builds
a `WebPkiServerVerifier` (with the public WebPKI roots when no custom roots are
provided), and the pin verifier checks the leaf certificate before the
connection is returned. Ordinary WSS continues to use the backend's stock
connector when no extra policy is requested.

## Scope

- `zircon_plugins/net/features/websocket/runtime/src/backend/security.rs`
- `zircon_plugins/net/features/websocket/runtime/src/backend/client.rs`
- `zircon_plugins/net/features/websocket/runtime/src/tests/security.rs`
- `zircon_plugins/net/runtime/src/transport/tls.rs`
- `zircon_plugins/net/runtime/src/transport/mod.rs`
- `zircon_plugins/net/runtime/src/lib.rs`
- `zircon_plugins/net/runtime/Cargo.toml`
- `zircon_plugins/Cargo.lock`
- `docs/wiki/plugins/runtime-families.md`
- `docs/crates/zircon_plugins/net/runtime.md`

## Evidence

- `rustfmt --edition 2021 --check` passed for all touched Rust files.
- `git diff --check` passed for the touched Rust, lockfile, and plan files.
- Focused source tests cover URL host normalization, admission of configured
  pins/roots, plaintext downgrade rejection, default public-root configuration,
  and malformed-root rejection.
- The focused static regression batch ran 24 tests across plugin structure,
  manifest/catalog contracts, and network performance guards with zero
  failures; this is source/static evidence only, not a native acceptance
  receipt.
- Managed job `41573a1dfef9408b94c4af9a98dbade8` exited before compilation
  under the current workspace/external-dependency gate and emitted no Rust
  diagnostics; its retained target directory is not a passing receipt.
- The managed Cargo/native run remains pending because the external
  `E:\Git\zr_vm` worktree is dirty; no compile or product acceptance claim is
  made here.

## Remaining boundary

Real multi-process WebSocket simulation, managed Cargo acceptance, and a
Windows release handshake trace remain pending. Until those receipts arrive,
this slice stays `implemented_pending_validation`.
