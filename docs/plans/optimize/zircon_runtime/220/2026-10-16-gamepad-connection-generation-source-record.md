# Runtime220 gamepad connection generation source record — 2026-10-16

Runtime220 INP-P1-035 now has the authorized canonical source repair across 32 paths (30 existing files and two new modules). Source activation and bounded static review are complete. This record does not close full G21, P1-022, native validation, performance or milestone acceptance.

The trigger was a disconnected gamepad button/axis event that could still mutate manager state. A receiver-only connected-set check would also have broken existing synthetic fixtures and would not identify a stale sample across reconnect. The repair carries a nonzero producer-captured connection generation through typed Rust events and a separate V2 gamepad payload, then admits and mutates under the manager's existing mutex before frame/record/UI publication.

## Source behavior and compatibility

- GamepadConnectionInfo and typed GamepadButton/Axis carry NonZeroU64. Missing or zero serde generation fails rather than filling a current value.
- Required object-safe checked admission returns Accepted or a typed rejection. Unit submit_event delegates to it; generic unstamped Gamepad button ingress fails closed.
- Unknown, stale, future and closed samples cannot change input state, edges, frame events or recording. A closed generation cannot reopen; higher disconnect creates a closed tombstone, and duplicate closed disconnect produces no duplicate state edge.
- App owns the session-local producer tracker. Inventory and connect share one path; duplicate active connect is idempotent, reconnect increments with checked overflow, and samples keep their capture stamp through forwarding.
- Existing repr(C) V1 envelope, ABI/function table and V1 constructors stay byte-compatible. New kinds21/22/23 carry a borrowed-checked 16-byte ZRG2 header with a nonzero u64 generation. Legacy V1 gamepad ingress now returns diagnostic InvalidArgument; external producers must migrate explicitly.
- Same-session dispatch_owned admission precedes real UI dispatch. Only Accepted reaches the nonempty published UI route. This does not hold a manager gate across callbacks or establish a global direct-manager/UI transaction.
- GamepadAxisTransition deliberately remains the documented per-routing-ID lossy frame aggregate. Retiring an ID clears only its private indices and retains old vector slots; raw InputEvent carries generation. The regression asserts raw generation order, four previous/value endpoints, another ID's accumulation, final axis value and action edges.
- Rumble retains device-ID routing and current active-App-tracker checks. Replay preserves exact recorded generations without relabeling; incomplete histories are not claimed self-contained.
- Old synthetic fixtures now connect explicitly with generation1 and keep their original numerical/action assertions. The App protocol source guard names the real V2 producers.

## Actual proof

- Frozen normal patch: 111207 B / `58b0784398d9cc88b0fc5e28c28c44914283ebd80b34d2513393f98a353bbea4`.
- Frozen binding: 127167 B / `b759c87ce755f56772956d0aed9157e2473dbd3d60153669d706673baca07c55`.
- Before-source notice: 16440 B / `40d8c8d0bf732d10295d3400be955dc1aa839d192549de624ab24ad58c9f1656`.
- Root grant: `/runtime220_source_activation_grant`; whole artifacts.json identity is not frozen across parallel authorized metadata appends.
- Fresh 30 existing before files totaled405490 B; two new paths were absent. Normal32 sections/103 hunks reconstructed all exact after values and reversed to every complete before in RAM.
- Every actual after file was read twice, matched the fixed bytes/hash, and reversed exactly to the captured before; two new files reverse to absence. The source state.rs keeps809 CRLF lines.
- Source writer cell125/`05fba8` → session61843 → cell126/`9fbc37`: actual exit0 and INNER_EXIT0, PS17144/Py35720; Git19728 and34204 both communicate+wait0 joined.
- HEAD `f28ba39a5095f4964c046ecc702778627f80cfd5` and index7478958 B/`8ae41eb940729cf4f1ea1468d72fdbe257581bc1fc38ace9dce024c481400893` stayed unchanged.
- The complete preexisting M content in Interface lib.rs and session/events.rs is retained through exact normal hunks. Other actor/semantics remain UNKNOWN; no attribution of parallel E05 or earlier work is claimed.
- Independent core peer `/root/runtime03_fence_core18`: PASS_SCOPED_CORE_NOT_NATIVE, no proven P0/P1; final79/d06009 actual0 with bounded prior readers. This is not native execution.

Machine records:

