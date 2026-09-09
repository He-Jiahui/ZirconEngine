---
related_code:
  - zircon_runtime/src/graphics
  - zircon_runtime/src/asset
  - zircon_editor/src/core/project
  - zircon_runtime_interface/src/world_sync
implementation_files:
  - zircon_runtime/src/graphics
  - zircon_runtime/src/asset
  - zircon_editor/src/core/project
  - zircon_runtime_interface/src/world_sync
plan_sources:
  - user: 2026-09-09 扩展 ZirconEngine Wiki 教程与机制说明
  - docs/wiki/concepts.md
tests:
  - zircon_runtime/src/graphics/tests
  - zircon_runtime/src/asset/tests
  - zircon_editor/src/tests
  - zircon_runtime_interface/src/tests/world_sync_contracts.rs
doc_type: category-index
---

# 引擎方案配方

配方面向已经理解模块和生命周期、现在要完成一个真实产品场景的读者。每篇配方都把多个子系统串成可审计的方案，并说明何时降级、何时重建、何时必须停止。

## 场景索引

| 结果 | 配方 | 主要组合 |
| --- | --- | --- |
| 提交并呈现一帧场景 | [RenderFrame 提交](render-scene-frame.md) | viewport、extract、RenderFramework、surface、present |
| 导入后安全消费模型/纹理 | [资产导入与消费](asset-import-and-consume.md) | ProjectManager、typed handle、readiness、lease |
| 完成一次作者态改动并保存 | [编辑器作者态会话](editor-authoring-session.md) | ProjectAuthority、Scene route、transaction、history、save |
| 让宿主增量同步运行时 World | [World Sync 宿主集成](world-sync-host-integration.md) | query、watch、invalidation、generation、ABI output |

## 组合方式

```text
目标场景
  -> 明确权威 owner
  -> 选一个稳定 identity（session / resource / viewport）
  -> 用 DTO、handle 或 operation 跨层
  -> 在 commit 后发布
  -> 用 receipt / diagnostics 验收
```

不要把配方当作复制粘贴的单一函数。它们故意把宿主装配、错误处理和退出顺序写出来；真正产品应把这些阶段接入自己的 coordinator、日志和测试夹具。

## 方案评审清单

- [ ] 每个跨层箭头都有明确 owner 和数据类型。
- [ ] 每个异步结果都携带或重新验证 generation/session identity。
- [ ] 失败路径说明了保留 last-good、重试、完整刷新或终止中的哪一种。
- [ ] 预算（字节、items、时间、in-flight）在入口处被限制，而不是出错后才截断。
- [ ] 配方中的“示意 Rust”已经替换成当前版本的真实 facade 签名。

配方依赖的基础知识：[基础概念](../concepts.md)、[产品生命周期](../product-lifecycle.md)、[Rust API 约定](../rust-api.md)。
