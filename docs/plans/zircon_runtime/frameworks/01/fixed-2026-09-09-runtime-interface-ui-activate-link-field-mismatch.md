---
handoff_kind: fixed
status: fixed
created_at: 2026-08-31
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/03
plan_link_mode: child_record_only
summary_slug: runtime-interface-ui-activate-link-field-mismatch
observed_at: 2026-08-31
related_code:
  - zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui/host_requests.rs
tests:
  - cargo test -p zircon_runtime_interface --locked
  - cargo test -p zr_resource --locked --release --lib resource_management_projection_current_source_profile -- --ignored --test-threads=1 --nocapture
resolved_at: 2026-09-09
---

# Runtime Interface UI Activate-Link Field Mismatch

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：R14 current-source ResourceManagement release profile
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 交接原因：最低共享原因位于 RuntimeInterface typed UI dispatch 到 generic host request 的公共投影，不属于 Frameworks01 或 `zr_resource`。

## 失败现象与复现证据

Frameworks01 managed current-source ResourceManagement release profiling reached the
`zircon_runtime_interface` dependency and failed before compiling `zr_resource`:

- Cargo job: `64107e1527764083b78888c199cccf5f`;
- run: `4630a785c5304a90957462c8f04c6581`;
- profile command: `cargo test -p zr_resource --locked --release --lib
  resource_management_projection_current_source_profile -- --ignored --test-threads=1
  --nocapture`;
- compiler: Rust 1.94.1 Windows MSVC managed lane;
- `zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs:139`: E0026/E0027.

`UiDispatchHostRequestKind::ActivateLink` now has fields `target` and `link_target`, with
`link_target` serialized as `href`. The runtime host-request projection still matches the Rust field
as `href`:

```text
UiDispatchHostRequestKind::ActivateLink { target, href }
```

The pattern names a removed field and omits `link_target`, so the support crate does not compile.

## Exact current source

| File | State | SHA-256 |
| --- | --- | --- |
| `zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs` | untracked, coordinator `attribution_missing` | `035671d851407158cffbfc8054e9b623c68ccb1221b4eaa840cdc00d6f329068` |
| `zircon_runtime_interface/src/ui/dispatch/input/result.rs` | modified, archived attribution stale | `d94bfb575396554c15221b6bd88845857679b6a896ba5be4adf56d3818aa8278` |

Coordinator evidence:

- bridge ownership matrix request `c7718b340a604ba9a03aea994939f152`: unowned,
  `attribution_missing`;
- dispatch definition matrix request `0b8dff12fad34de7a68080d0f946f442`: archived source owner,
  `attribution_hash_stale`, `attribution_baseline_stale`, `owner_not_executable`, and
  `live_lease_missing`.

No existing `failure-*.md` matched this source/error before this record was created.

## 最低共享层根因

Frameworks01 did not claim or edit either Runtime Interface source. The fixing owner must first
resolve whether the new runtime host-request bridge and the UI dispatch hard cut belong to the Runtime
Interface host ABI plan or the UI dispatch/link plan, then claim/attribute the exact files through the
coordinator. Frameworks01 must not absorb an untracked mixed-era bridge merely to unblock its profile.

## 架构修复验收

The fixing owner must:

1. preserve the Rust field hard cut to `link_target`; the serialized external key may remain `href`
   through the existing serde rename;
2. update every typed projection, constructor, pattern, and test consistently, without a legacy
   `href` Rust field or compatibility variant;
3. validate `zircon_runtime_interface` on Windows Rust 1.94.1 with `--locked`;
4. claim and attribute the current files, including the untracked bridge, before coordinator commit;
5. return the commit/current hashes and a fixed artifact or canonical return to this Failure.

## 禁止临时方案

- 不得恢复 Rust `href: String` 字段、兼容 variant、字符串 fallback 或第二份 link parser。
- 不得 wildcard/suppress E0026/E0027、弱化 typed projection test，或让 Frameworks01 吸收 foreign UI source。
- 不得在 managed lower/upward gates 完成前把 source repair 标成 fixed 或 accepted。

