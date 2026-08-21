# Runtime Post-Main Aggregate Optimization Integration

- Integration owner: `optimize-runtime-full-post-main-integration-r3-01a00797-20260821`
- Base HEAD: `be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`
- Baseline epoch: `336`
- Status: Runtime and Plugin ownership converged; waiting for active MVP00 Editor test repair

## Failed Validation Evidence

- Coordinator validation copy: `d875afbc26f7465b8e46c07314d02d69`; run:
  `e9bdf6c199ed49cd99d21b6148c2ae9b`; input manifest SHA-256:
  `ae462e164a145f5e4e8b6168744fe3ac9c8be430a8396160f863283d8c05f738`.
- Pinned root validator SHA-256:
  `715B8C3134AC65450660DF4A4796960024205D5FE2F8D230FF5F2CAC78920640`.
- Runtime07 passed three behavior tests for deterministic work sizes `1`, `1000`, and `100000`.
  Runtime45, Runtime48, Runtime49, the animation event path, and the glTF target-path path also
  passed before the batch reached its first compile failure.
- The immutable copy failed in `runtime08c_masked_base_blend_regressions`: the Runtime-owned module
  factory already supplied `&CoreWeak`, but the copy did not include the Plugin-owned companion
  conversion of `DefaultPhysicsManager::{new,attach_core}` from `&CoreHandle` to `&CoreWeak`.
  Cargo reported two `E0308` errors and the coordinator removed the failed copy normally.
- This is a cross-session materialization failure, not accepted test or performance evidence. The
  Plugin Session was subsequently cancelled and its candidate ownership transferred into this
  Runtime Session so one immutable copy and one coordinator-owned integration commit carry both
  records without a mixed snapshot.
- Correctly authenticated copy `3ced039b98894fc49deaed8a69a9597f`, run
  `25c7f008cc1c4ba78c8cef4c9b2aa495`, input manifest SHA-256
  `03addb7dd2f624372eced0c7304379c0dbe4506633e0105f1290b7c64027715d`, and pinned root validator
  SHA-256 `F90B4E4B1547C98EF9F96C1D6BB017A6E1EEDCB0F1729F4A0D80F78D81CDED13` failed before compilation:
  the manual closure omitted `zircon_hub`, `tools/cargo-zircon`, and `zircon_runtime_host`, so Cargo
  could not load the root workspace. No behavior or performance result from that run is accepted.
- Replacement copy `a25ed2d18c064bb7bc3c67229ce59d9c` expands the manifest from 17,886 to
  18,185 paths by adding all 299 tracked files under those three workspace members. Its input hash
  is `105c965ca6d186accf6b44c29fe805b57df37c78b739f86876f4e1e121bcec7b`; asynchronous execution is
  diagnostic-only because candidate formatting changed after materialization. Its terminal result
  may identify functional defects but cannot close acceptance. Run
  `17213b728e9a4c569ec88f284572dbda` reached rustc and failed its first Cargo group with exit code
  `101`: the copy omitted the 17 tracked files embedded by
  `zircon_runtime_interface/src/project/template_pack/embedded.rs` from
  `templates/projects/renderable-empty`. No behavior test or performance gate ran. The next copy
  adds those 17 immutable inputs plus this Runtime aggregate record, for 18,203 paths total.
- Corrected copy `062a5d9e099648d4a7e7861aeb8193cf`, run
  `b50406bd81d44850b636cec286d3972a`, input manifest SHA-256
  `ee24bf6e741716cf714172e3d1bc8ee4bef22b73e347bd39a1f935798c48da17`, and the same pinned root
  validator reached the first Runtime07 Cargo group. After 1130.360 seconds, rustc exited `101`
  because the lib-test tree embeds architecture, archived plan, module-convention, and UI docs that
  were not present in the manually selected closure. The reported 873 compiler errors are missing
  `include_str!` inputs plus their type-inference cascades; no behavior or performance gate ran.
  A literal `include_str!`/`include_bytes!` preflight then found 23 additional tracked audit/tool/App
  inputs and one clean Bevy audio fixture. The replacement closure therefore contains 23,558 main
  repository paths, the existing pinned `zr_vm` source, and
  `dev/bevy@fb89a8649d9b359e53ffb6e5492ebb7c059ac8af` with fixture SHA-256
  `A83896F7DFB64B6A8AD24BEB44BFA6665B4275D208EC38DE894776DAC866D7CF`. This avoids another
  per-file missing-input repair cycle before Cargo.
- Expanded copy `9733f98a87094b049f2f8cc434fd9db5`, run
  `629c3d963d0a4445bafe26b095536353`, input manifest SHA-256
  `dec98ef6805b09f5f53f64d8288b1113dd66703c0541db03c46ed7da23a4e976`, and the pinned root
  validator reached the first Runtime07 Cargo group and compiled the Runtime dependency graph for
  1535.547 seconds. Rustc then exited `101` because the copy omitted
  `tests/fixtures/serialization/scene-reflection/v0/reflected-value.json`, referenced through
  `concat!(env!("CARGO_MANIFEST_DIR"), ...)`; no behavior or performance gate ran. A repository-wide
  preflight resolved all 137 such macros to 114 unique existing inputs with zero unsupported or
  broken targets. Four were absent from the 23,558-path manifest: the reflection fixture, the
  dynamic-scene fixture, and two MVP automation JSON inputs. The replacement copy adds all four at
  once and preflights both literal and concat include forms before Cargo.
- Copy `b3935eafc23e431387ad09b67433e00c` materialized those 23,562 paths with input manifest
  SHA-256 `640643b8d13ad10cd099beb59f3d875037de3239f7e477ccf3f09797f1004b66`. The sealed-copy preflight
  passed the concat closure but found five invalid literal include occurrences in the baseline-HEAD
  Editor jobs admission-scaling test. The referenced `tests/system` and `tests/category.rs` paths do
  not exist in HEAD. The current worktree already repairs them to the production owner paths and
  restores the required `mpsc` import in
  `zircon_editor/src/core/jobs/tests/admission_scaling_contract/indexed.rs`; that exact change belongs
  to active Session `mvp00-current-source-convergence-r2-01a00797-20260818`. Coordinator transfer
  preview `af6d09a9fa6858f518c3ee8a83ad9724ea0513bc7f2b19600136ae919b1c1fe2` correctly rejected takeover
  with `source_owner_executable`. Cargo was not launched. The joint copy must be rematerialized only
  after that owner commits or explicitly transfers the repair, then rerun the same dual-form
  preflight before the joint batch.

## Grouped Validation Contract

- Run the pinned grouped validator in one coordinator-owned immutable copy containing the Runtime
  and Plugin candidate closures, require all behavior and compile groups to pass, and accept exactly
  33 Runtime plus 31 Plugin performance rows.
- Update this record with the exact validation identity, performance data, independent review, and complete coordinator commit manifest.
- Rust 1.94.1 file-scoped rustfmt now passes for all 221 owned Rust files, and scoped
  `git diff --check` passes for all 337 Session paths. A fresh immutable copy with those exact hashes
  plus the complete tracked-doc dependency closure must rerun the entire joint validator before
  submission.

## Acceptance Contract

- Validation is serialized as one aggregate job rather than one Cargo submission per task.
- The joint release gate requires 69 optimization tasks and 64 accepted performance rows; the
  Runtime child retains its own 61-Cargo-group, 37-task, 33-row contract.
- No result is complete until this record reports fresh terminal evidence and zero open Critical or Important review findings.
- The coordinator owns the scoped commit and the idempotent WeCom notification containing performance data.
