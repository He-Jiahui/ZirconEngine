# ZirconEngine docs/plans 总索引

本文档只作为 `docs/plans` 的入口导航与规则概述，不承载具体 workflow 产出、失败记录或子计划验收明细。

## 产出记录迁移说明

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

历史 workflow 整理状态、缺口表、成功/失败记录与恢复线索已迁入根级产出目录。

- 迁入记录：[`00/2026-07-09-root-index-output-records.md`](00/2026-07-09-root-index-output-records.md)

详细来源审计仍见 [workflow-source-audit.md](workflow-source-audit.md)。

## 计划集入口

| 计划集 | 入口 | 用途 |
| --- | --- | --- |
| Runtime 渲染管线 | [zircon_runtime/render/index.md](zircon_runtime/render/index.md) | 渲染 RDG、MeshDrawCommand、GPUScene、可见性、光照、时域、后处理、permutation 与能力层计划入口。 |
| Runtime 架构收束 | [zircon_runtime/runtime/index.md](zircon_runtime/runtime/index.md) | runtime 技术栈、core spine、调度、资产、边界、性能、ECS、UI、动态 API、JobSystem、输入与模块族计划入口。 |
| Runtime / Editor UI | [zircon_editor/editor_ui/index.md](zircon_editor/editor_ui/index.md) | editor UI 运行时能力、工作台壳、交互、资源与运行时 UI 关系入口。 |
| Zircon Hub | [zircon_hub/index.md](zircon_hub/index.md) | Hub action、payload、layout、visual standard 与相关计划入口。 |
| 插件生态 | [zircon_plugins/index.md](zircon_plugins/index.md) | 插件架构、能力插件、导出发布、Editor 集成、bridge 与结构框架入口。 |

## 使用规则

1. 进入某个子系统前，先读上表对应的 `index.md`。
2. 计划正文已细化和来源审计冲突时，以计划正文为准，但执行前必须按该计划的检查清单重核 live worktree。
3. 来源审计只能作为继续细化输入，不能直接替代源码事实核验。
4. substantial work 继续按里程碑推进：实现切片与测试阶段分离，里程碑末在子计划记录构建、测试、修复和接受证据。
5. 旧路径不打算保留时，执行计划必须硬切换：迁移调用方并删除旧路径，不留 re-export、alias、shim 或双轨兼容。

## 引擎级结构规范

[`engine-code-structure-convention.md`](engine-code-structure-convention.md) 是 ZirconEngine **代码结构与模块接口约定的唯一权威**。各计划集只引用、不重定义其规则。

补充输入：[`engine-code-review-findings-2026-06.md`](engine-code-review-findings-2026-06.md) 是聚焦代码审查的发现目录；具体产出记录由对应子计划与产出目录维护。
