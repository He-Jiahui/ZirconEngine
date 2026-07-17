---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: renderdoc-cold-warm-capture
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - docs/plans/performance/01/renderdoc_capture_audit.py
tests:
  - current-source MVP cold-frame RenderDoc capture
  - same-process second stable-frame RenderDoc capture
  - GPU timestamp availability and missing-counter reporting
---

# Render17：旧 D3D12 capture 显示 copy storm，但缺当前源码冷暖帧对照

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：RenderDoc 1.44 工具链探测与旧 D3D12 capture replay
- 来源证据：`docs/plans/performance/01/2026-07-17-renderdoc-toolchain-probe.md`
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：冷/暖帧捕获规范、GPU counter 缺失语义和跨 render owner 路由属于 Render17。

## 失败现象与复现证据

RenderDoc 1.44 已成功 replay 现有 D3D12 capture：4,357 actions、58 draws、39 dispatch、3,506 copy、51 clears；3,203 copy 发生在 event 4,000 前，copy 占 actions 80.47%。GPU duration counter 可枚举但没有样本，不能写成 0 ms。

## 最低共享层根因

该 capture 是旧高级场景且 copy 高度集中于早期事件，当前只能判定为冷帧初始化候选，不能证明当前源码稳定帧存在上传风暴。

## 架构修复验收

- 用当前源码、同一进程、同一 MVP 场景捕获 cold frame 与至少第二个 stable frame。
- 报告 draw/dispatch/copy/clear/barrier、upload bytes、pipeline/资源创建和 GPU duration；缺失 counter 显式记 unavailable。
- 静态稳定帧上传应归零或每项有资源生命周期理由；若仍有 copy storm，再路由至 Render01/02/03/13 的最低 owner。

## 禁止临时方案

- 不得用旧 capture 冒充当前源码或稳定帧。
- 不得把缺失 GPU timing 写成零，也不得通过降低画质掩盖上传/同步问题。

## 修复结果与回传

Open state: `待 Cargo lane 构建当前源码 MVP 后完成 RenderDoc cold/warm capture`。
