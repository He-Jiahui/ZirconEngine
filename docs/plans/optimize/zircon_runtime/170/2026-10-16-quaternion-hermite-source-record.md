# Runtime170 Quaternion Hermite source record

Date: 2026-10-16. Fixed six-file source correction implemented; scoped static byte proof complete; acceptance OPEN.

Executor: editor06_owner_core_peer20 / gpt-6.1-sol max. Packet/design owner: runtime170_hermite_core22 / gpt-6.1-sol max. Root notice writer 51b84d exited 0 (PS7872/Py2284; Git33196 communicate/wait0). Before-source notice: 6366 B / a2a7447058741dc383f8142106ed2e28d2cd55ab93c28d3634872c4c30d9258a.

Both public entries, AnimationClipEvaluator::sample_clip and DefaultAnimationManager as AnimationManager::sample_clip_pose, now use Quaternion component Hermite derivatives times actual positive key duration. Authored signs/magnitudes remain; only left out/right in tangents contribute, missing tangents are zero, and the final validated sample is normalized. Existing hermite_array is reused. Scalar/Vec duration behavior and Quaternion Linear/Step contracts stay as before.

Nonfinite components, norm overflow and zero-length samples return existing typed errors before normalization. Compiled errors retain the actual dense bone row and existing From conversion; raw errors retain AnimationError types. Result errors expose no pose. Existing pool release is unconditional after Result, clear/reset prepares reuse, and resource revision cache identity is unchanged.

Complete fresh before6 double raw checks and RAM normal forward/reverse passed before writing. Writer 409282 exited0, PS7816/Py24828, children0, argv9200 ASCII. Actual after6 was double-read against fixed pins and passed exact normal forward/reverse. Original LF bytes outside hunks remain. HEAD stayed f28ba39a5095f4964c046ecc702778627f80cfd5. Index stayed 7478958 B / 8ae41eb940729cf4f1ea1468d72fdbe257581bc1fc38ace9dce024c481400893 / ticks639276485811354535.

Only the six source paths were mutated. Disjoint E06/Runtime160 groups and foreign bytes were not overwritten. No root manifest/log, immutable packet, index, control or unrelated source write occurred.

The original 13 ordinary integration identities/attributes are preserved. Four new real-public-entry regressions are written, not executed. They cover dt=1/4/2^-30, negative start, signs/nonunit magnitudes, missing/adjacent tangents, zero/component/norm overflow, cache recovery and pool reuse. Full old/new FQNs and cfg are in the static JSON.

- `animation_compiled_evaluator_contract::quaternion_hermite_public_samplers_use_component_derivatives_and_duration`; integration #[test]; resolved feature graph OPEN; not run.
- `animation_compiled_evaluator_contract::quaternion_hermite_public_samplers_preserve_authored_signs_and_magnitudes`; integration #[test]; resolved feature graph OPEN; not run.
- `animation_compiled_evaluator_contract::quaternion_linear_and_step_public_samplers_preserve_existing_contracts`; integration #[test]; resolved feature graph OPEN; not run.
- `animation_compiled_evaluator_contract::quaternion_hermite_public_samplers_reject_degenerate_samples_and_recover`; integration #[test]; resolved feature graph OPEN; not run.

Ignored producer `evaluation::clip_evaluator::sample::optimization_tests::optimization_batch_20260830cd_pose_pool_miss_delta_p95` remains 3738 B / 99d99712c47df875165fe6ddb83e995557c13951a0b7a099a98a8180dd3dc3a3. Marker `RUNTIME170_POSE_POOL_MISS_DELTA_BENCH_V1` and original optimized_p95*100 <= baseline_p95*20 threshold remain unchanged and unexecuted.

Proposed integration recipe, not run: `cargo test --manifest-path zircon_plugins/animation/runtime/Cargo.toml --test animation_compiled_evaluator_contract`. Declared animation/physics-contracts dependencies are unchanged; resolved features remain OPEN. Root owns the later coalesced B20 request.

No Cargo, build, Rust/native test, benchmark, Python unittest, change guard, CI execution or Jenkins status query ran. Neither narrowed package nor workspace-wide compilation ran. Resolved features/full compiler closure, native, guard, performance, milestone and product gates remain OPEN. Source inventory, static analysis and mathematical counterexamples are not native test execution.

Fixed patch19380 B / 3182fd49347d71fe7f7745a51286da35baad6207b9f925a4c6642f5f98010e39; binding11057 B / 0be90c850b35035fe07e2014fcd323ad15b8c994082f314f23ed6bdfd32beafa. Implementation receipt and static JSON under the task report directory retain all six before/actual-after pins, process receipts, full test FQNs/cfg and unexecuted recipe.

Peer readers e283c6/811f84/a83a0f exited0/children0. The guessed missing channel_validation path and subsequent true module path are preserved. An overlength inline post command was rejected by the prelaunch length check (21804 chars, no PID/process/write) and shortened. Original prior failures/UNKNOWN remain in their records; no pending/UNKNOWN becomes PASS.

Released0 B. No helper, source copy, temporary/build/cache product or generated fixture was created. Retain fixed packets and four delivery records for validation; foreign/shared artifacts remain untouched.

| Source path | Actual after bytes | Actual raw SHA-256 |
| --- | ---: | --- |
| `zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/hermite.rs` | 4884 | `5978611a392dc6ac75d9671998183fcb8db0a697e0642e44da95aac090765abd` |
| `zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/sample.rs` | 11948 | `7be9307687bb88c30fec3e0d713f35483c69ee5a4ebae1f261fda51436eaae55` |
| `zircon_plugins/animation/runtime/src/channel_sampling/interpolation.rs` | 5223 | `656be3b9ff7b75657ec487193ce10901bc24d656bd7bf320be1f1f1b09c5227f` |
| `zircon_plugins/animation/runtime/src/manager/sampling.rs` | 4273 | `aa3bbab65f389b01a00fa2f731de2d3a70cf513bab7192f45eb1e4cdd7eb20a9` |
| `zircon_runtime/src/core/framework/animation/asset/channel.rs` | 5508 | `d86fa3eec06761325df599b37c066419fcd46ba7bf8e3a330032593a0309053f` |
| `zircon_plugins/animation/runtime/tests/animation_compiled_evaluator_contract.rs` | 27177 | `d1d9db5ba24fec7db5fb5e0f5784e71ab6f3d31d43e70cae2cd8e1693521d0ee` |
