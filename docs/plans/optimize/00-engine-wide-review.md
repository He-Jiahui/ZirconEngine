---
related_code:
  - Cargo.toml
  - zircon_app
  - zircon_runtime
  - zircon_runtime_interface
  - zircon_editor
  - zircon_plugins
  - zircon_hub
  - tools/cargo-zircon
  - zircon_reflect_derive
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/zircon_runtime/frameworks/index.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/engine-architecture/workspace-root-rules-and-hard-cutover.md
reference_engines:
  - dev/UnrealEngine
  - dev/Fyrox
  - dev/bevy
  - dev/godot
  - dev/Graphics
---

# 00 · ZirconEngine 工程级能力差距全量审查

## 1. 目标

本计划以“可长期演进的工程级游戏引擎”为验收标准，对 ZirconEngine 的生产代码、测试、构建配置、工具链和既有计划进行逐域审查，并将可复现的差距写入 `docs/plans/optimize/` 下对应 crate 或工具域的编号子计划。

本阶段只做 review、证据固化和重构路线设计，不修改生产实现。后续实现必须从子计划的最低共享层开始，按依赖顺序推进，不得以 demo 可运行、单测存在或 API 名称齐全代替工程化验收。

目标不是机械复制参考引擎，也不是凭功能表判断优劣。每项结论必须回答：

1. Zircon 当前行为、所有权边界和失败语义是什么；
2. 该行为在并发、异常、热重载、停机、长时间运行或大规模内容下是否仍成立；
3. Unreal、Fyrox、Bevy、Godot 或 Unity Graphics 中哪一类成熟机制提供了可核对的设计证据；
4. Zircon 应吸收什么契约，哪些参考实现的历史包袱不应照搬；
5. 修复应落在哪个 owner、先补什么测试、怎样完成硬切和反向验证。

## 2. 审查原则

### 2.1 证据等级

| 等级 | 必要证据 | 允许的结论 |
|---|---|---|
| E0 | 文件名、模块名或搜索命中 | 仅登记待审查线索 |
| E1 | 生产调用链与数据所有权已读 | 可描述当前机制，不得宣布缺陷已证实 |
| E2 | 生产链、失败路径和相关测试已交叉核对 | 可登记 Zircon 差距与风险等级 |
| E3 | 至少一个适配的参考引擎源码机制已核对 | 可提出目标契约和重构方向 |
| E4 | 已有复现测试或可执行验证设计 | 可进入后续实现里程碑 |

`coverage.md` 只登记物理覆盖和证据等级；编号子计划拥有详细发现。未达到 E2 的域不得写成“已完整审查”，未达到 E3 的发现不得用“对齐 Unreal”等措辞。

### 2.2 比较纪律

- Unreal 用于模块装载、对象寿命、编辑器/运行时边界、渲染架构和大型工程运维参考；不得把其兼容性债务直接复制为 Zircon 目标。
- Fyrox 用于 Rust 引擎插件、场景、资源和动态库重载参考。
- Bevy 用于 Rust ECS、调度、App/Plugin 生命周期和数据驱动系统参考；其不支持的运行时卸载能力不能被当作 Zircon 卸载正确性的证据。
- Godot 用于明确的初始化层级、反向清理、扩展重载和工具/运行时分层参考。
- `dev/Graphics` 用于 Unity Graphics 的渲染、着色器、资源和平台图形实现参考；Unity 闭源主引擎中无法从本仓源码证明的机制不得臆测。
- 优先比较契约、状态机、资源所有权、线程模型、失败恢复、可观测性和验证方法，不做仅按类名/API 名称的一一映射。

### 2.3 风险分级

| 等级 | 定义 |
|---|---|
| P0 | 可破坏生命周期原子性、内存/线程/动态库安全、持久化数据或产品停机正确性；实现前必须先处理 |
| P1 | 规模、并发、扩展性或跨模块契约在真实工程中不可持续，容易形成系统性返工 |
| P2 | 性能、诊断、测试可信度或维护成本明显弱于工程目标，但不立即破坏基础正确性 |
| P3 | 局部一致性、开发体验或文档债务；应在所属重构里程碑中收敛 |

性能结论必须有 profile/benchmark 入口或明确的测量计划。没有数据时只能登记“需要测量”，不得宣称 Zircon 已优于或必然劣于某参考引擎。

## 3. 分类与 owner

| 分类 | 主要审查范围 | 输出目录 |
|---|---|---|
| Host | 启动、产品循环、停机、动态库会话、配置与产品模式 | `zircon_app/` |
| Runtime | kernel、ECS/scene、asset、render/RHI、UI、input、script、physics、audio、network、diagnostics | `zircon_runtime/` |
| Interface | ABI/FFI、DTO、句柄、版本协商、错误与内存所有权 | `zircon_runtime_interface/` |
| Editor | authoring state、transaction/undo、selection、viewport、asset workflow、tool extensibility | `zircon_editor/` |
| Plugins | 插件边界、发现/装载/卸载、SDK、版本与隔离 | `zircon_plugins/` |
| Hub | 项目/引擎管理、安装、进程和更新工作流 | `zircon_hub/` |
| Tooling | Cargo workspace、代码生成、验证器、打包、CI、性能与诊断工具 | `zircon_tooling/` |

