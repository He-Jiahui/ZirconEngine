# ZirconEngine 全项目架构与代码扫描计划

## Summary
- 范围：扫描根 workspace、`zircon_plugins`、`docs/plans/tests/tools`，排除 `dev/` 参考引擎树、`target*` 构建产物和 `temp/`。
- 目标：先形成架构地图、明显 bug/编译风险、代码冗余热点、结构整理优先级；本轮扫描不改代码、不重构。
- 基线事实：当前在 `main`，工作区很脏；需把“已有未提交改动/并行会话改动”和“当前扫描发现”分开记录。

## Key Changes / Review Work
- 建立项目索引：统计 root/plugin workspace 的 crate、源码规模、顶层模块、公开 re-export、超大文件、测试/生产路径分布。
- 编译与迁移一致性扫描：优先检查 deleted/new 文件对应的 `mod` 引用、`Cargo.toml` 成员、`Cargo.lock`、root workspace 与 `zircon_plugins` workspace 是否一致。
- 架构边界审查：按最新收束规则检查 `zircon_app`、`zircon_runtime::core::{runtime,manager,framework,math,resource}`、`zircon_editor::{core,scene,ui}`、`zircon_runtime_interface` 和插件目录的职责是否混杂。
- Bug 热点扫描：区分生产代码与测试代码，检查生产路径中的 `unwrap/expect/panic/unsafe/todo`、资源生命周期、FFI/native plugin ABI、asset importer/resource variant 穷尽性、editor/runtime 状态越界。
- 冗余和结构热点：重点审 `zircon_runtime/src/lib.rs` 宽重导出、`zircon_editor/src/ui/**` 大体量 UI 聚合、VG/HGI 测试资源超大文件、重复插件 manifest/descriptor/service 类型样板。
- 交付物：输出一份分级 review 报告，包含架构图、P0/P1/P2 findings、冗余清理候选、后续重构里程碑；不在第一轮直接整理代码。

## Public APIs / Interfaces
- 第一轮扫描不修改 public API、crate 成员、类型、模块路径或文档。
- 若报告建议后续重构，必须把 public API 影响单独列出：新增/删除 re-export、移动模块、插件 manifest 字段、runtime interface DTO/ABI 变化都要进入下一份实施计划。

## Test Plan
- 轻量事实检查：
  - `cargo metadata --no-deps --format-version 1`
  - `cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1`
  - `git diff --check`
- 编译验证按阻塞程度执行并记录证据：
  - `cargo check -p zircon_runtime --lib --locked --jobs 1`
  - `cargo check --workspace --locked --all-targets --jobs 1`
  - `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --jobs 1`
- 若遇到并发 Cargo 或长时间依赖编译阻塞，记录为 inconclusive，并用最近 `.codex/sessions/*.log` 与更窄 package checks 归因，不擅自接管其他 active session 的模块。

## Assumptions
- “完整扫描”按你选择的项目代码范围执行，不包含 `dev/` 外部参考树。
- 当前大量未提交变更视为用户/其他会话已有工作，扫描只读，不 revert。
- “明显 bug”定义为：编译错误、模块/文件不一致、生产路径 panic/unsafe 风险、资源/ABI/lifetime/variant 穷尽性问题、架构边界泄漏；深层算法正确性只在已有测试或可复现证据支持时下结论。
