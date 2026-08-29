---
handoff_kind: fixed
status: fixed
created_at: 2026-08-29
resolved_at: 2026-08-29
summary_slug: random-checkpoint-mixed-era-snapshot
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/22
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/random/authority.rs
  - zircon_runtime/src/core/runtime/random/registry.rs
  - zircon_runtime/src/core/runtime/random/service.rs
  - zircon_runtime/src/core/runtime/random/tests/service.rs
  - zircon_runtime/crates/zr_contracts/src/random/service_checkpoint.rs
tests:
  - deterministic checkpoint/reseed interleaving regression
  - RandomService checkpoint restore and next-draw replay regression
  - managed Windows zircon_runtime focused random-service test batch
---

# Runtime22: make random checkpoints atomic across seed and stream eras

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 random contract/kernel partition review
- 修复责任计划：`docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md`
- 交接原因：Runtime22 owns random checkpoint, restore, replay, reseed, and scheduling determinism.

## 失败现象与复现证据

`RandomService::checkpoint()` first obtains canonical parked-stream entries through
`RandomStreamRegistry::checkpoint_entries()`. That call releases the registry mutex before the
service calls `RandomAuthority::snapshot()` for the master seed and generation. In contrast,
`RandomAuthority::reseed()` holds the registry mutex through `clear_if_idle_with`, then locks and
updates the seed authority before clearing parked streams.

A legal schedule can therefore capture old parked streams, release the registry lock, complete a
reseed, and finally snapshot the new seed and generation. The returned checkpoint combines two eras
and never existed as an atomic service state. `RandomServiceCheckpoint::try_new` correctly validates
algorithm agreement and canonical key ordering, but stream checkpoints do not carry the seed
generation, so the contracts layer cannot reject this structurally valid mixed-era value.

Delegated source hashes at discovery were:

- `authority.rs`: `c36e40928be115cbc366d9c3465deb78204e4984329a696d391830efdc297cbc`
- `registry.rs`: `6836b35717592002215e80c67e1c051c2aeb076bb8c5368a7ba2d5c25a0800f6`
- `service.rs`: `2b45153c3a9ee4d2d0c6d0e293d95e491600bbc3022388766ad459e05ca0ba59`
- `service_checkpoint.rs`: `188b0423df6b14ed10295baf41909ae4336aae4b80531fef5f4bfff000c11917`

Frameworks01 retained a live lease on `authority.rs` while this handoff was claimed, and that file's
current hash changed to `37ea7baed62566499cf225400274e460d23bbba46da7033276f4e70e665dd408`.
Frameworks01 completed the receipt-contract edit, released that exact lease, and Runtime22 then
claimed and attributed the five-file repair scope without forcing ownership. The fresh dependency
baseline is:

- `authority.rs`: `37ea7baed62566499cf225400274e460d23bbba46da7033276f4e70e665dd408`
- `zr_contracts/random/service_state.rs`:
  `c10422917a2d8d1896c428c75173d6a7e9821adac2fdadb2af2f75aa93203f21`
- `runtime/random/error.rs`: `8b7bfcb8ab4b1519acd7f3ecf68a9cd0dd9c51bee0ea1e90b575c7f24216ef64`
- `framework/random/mod.rs`: `4be5d396b086d3ebcd63c0804689a2f3dc51c8c8fdce8e7320c4ed8919fdf90b`

The contracts baseline now enforces successor generations through `RandomSeedReceipt::try_new` for
both construction and deserialization. Runtime22 must preserve that typed invariant unchanged.

## 最低共享层根因

The random service has one consistency boundary but checkpoint capture currently spans two mutex
epochs. The registry owns the stream set, while the authority owns seed and generation. Checkpoint
must observe both while holding the registry mutex and then the seed mutex, matching the existing
global lock order used by reseed and stream acquisition: `registry -> seed`.

The contracts schema is not the repair layer. Adding generation fields, compatibility constructors,
or post-hoc validation would not make the runtime observation atomic and would create a second
public surface for one consistency rule.

## 架构修复验收

- Route checkpoint capture through one registry-locked operation that collects canonical stream
  entries and snapshots seed/generation before releasing the registry mutex.
- Preserve the single global lock order `registry -> seed`; introduce no `seed -> registry` path.
- Keep the public checkpoint schema and restore surface unchanged.
- Add a deterministic regression hook that pauses checkpoint after stream-entry capture. Start
  reseed while checkpoint remains paused and prove reseed cannot complete until checkpoint resumes.
- Verify the completed checkpoint contains the old seed generation and old parked-stream state, then
  restore it and reproduce the next draw and draw index.
- Run the focused random-service checks as part of one managed Windows validation batch. Record
  correctness and checkpoint-boundary performance evidence without claiming draw-path gains.

## 禁止临时方案

- Do not add a second lock-order path, retry loop, optimistic generation check, or mixed-era fallback.
- Do not change `zr_contracts` to mask the runtime atomicity defect.
- Do not expose a public test hook or compatibility checkpoint API.
- Do not overwrite the active Frameworks01 `authority.rs` blob before ownership is released or
  transferred through the coordinator.
- Do not claim completion from static inspection or an unmanaged build alone.

## 修复结果与回传

