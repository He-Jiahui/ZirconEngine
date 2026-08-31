---
status: architecture-review-complete-correctness-containment-implemented-runtime72-baseline-blocked
created_at: 2026-08-31
parent_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
session_id: root-editor08-keymap-playmode-exact-successor-20260831
related_code:
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/commands/keymap/tests.rs
  - zircon_editor/src/core/commands/when.rs
  - zircon_editor/src/ui/host/editor_manager.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/InputBindingManager.cpp
---

# Editor08 keymap binding-context current-source architecture review

## 结论

current source 已新增 signature bucket、按当前 `CommandEvalCtx` 过滤同 chord 候选的输入分派，以及 `WhenClause::can_overlap_in_interactive_context`。这些文件的历史 owner 均已 archived；本轮只通过 audited current-blob transfer 接管，transfer request 为 `89e2a67272d64321bb8d48a2d24a60e0`，未复用历史 blob、未使用 maintenance override。

当前实现不能作为 Editor08 M2.2 完成态。`when` 是命令 enablement predicate，不是稳定的 binding context 身份；用任意布尔可满足性求解器推导快捷键域，把命令注册、插件 capability 和输入域混为一层。生产代码也没有消费 `conflicts_with_when`：current workspace 的 5 个引用中，1 个是方法定义、4 个都在单元测试。设置页、注册事务和 extension materialization 均未发布冲突 generation，因此当前只有上下文 enablement dispatch，没有产品级域冲突闭环。

## Current baseline 与复杂度

- 内建 keymap 当前有 9 个 binding；按 `EditorKeyChordSignature` 建立 bucket，键盘热路径只扫描命中 bucket，并在两个 enabled command 同时存在时无副作用拒绝。该路径的结构方向可保留。
- 默认 chord 同时存在于 `default.keymap.toml` 与 `EditorCommandDescriptor::default_chord`。manager 输入服务从 TOML 构造 base bindings，menu/palette 先读 descriptor、再由部分 workbench projection 覆盖 effective chord。registry 的 `missing_default_keymap_bindings` 只能发现缺项，不能发现同 command 的 chord 值漂移；extension command 的 descriptor 默认 chord 也不会自动进入 manager keymap generation。这是双权威，不是可保留的 preset 设计。
- extension contract 还缺少承载该结构的 ABI：`zircon_runtime_interface` 的 `SerializedEditorContribution::Command` v3 只有 id、本地化 key 与 execution contract，插件 SDK builder 没有 default chord、binding context 或 structured when 参数；`core/plugin/materializer.rs` 因而只能注册 `default_chord = None`、`when = Always` 的 native descriptor。仅在 `zircon_editor` 内新增 context 类型会留下第二条无 context 的插件路线，必须作为跨 crate schema 硬切处理。
- `conflicts_with_when` 先按 chord 分组，再对 bucket 内 command 做两两比较，外层为 `O(k^2)`。
- 每对 command 的 overlap 会收集全部原子并递归枚举 truth assignment；一般布尔表达式最坏为 `O(2^a * expression_size)`。`BTreeSet` 只在入口收集原子一次，递归克隆的是包含 `BTreeMap` 与原子拥有数据的 `WhenAssignment`，其中 capability 使用拥有的 `String`；current schema 没有 atom/depth/branch budget。
- 该 overlap API 尚无生产 consumer，所以目前不能把它描述成已观测产品瓶颈；也不能因为静态最坏界就宣称优化收益。接入产品前必须先增加 branch-count、wall-time、allocation 与 1/1k/10k command-context profile。
- 当前没有可运行的 current Editor binary，未取得输入到 dispatch 的 ETW/CPU/alloc 产品数据；本记录不包含延迟、功耗或相对 Unreal 的性能声明。

## 已证实的正确性 RED

`CommandEvalCtx.play_state` 始终是 `Edit | Building | Playing | CleanupFailed` 之一，但 current solver 把这一轴建模成“最多一个”，允许四个 play atom 全部为 false。于是以下两个真实互斥谓词会被错误判为可重叠：

1. `Not(PlayMode(Edit))`；
2. `Not(Building) && Not(Playing) && Not(CleanupFailed)`。

