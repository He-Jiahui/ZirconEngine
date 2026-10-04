---
title: Editor05 Inspector Transform Edit Entity Latch
category: zircon_editor
report_id: Editor05-transform-edit-entity-latch-2026-09-29
date: 2026-09-29
last_rechecked: 2026-10-01
implementation_status: pending_apply
validation_status: not_run
performance_status: pending_editor_product_gate
candidate_id: root-editor05-f4-origin-guard-v7
---

# Editor05 Inspector Transform Edit Entity Latch

## Finding

Transform-axis text editing had no stable entity identity across the full
native Focus/Edit/Submit lifecycle. A typed Submit could fall back to the live
`entity://selected` alias, so selection changes could retarget the write. The
native text buffer could also retain A's draft while an Inspector sync
projected B into the visible axis field. Finally, the commit bridge consumed
the latched identity before typed runtime dispatch was known to have succeeded.

## Candidate behavior

- Native Focus captures the current Inspector `NodeId` for that axis. If a
  legacy caller sends Change without Focus, the first Change captures the
  current exact Inspector id before editing the field. Subsequent Changes keep
  that id; a no-target Change blocks the gesture. No Submit resolves the live
  selection or `entity://selected`.
- The retained-host automation adapter sends the same private Focus action
  before its componentized transform Commit, so scripted edits use the entity
  identity lifecycle instead of an unarmed Submit fallback.
- A selection identity change abandons the active axis edit before the bridge
  projects the new Inspector snapshot. The host clears native focus data, sends
  the stored Blur action through the existing private surface-action/bridge
  route, and blocks the old Submit until a fresh Focus. Inspector values and
  native input state therefore move to B together.
- Typed Submit peeks the armed `node://<id>` and preserves the existing finite
  `f32` parsing and one-axis batch. The bridge consumes the latch only after the existing typed Inspector
  transaction dispatch succeeds. Parse and runtime dispatch errors leave
  the original gesture target available for correction and retry.
- Abandonment uses the stored Blur action on primary-pointer focus clearing,
  Escape, native window-focus loss, window hide/close, Inspector selection
  synchronization, and the Play-preview rising edge. The two app-owned paths
  dispatch inline after releasing native window-state borrows, avoiding a
  nested `Rc<RefCell<RetainedEditorHost>>` callback borrow.
- Axis-only writes, M0 high-precision preservation, and no-op transaction
  behavior remain covered.

## Regression coverage

`zircon_editor/src/tests/editor_event/runtime/when_evaluation/transform_edit_identity.rs`
covers Change-without-Focus capture, A-to-B selection cancellation, stale
Submit rejection, successful fresh B edit, and an injected typed dispatch
failure followed by corrected retry that remains addressed to A. Existing
finite-value and exact-axis typed batch fixtures in
`zircon_editor/src/tests/editor_event/runtime/when_evaluation.rs` now arm edits
through the real snapshot and Focus bridge route.

`zircon_editor/src/ui/retained_host/app/tests/inspector_transform_edit_lifecycle.rs`
uses the actual native pointer and text adapters, the `Rc` host callbacks, the
hierarchy selection/sync path, and the real host tick. It covers native
pointer-focus → text edit → select/sync B → Enter (no write and no stale
buffer), a fresh native B edit, and Play-preview focus abandonment before
Enter. `zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/dispatch/owned_control_id_tests.rs`
continues to cover focus-owned control dispatch.

The existing automation regressions remain in
`zircon_editor/src/ui/retained_host/app/tests/retained_host_automation.rs`; the
production automation adapter now arms transform commits with a Focus action
before Submit.

The existing M0 high-precision empty-apply regression and transaction identity
no-op regression remain in
`zircon_editor/src/tests/host/binding_dispatch/inspector.rs::inspector_binding_empty_apply_preserves_high_precision_transform_and_history`
and
`zircon_editor/src/tests/editor_event/runtime/stack_play.rs::inspector_no_op_trace_does_not_reuse_a_prior_transaction_identity`.

## Candidate boundary

This is an offline candidate only. No source file has been applied and no Rust
or managed validation has run. The exact source preimages, candidate hashes,
and exact-byte patch digest are recorded in the immutable V7
candidate manifest. V3 is historical and its same-ID world boundary was incomplete. Its two document drafts are superseded by the current
shared records; prepare current document preimages before applying any bundle.
Independent review and complete source admission must precede validation.

## Acceptance

Keep this record at `pending_apply` until the patch is admitted against its
exact preimages. Then include the runtime identity and native lifecycle
regressions in the grouped managed Editor validation batch and record its
actual receipt. Static review is not a passing Rust test or runtime acceptance.

## 当前实施记录（2026-10-01）

| 范围 | 状态 | 证据与剩余门 |
| --- | --- | --- |
| 历史 V3 源码保留型候选 | `pending_apply` | V3 manifest `da6f8eca62f7c2b247960f76595fe8ea383eb44e055b98559eb517b7ba7a3727`；3 处当前外部修改已合并，24-target 精确回放与 21-file 格式检查通过。 |
| 历史 V3 source admission | `blocked_by_active_source_owner` | 本次公开 preview：23 eligible、1 blocked；tick.rs 属于活动 Session astra-ui-time-20260929-01a0f033-r2，未覆盖、未强制转移、未应用部分代码。 |
| 独立审查、受管回归、产品性能 | `open` | 旧 review receipt 不存在；当前 V7 与 gpt-6-sol 独立审查见下文；Cargo/产品测试尚未运行。 |

完整完成列表及当前验证边界：
[Editor1063](../../../astra/features/editor/1063-editor05-transform-edit-entity-identity-completion-list.md)。

