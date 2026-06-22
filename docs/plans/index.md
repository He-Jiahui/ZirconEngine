# ZirconEngine docs/plans 总索引与 Workflow 整理状态

本文档把 `C:\Users\HeJiahui\.claude\projects\E--Git-ZirconEngine` 中 workflow 留下的有效信息收束到 `docs/plans` 的入口层。

有效信息的采纳口径：

- 已经写入 `docs/plans/**` 的计划文档，作为当前可读计划入口。
- workflow 脚本中的计划目标、阶段、文件清单和 focus，作为恢复或继续细化时的输入。
- workflow 运行结果中明确成功的子计划摘要，作为已落盘状态说明。
- API 502、断连、stall、工具噪声和未经核验的完整草稿，不直接当作事实写入子计划正文；只在来源审计中登记为可恢复资料。

详细来源与缺口见 [workflow-source-audit.md](workflow-source-audit.md)。

## 计划集入口

| 计划集 | 入口 | 来源 workflow / memory | 当前整理状态 | 下一步口径 |
| --- | --- | --- | --- | --- |
| Runtime 渲染管线 | [zircon_runtime/render/index.md](zircon_runtime/render/index.md) | `memory/render-alignment-plan-set.md`，origin session `fcc4034f-230c-490a-af30-7036916f8ad9` | 已形成 `index.md + 01-16` 子计划，是渲染 RDG / MeshDrawCommand / GPUScene / visibility / lighting / temporal / postprocess / permutation / 能力层计划的权威入口。 | 渲染任务先读该 index；与旧 `.codex/plans` 冲突时以该目录为准。 |
| Runtime 架构收束 | [zircon_runtime/runtime/index.md](zircon_runtime/runtime/index.md) | `wf_a76fbb0c-aac` (`refine-runtime-plans`) | `01-tech-stack-and-dependency-governance.md` 已出现 `last_refined: 2026-06-12` 和执行前检查清单；`02-07` 仍主要是基线版。workflow 的 gather 阶段 01-07 全部完成，部分 draft 完成，但 verify/落盘多处失败。 | 先核验 01；再按来源审计恢复或重跑 03/04/05/07 草稿，重新生成 02/06，最后统一更新 runtime index。 |
| Runtime / Editor UI | [zircon_editor/editor_ui/index.md](zircon_editor/editor_ui/index.md) | `editor-ui-plan-engineering-deepening-wf_cbcbc1a2-cd3.js` | 现有 `index.md + 01-09` 是可读基线；源目录没有保存顶层 workflow 结果 JSON，仅有脚本和子代理日志，不能证明细化/核验完成。 | 以脚本中的 9 个领域和全局约束为继续细化输入；需要重跑 Explore -> Refine -> Verify -> Repair -> Index 或人工逐文档补齐工程化章节。 |
| Zircon Hub | [zircon_hub/index.md](zircon_hub/index.md) | `wf_234cfcdf-454` (`deepen-hub-plans`) | `01-action-dispatch-and-typed-payload.md` 与 `06-layout-and-visual-standard.md` 已被工程级细化；`02-05/07` 仍是短基线版。index-update 阶段失败。 | 保留 01/06；补跑 02-05/07 的 deepen + verify；最后同步 Hub index 的跨计划所有权地图和切片执行清单。 |
| 插件生态 | [zircon_plugins/index.md](zircon_plugins/index.md) | `wf_39de3956-0fa` (`deepen-plugin-plans`) | `index.md + 01-10` 基线计划存在；workflow 在 01 core 深化阶段因 502 失败，未进入下游插件并行深化。 | 必须先定稿 01 插件核心 API，再让 02-10 照用同一调度锚点、注册 API、plugin.toml schema 与 ABI v3 名称。 |

## 使用规则

1. 进入某个子系统前，先读上表对应的 `index.md`。
2. 计划正文已细化和来源审计冲突时，以计划正文为准，但执行前必须按该计划的检查清单重核 live worktree。
3. 来源审计记录了失败 workflow 的可恢复信息；这些信息只能作为继续细化的输入，不能直接替代源码事实核验。
4. substantial work 继续按里程碑推进：实现切片与测试阶段分离，里程碑末记录构建、测试、修复和接受证据。
5. 旧路径不打算保留时，执行计划必须硬切换：迁移调用方并删除旧路径，不留 re-export、alias、shim 或双轨兼容。

