# ZirconEngine 全仓源码注释审查计划

**目标：** 逐项追踪第一方源码的调用关系和设计意图，在确有信息价值的位置加入中文注释；用当前内容哈希证明覆盖范围，并对确认问题和疑问留下可搜索的 `BUG:` / `TODO:`。本计划不修改程序行为或公共 API。

**执行边界：** 使用共享 `main` 和协调器的精确路径归属。既有未提交改动、参考仓库、上游代码、生成物及构建产物不得混入本任务提交。计划定义单独走授权的维护候选；普通模块里程碑只提交已归属的源码、审查账本和子记录。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-source-comment-audit",
  "goal": "Review every first-party source file and document verified call-chain intent without changing behavior",
  "milestones": [
    {"id": "M1", "title": "Comment standard and coverage ledger", "depends_on": []},
    {"id": "M2", "title": "Interface, reflection, and SDK contracts", "depends_on": ["M1"]},
    {"id": "M3", "title": "Runtime core and framework", "depends_on": ["M2"]},
    {"id": "M4", "title": "Runtime asset, resource, and scene", "depends_on": ["M3"]},
    {"id": "M5", "title": "Runtime graphics and rendering", "depends_on": ["M3"]},
    {"id": "M6", "title": "Runtime text and UI", "depends_on": ["M3"]},
    {"id": "M7", "title": "Remaining runtime systems", "depends_on": ["M4", "M5", "M6"]},
    {"id": "M8", "title": "Editor core and scene", "depends_on": ["M2", "M7"]},
    {"id": "M9", "title": "Editor UI and retained host", "depends_on": ["M8"]},
    {"id": "M10", "title": "First-party plugins", "depends_on": ["M2", "M7"]},
    {"id": "M11", "title": "App, hosts, Hub, tooling, examples, and authored resources", "depends_on": ["M9", "M10"]},
    {"id": "M12", "title": "Whole-workspace coverage and issue reconciliation", "depends_on": ["M11"]}
  ]
}
```

## 共同执行规则

1. 从 Git 当前工作树枚举已跟踪及未忽略的未跟踪文件，按仓库注释审查规范分类。测试随所属生产模块审查；资源同时追踪加载器与消费者。以实际路径和最终 SHA-256 建立覆盖账本，内容变化即重新审查受影响部分。
2. 每个子模块交给一个有明确写入路径的 agent。大型目录按直接子模块拆分；同一文件只能有一个写入者，其他 agent 可跨仓只读寻找调用方。审查定义、直接调用、trait/宏/动态入口、测试、错误路径和调用目的后，才决定是否注释或登记问题。
3. 每个完成的子模块均需主审独立复核、局部格式或解析检查、`git diff --check`、标签与问题账本对应检查，以及协调器实际要求的精确快照验证。纯注释切片不例行运行 Cargo；当服务门槛或具体 ABI/行为风险要求时，使用受管验证通道。
4. 一个里程碑可能包含多个独立子模块。子模块完成且通过门槛后可作为精确范围的 integration candidate 阶段提交；最后一个子模块连同单条里程碑证据完成该里程碑。未解决的外来文件归属或待验证票据只阻挡相关范围，不阻挡其他子模块。
5. `BUG:` 只用于复现或可证明的错误路径；意图疑问、潜在风险和缺失证据用 `TODO:`。问题账本关联已有负责人，本任务不混入行为修复。不得用注释文本使源码字符串守卫假绿。

## Milestone M1: 规范与覆盖账本

**实施：** 建立统一注释规范、确定性的第一方源码清单、按文件哈希记录的覆盖账本和问题账本；先保留所有状态为待审，不以工具枚举代替人工调用链审查。

**测试阶段：** 对工具运行聚焦单测、规范文档结构检查、确定性重跑与差异检查；由独立 reviewer 审核范围排除规则及账本是否会错误声称完成。

## Milestone M2: 公共接口、反射与 SDK

**实施：** 审查 `zircon_runtime_interface`、反射宏及插件 SDK 的接口、ABI、所有权和调用方；测试与加载端一并取证。

**测试阶段：** 局部格式和结构检查；对实际改动的 ABI 文档注释运行相关契约门槛并核对所有跨动态库调用约束。

## Milestone M3: Runtime 核心与 framework

**实施：** 审查 `core/{runtime,framework,manager,math,resource}` 及基础注册、生命周期、服务解析路径，测试随模块推进。

**测试阶段：** 核对状态与所有权注释对调用端和守卫测试均准确；按受影响范围执行必要的核心契约门槛。

## Milestone M4: Runtime 资产、资源与场景

**实施：** 依子模块审查资源标识、资产流水线、场景数据与持久化调用链。

**测试阶段：** 检查资源所有权、失效与持久化注释，运行所需结构和受管契约门槛。

## Milestone M5: Runtime 图形与渲染

**实施：** 依渲染子模块审查资源创建、帧阶段、GPU 约束和着色器调用关系。

**测试阶段：** 检查 Rust/WGSL 解析、格式和与宿主契约相关的最小验证范围。

## Milestone M6: Runtime 文本与 UI

**实施：** 审查文本布局、UI 模板、事件、渲染和 ZUI 资源的生产及消费链。

**测试阶段：** 检查 Rust/ZUI 解析和相关模板、布局契约门槛。

## Milestone M7: 其余 Runtime 系统

**实施：** 覆盖前述里程碑未归入的 Runtime 代码、内部 crate、平台和插件运行路径。

**测试阶段：** 运行每个实际 owner 的局部检查，并确认 Runtime 覆盖账本无遗漏。

## Milestone M8: Editor 核心与场景

**实施：** 审查编辑事务、场景操作、项目生命周期及其 Runtime 调用边界。

**测试阶段：** 核对状态转换、回滚和跨模块注释，按受影响契约执行门槛。

## Milestone M9: Editor UI 与 retained host

**实施：** 按子模块审查 Editor UI、宿主状态、资源与输入链，避免把大目录合成单次审查。

**测试阶段：** 检查 UI 语言解析、局部结构门槛和调用端证据。

## Milestone M10: 第一方插件

**实施：** 每个插件家族独立审查；大型插件继续按 runtime/editor/feature 子模块拆分，测试随 owner 审查。

**测试阶段：** 核对插件入口、能力协商、卸载约束和对应的最小验证门槛。

## Milestone M11: 入口、Hub、工具、示例与资源

**实施：** 覆盖 App、Runtime host、Hub、仓库工具与技能脚本、示例、模板和此前未归入的第一方可执行文本资源。

**测试阶段：** 按语言运行局部解析、格式、结构检查，并比对最终第一方清单的未审查项。

## Milestone M12: 全仓覆盖与问题核对

**实施：** 重新枚举当前工作树；对新增、变更、未知后缀和延期归属文件补审，反查已变调用端影响的被调定义并复核当前调用证据，逐一核对 `TODO:` / `BUG:` 与问题账本。

**测试阶段：** 覆盖工具的完成模式必须证明全部纳入范围的文件哈希仍与审查记录一致，排除项有理由且待处理数为零；人工复核跨文件调用关系的当前证据及各里程碑提交、实际验证回执后关闭 Goal。