Frameworks01 can then rerun the unchanged 31-sample ResourceManagement profile. This failure blocks
that managed profile only; it does not block readiness architecture review or behavior/profile
infrastructure work.

## 修复结果与回传

- 根因：The migrated typed ActivateLink dispatch uses Rust field link_target with serde wire key href, but the runtime host projection and queue constructors still destructured the removed Rust href field.
- 架构修复：Kept one typed UiRichLinkTarget model end to end: the host bridge and runtime constructors now use link_target, preserve the existing href wire name, clone the admitted target, and carry no legacy href field, compatibility variant, string fallback or second parser.
- 验证：Managed Windows interface job f95f64a6a06345d3940884140d9e3e50 executed the ActivateLink roundtrip/Debug and typed projection tests in the current interface library; overall result was 739 passed, 23 failed and 101 ignored, with this chain green. Managed zr_resource locked release library job 64402e468e94498f96357bb26d716f81 passed 224, 0 failed and 10 explicitly ignored. Static rich-link contracts passed 22/22 and the all-source legacy typed-field scan reported zero matches; remaining unrelated full-library failures are explicit.
- 回传：Returned the current-source typed ActivateLink bridge across ui_host_request.rs ee0d618872cecf058dae9eb982bb6448765bb14b09c2bc4e765feb0c01b64dcc, dispatch result bc41084f0e62240cedb7dde85f91fd20347b900181f2bed5ccfedab04addb195, dispatch effect 4d5c62c9c64f42e8334fa91f918e5994d077f5f59da885f8114a79e03acaa006 and Runtime host constructors f8e86341200d4525408a37192b46acb0105e814659c09d6ea984701f060aba2f. The original Frameworks01 compile/profile blocker is absent.

## 2026-09-02 coordinated current-source batch receipt

RuntimeInterface03 renewed one copy-complete lease for the three open interface failures and their
shared typed projection under request `437bd9bac8194aaf9eaff5849b4da574`; attribution request
`cd08b4bd45e24aa19c63fcb63a67cf33` accepted the exact current hashes. The current ActivateLink
union remains:

- `ui_host_request.rs` `C977C4D6689FB2487A9D7A179ADDF6977450AEFDEBFF96D64BE7C59BC1707B51`;
- `result.rs` `BC41084F0E62240CEDB7DDE85F91FD20347B900181F2BED5CCFEDAB04ADDB195`;
- `effect.rs` `4D5C62C9C64F42E8334FA91F918E5994D077F5F59DA885F8114A79E03ACAA006`;
- Runtime `host_requests.rs` `F8E86341200D4525408A37192B46ACB0105E814659C09D6EA984701F060ABA2F`.

The product-source scan over Runtime, Runtime Interface, Runtime Host, Editor, App, and Plugins
excluding target trees reports `legacy_typed_href_file_count=0`. The reproducible static command
`python -m unittest tools.tests.test_runtime_rich_link_command_index_performance_contract
tools.tests.test_runtime_text_rich_source_contract` passes `22/22`. Combined managed request
`runtime-interface03-runtime200-current-source-20260902-r1` was submitted for Windows Rust 1.94.1
release tests of `zircon_runtime_interface` and `zircon_runtime`, but admission rejected it before
ticket creation and before Cargo execution with `validation_ticket_external_worktree_dirty` for
external worktree `E:\\Git\\zr_vm`. No compile, test, performance, integration, or return receipt is
claimed. This Failure remains `open`, and the external worktree remains untouched.

After this receipt was appended, document-only lease request
`134b23729daf4ba39e17f989d6773017` and attribution request
`cde75b5e97f64013bb50d587aa61c7c7` refreshed all three canonical Failure artifacts.

## 2026-09-08 Current Contract Test Admission

