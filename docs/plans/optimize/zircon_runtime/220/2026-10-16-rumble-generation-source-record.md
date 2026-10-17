# Runtime220 queued rumble: enqueue-time generation

Source slice delivered2026-10-16; full acceptance remains OPEN.

- [x] Exact19 existing source application;19sections/38hunks; source351214->368868B.
- [x] Actual fresh-before/after double raw, exact forward/reverse, EOL/foreign/HEAD/index preserved.
- [x] Independent PASS_SCOPED_PACKET_NOT_NATIVE; no scoped P0/P1.
- [ ] Execute6new ordinary+12preserved named requirements.
- [ ] Supported guard/full compiler-config-toolchain-environment pre/post seals.
- [ ] Native/hardware/performance/product and full INP-P1-043/G25 acceptance.

| Literal FQN | Case | Package/binary | Arm | Parent cfg | Status |
|---|---|---|---|---|---|
| `input::tests::input_manager::host_requests::gamepad_rumble_rejects_unobserved_and_closed_targets_without_retention` | new | zircon_runtime/lib | #2 | cfg(test); lib input module requires feature="native-host" | UNRUN |
| `input::tests::input_manager::host_requests::gamepad_rumble_queue_retains_enqueue_generations_across_reconnects` | new | zircon_runtime/lib | #2 | cfg(test); lib input module requires feature="native-host" | UNRUN |
| `dynamic_api::session::host_requests::tests::encoded_rumble_page_keeps_enqueue_generation_after_reconnect` | new | zircon_runtime/lib | #2 | cfg(test); lib dynamic_api requires feature="dynamic-api" | UNRUN |
| `entry::runtime_entry_app::gamepad::rumble::tests::rumble_generation_guard_rejects_queued_add_and_stop_after_reconnect` | new | zircon_app/lib | #6 | cfg(all(test, feature="gamepad-gilrs")); native-host -> entry; platform-winit -> runtime_entry_app | UNRUN |
| `tests::contracts::runtime_host_request_batch_serializes_generation_bound_rumble_v2` | new | zircon_runtime_interface/lib | #25 | cfg(test); no additional parent feature/OS gate | UNRUN |
| `tests::contracts::runtime_gamepad_rumble_v2_rejects_missing_or_invalid_generation` | new | zircon_runtime_interface/lib | #25 | cfg(test); no additional parent feature/OS gate | UNRUN |
| `input::tests::input_manager::host_requests::gamepad_rumble_requests_are_frame_local_and_drainable` | preserved | zircon_runtime/lib | #2 | cfg(test); lib input module requires feature="native-host" | UNRUN |
| `input::tests::input_manager::host_requests::cursor_host_requests_are_frame_local_and_drainable` | preserved | zircon_runtime/lib | #2 | cfg(test); lib input module requires feature="native-host" | UNRUN |
| `dynamic_api::session::host_requests::tests::runtime_ime_host_request_preserves_its_viewport_target` | preserved | zircon_runtime/lib | #2 | cfg(test); lib dynamic_api requires feature="dynamic-api" | UNRUN |
| `dynamic_api::session::host_requests::tests::runtime_ime_host_request_bounds_and_remaps_its_utf8_context_window` | preserved | zircon_runtime/lib | #2 | cfg(test); lib dynamic_api requires feature="dynamic-api" | UNRUN |
| `dynamic_api::session::host_requests::tests::borrowed_host_request_page_matches_the_owned_v1_batch_encoding` | preserved | zircon_runtime/lib | #2 | cfg(test); lib dynamic_api requires feature="dynamic-api" | UNRUN |
| `dynamic_api::tests::host_request_payloads::host_request_batch_encodes_runtime_ime_requests` | preserved | zircon_runtime/lib | #2 | cfg(test); lib dynamic_api requires feature="dynamic-api" | UNRUN |
| `dynamic_api::tests::host_request_payloads::host_request_batch_encodes_gamepad_rumble_requests` | preserved | zircon_runtime/lib | #2 | cfg(test); lib dynamic_api requires feature="dynamic-api" | UNRUN |
| `dynamic_api::tests::host_request_payloads::host_request_batch_encodes_cursor_requests` | preserved | zircon_runtime/lib | #2 | cfg(test); lib dynamic_api requires feature="dynamic-api" | UNRUN |
| `entry::runtime_entry_app::gamepad::rumble::tests::rumble_effect_admission_has_a_fixed_per_gamepad_limit` | preserved | zircon_app/lib | #6 | cfg(all(test, feature="gamepad-gilrs")); native-host -> entry; platform-winit -> runtime_entry_app | UNRUN |
| `entry::runtime_entry_app::gamepad::rumble::tests::rumble_add_admits_before_backend_creation_and_publish` | preserved | zircon_app/lib | #6 | cfg(all(test, feature="gamepad-gilrs")); native-host -> entry; platform-winit -> runtime_entry_app | UNRUN |
| `tests::contracts::runtime_host_request_batch_serializes_gamepad_rumble_requests` | preserved | zircon_runtime_interface/lib | #25 | cfg(test); no additional parent feature/OS gate | UNRUN |
| `tests::runtime_absorption::dynamic_api_session::mirror_docs::runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` | preserved | zircon_runtime/lib | #2 | cfg(test); actual path alias tests::runtime_absorption -> dynamic_api_session -> mirror_docs | UNRUN |

