---
doc_type: feature-completion
status: pending_apply
last_rechecked: 2026-10-01
candidate_id: root-editor05-f4-origin-guard-v7
implementation_status: offline_current_preserving_candidate_only
validation_status: not_run
performance_status: pending_editor_product_gate_no_performance_claim
plan_sources:
  - docs/plans/optimize/zircon_editor/05/2026-09-29-inspector-transform-edit-entity-latch.md
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui
  - zircon_editor/src/tests/editor_event/runtime/when_evaluation.rs
  - zircon_editor/src/ui/retained_host/app/automation.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface/edit.rs
  - zircon_editor/src/ui/retained_host/app/tests/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/transform_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/text_focus.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/text_focus/activation.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/text_focus/clear.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/focus.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/dispatch/owned_control_id_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard.rs
  - zircon_editor/src/ui/workbench/snapshot/data/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/workbench/state/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/host/play_inspector_projection.rs
  - zircon_editor/src/ui/layouts/views/inspector.rs
  - zircon_editor/src/ui/workbench/fixture/preview_inspector_into_snapshot.rs
  - zircon_editor/src/tests/ui/inspector/bootstrap_assets.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/host/pane_presentation/support.rs
  - zircon_editor/src/tests/host/retained_inspector_template_body.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/scene_snapshot.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_inspector_property_edit.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/mod.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/play_inspector.rs
  - zircon_editor/src/ui/retained_host/app/inspector.rs
  - zircon_editor/src/ui/binding_dispatch/error.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/inspector_event.rs
  - zircon_editor/src/ui/host/editor_event_execution/mod.rs
  - zircon_editor/src/core/editing/authoring_world.rs
tests:
  - zircon_editor/src/tests/editor_event/runtime/when_evaluation/transform_edit_identity.rs
  - zircon_editor/src/ui/retained_host/app/tests/inspector_transform_edit_lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/dispatch/owned_control_id_tests.rs
---

# Editor1063: Editor05 transform edit entity identity

The candidate latches a transform axis's exact Inspector `NodeId` at native
Focus (or first Change for legacy callers). Submit requires an armed
`node://<id>` and never falls back to the live selected-entity alias. A typed
runtime failure leaves the latch available for correction; only successful
dispatch consumes it. When selection changes, the host clears native focus and
routes the stored Blur action before syncing B's axis values, so a stale Enter
cannot write either entity. Play-preview activation uses the same stored Blur
route.

| Case | Regression assertion | Status |
| --- | --- | --- |
| Focus/Edit A, selection sync to B, then old Submit/Enter | Old draft is canceled, input buffer is cleared, and A/B transforms remain unchanged. | Candidate added; grouped managed run pending. |
| Fresh native Focus/Edit/Enter on B | Only B's requested axis changes; A and every other axis remain unchanged. | Candidate added; grouped managed run pending. |
| Typed dispatch fails while editing A, then correction and retry | Failed dispatch preserves A's latch; corrected Submit writes A and cannot retarget B. | Candidate added; grouped managed run pending. |
| Play-preview rising edge abandons an active edit | Stored Blur is dispatched, native focus is cleared, and stale Enter does not write. | Candidate added; grouped managed run pending. |
| Same-value Submit and M0 precision behavior | No-op does not advance world generation; typed axis batch retains `f32` precision and writes only its axis. | Candidate added; existing M0 fixtures retained; grouped managed run pending. |
| Retained-host transform automation | Automation sends Focus before its componentized transform Submit, preserving the exact-entity lifecycle. | Existing callback regressions retained; grouped managed run pending. |

No source has been applied from this offline bundle. Rust execution and managed
Editor validation are unrun; keep this record at `pending_apply` until root
admits the exact patch and promote it only from the grouped validation receipt.

## 当前完成列表（2026-10-01）

- [x] 对当前源码重新合并 3 个漂移文件；保留 Runtime frame owner/backoff、Hierarchy EntityId 展开与筛选状态、关闭窗口的 pointer-capture 处理。
- [x] 私有候选 V3 的 24 个目标按实际字节回放成功；21 个 Rust 目标格式检查通过（历史节点）。
- [x] 当前记录路径已完成精确 ownership transfer、lease 与 plan maintenance 授权。
- [x] 当前 V7 的 gpt-6-sol 独立静态审查返回，未发现确认的阻塞缺陷；构造点、事务路径和真实夹具 API 已核对。历史缺失或失败的审查未计为通过。
- [ ] 完整源码合法应用。tick.rs 当前属于活动 Session astra-ui-time-20260929-01a0f033-r2；当前仅写入记录，没有部分应用相互依赖的代码。
- [ ] 合并受管 Editor check、普通回归和相关边界测试批次并核验 terminal receipt。
- [ ] 执行当前 V7 scene/document/同 NodeId、selection ABA 与精确目标回归并核实 authoritative 拒绝边界。
- [ ] 满足依赖门后运行真实 Windows 编辑、保存、销毁与重开，以及计划要求的产品性能矩阵。

## 证据与边界

候选 manifest：`.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor05-f4-current-preserving-v3/prepared.json`
（SHA-256 `da6f8eca62f7c2b247960f76595fe8ea383eb44e055b98559eb517b7ba7a3727`）。
精确回放 receipt：同目录 `exact-replay-receipt.json`
（SHA-256 `6cfad265b999ea774f41d6305c37b2e7dcbac2dede1d81fb8c856702ba3f7494`）。
该回放发生于记录创建前；冻结候选中的两个旧文档草稿现已由本次记录取代，后续应用必须重新生成文档 preimage，不能直接应用旧 24-target 补丁。

原计划为 `docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md`；
本候选没有关闭其全部 M0-M5 或产品验收门。当前没有 Cargo 执行结果和性能达标数据；静态检查不能代替这些门。

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
