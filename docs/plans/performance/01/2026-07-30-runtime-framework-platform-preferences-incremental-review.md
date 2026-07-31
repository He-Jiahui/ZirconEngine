# Runtime framework Platform preferences current-source incremental review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Accounting: keep in `pending.md`; do not add to `review.md` before current-source managed Cargo and filesystem/product counters are GREEN.
- Code disposition: no Rust source was changed. All dirty and untracked Platform preference sources were preserved as external work.

## Exact scope

| module or file | files | physical lines | tests | current-source fingerprint |
|---|---:|---:|---:|---|
| `zircon_runtime/src/core/framework/platform` | 9/9 | 283 | 0 | `10099ac60aeea2995699a1029076b10b62d4e55bfa52ad8e84b8f5a188691fbf` |
| `zircon_runtime/src/platform/{preferences,service_types}` | 6/6 | 427 | 1 | `f3ba1fcf5b69c469de39cf634f3299c96cd7b8b244e125de9fae129b61018d7d` |
| `zircon_app/src/entry/platform_preferences.rs` | 1/1 | 291 | 6 | `894403cab324a1dcef52ce052588f9727311c88b33d6714f2e28eee339b897f2` |
| `zircon_runtime/src/platform/tests/preferences.rs` | 1/1 | 216 | 7 | `c53e3e87a0661e590568de2c79925231ee530bf2ae6d682785ada53d948e1b41` |

All 17 exact owner/test Rust files were read after their current hashes changed during review. The shared `zircon_runtime/src/foundation/persistence/atomic_file.rs` commit path and WOC's preference, keybind and gamepad storage files were also read as support/consumer evidence. The older bridge review's Platform `3/3` count describes the pre-preferences tree and is corrected to current `9/9` here.

## Confirmed performance boundary

1. `PreferenceStorage` and `PreferenceStorageBackend` expose synchronous `read`, `write`, `remove` and `flush`. `PlatformManager` clones the backend `Arc` under a short `RwLock` guard and invokes it after the guard is dropped. This is a sound lock boundary, but it leaves all backend latency on the caller.
2. The desktop atomic-file backend rebuilds two BLAKE3 digests, two hex `String`s and a `PathBuf` for each operation. Reads call `fs::read`. Writes run the shared staged atomic writer, which writes, flushes and `sync_all`s staging, then reopens and `sync_all`s the committed target and, on Unix, its parent directory. Remove also syncs the parent on Unix.
3. The post-commit sync work is tied to the durability contract: the shared writer syncs staging and performs the atomic replace but does not itself sync the Unix parent after rename. It is therefore not a safe trivial deletion without first changing and testing crash semantics.
4. No production engine preference read/write consumer is currently wired. WOC still owns a local `PreferenceStorage` trait and its keybind/gamepad stores use that trait; current engine uses are host assembly and tests. The synchronous cost is consequently a latent pre-wiring risk, not a measured current frame hotspot.

## Plan and acceptance

- `PERF-MVP-589` / Frameworks05 with Runtime11/Runtime02: before WOC or Editor wiring, put blocking persistence behind the shared bounded execution authority; coalesce same-key writes by latest generation; publish read-your-write state; provide bounded flush/shutdown fences; cache canonical key hash/path derivation; keep host-provided backends off frame/UI callers unless explicitly affinity-safe.
- Dynamic matrix: keys `1/1k/100k`, values `0/1KiB/1MiB`, same-key bursts `1/1k/1M`, filesystem latency `0/10ms/2s`, writers `1/16`. Record caller filesystem wall, hash/path builds, staged writes/fsyncs, queue entries/bytes/oldest age/coalesce/drop, flush/shutdown time, RSS and p95. Require frame/UI caller filesystem wall `0`, bounded memory and durable writes, read-your-write, crash old-or-new visibility and explicit error/retry/cancel outcomes.

## Reference check

- Godot `dev/godot/editor/settings/editor_settings_dialog.cpp` restarts a one-shot 1.5-second timer when settings change and saves on timeout. This supports burst coalescing before persistence, but its synchronous save is not a complete Zircon worker/affinity design.

## Static gates executed

- Read 17/17 exact owner/test Rust files plus the shared atomic writer and three WOC consumer-contract files.
- `rustfmt --check --edition 2021` passed the 9 framework files, 6 runtime implementation files and the runtime preference test. The externally dirty App host file has current import/assert formatting drift, so the complete exact scope is not formatting GREEN.
- No managed Cargo, filesystem latency/fault injection, ETW/WPR product trace or scale counter ran. RenderDoc is not applicable to this non-rendering persistence slice. The module remains pending.
