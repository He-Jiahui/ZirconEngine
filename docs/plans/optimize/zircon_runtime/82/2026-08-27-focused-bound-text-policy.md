# Runtime Text focused bound-value refresh architecture and profile plan (2026-08-27, updated 2026-08-28)

## 状态

`unreal_focused_refresh_policy_reviewed /
focused_bound_model_update_gateway_implemented_unvalidated /
revision_compare_and_swap_conflict_implemented_unvalidated /
secure_pending_surface_owned_unvalidated /
fixed_content_free_profile_counters_implemented_unvalidated /
managed_profile_power_wgpu_pending`

本报告仍不是验收产物。current-source 已完成非验收网关与测量点；managed Runtime、真实产品动态
binding producer、平台 IME、分配/RSS/功耗、matched Unreal 和 WGPU 尚无终态，不能宣称 focused
binding 或 Runtime82 gate 已验收完成。

## Unreal reference

本地参考源码
`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp`
给出两条一致规则：

- `3622-3636`：widget 聚焦时 editable text 本身不从 bound value 刷新；只刷新 password 和
  marshaller 状态。失焦后才执行 full refresh。
- `4508-4547`：`OnBoundTextChanged` 在聚焦且没有 `bForceBoundTextReview` 时返回。显式
  SetText/LoadText 可开启 force review；实际值不同才替换，必要时把 caret 移至末尾，并只在
  内容真的变化时发 changed event。

这里的重要结构不是一个 focus 条件，而是 `BoundText` 与 `EditableText` 本来就是两个状态，
并且显式 replace 与普通 attribute refresh 有不同策略。

## Current-source defect and correction

`UiPropertyMutationRequest` 只有：

- `source: UiReflectedPropertySource`，默认是 `RuntimeState`；
- `binding_source_kind`，用于把 mutation 投影为哪类下游 binding update。

二者都不能稳定回答该请求是外部 bound-model refresh、显式 SetText/LoadText replacement，还是
一次已经提交的 edit projection。当前 `UiSurface::mutate_editable_text_property` 对正文属性统一立即
生成 display text、修正 caret、清 selection/composition 并提交。retained template metadata 同时是
外部可见 property 与 editable buffer，缺少 Unreal 已有的 model/edit 双状态。

所以两个看似简单的修改都错误：

- 聚焦时继续立即写：外部刷新覆盖尚未提交的用户编辑；
- 聚焦时 early-return：模型新值既没有保存也没有 typed deferred receipt，更新静默丢失。

`UiPropertyMutationStatus` 仍只有 `Accepted/Unchanged/Rejected`，因此 generic property mutation 保留
为显式 replacement 兼容入口，不承担 bound refresh。新能力没有向该 DTO 塞 focus 特判，而是在现有
product document owner 上新增独立、版本化 model-update gateway。

current-source correction：

- `zircon_runtime_interface::ui::text` 新增 `UiTextModelUpdateRequest/Receipt`、request UUID、
  `UiTextDocumentKey` 和 `BoundRefresh | ExplicitSetText | ExplicitLoadText`；
- `UiInputManager` 的 535 行 queue owner 持有每 owner 最新 pending 元数据和 terminal receipts；
- 282 行 transaction child 基于 committed document state 复用现有 document+Surface 双 prepare/commit；
- secure pending 正文只进入 Surface secure store。manager 只持 metadata/bytes 与 clear-only opaque
  handle，Surface switch、policy change、detach、supersession 和 manager Drop 都可撤销旧值；
- IME preedit 不是 document source；显式 Set/Load 先恢复 committed base，再做全量 exact replacement，
  只推进一次 revision；
- latest unchanged bound refresh 也 supersede 旧 pending，防止旧模型值在失焦时复活。

## Required authority and DTO

产品 document/edit session 现在作为唯一 owner，并区分：

1. model value identity：绑定模型的 value 与 model revision；
2. editable snapshot identity：当前 edit base revision、document revision 与 selection/composition；
3. mutation origin：`BoundRefresh`、`ExplicitSetText`、`ExplicitLoadText`；本地
   `EditProjection` 继续由 existing exact edit intent 表达；
4. refresh policy：普通 refresh defer 或 Set/Load force review；
5. typed outcome：applied、unchanged、deferred、conflict/rebase-required、rejected。

版本化 schema 当前为 1。receipt 不携带正文，并校验状态/失败/current key/可选 document edit receipt
的一致性；malformed request rejection 可以安全回显无效身份而不成为有效 document claim。dynamic
Runtime host ABI 尚未扩展；在宿主 model contract 冻结前不擅自增加第二条 FFI 路径。

## State transition

| 输入 | 未聚焦 | 聚焦 |
|---|---|---|
| 普通 bound refresh | revision-compatible 时应用 | 不替换 edit buffer；更新每 owner 唯一 latest pending refresh |
| 显式 replace/load | 经完整 edit transaction 应用 | force review，经完整 transaction 应用并按策略修正 caret |
| 本地 edit projection | 更新 edit/document revision | 更新 edit/document revision，不消费不兼容 pending model value |
| blur/commit | 无额外动作 | pending 与 edit base 兼容则应用，否则返回 typed conflict/rebase receipt |
| detach/policy/session teardown | 清 session state | 清 pending；secure value 由 secure owner 销毁 |