分类以 canonical owner 为准。跨域问题由最低共享 owner 的子计划承接，其他目录只保留链接和影响说明，禁止复制同一条发现。

## 4. 覆盖方法

每个审查单元按相同顺序执行：

1. 统计 production/test/build/doc 物理范围并记录工作区已有改动；
2. 读取模块入口、公开契约、核心状态、失败路径、Drop/cleanup 和生产调用点；
3. 读取行为测试、结构守卫、集成测试、基准和缺失的反例；
4. 从 `dev/` 中选择最匹配的参考子系统，读取其实现与测试；
5. 写 E2/E3 发现、影响、目标契约、硬切边界和可执行测试矩阵；
6. 将尚未证明的怀疑留在覆盖账本，不升级为结论；
7. 在工作区变动重叠时标记 `recheck_required`，不得覆盖或回滚其他会话修改。

审查以“纵向闭环”为最小单位。例如生命周期审查必须从 descriptor/registry 一直读到产品启动与停机调用，不能只读 trait；渲染审查必须贯穿 authoring/runtime extraction、render graph、RHI submission、资源回收和设备丢失。

## 5. 依赖分层与审查顺序

| 里程碑 | 层级 | 审查内容 | 退出条件 |
|---|---|---|---|
| R0 | 基线 | workspace、crate、feature、生产/测试规模、现有计划与活跃会话 | 建立可重复的覆盖账本，不对未读域下结论 |
| R1 | Kernel | runtime 生命周期、服务/模块图、任务、事件、配置、诊断、ABI 基础 | 所有核心状态转换与产品调用链达到 E3 |
| R2 | Data | resource/asset、scene/ECS、serialization、reflection、project state | 加载、变更、回滚、卸载和版本迁移达到 E3 |
| R3 | Platform | platform、input、window、filesystem、process、threading | 多窗口、设备变化、后台/恢复和停机达到 E3 |
| R4 | Runtime systems | physics、audio、animation、navigation、script、network | 每域覆盖 scheduling、ownership、hot reload、determinism/latency 约束 |
| R5 | Graphics | RHI、render graph、renderer、shader/material、streaming、UI/text | CPU/GPU lifetime、barrier、device loss、frame pacing 和 profile 达到 E3 |
| R6 | Host/Plugin/ABI | app、runtime interface、dynamic session、plugins | 跨库所有权、版本协商、装载/卸载和失败隔离达到 E3 |
| R7 | Editor/Hub | transaction、authoring/runtime bridge、viewport、content workflow、项目管理 | 编辑器状态可恢复、可撤销、可扩展，产品流程有端到端失败设计 |
| R8 | Tooling/acceptance | build、codegen、validation、CI、packaging、performance | 每条 P0/P1 有 owner、依赖序和可执行验收门槛 |

R1-R8 可以在无依赖冲突的范围内交错审查，但重构实施顺序必须从最低共享层向上。MVP 当前优先级仍以 `docs/plans/mvp/index.md` 为准；本计划发现的高级能力可先设计，不得借 review 绕过 MVP gate 开始实现。

## 6. 子计划要求

每篇编号差距文档至少包含：

- 审查边界、已读文件与明确未覆盖范围；
- 当前实现和生产调用链；
- 按 P0-P3 排序、带源码位置的发现；
- 参考引擎证据和适用边界；
- 目标状态机/所有权/线程/错误契约；
- 不保留兼容 shim 的硬切范围；
- 测试先行的重构里程碑；
- 单元、集成、并发/压力、故障注入、产品闭环和性能验证矩阵；
- 与既有计划的冲突、需要重开或纠正的状态；
- `## 状态与产出记录`，只在后续里程碑真正验收后追加一行。

## 7. 完成定义

整个 review 只有在以下条件同时成立时才能宣称完成：

1. `coverage.md` 中所有 production 域达到 E2，所有 P0/P1 所在域达到 E3；
2. 每条差距只有一个 canonical 编号子计划 owner；
3. 每个 crate 的启动、稳态、异常、恢复与停机路径均有审查结论；
4. 所有参考比较均指向仓内可读取源码，不使用功能宣传替代实现证据；
5. 每条 P0/P1 都有最低层 owner、依赖顺序、失败测试和产品级验收设计；
6. 文档链接、计划输出规则和 `git diff --check` 通过；
7. 工作区已有改动影响的结论完成二次复核。

当前阶段不满足这些条件，因此本计划状态保持 `in_progress`。任何单篇报告完成只代表对应审查单元完成，不代表全引擎 review 完成。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