异步验证按里程碑合并 check、普通回归与边界检查后统一提交。提交记录保存后继续独立修复；到下一个有实际修复结果的边界再核验既有 receipt，不实时跟踪编译。

## 场景替换、选择代次与精确目标修复（当前 V7 候选）

旧 V3 只比较 NodeId，场景重载后复用同 ID 时会保留旧草稿的写入资格，不能作为完整身份边界接受。V7 在同一次快照状态和世界借用中记录现有 GatewaySessionIdentity、文档 ID、选择 revision、WorldDomain 和实体 ID。来源使用 AuthoringWorld 已保留的实际 gateway identity；若 context 的当前 identity 与它不一致，则不授予编辑来源。InspectorEditOrigin 是只读来源描述，未建立第二份编辑权限。

宿主在 Transform Focus/Change/Submit/Blur 前比较投影与当前来源；失效时取消缓存编辑，根窗口原生焦点经存储的 Blur 路径清除并请求现有 presentation/render 刷新。投影重建也比较完整来源，覆盖同 ID 重载。Typed Submit 保留正常 event journal、Inspector 应用、事务和回滚路径；执行入口在同一 shell mutation lock 内再次校验来源，并要求 node:// 的精确实体等于来源实体，防止使用 B 的当前来源提交仍指向 A 的旧绑定。失败后修正重试与成功后消费 latch 的原行为保留。分离窗口的真实 transform/focus 路径仍在独立审查，不能由根窗口回归推断已覆盖。

六个新回归断言覆盖：同 ID 重载后下一次 tick 前 Enter、同 ID 重载后的投影重建、A→B→A、文档重新绑定、绕过 UI 后的 authoritative Submit，以及用另一个实体的当前来源提交旧绑定。重载夹具注册并激活真实 SceneModule，通过 prepare_authoring_world/replace_world 替换，随后重新绑定原文档并走 hierarchy selection event 重选同 ID；这样新的有效输入仍经过真实历史事务。五个原来源用例及第六个目标用例分别先于对应实现写出；Cargo 尚未执行，没有观察到 red/green。

- [x] 私有 V7 来源与目标校验、六个行为回归已写出；49 个源码/asset 目标精确补丁回放、48 个 Rust 格式检查通过。
- [x] 所有 canonical InspectorSnapshot 构造点补齐来源字段；展示 fixture 无来源，不能据此取得 Typed Submit 写入资格。Play 投影按缓存 gateway/entity 匹配后附来源。
- [x] 修正世界重载回归的真实 level 服务注册与文档绑定依赖。
- [x] 当前 gpt-6-sol 独立静态审查成功返回，49 个候选哈希匹配，未发现确认的必修项。此前失败或中断的审查不计为通过；测试仍未执行。
- [ ] 全部 49 路径完成当前来源准入；V7 当前 public preview 为 48 eligible、1 active executable owner 阻挡，未移交或部分应用。
- [ ] 完整源码合法应用，并合并 check、普通与原生输入回归批次。
- [ ] 真实 Windows 场景保存、关闭、重开与产品性能验收。
- [ ] 原 Inspector 计划的 scene/document/schema/selection-generation 和 M0-M5 门闭合。来源尚未包含 schema generation，不能推断完整门通过。

| 当前证据 | 实际结果 |
| --- | --- |
| V7 manifest | offline-candidates/root-editor05-f4-origin-guard-v7/prepared.json；SHA d1e723bee5ba77125a142066ec56c584536ab1ca5649442a5d22d28f675c9b20。 |
| V7 exact patch | 同目录 candidate.patch；SHA 7b53b91a434f6ccb885d91ac2a48003e7ccdb971ed50a9563ab94da3b63f795c。 |
| V7 exact replay | 同目录 exact-replay-receipt.json；SHA 2c489b16ef4bda5bb442f8d18a696a93c2e0fcfe64b0a740b78978f12258d1ff；49 字节一致目标，48 Rustfmt exit 0。 |
| V7 独立静态审查 | 同目录 sol-independent-review.json；SHA 00643e626023b48d7c9a25d3c4b2e72490e8f47b304f56637f08ce706f192a21；findings 为空，未编译或执行回归。 |
| V7 当前 49 路径 source preview | 2026-10-01-editor05-f4-origin49-reviewed-public-source-admission-preview.json；SHA 53e8fe3a7f930dd9e48beb2afff53ee42bbffdee6758ff281af47528995d6131；preimages 无漂移，48 eligible、1 阻挡。 |
| 历史 48 路径 source preview | 2026-10-01-editor05-f4-origin48-public-source-admission-preview.json；SHA 2f07f643c258061114f722919123e94ac4ed950c80f0e1018b97d80d22e6e14e；当时 47 eligible、1 executable owner 阻挡，不能作为当前 49 路径准入通过。 |
| 来源准入 | 当前 V7 preview 的 tick.rs owner 为 astra-ui-time-20260929-01a0f033-r2，状态 active；未强制转移、未部分应用跨 API 改动。完整准入保持开放。 |
| 编译与性能 | 未提交 Editor validation ticket、未运行 Cargo 或产品性能采样；目标 active，记录 pending_apply。 |

原来的 V5 记录由本次 V7 记录替代。四记录租约与归属回执 2026-10-01-runtime51-editor05-v7-four-current-record-fresh-lease-attribution.json（SHA 7ccbd5cde9b2868cfd80fcccd82ed94473f55c4932ee23889c84d643f53461f8）证明此前记录字节已归属；这次补入审查与 preview 的新记录字节仍需重新归属。实际源码准入、编译及产品验收保持开放；未收到 terminal 结果不计通过。