- 根因：Checkpoint released the registry mutex before capturing seed/generation, allowing reseed to combine old parked streams with a new authority era.
- 架构修复：Capture canonical streams and exactly one RandomServiceState while holding the registry guard, preserving the sole registry-to-seed lock order and unchanged public schema.
- 验证：Stable rustc and coordinator Cargo job d25495f39d2642d5983eaa75bc38f57d run d2cc1fe7f6fd4c38b277fd384df6236e passed 18/18 random tests; independent review C0/I0/M0; 65536-stream checkpoint p50 22.925 ms and p95 41.934 ms.
- 回传：Runtime22 fixed mixed-era random checkpoints, preserved restore/replay next-draw identity, and returned the lifecycle to Frameworks01.

## Current state

Fixed: `atomic_checkpoint_green / managed_validation_green / support_fixed_record_returned`.

Durable Windows RED run (the earlier `fb05a48e3d7f40038e23984080269333` launch only proved a
command-line forwarding error and did not compile or execute the test):

- Cargo job: `9bde9a9769d549f4b41241fae25d8e9b`
- Cargo run: `0e86161a26af460883ba1a9461142e30`
- Test filter: `checkpoint_prevents_reseed_from_crossing_the_captured_stream_era`

That early managed Cargo run completed with exit 101 after reaching 398 pre-existing full-package
compile errors in the shared `zircon_runtime` worktree. It started before the final Runtime22 source
hashes and is retained only as external baseline evidence, not as validation of this repair.

The final coordinator-managed Windows Cargo batch compiled the current random module and current
`zr_contracts`, then passed all 18 tests in 0.07 seconds:

- Cargo job: `d25495f39d2642d5983eaa75bc38f57d`
- Cargo run: `d2cc1fe7f6fd4c38b277fd384df6236e`
- Command: `cargo test --manifest-path .codex/state/runtime22-random-validation-harness/Cargo.toml`

An isolated Windows `rustc --test` build of the same runtime random module produced the required
implementation RED in 0.04 seconds: `0 passed / 1 failed / 17 filtered`, failing because reseed
completed while checkpoint was paused. The temporary contracts rlib used for that historical RED
required `RUSTC_BOOTSTRAP` only to pass the separately handed-off Frameworks01 `const fn` compiler
blocker; the Runtime22 test binary itself used the current Runtime source and stable rustc 1.94.1.

After the lock-boundary and contracts fixes, stable rustc 1.94.1 compiled the current
`zr_contracts` rlib without `RUSTC_BOOTSTRAP`. The final deterministic regression passed in 0.00
seconds. It uses a test-only registry `try_lock` observation after entry capture, then lets the
reseed worker reach its
call boundary before resuming checkpoint; it does not infer lock ownership from a wall-clock
timeout. The complete random kernel batch then passed `18/18` in 0.06 seconds on one test thread,
including the 4,096-key
canonical checkpoint stress test, checkpoint restore/next-draw replay, concurrent same-key
admission, active-lease rejection, reseed exhaustion, bounded-draw vectors, and the static
lock/hash-free draw-owner guard.

The implementation replaces the old split `checkpoint_entries()` operation with the typed
`checkpoint_with_authority_snapshot()` boundary. The registry guard remains live while canonical
entries are collected and the supplied closure returns exactly one `RandomServiceState`. The only lock order is therefore
`registry -> seed`, identical to acquire and reseed. No draw, acquire, release, contracts schema, or
public API code changed.

An optimized isolated checkpoint probe (`rustc 1.94.1`, `-C opt-level=3`) measured the current fixed
implementation on this machine:

| Streams | Repetitions | p50 | p95 | Mean | Typed payload |
|---:|---:|---:|---:|---:|---:|
| 1 | 2,000 | 0.5 us | 0.6 us | 0.705 us | 120 B |
| 64 | 1,000 | 13.2 us | 15.8 us | 16.215 us | 6,168 B |
| 1,024 | 200 | 211.7 us | 321.8 us | 233.364 us | 98,328 B |
| 65,536 | 20 | 22.925 ms | 41.934 ms | 26.685 ms | 6,291,480 B |

This is preliminary current-implementation evidence only. It confirms the required one-vector
`O(N)` checkpoint boundary and does not claim a draw-path gain, allocation census, contention
profile, package-power result, or completed managed release benchmark.

The prior Frameworks01 dependency hash
`14765cd21ef94de6421c167698b9724c9d07fca13112b460addd5028d1abe7af` introduced a separate stable
compiler blocker: `RandomSeedReceipt::try_new` was a `const fn`, but its Option inequality required
unstable const `PartialEq` on rustc 1.94.1 (`E0658`, `service_state.rs:80`). Runtime22 returned that
support-layer failure without editing contracts. Frameworks01 fixed it at
`c10422917a2d8d1896c428c75173d6a7e9821adac2fdadb2af2f75aa93203f21` by matching
`checked_add(1)` and comparing primitive `u64`; stable rlib compilation and the const/overflow/jump
and serde harness are green. Frameworks01 returned the canonical fixed record atomically at
`fixed-2026-08-29-zr-contracts-random-seed-receipt-const-compile.md` with SHA-256
`6178a12dae56a647c7738f21c3600e16ce5d33b3b0f2b3c01b677e9edab66b72`.