- `E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/runtime220-gamepad-connection-generation-v1-implementation-receipt.json`: 26502 B / `985fe50426429a383c8fd2b8830ac8ddafbc697551d3b28cfc6b37b306dc8dec`.
- `E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/runtime220-gamepad-connection-generation-v1-static.json`: 34190 B / `af09328c41c00c2df49a34d19e57f101cbff0896ece70267b9c63508ac662642`.

## Test and handoff inventory

All68 old and14 new ordinary tests (82 exact FQNs) remain **UNRUN**. These are lexical source identities, not a native floor. The complete old/new literal names are in the fixed binding /tests/inventory and the static JSON. The Interface session/events.rs contains no same-scope test function.

| Actual module prefix | Old ordinary | New ordinary | Required source cfg |
| --- | ---: | ---: | --- |
| `entry::runtime_entry_app::gamepad::producer::tests` | 0 | 3 | all(test, feature=native-host) plus feature=gamepad-gilrs |
| `input::tests::input_manager::touch_gamepad` | 4 | 0 | all(test, feature=native-host) |
| `input::tests::input_manager::frame_state` | 12 | 1 | all(test, feature=native-host) |
| `input::tests::action_axis_transitions` | 2 | 0 | all(test, feature=native-host) |
| `input::tests::action_mapping` | 17 | 0 | all(test, feature=native-host) |
| `input::tests::recording_replay` | 5 | 1 | all(test, feature=native-host) |
| `input::tests::gamepad_bridge` | 2 | 4 | all(test, feature=native-host) |
| `dynamic_api::session::events::tests` | 6 | 1 | all(test, feature=native-host) |
| `dynamic_api::tests::input_events` | 4 | 1 | all(test, feature=native-host) |
| `tests::runtime_absorption::input_stack::gamepad_bridge` | 1 | 0 | all(test, feature=native-host) |
| `runtime_api::session::gamepad_generation_v2::tests` | 0 | 2 | test |
| `entry::tests::runtime_entry_input_guards::protocol` | 1 | 0 | all(test, feature=native-host) |
| `dynamic_api::session::runtime_ui::tests` | 14 | 1 | all(test, feature=native-host) |

New ordinary cases:

- `entry::runtime_entry_app::gamepad::producer::tests::producer_captured_sample_keeps_old_generation_across_reconnect` — all(test, feature=native-host) plus feature=gamepad-gilrs; **UNRUN**.
- `entry::runtime_entry_app::gamepad::producer::tests::producer_duplicate_inventory_and_connect_keep_generation` — all(test, feature=native-host) plus feature=gamepad-gilrs; **UNRUN**.
- `entry::runtime_entry_app::gamepad::producer::tests::producer_generation_overflow_fails_without_wrap` — all(test, feature=native-host) plus feature=gamepad-gilrs; **UNRUN**.
- `input::tests::input_manager::frame_state::gamepad_same_frame_reconnect_preserves_raw_generations_and_routing_axis_aggregates` — all(test, feature=native-host); **UNRUN**.
- `input::tests::recording_replay::gamepad_replay_preserves_generation_and_rejects_missing_connection_history` — all(test, feature=native-host); **UNRUN**.
- `input::tests::gamepad_bridge::gamepad_samples_require_an_open_matching_generation` — all(test, feature=native-host); **UNRUN**.
- `input::tests::gamepad_bridge::gamepad_reconnect_rejects_stale_samples_and_stale_disconnect` — all(test, feature=native-host); **UNRUN**.
- `input::tests::gamepad_bridge::gamepad_closed_generation_cannot_reopen_and_future_disconnect_closes_once` — all(test, feature=native-host); **UNRUN**.
- `input::tests::gamepad_bridge::gamepad_generic_button_events_require_generation` — all(test, feature=native-host); **UNRUN**.
- `dynamic_api::session::events::tests::checked_gamepad_session_admission_returns_the_manager_generation_rejection` — all(test, feature=native-host); **UNRUN**.
- `dynamic_api::tests::input_events::gamepad_v1_ingress_and_malformed_v2_fail_closed` — all(test, feature=native-host); **UNRUN**.
- `runtime_api::session::gamepad_generation_v2::tests::gamepad_v2_round_trips_nonzero_generation_and_borrowed_name` — test; **UNRUN**.
- `runtime_api::session::gamepad_generation_v2::tests::gamepad_v2_rejects_invalid_generation_payload` — test; **UNRUN**.
- `dynamic_api::session::runtime_ui::tests::gamepad_generation_admission_gates_published_runtime_ui_navigation_and_analog` — all(test, feature=native-host); **UNRUN**.

