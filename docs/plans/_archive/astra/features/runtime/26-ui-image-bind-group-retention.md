---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11c/2026-09-09-image-bind-group-idle-retention.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
---

# UI 图像绑定缓存保留

## 当前源码确认

`zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs` 的 image bind-group 产品按
GPU texture `Arc` 身份复用；空帧只推进有界 prepare epoch 并保留近期或仍被 segment 引用的
产品，名义 512 项上限压力下只淘汰未固定的非当前项，绝不为满足上限丢弃活跃产品。资源解析映射仍按 management/readiness/
binding-product generation 失效，不改变异步上传 authority。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M1 | 两个 prepare epoch 的 idle 保留、活跃 Arc 固定与 512 项上限 | implemented_pending_validation | Runtime GPU-image Python 契约 `10/10`；Runtime/Editor 合并性能合同批次 `1723/1723`（Runtime `1143/1143`、Editor `580/580`）通过；后续有界 Runtime/Editor 源码合同批次 `70/70` 通过；现有确定性模型的 registry visit 降幅为 `1,024x`；Rust owner `rustfmt --check` 与 scoped diff check 通过；WGPU/release p95 仍待受管验证 |