普通 refresh 的 pending store 只保存每个活动 editable owner 的最新值与 revision，当前
`BTreeMap` lookup/update 为 `O(log E)`，`E <= 256`；resident state 为
`O(active editable sessions + pending bytes)`。它不是 document snapshot cache；不保存多版本队列，
也不为每次 tick 复制 UI tree 或全文 document。单值上限 4 MiB，pending 总正文 16 MiB，pending 与
terminal receipt 合计 256 行。

## Correctness and security gates

- pending refresh 带 owner、model revision、edit base/document revision，stale completion fail closed；
- binding transaction 不把 deferred 计为 applied，也不提前发布 dirty/value-changed；
- force review 仍复用 editable property/document transaction，不能绕过 grapheme/IME state repair；
- blur 冲突不能 last-writer-wins 静默覆盖；由产品 model/document owner选择 rebase、accept local 或
  accept remote，并发布可解释 receipt；
- secure pending raw value 不能进入 generic cloned/serialized Surface map，只能进入 secure store，
  且 clone、serde、detach、policy change、surface switch、manager Drop 后不可解析；内存 zeroization 和
  crash-dump policy 仍是独立开放项；
- profile 记录 refresh/apply/defer/conflict 的低基数计数和耗时，不能记录原文、property 动态标签或
  secure 值。

## Implementation order and current completion

1. [implemented_unvalidated] 建立产品 `TextDocumentId + Revision` edit session 与 model/edit 双 identity；
2. [partially_implemented_unvalidated] 独立 request 已有 typed origin/policy；真实 dynamic binding producer
   尚未接宿主 ABI，generic property mutation 不伪装成 bound refresh；
3. [implemented_unvalidated] 增加有界 pending refresh 与 typed deferred/conflict/terminal receipt；
4. [implemented_unvalidated] 接 blur CAS/force review、IME committed-base 与 secure lifecycle；stale 时发布
   conflict，不做未经产品 policy 授权的自动三方 merge；
5. [pending] 执行 managed fault/profile/power、matched Unreal、平台 IME 与 WGPU 产品验证。

## Profile matrix before any optimization

已有 16 个固定、无正文 `ui_text.model_update.*` counters：request count/bytes、bound/explicit、
focused/secure、Applied/Unchanged/Deferred/Conflict/Rejected、pending admission/release bytes 与
supersession。标签中不含 request/tree/node/property/source text。后续 managed profiler 必须使用相同
source snapshot，先冻结 raw evidence 到 `docs/tests/runtime/text`，再决定是否优化：

| Lane | 规模 | 必测行为 |
|---|---:|---|
| unfocused unchanged/replace | 1/100/1k/10k requests | no-op 与 full replacement 分开，不混平均值 |
| focused latest refresh | 1/100/1k/10k same-owner requests | defer、supersession、pending bytes、terminal drain |
| owner fan-out | 1/16/64/256 owners | `BTreeMap O(log E)`、queue cap、RSS、terminal backpressure |
| blur compatible | 1/100/1k/10k cycles | expected-key match、document+Surface apply、revision advance |
| blur conflict | 1/100/1k/10k cycles | local edit 后 conflict，零 remote mutation |
| explicit/preedit | SetText/LoadText × plain/IME | committed-base restore、一次 revision、caret end |
| secure lifecycle | defer/apply/policy/detach/switch/drop | plaintext copy count、resident bytes、清理与日志泄露 |

每 lane 预热后采 31 个非 ignored 样本，记录 p50/p95/p99、allocation count/bytes、CPU time、RSS、
固定 counters、document revision 与结果分布。Windows power/ETW 若仍被策略阻断，报告原始错误码，
不得以 wall time 推断功耗。matched Unreal 使用相同文本长度、owner 数、刷新/聚焦序列、build mode 与
硬件；不能用 Slate 编辑器交互经验值替代数据。

当前允许的结构结论只有：focused defer 不扫描 source/layout/tree，pending move 不额外复制 request
String；owner lookup 为 `O(log E)`；accepted replacement 因 Surface/property 仍持完整 String 而至少
`O(N)`。在 profiler 证明 `BTreeMap`、secure Mutex 或 Surface full-value projection 中哪一项主导前，
禁止改 HashMap、lock-free queue、rope handle、阈值或自动 merge 算法。本切片没有渲染变化，因此不生成
策略截图；最终真实 WGPU 文本图仍只能写入 `docs/tests/runtime/text`。

## Managed validation attempt (2026-08-28)

Windows validator 从批准的 D 盘共享 pool 执行 default `zircon_runtime` package build，成功编译 interface
并进入 Runtime crate，但共享脏工作树最终报告 154 个错误；可见终端错误包含与本专题无关的 SDF atlas
旧函数名调用，完整 crate 因此没有形成 compile ticket。随后一次仅用于筛选诊断的受管 lane acquire 在服务
端已接收后发生 `command_post_timeout`；按策略没有查询、轮询或直接 Cargo 重试。此结果只证明当前 default
工作树不可验收，不证明本 gateway 通过或失败；聚焦 test/profile/power/WGPU gate 全部保持 pending。
