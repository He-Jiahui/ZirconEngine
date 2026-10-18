# 全仓 Rust 模块化整改范围计划

## Summary

本轮按新模块边界 skill 的**字面全量范围**执行：审计整个 `zirconEngine` 工作区内**全部 235 个跟踪中的 Rust 文件**，但**只修改违规文件**，不为了“碰过每个文件”而强制改动。

当前锁定的范围是：

- `165` 个生产 Rust 文件
- `69` 个测试 Rust 文件
- `1` 个 `build.rs`
- `5` 个 `.slint` 文件不作为主整改对象，只在 Rust 模块拆分导致导入、绑定或 UI 投影接口联动时被动调整

## Implementation Changes

### 审计与改动边界

- 覆盖全部 `22` 个 `zircon_*` crate；所有 Rust 文件都要被审计是否违反“单文件单职责、根模块只做 wiring、相似模块必须目录化、声明与实现拆分”的规则。
- 已经合规的文件只记录为“通过审计”，不强制改动。
- 不把这轮工作扩展成功能变更；目标是模块树重组、职责解耦、测试镜像调整和必要的 `pub use`/模块声明收口。
- `.slint` 文件不单独按同一 skill 做系统性重组；仅在 Rust 侧拆分后为保持编译与绑定关系正确而做最小联动。

### 波次顺序

1. **Wave 1: 共享基础层**
   - `zircon_core`、`zircon_math`、`zircon_module`、`zircon_resource`、`zircon_manager`、`zircon_ui`、`zircon_input`
   - 约 `51` 个 Rust 文件
   - 目标：先清理基础 crate 的 root file、共享数据结构、事件/绑定/布局等公共边界，避免上层 editor/scene/asset 后续重复适配

2. **Wave 2: 资源与场景渲染层**
   - `zircon_asset`、`zircon_scene`、`zircon_graphics`
   - 约 `87` 个 Rust 文件
   - 目标：按资源、场景、渲染子系统切成目录化树，保证 `lib.rs` 和中间 `mod.rs` 只做导航和导出

3. **Wave 3: 编辑器契约与壳层**
   - `zircon_editor_ui`、`zircon_editor`、`zircon_entry`
   - 约 `88` 个 Rust 文件，外加 `5` 个可能联动的 `.slint`
   - 目标：重点处理你点名的 binding/host/workbench/slint_host 等混装模块；`binding.rs` 类文件收敛为最薄绑定入口，命令/载荷/路由/转换按域拆目录

4. **Wave 4: 轻量单文件 crate 审计波**
   - `zircon_animation`、`zircon_navigation`、`zircon_net`、`zircon_particles`、`zircon_physics`、`zircon_platform`、`zircon_script`、`zircon_sound`、`zircon_texture`
   - 共 `9` 个 Rust 文件
   - 目标：全量审计，只有在单文件已明显混多职责时才拆；否则保留现状并记录“审计通过”

### Public APIs / Interfaces

- 这轮**不以新增公共 API 为目标**。
- crate 对外导出语义应保持稳定；允许内部模块路径、`mod` 组织、`pub use` 汇聚位置重排，但不应把纯结构性重构升级为对外行为改动。
- 测试树要随生产模块同步镜像调整；不允许生产代码更分层、测试还停留在旧的扁平结构。

## Test Plan

- 每一波结束都要完成该波 crate 的编译与测试验证，而不是只做最终一次总验证。
- Wave 1 验证共享基础 crate 的单元测试与直接依赖编译通过。
- Wave 2 验证 `asset/scene/graphics` 三层的单元测试、交叉依赖编译与关键公共 re-export 不回归。
- Wave 3 验证 `editor_ui/editor/entry` 的单元测试、集成测试、Slint 构建链路和绑定分发相关编译。
- Wave 4 验证所有单文件 crate 经审计后仍能通过最小编译/测试。
- 最终必须补一次 workspace 级验证，确认整个 235 Rust 文件范围内没有因模块树重排留下的导入、可见性或测试路径回归。

## Assumptions

- “所有脚本”在本轮定义为**全部跟踪中的 Rust 文件**；不包含仓库里不存在的 `py/ps1/sh` 代码面。
- `.slint` 不算这轮主整改面，但允许被 Rust 重组带动调整。
- “所有脚本”表示**全量审计**，不是**强制所有文件都改**。
- 计划默认采用**自底向上分层波次**，避免 editor 层先拆后又被下层结构重组反向冲刷。