B21 must preserve all original31 Cargo and2 Python arms,16 producer identities and legacy11 producer gates, profiles/floors and every quantile/ratio/absolute/RSS/raw gate independently AND. Reuse the fixed B20 contract1430659 B/`bfcab865d209cdd46c8180ad9fe48a31392a5ec403e8e83c4178e45d08f6cbb2`:
- Runtime #2 keeps its exact old array, then OR-adds all10 real Runtime inventory namespaces; require old67 and new9 names independently.
- App #6 adds --lib while retaining --test f0_editor_product_startup_diagnostics and no global filter; old protocol guard plus producer3 and original integration floor1 remain required.
- Interface #25 OR-adds the real V2 codec namespace while retaining retained_memory::tests:: and its original2 cases.
No command, CI status query, change guard or performance run was executed by this source task. Supported guard and complete owner source/config pre/post seals must precede native acceptance.

## Exact source after pins

| Canonical source path | Bytes | SHA256 |
| --- | ---: | --- |
| `zircon_runtime/src/core/framework/input/gamepad/device/connection_info.rs` | 5494 | `99a4f855794a62cce9dd75fedb18477be42a56663f0e2fccb8f5ec1077d53fdf` |
| `zircon_runtime/src/core/framework/input/input_event.rs` | 32398 | `3ad8b1ea49f48f0de3ac7f46440691c5abe636d8fe70292e29d79f51f2615452` |
| `zircon_runtime/src/core/framework/input/input_manager.rs` | 21692 | `f535e94f3c0c73e6b0f5a9257ba0f8f9db125e4f5cc2eca228230422ca5c2152` |
| `zircon_runtime/src/core/framework/input/mod.rs` | 3203 | `c5cbcc9a703dfe2c9d1bde1e3b902ab3db7c8429d341ece6363973e5ba74f24a` |
| `zircon_runtime/src/input/runtime/input_state.rs` | 19283 | `31823b26eb63b8ec4a6eaf0208478ae16b514534d6b8cbcfe999ea636f4db3bf` |
| `zircon_runtime/src/input/runtime/default_input_manager.rs` | 32148 | `3c0d4815511cd122ad28d3173e81d709353b24e858aafbd67176bbaed9573c17` |
| `zircon_runtime_interface/src/runtime_api/mod.rs` | 10083 | `92fcba6686213b152046b0957d621753a47132d9a39041eb9f2cd0c26d52553e` |
| `zircon_runtime_interface/src/lib.rs` | 16179 | `215c30fd402ed9b984da46cd8aa922f6dc9a8066e30af63b5ba41bedd6319890` |
| `zircon_app/src/entry/runtime_entry_app/gamepad/producer.rs` | 4043 | `9892410cf91510f9b8cea32465cd68e539dae69026fd624b2c69df7bc94fefa5` |
| `zircon_app/src/entry/runtime_entry_app/gamepad/mod.rs` | 564 | `7e2cede0cd88953246e7b9bc978340a78d263a79465ce854c54d67662045dbc9` |
| `zircon_app/src/entry/runtime_entry_app/mod.rs` | 10627 | `7330ea4d4843d1cfb69c6102ab59e6765ff8d392a40744dcbaf3fc67221a91d7` |
| `zircon_app/src/entry/runtime_entry_app/construct.rs` | 3839 | `5579e8441c0a30afb59042d18bd3ab5257e0b7c0ea74c6c482a79eeac05cc233` |
| `zircon_app/src/entry/runtime_entry_app/gamepad/events.rs` | 4103 | `af08d5cd1e32e340951ff360a89d142c02a33f1a0cbec5fabb76c52c43aa7fa5` |
| `zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs` | 8100 | `4469c8bfd41453d944d6460b51914c5088e945441eb30cf615928905ea09da64` |
| `zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs` | 9263 | `2cbf3c0fb5f6ec4701a17a7a7af4a983066f994ec799aead6384aef4675e477e` |
| `zircon_runtime/src/input/tests/input_manager/touch_gamepad.rs` | 18001 | `ae7b59338f6fda76a38ff2dac3e5f4a718c485ec77b29c11ee295290d5f85639` |
| `zircon_runtime/src/input/tests/input_manager/frame_state.rs` | 31867 | `8d8844c85790d4c344e1efbad4ca78b0b09c501b0ddab87fd6d367bfdd4fd40f` |
| `zircon_runtime/src/input/tests/action_axis_transitions.rs` | 4327 | `310017f74030c7e5c4465ee9af1b49b18fb652f5682b8e3e366a9e27099d2c15` |
| `zircon_runtime/src/input/tests/action_mapping.rs` | 29064 | `faf428e7bdce44867c1126b54cd1b07a3456a6dcad6f37bdc00c91c811139044` |
| `zircon_runtime/src/input/tests/recording_replay.rs` | 14803 | `c5f4f2a4539c3b693ba73437f8d351189386654a1bc5c57e5367707affa66c7d` |
| `zircon_runtime/src/input/tests/gamepad_bridge.rs` | 14735 | `34ecd719ad08401a9c331d761bf2307a2169062d169b537797b7e0f82e0d5cd1` |
| `zircon_runtime/src/dynamic_api/session/tests/events.rs` | 7607 | `266e61df8065299341b51626fcc42da3b290b467268dfafc24ee820953042fc0` |
| `zircon_runtime/src/dynamic_api/tests/input_events.rs` | 8395 | `c00116104e0a95c04442980bdf3a3407e625f2bf387194a2938996766676d219` |
| `zircon_runtime/src/tests/contracts/input_stack/gamepad_bridge.rs` | 3853 | `6cdf294ace09fac8362c225832d3e5c7b58b1894ca6a44726aaf3681602cfa5f` |
| `zircon_runtime/src/dynamic_api/session/state.rs` | 33231 | `04c9cde16d3e905ef8c408a33ed3381d50b101e14e69787997f14445ba739c79` |
| `zircon_runtime/src/dynamic_api/session/events.rs` | 28292 | `899546ce7ae96ba7b509328b8d3ff8fe7bea47e8287d91a42817f75939cff83d` |
| `zircon_runtime/src/dynamic_api/session/events/gamepad.rs` | 7046 | `c12bc9e25a7ede70566e283865955c4714a0cb97543c7b028bc9cd3bd00fa07f` |
| `zircon_runtime_interface/src/runtime_api/session/gamepad_generation_v2.rs` | 6129 | `aee2961cc2464d94b79c5e4752c1ae3eb15b524efb3f0f70f00dc6677d1e8b4b` |
| `zircon_runtime_interface/src/runtime_api/session/mod.rs` | 4668 | `37c2e1c74c1d310572349038bce1d3531cb022c1a8764879f0ccc47e5c2383e5` |
| `zircon_runtime_interface/src/runtime_api/session/events.rs` | 23064 | `d6185cde739619f1c43d49582daa22bde5b3a573bfb68429533ce0f42fd2af47` |
| `zircon_app/src/entry/tests/runtime_entry_input_guards/protocol.rs` | 13754 | `c443cd2fd4f60afb4cb3d81705a24265792a89d09b9d724586b0d05af3b05022` |
| `zircon_runtime/src/dynamic_api/session/runtime_ui/tests/cases.rs` | 30396 | `49146ddffb37c166c7ffb8b30851607d85fd7427b7051f4af7f913fab081b26c` |

