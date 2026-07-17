---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: gateway-stable-call-lock-and-clone
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/gateway/capabilities.rs
tests:
  - stable gateway tick/event/capture call lock-count regression
  - gateway replacement concurrency and lifetime matrix
  - editor idle and interaction WPR trace
---

# Editor01：gateway 稳态调用锁与快照复制

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/core/gateway` 逐文件性能审查
- 修复责任计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 交接原因：gateway generation 与稳定调用所有权属于 Editor01 内核边界，不能由上层调用点各自缓存。

## 失败现象与复现证据

`EditorRuntimeGatewayHandle` 用 `Arc<RwLock<SharedEditorRuntimeGateway>>` 保存可替换 gateway，但每次 capabilities、session、tick、event drain、capture、profile、subscription 和 operation 调用都先获取读锁，再 clone trait-object `Arc`。`capabilities()` 随后还深 clone 包含多个 `String`/`Vec<String>` 的 `RuntimeCapabilities`。gateway 替换属于低频控制面，锁和 owned capability snapshot 却落在稳定数据面，放大编辑器 tick、事件泵和 viewport capture 的主线程成本。

## 最低共享层根因

可替换 gateway 的控制面 owner 与稳定数据面共用一个 `RwLock`，且 capability projection 没有绑定 gateway generation 的共享快照。

## 架构修复验收

- 把 gateway generation/replacement 与稳定调用快照分离；稳定调用读取 immutable `Arc` snapshot，不经过共享 `RwLock`，替换时原子发布新 generation。
- capability projection 随 gateway generation 构建一次并共享借用或 `Arc`；不得逐查询复制字符串集合。
- 旧 snapshot 在并发调用结束前保持存活；replacement、shutdown、session invalidation 与 poison/recovery 语义有并发测试。
- WPR/计数测试证明 idle/tick/event/capture 稳态 gateway read-lock 次数为零，并记录替换路径成本。

## 禁止临时方案

- 不得只把 `RwLock` 换成 `Mutex` 或给每个方法加独立缓存。
- 不得暴露借用跨越可替换 owner 的悬垂引用。
- 不得在未测调用频率时把该项宣称为帧占比结论。

## 修复结果与回传

Open state: `待 Editor01 实现 generation-bound immutable gateway/capability snapshot，并回传并发语义、lock-count 与产品 trace`。