Actual source receipt: {"actual_exit": 0, "actual_source_writes": 19, "after_total_bytes": 368868, "all_19_after_double_raw": true, "before_source_notice": {"bytes": 13708, "path": "E:\\Git\\ZirconEngine\\.jenkins\\records\\optimize-01a13451-20261016-runtime220-rumble-source-activation.json", "raw_sha256": "53b1790dd69766ced8f9956420fb6a50bb9b12dcb88d04a19a9bad1472cbbabc"}, "before_total_bytes": 351214, "cell": 3, "children": [{"argv": ["git", "rev-parse", "HEAD"], "communicate_wait_joined": true, "exit_code": 0, "pid": 9192, "scoped_path_count": 0, "stderr": ""}, {"argv": ["git", "ls-files", "--stage", "--"], "communicate_wait_joined": true, "exit_code": 0, "pid": 32896, "scoped_path_count": 19, "stderr": ""}, {"argv": ["git", "ls-tree", "HEAD", "--"], "communicate_wait_joined": true, "exit_code": 0, "pid": 20848, "scoped_path_count": 19, "stderr": ""}, {"argv": ["git", "diff", "--cached", "--no-ext-diff", "--no-textconv", "--numstat", "--"], "communicate_wait_joined": true, "exit_code": 0, "pid": 18548, "scoped_path_count": 19, "stderr": ""}], "chunk_id": "771db9", "ci_queries": 0, "eof_and_eol_preserved": true, "guard_requests": 0, "head": "f28ba39a5095f4964c046ecc702778627f80cfd5", "index_before_after": {"bytes": 7478958, "mtime_after_ns_decimal": "1792051781135453500", "mtime_before_ns_decimal": "1792051781135453500", "raw_sha256": "8ae41eb940729cf4f1ea1468d72fdbe257581bc1fc38ace9dce024c481400893"}, "native_requests": 0, "normal_forward_reverse_exact": true, "normal_patch": {"bytes": 42759, "path": "E:\\Git\\ZirconEngine\\.codex\\outbox\\reports\\optimize-20261013-01a13451\\runtime220-rumble-generation-v1.patch", "raw_sha256": "7fecc153627738e664fb51e720c269b3e2730e95933351e1dbb4c969e0cf2d97"}, "parent_pid": 17048, "pid": 17648, "session_id": null, "source_binding": {"bytes": 118919, "path": "E:\\Git\\ZirconEngine\\.codex\\outbox\\reports\\optimize-20261013-01a13451\\runtime220-rumble-generation-v1-source-binding.json", "raw_sha256": "0cac9296dbc199aec811b638ad4bb17d45a6bbf812891cf7e0ccd43c049cb922"}, "source_frozen": true, "source_write_failures": [], "status": "SOURCE_APPLIED_PASS_RAW_ONLY_NOT_NATIVE"}

