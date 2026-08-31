# Runtime25 Filesystem Capability Truth Freeze

- Date: 2026-08-23
- Owner: `optimize-runtime25-filesystem-capability-truth-freeze-r1-20260823`
- Source plan: `docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md`
- Findings: `FILESYSTEM-P1-011`, `FILESYSTEM-P1-012`
- Status: source implementation complete; managed validation pending

## Current-Source Review

The review was repeated at baseline HEAD
`471bb732e3683fd7c12d7b69a9e85a22048efcba` before changing production code.

| Surface | Current source | Consumer evidence | M0 decision |
|---|---|---|---|
| `ResourceIo` | Three synchronous `read/write/exists` methods | Zero implementations and zero consumers outside its definition/re-exports | Keep the planned name, but seal it as an unimplemented capability until M2 provides a real provider |
| `AssetIoDriver` | Empty public unit struct registered as an immediate driver | Zero service dependencies and zero runtime consumers | Make the type uninhabited and remove the driver descriptor |
| Asset module description | Descriptor claims asynchronous I/O while `EngineModule` describes project/import/index work | The module actually owns project/import/resource managers | Use one private description authority for both projections and state only implemented capability |

The production-like direct-filesystem inventory currently contains 236 Rust
files under the five product roots scanned with
`std::fs|fs::|File::(open|create)|OpenOptions::`: Runtime 108, Editor 79,
Hub 29, App 12, and plugins 8. Runtime contributes 40 asset files and 12 core
files. This is an M0 routing inventory, not a claim that all direct calls are
invalid: provider backends, durable transactions, and fault tests remain
approved low-level owners.

## Reference-Engine Decision

- Unreal exposes a real `IPlatformFile` chain with concrete open, move,
  delete, directory, async-read, and lower-level provider contracts. The
  platform-file manager resolves installed providers; type existence alone is
  not a capability receipt.
- Bevy registers `AssetSource` instances that contain actual reader, optional
  writer, watcher, and processed-source objects. Capability absence is explicit.
- Fyrox's `ResourceIo` has a concrete `FsResourceIo` implementation and is
  consumed by resource loading. Zircon's same-named dead trait must not be
  presented as equivalent.

M0 therefore freezes capability truth without adding a second placeholder
facade. It does not change `ProjectPaths`, watcher, atomic-file, durable
transaction, importer, or project-manager behavior. M2 remains responsible for
the local provider, source/mount registry, real asset-reader consumption, and
final deletion of the temporary sealed surface.

## Acceptance Contract

- The Asset module descriptor has zero drivers and retains its three real
  managers.
- Its description contains no asynchronous-I/O claim.
- `AssetIoDriver` is uninhabited and cannot be constructed or registered.
- `ResourceIo` cannot be implemented outside its private M0 capability seal.
- No new compatibility alias, facade re-export, filesystem behavior, or Cargo
  dependency is introduced.

## Quantified Impact

The cut removes one false driver descriptor, one service registration/index,
one immediate empty factory invocation, and one empty `Arc` service allocation
per Asset-module lifecycle construction. It changes no per-frame path and makes
no throughput, latency, or power claim. Those measurements begin only after M2
has a real provider and a reproducible asset-read workload.

## Status And Output Record

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| M0 review | Current source, direct-I/O inventory, Unreal/Bevy/Fyrox routing | `review_complete` | 2026-08-23 | 0 implementations/0 consumers; 236 production-like direct-I/O files |
| M0 truth freeze | Descriptor, unimplemented driver/resource-I/O boundaries | `implementation_complete_validation_pending` | 2026-08-23 | Static truth guard, exact rustfmt check, and scoped diff check pass; managed validation and independent review pending |
