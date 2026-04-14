---
related_code:
  - zircon_entry/src/lib.rs
  - zircon_core/src/lib.rs
  - zircon_module/src/lib.rs
  - zircon_manager/src/lib.rs
  - zircon_scene/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_script/src/lib.rs
  - zircon_graphics/src/lib.rs
implementation_files:
  - zircon_entry/src/lib.rs
  - zircon_core/src/lib.rs
  - zircon_module/src/lib.rs
  - zircon_manager/src/lib.rs
  - zircon_scene/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_script/src/lib.rs
  - zircon_graphics/src/lib.rs
plan_sources:
  - user: 2026-04-13 将架构优先规则保留到 docs 下面用于生产项目 wiki
  - .codex/plans/全系统重构方案.md
tests:
  - cargo check --workspace
  - zircon_core/src/lib.rs
  - zircon_scene/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_script/src/lib.rs
doc_type: module-detail
---

# Architecture-First Development

## Purpose

这份文档定义 `zirconEngine` 的系统级开发约束。目标不是描述某个单点实现，而是约束未来所有跨 crate 功能、子系统重构和核心能力扩展必须如何进入引擎主架构。

核心原则只有两条：

- 所有功能先抽象出可复用的框架结构，再写具体行为。
- 如果某个方案与主流游戏引擎的典型分层相比过于浅薄、过于特化，优先深化架构设计，而不是先求最短实现路径。

## Related Files

- `zircon_entry/src/lib.rs`
- `zircon_core/src/lib.rs`
- `zircon_module/src/lib.rs`
- `zircon_manager/src/lib.rs`
- `zircon_scene/src/lib.rs`
- `zircon_editor/src/lib.rs`
- `zircon_script/src/lib.rs`
- `zircon_graphics/src/lib.rs`

## Architecture Baseline

当前默认权威路线图是 [全系统重构方案](../../.codex/plans/全系统重构方案.md)。除非用户明确替换目标，否则所有设计和实现都应以这份方案定义的最终引擎形态为准。

### Runtime Spine

运行时主干固定为：

`zircon_entry -> zircon_core -> zircon_module/zircon_manager -> subsystem modules`

这条主干的含义是：

- `zircon_entry` 只负责 entry profile 选择、内建模块注册、shell 启动和主循环接通
- `zircon_core` 负责进程级唯一 `CoreRuntime`、生命周期、依赖排序、事件、配置和调度
- `zircon_module` 负责模块、驱动器、管理器、插件 descriptor 和上下文
- `zircon_manager` 负责稳定 façade、trait、resolver 和 handle surface
- 具体领域能力由 `zircon_*` 子系统模块实现，并通过 core/module/manager 模型接入

### ECS Authority

- `zircon_scene::World` 只是 ECS 数据与调度容器
- `zircon_scene::LevelSystem` 才是运行中的系统实例层，子系统严格挂在它下面
- Scene tree、editor hierarchy、render extract 都是派生视图，不是权威运行时模型
- `SystemStage` 与调度阶段是运行时执行单元，不再允许把场景节点对象本身当作行为宿主

### Editor and Runtime Boundary

- editor 和 runtime 都必须通过 core 查询 façade 或 manager，而不是直接拼接底层实现对象
- `zircon_editor` 负责 editor state、命令系统、选择集、宿主 UI 和 workbench；它不是运行时世界的拥有者
- editor 对世界的修改应通过 `LevelManager`、`LevelSystem` 或命令层落入 ECS world

### Plugin Boundary

- 插件边界按 VM 插件协议设计，而不是按 Rust 动态库对象共享设计
- 宿主侧必须强调稳定 handle、capability 边界和状态迁移语义
- 热替换保证的是协议连续性，不是 Rust 对象地址连续性

## Non-Negotiable Rules

### 1. Define the Framework Slot Before the Feature

在实现任何新功能前，先明确它落在哪个框架槽位：

- core contract
- module descriptor
- driver or manager
- manager façade or stable handle
- ECS component, resource, or system stage
- config object or manifest
- editor command surface
- VM plugin protocol

如果一个功能无法明确归属这些边界之一，说明架构还没准备好，不能直接实现。

### 2. Reject Direct Cross-Layer Construction

不允许从上层直接 new 或持有下层具体实现对象来走主链路。上层应依赖：

- descriptors
- typed façade traits
- stable handles
- manager interfaces
- ECS queries or command surfaces

如果实现需要 editor、runtime 或 plugin 直接知道某个底层具体类型，优先补边界，而不是继续穿透。

### 3. Treat Simplicity as a Design Check, Not an Automatic Win

“这个功能很简单，所以先直接写了” 不是有效理由。每次觉得实现很简单时，先问两个问题：

- 这是因为系统已经有足够强的抽象，所以 leaf implementation 很薄吗？
- 还是因为我们绕过了 descriptor、resolver、façade、`LevelSystem`、plugin contract、lifecycle 这些必要层？

只有第一种情况可以继续实现。第二种情况必须先补架构。

### 4. Align With Mainstream Engine Patterns

在新增或重构子系统时，先检查它是否与成熟引擎的常见模式相匹配。重点不是机械复制，而是避免做出明显低阶、短命、不可扩展的结构。