## 引擎级结构规范

[`engine-code-structure-convention.md`](engine-code-structure-convention.md) 是 ZirconEngine **代码结构与模块接口约定的唯一权威**（模块布局 / 命名 / 公共 API / 测试组织 / 资源放置 / 插件 DX 框架）。各计划集只引用、不重定义其规则。落地与覆盖：

| 计划集 | 结构优化落地 | 强制门禁 |
| --- | --- | --- |
| Runtime | [runtime/15-code-structure-and-module-conventions.md](zircon_runtime/runtime/15-code-structure-and-module-conventions.md) | `module_convention_gate` + `runtime_structure_audits/module_convention_boundary.py` |
| Editor UI | [editor_ui/10-code-structure-and-module-conventions.md](zircon_editor/editor_ui/10-code-structure-and-module-conventions.md) | editor `module_convention_gate` + `editor_structure_audits/` |
| 插件生态 | [zircon_plugins/12-plugin-dx-and-structure-framework.md](zircon_plugins/12-plugin-dx-and-structure-framework.md) | `plugin_skeleton_gate` + `tools/plugin_structure_audits/` |
| Runtime 渲染 | render index「代码结构规范」节 | graphics 热点纳入 Runtime 15 + `large_file_ownership_gate` |
| Zircon Hub | hub index「代码结构规范」节 | 巨型文件 + 前端组件化纳入规范 §1/§3/§4 |

补充输入：[`engine-code-review-findings-2026-06.md`](engine-code-review-findings-2026-06.md) 是 2026-06 一轮聚焦代码审查的发现目录（F1–F19 接口/质量 + D 系列插件 DX，带 file:line 证据），规范级结论已并入 convention §7.5，结构级发现已并入三套结构子计划，安全/性能类 P0（F1 native 回调缺 catch_unwind、F2 scene 每帧 lock().unwrap、F3 每帧整帧 clone）登记建议补入 Runtime 06/07。

## 当前高优先缺口

| 优先级 | 缺口 | 原因 | 建议动作 |
| --- | --- | --- | --- |
| P0 | Hub index 未同步 01/06 的工程化细化 | `index-update` 代理 502 失败；现有 index 还不能反映跨计划共享类型所有权。 | 读 `zircon_hub/01`、`06` 和基线 `02-05/07`，补充 Hub index 的所有权地图与切片执行清单。 |
| P0 | Runtime 02/06 未生成可落地稿 | `draft:02` 502，`draft:06` 断连。 | 基于 `wf_a76fbb0c-aac` 的 focus 与 gather 证据重新生成并核验。 |
| P1 | Runtime 03/04/05/07 有 draft 但未 verify/落盘 | draft 代理成功，verify 代理 502；未经核验不能直接覆盖计划。 | 从对应 subagent StructuredOutput 恢复草稿，逐路径、标识符、测试名核验后再写回。 |
| P1 | Hub 02-05/07 工程化细化未完成 | deepen 代理 502 或断连。 | 按 `deepen-hub-plans` 脚本中的 focus 逐份补齐目标代码形状、文件变更清单、实施步骤和契约联动。 |
| P1 | 插件计划 01 core 未定稿 | 下游 02-10 依赖 01 的 API/锚点名；workflow 停在 Core。 | 单独完成 `zircon_plugins/01-plugin-architecture-core.md`，再扩展下游计划。 |
| P2 | Editor UI 计划只有基线版 | 缺少顶层 workflow 结果 JSON，无法确认工程化重写已完成。 | 以 editor workflow 脚本为规格，补齐 01-09 的接口草案、模块落点、切片、测试矩阵、依赖表和完成定义。 |
| P1 | 引擎结构债散乱（模块布局 / 命名 / façade / 测试 / 插件 DX）损害 code review | 框架成熟但结构散乱；已立 `engine-code-structure-convention.md` 与 Runtime 15 / Editor UI 10 / Plugins 12 三套结构子计划。 | 按各结构子计划里程碑推进；以 `module_convention_gate` / `plugin_skeleton_gate` 机器化验收，存量按硬切 / touch-it-conform-it 降债至 0。 |

