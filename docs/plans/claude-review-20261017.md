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
- H3 (in-repo parts): Disable fsmonitor/hooks on Jenkins Git entry points, pin
  the delivery import and execute the verified bytes without rereading. Narrow
  the canonical Windows profile: Git data, Jenkins runtime/state, workflows and
  executable helper directories are read-only. Commit the profile through the
  index while preserving the preexisting Linux overlay. 29 focused Python tests
  passed (four Windows-only cases skipped); 205 production Python files parsed.
  Skipped: relocating the live Jenkins install/authorization DB, immutable
  external bootstrap and interpreter pinning; those need operator installation
  changes. No service configuration or runtime was modified or activated.
- H4: Self-hosted jobs run only on main/master pushes; PR source contracts use
  GitHub-hosted runners. Limit tokens to contents:read and disable persisted
  checkout credentials. Add workflow concurrency and fresh run/attempt/job/matrix
  roots for Cargo, rustup, temp, targets, Miri and Zig. Physical/reparse guards
  reject reused output roots. 48 focused Python contracts passed. YAML parser
  unavailable here; actual Windows runner execution remains pending.
- M1: Parse WebSocket URIs with the connection library's HTTP URI parser; policy,
  pin host and connection share the parsed URI. Normalize schemes and hosts.
  Add query/fragment loopback-spoof and uppercase-scheme regressions. Source
  wiring checks passed; Rust regressions await the guarded build lane.