前者只在 Building/Playing/CleanupFailed 为真，后者只在 Edit 为真，不存在共同 interactive context。`keymap_allows_same_chord_for_exhaustive_disjoint_play_mode_domains` 已加入 `keymap/tests.rs` 作为 RED 合同。containment 已让 overlap 在谓词引用 PlayMode 时显式尝试 4 个真实状态，禁止空状态进入布尔求解；未引用 PlayMode 时仍只执行一次求解。managed execution 尚未取得，不将源码测试存在误报为通过。

## Unreal 参照与适用裁决

UE `FBindingContext` 给命令一个稳定 context name 与可选 parent；父子 context 之间禁止重复 binding，兄弟 context 可以独立。`FInputBindingManager` 对 active chord 使用 context-local map，并沿当前 context 的 parent/children 查找；command filter 只决定命令是否可执行，不反向定义 context 拓扑。

Zircon 应复用这一结构原则，而不是复制 Slate 类型：

```text
EditorCommandBindingContextRegistry
  global
    workbench
      project
        document.scene
        document.material
        document.animation
        scene_mode.<id>
        toolkit.<id>

(keyboard signature, binding context) -> command slot
active context stack -> most specific to parent -> global
WhenClause/required_capabilities -> final enablement only
```

## 目标结构

1. 新增 typed `EditorCommandBindingContextId` 与 registry-owned context descriptor，包含 stable id、parent id、owner ticket 和确定性 priority；注册时拒绝 missing parent、cycle、超深链与重复 id。
2. 共享贡献 ABI 同批硬切：退役 command v3，新增不兼容的 command schema 与 binding-context contribution；插件必须显式提交 context id，可选提交 default chord，并通过 shared typed when DTO 表达可序列化 enablement。SDK builder、runtime validation、editor materializer 与 registry transaction 同一切片切换，不保留 v3 fallback 或 `Always/workbench` 静默补值。
3. `EditorCommandDescriptor` 显式引用一个 binding context；内建 command 默认 `workbench`，document/toolkit/scene-mode owner 声明自己的子 context。插件 capability 不再隐式产生域。
4. command registry 的 immutable catalog 成为默认 chord 唯一 owner。keymap generation 按 `descriptor default -> selected preset delta -> typed settings override` 物化；默认 preset 不再重复列出同一批 descriptor chord，旧 `default.keymap.toml` 清单硬删除或改成只含真实差异及 version/preset id 的 delta artifact。
5. keymap generation 建立 `(signature, context_slot)` 索引；冲突只在同 context 或 ancestor/descendant 链上成立，复杂度受 context depth 上限约束，不执行通用 SAT。
6. `CommandEvalSnapshotHandle` 发布 immutable active-context generation。输入热路径按最具体 context 向父级探测，再用现有 when/capability snapshot 做最终 enablement；不得在每个 key event 重建 context 图或全表扫描。
7. 设置 override 重建时发布 immutable conflict generation，设置页和 extension registration 读取同一结果；missing command/context fail-close，并给出 typed diagnostics，不按 operation path 静默择一。
8. `WhenClause::can_overlap_in_interactive_context` 在 binding-context 切换完成后退出 keymap 权威；若其他静态分析仍需要它，必须独立限额、补齐 exact-state 语义并标明不是 dispatch owner。

## 实施与性能门

### R0 · RED 与 profile harness

- 保留 PlayMode exact-one RED；补 same-context、parent-child、sibling、missing-context、cycle、depth-limit、两个 enabled candidate 的合同。
- 加入 descriptor/TOML chord mismatch、extension default chord 自动进入 generation、preset delta/setting override precedence 与 registry generation 失效合同。
- 在受管 target 上记录 9/1k/10k commands、1/10/100 contexts 的 generation build、conflict publish、single-key dispatch：p50/p95/max、visited bucket/context count、allocation bytes 和 command-registry lock wait。

### M1 · Correctness containment

- `implemented / managed pending`：PlayMode exact-one 已用 4 个真实状态枚举修复，未把 current SAT conflict API接入设置或插件注册链。
- atom/depth/branch fail-close budget 仍 pending；该修复只是 containment，不作为最终架构。

### M2 · Binding context hard cut