Frozen patch: E:\Git\ZirconEngine\.codex\outbox\reports\optimize-20261013-01a13451\runtime220-rumble-generation-v1.patch; 42759B/7fecc153627738e664fb51e720c269b3e2730e95933351e1dbb4c969e0cf2d97.

Frozen binding: E:\Git\ZirconEngine\.codex\outbox\reports\optimize-20261013-01a13451\runtime220-rumble-generation-v1-source-binding.json; 118919B/0cac9296dbc199aec811b638ad4bb17d45a6bbf812891cf7e0ccd43c049cb922.

Implementation: E:\Git\ZirconEngine\.codex\outbox\reports\optimize-20261013-01a13451\runtime220-rumble-generation-v1-implementation-receipt.json; 112105B/eef9bc5eafd971026af6ab2e8b3ad15eab5473d8dd7f530cdf2bba2abd1b4fd3.

Static: E:\Git\ZirconEngine\.codex\outbox\reports\optimize-20261013-01a13451\runtime220-rumble-generation-v1-static.json; 43526B/eeaca615eca96756e49abac1b53adb0f683c88804bce3b33749a3a4b8d0b6a83.

Connected generation is captured under the manager state lock. Unavailable targets retain nothing. FIFO/pending/encoded pages and rollback preserve captured generation; drain never relabels. App gates Add and Stop before effect/backend mutation; no manager lock crosses backend/UI work.

C API V8/optional drain V2/opaque result layouts stay unchanged. Outer V1 JSON adds mandatory-generation GamepadRumbleV2; legacy serde remains, but V1 device execution is rejected. Old-host unknown-tag rejection is failclosed, not an interoperability pass. Headless diagnoses unsupported without effects/ack.

Reuse Runtime#2 debug/lib target-editor-host,test-support; App#6 debug/lib+old integration/no globalfilter; Interface#25 debug/lib. Binding /validation_routes supplies selectors and named AND. Keep31Cargo+2Python,prior89names,original172/167/1/12/2 subsets,16producers+legacy11 and all old percentile/raw/relative/absolute/RSS AND. Lexical60->66ordinary/ignored0 is not a nativefloor.

Old Runtime adds connectedg1, keeping NaN/clamp/order/framewindow/crossframepending/full drain/secondempty assertions. Old App sourceguard remains structural. New App helper test is not hardware; page manager/converter/owned+borrowed encoders are not an executed FFI fault-rollback test.

History:129 actual1 byte-adapter SHA error;130 proved exact12500B prefix;133 exact append0. Preserve101 child2,109 PIDUNKNOWN,tool40/E05writer88 UNKNOWN. JS parser produced no exec receipt; cell7/root215/216 spawn_ready PID/session/terminal UNKNOWN. Success closes none of those unknowns.

No helper/sourcecopy/fixture/build product was created; released0B. Preserve all foreign/UNKNOWN content. ThreeCRLF and16LF sources retain EOF newlines. Source19 is frozen.

Raw frame windows remain unstamped intent observations; resubmitting recorded intent is fresh enqueue, not proof of its original physical generation. RequestID/deadline/completion/ack, physical Gilrs late events, replay/session remapping and full INP-P1-043/G25 remain OPEN.

OPEN: INP-P1-043 request ID, deadline, completion and acknowledgement remain OPEN

OPEN: INP-G25 full acceptance remains OPEN

OPEN: Physical Gilrs old OS events and real backend execution remain OPEN

OPEN: Arbitrary recording replay cannot prove original physical rumble identity; raw re-submission is a new enqueue

OPEN: Whole replay/persistence/session remapping is outside this slice

OPEN: Global direct InputManager/UI transaction is outside this slice

OPEN: Mixed-version host/runtime wire interoperability is fail-closed, not validated interoperability

OPEN: Full compiler/configuration/seal and supported guard remain OPEN

OPEN: Native ordinary tests, hardware, performance, whole milestone and product acceptance remain OPEN
