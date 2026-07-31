# Editor core script build current-source review

## Status

- Result: `static_complete / dynamic_pending / product_integration_pending`.
- Review date: 2026-07-30.
- Owners: Editor13 owns orchestration; Editor14/Runtime11 own bounded job execution; Runtime13 owns VM compilation; Editor04 owns Play resume; Editor16 owns commandlet integration.
- Accounting: keep `zircon_editor/src/core/script_build/**` in `pending.md`. No production caller exists beyond the module export, so this is an integration capacity gate rather than a measured current UI hotspot.
- Code disposition: no Rust source was changed. The current untracked module was preserved.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/script_build/**` | 4/4 | 912 | 13 | `3e664862dfc34c843495b2c5e218152b37fb7a753ad67b13414c7bf4bbc411a3` |

The fingerprint streams each sorted native workspace-relative path, a zero byte, raw file bytes and a zero byte into SHA256. All four files were read in full. Repository-index caller search found only `core/mod.rs` exporting the module; watch, command, Play, jobs, VM, diagnostics and commandlet product adapters remain absent as already stated by Editor13.

## Per-file review

| file | current-source performance result |
|---|---|
| `mod.rs` | Module wiring and exports only. |
| `orchestrator.rs` | Unique watch paths are hard-bounded at 20 and collapse to a full-rebuild sentinel; last outcome is shared by `Arc`. Remaining sliding debounce can be postponed forever, while Command/Play requests append to an unbounded FIFO with no generation, entry/byte/age budget, coalescing, cancellation or supersede. A failure also clears changes observed while the active build ran. |
| `request.rs` | Every request heap-allocates a fixed three-element step Vec. Dispatch clones the active step, including up to 20 PathBuf values for incremental compile. This is bounded per request but multiplies with the unbounded FIFO; a fixed step layout and generation-owned shared path batch remove it. |
| `tests.rs` | Thirteen tests cover debounce, 20/21/10K path bounds, explicit flush, step order, Play resume, failure clearing, stale completion and request-id exhaustion. No continuous-watch max latency, 1M trigger storm, queue memory/age, coalescing/cancel or slow-job integration test exists. |

## Corrected and remaining task

### PERF-MVP-557

The former unbounded watch-path and snapshot deep-clone findings are fixed. Current source retains at most 20 paths plus a sentinel, and snapshots share the last outcome. Remaining work is the unbounded Command/Play FIFO, indefinitely sliding deadline, repeated fixed-step/path ownership, and lack of source generation/cancel/supersede. Because no product consumer exists, Editor13 must solve these before M2/M3 wiring rather than hiding them behind a larger queue or private worker.

## Acceptance plan

- Watch: paths 1/20/21/10K, event rate 1/60/1K Hz for 60 seconds. Record resident paths/bytes, first-event age, sliding deadline and emitted build generations. Resident paths stay at 20 plus sentinel and max latency is bounded.
- Explicit triggers: Command/Play 1/1K/1M during idle/active/failure. Record queued entries/bytes/age, fixed-step/path clone bytes, compilations per source generation, observers and resume intent. Same generation compiles once; queue memory is hard-bounded and latest Play intent is retained.
- Execution: compile latency 0/16ms/10s, success/failure/cancel/supersede. Run through Editor14's shared job owner with one `script_artifacts` mutex group; UI callback/process/I/O wall is zero and stale completions cannot refresh bindings or resume Play.
- Run current-source managed 13 orchestrator tests, then VM/job/Play/commandlet integration and F4 save-storm/build/Play traces. No render path exists in this module, so RenderDoc is not applicable.

## Reference check

- Fyrox `dev/Fyrox/editor/src/lib.rs` keeps an ordered build command queue, one child owner and `play_after_build` in one mode. Zircon should retain that single lifecycle authority while adding admission bounds and generation coalescing.
- Unreal `dev/UnrealEngine/Engine/Source/Developer/HotReload` separates the reload mechanism from editor request orchestration; Zircon must not treat compile completion as safe state migration without Runtime13's explicit snapshot/replay contract.
- Bevy `dev/bevy/crates/bevy_tasks/src/usages.rs` separates cross-frame CPU and I/O task owners. Script compile and artifact I/O should reuse Editor14/Runtime11 queues, not create a ScriptBuild-private pool.

## Static gates executed

- Read all current 4/4 Rust files; caller search confirmed product integration is absent.
- `rustfmt --edition 2021 --check` passed all four current files.
- Per-file `git diff --no-index --check` reported zero whitespace errors for the untracked tree.
- `review.md` remained unchanged. No managed Cargo, queue/latency/allocation scale run, VM/job/Play/commandlet execution, WPR F4 trace or independent dynamic review ran.
