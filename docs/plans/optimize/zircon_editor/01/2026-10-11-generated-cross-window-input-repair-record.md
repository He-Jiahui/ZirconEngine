---
related_code:
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/generated_bottom_pane.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host/generated_bottom_pane.rs
  - zircon_editor/src/ui/retained_host/app/tests/generated_bottom_pane.rs
doc_type: source-repair-record
title: Editor Generated 跨窗口旧输入退休修复记录
date: 2026-10-11
implementation_status: source_adopted_native_and_performance_pending
validation_status: source_review_and_exact_adoption_complete
performance_status: original_contract_pending
---

# Editor Generated 跨窗口旧输入退休修复记录

本记录说明公开 Attach 将 Generated pane 从 native child A 移入 root Drawer 后，旧窗口输入退休路径会隐藏共享 Generated holder 的缺陷、Source 修复及其验收边界。Source 修复已实际采用并精确匹配四个 canonical postimages；本文不声称已运行 Native、性能或产品验收。

## 缺陷合同与触发顺序

冻结的 Source 合同描述以下真实所有权顺序；它是源码确认的缺陷路径，不是一次 Native 复现记录：

1. 激活 Workbench 并发布 editor.generated_bottom。
2. 通过实际 controller-backed detach 路径，让 native child A 同时持有 editor.assets#1 与 active Generated pane；在 authored filter 上接纳 normalized Down，再接纳 IME Preedit，令 A 保留 Generated 的旧输入地址。
3. 通过公开 AttachView 将 Generated 移到 root Bottom drawer。A 仍是可见的 native target，但 active pane 已变为 Assets。
4. 普通 projection 先发布 destination 的 Generated Source。随后旧窗口输入退休逻辑仍使用 A 的旧地址查找 Generated。
5. A 的查找受其 native_floating_window_id 与 active pane 限制；A 现在只呈现 Assets，因此它找不到 Generated。旧地址的退休随后走现有 deactivate 路径，隐藏由 registry 共享的 Generated holder，并清除其 publication、pointer/input-result proof。
6. 原 Generated reconciliation 只看到 destination Source 存在，因而跳过恢复。虽然目的窗口有 Source，实际 holder 已隐藏，Generated admission 会拒绝当前 owner；直到另一次正常 publication 才恢复输入。

简单的 root-to-child detach 不是这个反例：root 查找可以沿 floating_layer 找到已移动的 native pane。关键在于旧地址属于仍保留 Assets 的 child A。

## Source 修复与 epoch 边界

所有权仍由既有 EditorUiHostRuntime.template_action_registry 和它持有的 Generated holder、Surface、input manager、visibility、epoch 与 publication proof 管理。每个窗口的 GeneratedBottomPaneData.Source 和 projected node/token bindings 仍是派生 presentation data，不作为 holder 可见性或输入权限的权威。

修复增加只读的 crate 内 getter generated_bottom_pane_visible，在现有 registry mutex 下读取 holder.visible。旧窗口的 focus、IME、press/capture、pointer continuation 和旧 Source/token 权限仍通过现有 deactivate 路径退休；没有放宽输入取消、Source proof 或 epoch 检查。

当仍被窗口呈现的 Generated holder 已被旧窗口退休逻辑隐藏时，reconcile 标记全局 presentation dirty 并请求恢复。随后 recompute 最多执行一次普通 try_recompute_if_dirty 完整转换/native sync，让现有 projector 重新发布仍呈现的 holder。若额外 pass 失败，沿用既有 dirty 与 status error 路径留给后续正常 retry；不递归，也不建立无界重试。空 registry 或已可见且仍被呈现的 holder 不要求额外 projection。

epoch 的边界必须精确区分：hide 清理 focus/window-focus、wrapper visibility、publication、pointer proof 与最后输入结果，但 hide 本体不写 holder.epoch。外层 deactivate 在 hide 成功后推进 registry epoch 并赋给 holder；随后普通 project 在 publication 缺失时再产生新的 Source/token publication epoch。本文不把 epoch 推进归因于 hide 本体。

## Regression 与 Native 选择合同

canonical test file 增加一个非 ignored 的普通 Rust case：

ui::retained_host::app::tests::generated_bottom_pane::generated_bottom_public_floating_attach_retires_old_input_and_republishes_current_source

该 case 覆盖 child A 的真实 holder/Source identity、paint/hit、normalized input 与 IME、公开 Attach、旧地址及旧 token 拒绝、destination 即时 holder 恢复，以及恢复后的实际输入、producer association、journal、feedback 与 paint。旧 test file 的 830 行、31,769-byte 原始前缀逐字节保留。

ordinal 15 的原普通 Native case 数量下限为 191；追加的一个唯一普通 case 使该选择器下限变为 192。既有参数、feature、选择器和 required names 保持不变。冻结 selector 字段 originalPerformance.minimumSamples 中的 191 表示历史普通 Native case 数量下限，不是性能测量 sample，也不构成性能指标或验收。

完整后续批次保留原 15 条命令和 14 个原性能 producer，并加入两个 Runtime full-matrix producer，总计 16 个 producer。Editor 新增 ordinary case 不是性能样本。Batch06 正在准备一次完整异步启动；截至本记录，Native 尚未运行，本文不记录成已启动或已通过。

## Canonical adoption 证据

实际 apply receipt 报告 actualApplyExit=0，四个物理 postimages 与冻结结果精确一致：

- zircon_editor/src/ui/template_runtime/runtime/runtime_host/generated_bottom_pane.rs
- zircon_editor/src/ui/retained_host/app/generated_bottom_pane.rs
- zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
- zircon_editor/src/ui/retained_host/app/tests/generated_bottom_pane.rs

共享 .git/index 的前后 SHA、长度和时间相同，indexWrites=0。两个旧 foreign inputs 完整保留：zircon_editor/src/ui/retained_host/ui/pane_data_conversion/generated_bottom_projection.rs 与 zircon_editor/src/ui/retained_host/host_contract/window/retained_pane_input.rs。receipt 还报告 sourceCopies=0、nativeRuns=0、compilerInvocations=0、actualReleasedBytes=0。

参考证据：

- [实际 canonical apply receipt](../../../../../.codex/outbox/editor-generated-cross-window-retirement-repair-v1/actual-canonical-apply-v1.json)：SHA256 597e08967a0a82d7901e51a9c637bdc342dd6740d7cf6a8f313800129cdf6150，12,903 bytes。
- [Frozen Source contract proof](../../../../../.codex/outbox/editor-generated-cross-window-retirement-repair-v1/source-contract-proof-v1.json)。
- [Independent peer review](../../../../../.codex/outbox/editor-generated-cross-window-retirement-repair-v1/independent-review-v1.json)。
- [Frozen Native selector](../../../../../.codex/outbox/editor-generated-cross-window-retirement-repair-v1/native-test-entrypoints-v1.json)。
- [Source completion list](../../../astra/features/editor/2026-10-11-generated-cross-window-input-source-completion-list.md)。

## Acceptance 状态

Source review 与 canonical adoption 已完成。Native tests、原性能合同、Editor products、Jenkins 与 milestone acceptance 均未通过或尚未运行，保持 pending。actualReleasedBytes=0；本记录不声称有空间释放。完整状态见[Source completion list](../../../astra/features/editor/2026-10-11-generated-cross-window-input-source-completion-list.md)。