至少要对齐这些层次意识：

- 应用入口与核心 runtime 分离
- 模块注册和生命周期由统一核心管理
- 上层通过 façade 或服务访问领域能力
- `LevelManager -> LevelSystem -> World` 作为场景运行时权威分层
- editor 作为宿主和工具层，不直接吞并 runtime 内核
- plugin/script 通过稳定协议接入，而不是直接绑死具体实现

如果当前方案比主流模式更扁平、更特化或更依赖临时分支，默认视为设计不足。

### 5. Avoid Feature-Specific Branches in Shared Foundations

在 shared foundations 中，以下信号都视为架构失败：

- 根据某个功能名、类型名、节点名、资源名分支
- 只为一个上层场景添加 ad hoc service lookup
- 为某个功能临时缓存跨生命周期的强引用
- 在 core/server/module 层写“这次先特殊处理”的例外分支

出现这些情况时，先回答：缺失的底层能力到底是什么？

- capability contract
- lifecycle rule
- façade boundary
- ECS data model
- command surface
- serialization rule
- plugin protocol

然后补这个能力，而不是保留一次性分支。

### 6. Keep the Workspace Architecturally Coherent

如果一个跨 workspace 重构把某个 crate 升级成真实模块，相关 sibling `zircon_*` crates 也必须同步拥有最小架构骨架：

- module descriptor
- driver or manager descriptor
- config object
- lifecycle integration
- no-op or stub implementation

不能长期保留“主链路之外的 crate 继续只是占位名义模块”的状态。

## Design Workflow

### Step 1. Anchor the Feature in the Current Target Architecture

开始前先回答：

- 这个能力属于 core、module metadata、manager façade、`LevelManager`、`LevelSystem`、`World`、graphics extract、editor host 还是 VM plugin？
- 它依赖哪个生命周期层？
- 它需要哪些 sibling 模块同步接入？

如果这些问题答不清，就还没到写代码的时候。

### Step 2. Extract the Missing Abstractions

把功能实现前需要存在的抽象列出来，例如：

- descriptor types
- service traits
- handles
- config types
- ECS components
- system registration hooks
- manifest structures
- command surfaces
- host capabilities

如果列完发现其实没有新抽象，只是在一个已存在、边界清晰、外部不可见的 leaf 内部补逻辑，那才属于直接实现的例外。

### Step 3. Run the Architecture Depth Test

实现前必须能回答是：

- 未来另一个功能能否复用这个边界，而不再改 shared code？
- 上层依赖的是 contract，而不是 concrete implementation 吗？
- 生命周期、热重载、调度、提取阶段和序列化边界仍然清晰吗？
- 这次改动是否让 `zirconEngine` 更像完整引擎，而不是更像一次性 demo？

只要有一个答案是否，就应该继续设计。

### Step 4. Implement From Framework Inward

推荐顺序固定为：

1. 描述边界与契约
2. 接入 descriptor、注册、生命周期和 config
3. 补 no-op 或 stub 路径
4. 接入 façade、manager、ECS 或 plugin protocol
5. 最后再写 feature-specific behavior

这样可以保证具体功能是正常消费者，而不是反向定义架构。

## Red Flags

以下现象出现时，应立刻停下重审设计：

- “先直接从 editor 调这个具体类型，后面再抽象”
- “这个功能只有一个地方用，不需要 façade”
- “先把 world 放在 editor state 里，后面再拆”
- “这个 plugin 先拿 Rust 对象引用，热替换以后再说”
- “其他 crate 先不接，当前链路跑通就行”
- “shared code 里加一个 if 分支专门兼容这次需求”

这些都不是小问题，而是典型的架构债务入口。

## Acceptable Direct Implementations

只有同时满足以下条件，才允许先写直接实现：

- 改动完全封闭在一个既有抽象内部
- 不新增跨 crate public entry point
- 不改变 lifecycle、façade、ECS authority 或 plugin contract
- 未来同类能力不需要新的公共框架槽位复用

如果任何一条不满足，就先做架构设计。

## Validation Expectations

架构类工作不能只用“功能跑了”作为验收。至少要检查被触达的层次：

- lifecycle 注册、激活、停机和依赖顺序
- façade 或 handle 是否成为唯一上层入口
- ECS 数据权威是否仍然只在 world 一侧
- render extract / update 阶段是否仍然分离
- plugin state save/restore 和 capability 边界是否清晰
- sibling crates 是否已至少具备最小模块骨架

对正式实现，最终仍应以 workspace 级构建与测试为准，例如 `cargo check --workspace` 以及相关 crate 的单元/集成测试。

## Plan Sources

- 用户要求把这套规则沉淀到 `docs/`，用于项目 wiki 和后续生产开发协作
- [全系统重构方案](../../.codex/plans/全系统重构方案.md) 作为当前权威架构目标

## Open Issues or Follow-up

- 后续可以继续拆分本目录，分别补 `core-runtime-lifecycle.md`、`manager-facade-contracts.md`、`level-runtime-world.md`、`vm-plugin-host-contract.md`
- 当 `zircon_core`、`zircon_module`、`zircon_manager`、`zircon_entry`、`zircon_scene`、`zircon_script` 稳定后，应把这里的全局规则细化成每个子系统的叶子文档