- descriptor/context registry、registry-derived default bindings、preset delta、active-context generation 与 `(signature, context_slot)` index 同一切片落地。
- `zircon_runtime_interface` command schema、plugin SDK builder、serialized binding-context contribution、materializer 与 registry transaction 原子硬切；删除 command v3 接受路径并补 missing/unknown/cyclic context 的跨 crate 拒绝合同。
- 删除 keymap 对 `WhenClause::can_overlap_in_interactive_context` 的依赖，不保留旧 SAT compatibility route。
- 删除重复默认 chord 清单和 `missing_default_keymap_bindings` 这种只能发现缺项的双权威补丁，不保留两套 base binding owner。

### M3 · 产品闭环

- settings override、extension/toolkit registration、菜单 shortcut projection 与 native input dispatch 消费同一 context/conflict generation。
- 产品测试覆盖 focus/toolkit/scene-mode 切换、同 chord 兄弟域共存、父子冲突告警和失焦后 context 立即退出。

### M4 · 竞争性验证

- 只有 managed Cargo、产品交互和 profile 全部通过后，才比较 Zircon 与 UE 式 context-map 的规模趋势；目标是 key dispatch 随总 command 数不增长，conflict rebuild 为 `O(bindings * bounded_context_depth)`。

## Managed validation terminal 与既有 Runtime72 Failure

精确六路径 managed ticket `7a5ec9132cc74ea1871b19fd1e712e83` 已完成 materialization 并实际启动 Cargo；copy job 为 `4969e15f859c41e596ba5b5da7b43f6f`，source manifest 为 `0a2a1076ebb4251a81c31fc931eeda74ef8b24d077e02c2d18f51c5fea0eb6ed`。运行从 2026-08-31 04:31:03 +08:00 持续到 04:51:47 +08:00，最终 exit `101`，但 focused keymap test 尚未开始：依赖 `zircon_runtime` 编译在 `runtime.rs` 调用 `CoreHandle::active_module_shutdown_order` 时出现单个 `E0599`。

该错误已有 canonical Failure，禁止重复创建或 import：`docs/plans/optimize/zircon_runtime/72/failure-2026-08-22-active-ledger-owner-wiring.md`，当前 SHA-256 `3EF652FE5DD2A5C4BFB1CA6D3AD4C03289DCF43A69B0D21431D75A9947A46E04`，fixing plan 为 Runtime72。共享 HEAD `14c89f9776bed828cc85e05e4b9914b3f8d1e784` 包含 active-ledger shutdown callsite，但 `CoreHandle` accessor 未进入 HEAD；git history 将该 callsite 定位到 `08094b9b9e17f6c80372e15c17b01204038b305b`。共享 current worktree 已有 forward accessor，`runtime.rs` SHA-256 `51C155CFBE89FFBCD84653629A91F37B9E5C5EB8E8D7B7809B4B191552219A63`、`core_handle.rs` SHA-256 `0CAC0C5BB6F7FE8A3E9C996CEF6AC9A91A20AD25778752A8ABABF8FF4C474452`，但这两个 mixed current blob 尚未由 Runtime72 以完整 ledger owner/activation hooks 闭包集成，因此 Editor08 不能把它们附带进自己的 validation manifest。