Stable fixing Session `failure-roll-01a07160-interface03`, baseline 601, preserves this
lifecycle and the original typed link projection. Managed Windows job
`61f0ee23ee454d509a19f5a075dcd57a` attempted the shared interface library under
`--no-default-features --locked --lib serialization::tests::`, static linking, input
`runtime25-project-paths-support-3095-20260908`, digest
`32efa2999d8e8e6a86707ebecf8e10e53d6e68bf89287f60c2c02483b9f66870`.
It reported 23 test-compilation errors and executed zero tests. The ActivateLink test
partially moved `request.kind` before its dynamic-content Debug assertion.

Source snapshot 3128 (`d20d0abe769b4c2f9ed080f3ee7bd569`) repairs that assertion by
borrowing the kind and typed target. The wire key, roundtrip, link identity and content-free
Debug assertions remain intact; the bridge hash is now
`ee0d618872cecf058dae9eb982bb6448765bb14b09c2bc4e765feb0c01b64dcc`.
Transfer `d17942e4f35f41d1acfce70455bc53fd`, lease
`b2f566aa926644d0b2028e4901cb2d9c`, and pre-edit snapshot 3127 preserve the exact
HEAD-clean or archived bytes of the ten-path source admission batch.

The same snapshot corrects the shared interface test boundary needed by this gate:
crate-local binding imports; a borrowed expression array with one retained scratch stack;
public surface imports for parity DTOs; canonical text-shape access and the removal of an
obsolete editable-text fixture field; and an owned route clone where the original borrowed
route is still serialized later. The private parity-helper tests move atomically from
`src/tests/render_parity_performance_contracts.rs` to
`src/ui/surface/render/parity/performance_tests.rs`, with both module registrations updated.
All five test bodies, including two ignored performance thresholds, are retained; no
production visibility was widened. The two existing large contract suites receive only
consumer corrections and formatting, with no new subsystem or helper added.

Scoped rustfmt and whitespace checks pass. Derived-input validation, independent review,
the original ResourceManagement profile, formal fixing-Session closeout binding and
`failure return` remain pending. The original error evidence and lifecycle stay open;
`plan_link_mode: child_record_only` makes the existing child-record routing explicit.

## 2026-09-08 Shared Library Execution

Derived input `interface-library-consumers-3137-20260908`, digest
`484b58e5512bb5619941864b4906bbfaaeef0cff78f89267586fe6e4b00d2b63`, preserves 3128 and
adds separately owned Editor08 3134 / App07 3137 compilation repairs. Managed Windows job
`f95f64a6a06345d3940884140d9e3e50` compiled and ran the full locked/static interface library:
739 passed, 23 failed, 101 ignored, zero filtered out. Its
`results/interface-library-3137.{json,log}` retain all actual test names and receipt.
The ActivateLink roundtrip/Debug, borrowed expression-stack and parity tests executed;
the whole library remains red on project, ABI, boundary and other UI assertions.

Two directly actionable shared-layer roots now have separate canonical records:
[slot rebind precedence](../../../optimize/zircon_runtime_interface/03/fixed-2026-09-09-layout-slot-rebind-precedence.md) and
[resolved text metrics](../../../optimize/zircon_runtime_interface/03/fixed-2026-09-09-resolved-text-geometry-metrics-contract.md).
Their exact source snapshots 3147/3148 supersede only the selected input paths; other owner
identities and original performance/product gates remain intact. Independent review of 3128
and these increments was pending at that snapshot; ResourceManagement acceptance and formal closeout remain pending.

2026-09-08 independent review in the existing task "优化协调器验证效率" completed with
Critical 0 / Important 0 / Moderate 0. Effective source 3128 -> 3147 -> 3148 -> 3151
has 15 unique paths including one deletion; current attribution, ObjectStore and staged
validation inputs matched before and after review. Four reviewed records were snapshots
3149, 3152 (slot/text only) and 3160 (shadow). Report:
`.codex/tmp/interface03-3160-review-20260908-result.txt`. No reviewer ownership conflict.
This source review does not replace the outstanding full-library, upward product/performance,
formal fixing-Session validation, failure return or closeout gates.