## Completion and limits

- [x] Canonical32-path source activation and two new module wiring.
- [x] Exact normal forward/reverse, after double raw, EOL, HEAD/index and foreign-before preservation.
- [x] Bounded independent static core peer; no proven P0/P1 in reviewed scope.
- [ ] All68 old plus14 new ordinary cases: UNRUN.
- [ ] Supported guard, compiler/full cfg/SDK/ABI, complete source/config seals: OPEN.
- [ ] Real native UI/backend execution, physical late Gilrs samples and full G21/P1-022 identity: OPEN.
- [ ] Queued rumble incarnation and complete recording/replay histories: OPEN.
- [ ] All original performance/RSS/quantile gates and product/milestone acceptance: OPEN.

Historical missing-directory query exit2, incomplete capture exception UNKNOWN, spawn_ready PID/session/terminal UNKNOWN, failed assumed Git path exit1 and RAM anchor failures remain recorded in the fixed binding and receipt. Later success does not close unknown historical process identity. Root's ignore read-denial and LF-to-CRLF warnings are preserved as prior receipts; no configuration was changed. The interleaved E05 parameter reader exit1/PID UNKNOWN is also preserved separately.

No build product, cache, helper, source copy or generated fixture was created. No files were removed; actual released space0 B. Retain the fixed packets and these four source/static records for later B21 verification. Root metadata/log, index, staging, commits and pushes were untouched.