本次结果只证明精确 Editor08 overlay 已成功物化，以及 repository baseline 被既有 Runtime72 P0 阻断；不证明 keymap test 通过，也不推翻独立 review 的 `Critical 0 / Important 0 / Minor 0`。Runtime72 Failure fixed 并进入共享 baseline 后，Editor08 应以更新后的 parent/child plan hash 重提同一 focused command；不得添加 Editor 侧 stub、恢复 frozen-graph fallback、扩 validation 到整个 dirty runtime，或把 exit 101 标记为 keymap 失败。

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-31 02:58 +08:00 | `current-topology-reviewed / ownership-transferred / correctness-containment-implemented / static-verified / managed-pending / architecture-hard-cut-planned` | 复核 keymap/when/manager current blob 与全部 consumer，确认 9 个内建 binding、signature bucket 热路径、production conflict consumer 为 0、pairwise + 通用 SAT 最坏指数界；对照 UE `FBindingContext/FInputBindingManager`，确定显式 context registry、父子域、context-local signature index 与 immutable active-context generation 的硬切方向。进一步核对共享 contribution ABI、plugin SDK 与 materializer，确认 command v3 无 chord/context/when 且插件 descriptor 被物化为无默认 chord 的 `Always`，因此把不兼容 schema、binding-context contribution、SDK/materializer/registry transaction 原子切换列为 M2 前置，而非编辑器内部补丁。新增 PlayMode exact-one RED，并以 4 个真实 PlayMode 状态枚举完成 containment；`when.rs` SHA-256 `B3F10B51DB980D47B53CBBD077FF363E2E7644D7129E4B38D3E847FFFAE58CFB`，test owner SHA-256 `D75F775D1CE05083A30570369E361CF2D5F5FB0C8D717C945ACF25B2ACC886CA`，rustfmt/diff-check 通过。managed v1 ticket `1aae67f1ed964bf1833e59e83bd4d647` 在 Cargo 前终态 `snapshot_stale`；同一 exact manifest `f32ac802135753711b2cc6c9b887406d96280b12bbea538885c90dda05ca543b` 的 v2 ticket `e862dc1b389e4da1890e7562f89e8281` 也在 `owned_overlay` 终态 `validation_copy_attribution_stale`，copy job `f4970dfd06214bd09b3d2ab36589faa6` 指向该旧 Session write scope 内的 `zircon_editor/assets/i18n/en.toml`，未运行 Cargo。产品 profile、Cargo GREEN 与最终 binding-context hard cut 仍 pending，无性能收益声明。 |
| 2026-08-31 04:02 +08:00 | `independent-review-c0-i0-m2-resolved / clean-validation-admission-blocked` | 独立 reviewer 对 PlayMode 四态、三值剪枝、manager 双重 enablement 门禁与新增 RED 完成复审，结论 `Critical 0 / Important 0 / Minor 2`；已把 `conflicts_with_when` 当前引用修正为定义 1 + 测试 4，并纠正递归分配描述，两个 Minor 均已闭合。为绕开旧 Session 的 114-path stale overlay，fresh primary `root-editor08-keymap-playmode-exact-validation-20260831` 注册请求 `e16462a720794ac38e0c63397e31181a` 被 `plan_wip_limit_reached` 拒绝；reviewer Session 写 scope 重试请求 `5f759a9d` 被 `plan_wip_reviewer_write_scope_forbidden` 拒绝。故当前精确源码仍只有 rustfmt/diff-check 与独立静态复审证据；在旧 primary 合法落地或释放前不吸收 114 个无关路径，不误报 managed GREEN。 |
| 2026-08-31 04:24 +08:00 | `oversized-primary-cancelled / exact-successor-owned / managed-resubmit-ready` | 旧 primary `root-editor08-ticketed-command-router-hardcut-20260829` 的 terminal tickets 与历史 attribution 均保留，但因 114-path overlay 污染精确验证，以 request `ca0cbd41c2fd4f8f8504ef9982b9c412` 标记 `cancelled`；未删除或回退任何工作树文件。successor `root-editor08-keymap-playmode-exact-successor-20260831` 仅登记 parent/child plan、`keymap.rs`、`keymap/tests.rs`、`when.rs`、`editor_manager.rs` 六条路径，lease request `576f17519da541a1a88abf9e546fda8d`、exact attribution request `037c3d683e744a50b3bbe9956450238c` 均完成，后续验证不再吸收 `i18n/en.toml` 等无关 overlay。 |
| 2026-08-31 05:11 +08:00 | `exact-materialization-complete / cargo-started / external-runtime72-e0599 / no-duplicate-failure / keymap-managed-blocked` | exact ticket `7a5ec9132cc74ea1871b19fd1e712e83` 与 copy job `4969e15f859c41e596ba5b5da7b43f6f` 成功物化六路径 manifest 并运行 Cargo 20m44s，随后在依赖 `zircon_runtime` 的 `CoreHandle::active_module_shutdown_order` 缺失处以单个 `E0599`/exit 101 终止，focused keymap test 未开始。该根因已由 Runtime72 canonical Failure `active-ledger-owner-wiring` 持有，hash `3EF652FE5DD2A5C4BFB1CA6D3AD4C03289DCF43A69B0D21431D75A9947A46E04`；本轮只引用并补 managed reproduction，不重复 import、不修改 Runtime 源码、不误报 keymap GREEN。 |
