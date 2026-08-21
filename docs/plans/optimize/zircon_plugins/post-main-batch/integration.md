# Plugin Post-Main Aggregate Optimization Record

- Integration owner: `optimize-runtime-full-post-main-integration-r3-01a00797-20260821`
- Former Plugin owner: `optimize-plugin-full-post-main-integration-r2-01a00797-20260821`
  (`cancelled` after ownership consolidation)
- Source plans: `docs/plans/optimize/zircon_plugins` plus the Physics query record under
  `docs/plans/optimize/zircon_runtime/08a`
- Base HEAD: `be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`
- Baseline epoch: `336`
- Status: ownership transferred to the Runtime aggregate; waiting for active MVP00 Editor test repair

## Ownership Closure

- The aggregate takes over 127 live Plugin paths through ownership-transfer fingerprint
  `d9c825d6216d327c784f3e0e4815f69327854baae1469b539b21634995ff4181`.
- The removed phantom-authoring file
  `zircon_plugins/native_window_hosting/editor/src/extension_ids.rs` is leased and attributed to
  this Session with a missing content hash through deletion-bridge fingerprint
  `f02966f7771e233e84ea43f16afc2e4c4382b97e9a821a46a233516c9e8dd2d5`, preserving the intended
  deletion.
- Runtime22's scene-system clock-domain registration remains owned by the active Runtime aggregate.
  The Plugin aggregate will inherit that file from the Runtime commit instead of duplicating its
  ownership or validation input.
- The Plugin-owned `DefaultPhysicsManager::{new,attach_core}` conversion to `&CoreWeak` is required
  by the Runtime-owned module-factory conversion. Runtime validation copy
  `d875afbc26f7465b8e46c07314d02d69` proved that materializing only the Runtime candidate creates an
  invalid mixed snapshot (`E0308` at both call sites). The next managed copy therefore materializes
  both candidate closures and runs both aggregate validators serially before either integration
  record is finalized.
- All live Plugin paths and the intended `extension_ids.rs` deletion were transferred to the Runtime
  aggregate. The Plugin and Runtime records now travel in one coordinator-owned candidate and one
  WeCom-notified commit.
- The nested `zircon_plugins/Cargo.toml` and `zircon_plugins/Cargo.lock` are part of the aggregate
  candidate. Validation must materialize the nested plugin workspace Cargo closure, including
  unchanged package manifests and source files required by RPC, replication, and the other selected
  packages.

## Grouped Validation Contract

- Live root validator source: `zircon-validation-plugin-full-post-main-batch.ps1`, SHA-256
  `DC4EC76C28819285D1A4E1C4ECCD25B66A6D83B5E3F8E16192AD464EBBEB33C6`. The pinned execution-stage
  root is `6787B55BC67195B72CBEF01229CB99277235D7C4497AA6662792820BE6F6E613`; it adds only inherited
  single-job, single-test-thread, warning, and zero-debuginfo test-profile environment settings.
- The pinned validator tree contains 12 top-level child batches, 32 optimization tasks, and 31
  performance rows. Release benchmarks use 21 alternating legacy/optimized sample pairs and
  nearest-rank percentiles.
- The single immutable validation copy must run both the Runtime and Plugin behavior tests, compile
  gates, and performance gates as one serialized coordinator submission. Per-task Cargo submissions
  are not acceptance evidence.
- The Runtime aggregate is the only active Cargo lane. Joint copy
  `3ced039b98894fc49deaed8a69a9597f` reached Cargo but failed before compilation because three root
  workspace member directories were absent; replacement copy
  `a25ed2d18c064bb7bc3c67229ce59d9c` adds all 299 missing tracked files. Its run
  `17213b728e9a4c569ec88f284572dbda` then reached rustc but failed the first Cargo group with exit
  code `101` because all 17 tracked `templates/projects/renderable-empty` inputs embedded by
  `zircon_runtime_interface` were absent. No behavior test or performance gate ran in any of these
  attempts. The replacement input also predates candidate-wide Rust 1.94.1 formatting; the next
  18,203-path copy includes the template closure and exact formatted candidate before acceptance.
- Corrected copy `062a5d9e099648d4a7e7861aeb8193cf`, run
  `b50406bd81d44850b636cec286d3972a`, input manifest SHA-256
  `ee24bf6e741716cf714172e3d1bc8ee4bef22b73e347bd39a1f935798c48da17`, reached the first Runtime07
  Cargo group but exited `101` after 1130.360 seconds. The Runtime lib-test tree embeds broad
  architecture, archived-plan, module-convention, and UI documentation through `include_str!`;
  those unchanged baseline inputs were outside the manual closure. The 873 diagnostics are missing
  document inputs plus type-inference cascades, not Plugin implementation failures, and no behavior
  or performance gate ran. A literal include preflight adds all 5,332 missing tracked docs plus 23
  tracked audit/tool/App targets. The Plugin audio fixture is supplied from clean pinned source
  `dev/bevy@fb89a8649d9b359e53ffb6e5492ebb7c059ac8af` (fixture SHA-256
  `A83896F7DFB64B6A8AD24BEB44BFA6665B4275D208EC38DE894776DAC866D7CF`). The next immutable copy
  therefore carries 23,558 main-repository paths plus the pinned `zr_vm` and Bevy sources.
- Expanded copy `9733f98a87094b049f2f8cc434fd9db5`, run
  `629c3d963d0a4445bafe26b095536353`, input manifest SHA-256
  `dec98ef6805b09f5f53f64d8288b1113dd66703c0541db03c46ed7da23a4e976`, reached the first Runtime07
  Cargo group and compiled for 1535.547 seconds before rustc exited `101`: a Runtime scene-reflection
  test embeds a fixture through `concat!(env!("CARGO_MANIFEST_DIR"), ...)`, which the prior literal
  include closure scan did not collect. No Plugin behavior or performance gate ran. The full concat
  scan found 137 macros, 114 unique existing targets, zero unsupported or broken targets, and four
  inputs absent from the 23,558-path manifest. The next joint copy adds all four inputs together and
  preflights both include forms before the serialized Runtime-plus-Plugin batch.
- Copy `b3935eafc23e431387ad09b67433e00c` materialized the resulting 23,562 paths with input manifest
  SHA-256 `640643b8d13ad10cd099beb59f3d875037de3239f7e477ccf3f09797f1004b66`. Before Cargo admission,
  the sealed-copy preflight found five invalid literal references in a baseline-HEAD Editor jobs test;
  concat targets were closed. The current worktree repair points those references at the production
  owners and restores its `mpsc` import, but active MVP00 Session
  `mvp00-current-source-convergence-r2-01a00797-20260818` owns the exact test file. Coordinator transfer
  preview `af6d09a9fa6858f518c3ee8a83ad9724ea0513bc7f2b19600136ae919b1c1fe2` rejected takeover with
  `source_owner_executable`, as required. No Runtime or Plugin test ran; rematerialization waits for
  that owner to commit or explicitly transfer the repair.

## Acceptance State

Behavior test counts, compile results, per-task P50/P95 values, improvement ratios, managed job/run
identifiers, and the final validator hash are pending terminal coordinator evidence. This record must
be updated with those exact values before the integration candidate can be finalized or reported as
performance-complete.
