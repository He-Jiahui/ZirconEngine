**GI/VG 插件化激进迁移计划**

## Summary

把 Hybrid GI 与 Virtual Geometry 从 `zircon_runtime` 的 graphics/scene/runtime/type 层硬迁移到对应 `zircon_plugins` 插件内。迁移采用硬切换：不保留旧路径 re-export、compat module 或 shim；实现期间只做语法/类型检查，全部迁完后再跑单元测试。

## Key Changes

- 将 scene renderer 下的 GI/VG 专属目录迁入插件：
  - `zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/...`
  - `zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/...`
  - 同步迁移 shader 文件，并修正 `include_str!` 相对路径。
- 将 `zircon_runtime/src/graphics/types` 中所有 GI/VG 高级 DTO 抽到插件：
  - GI：prepare、resolve runtime、scene prepare/readback/completion 相关类型。
  - VG：prepare、cluster selection、node/cluster cull、raster draw、scratch/debug completion 相关类型。
  - runtime 仅保留通用 `GraphicsError`、`GpuResourceHandle`、`ViewportFrame*`，以及 core framework 公共 render/debug DTO。
- 在 `zircon_runtime` 增加最小中立渲染插件接口，让 runtime 不依赖 `zircon_plugins`：
  - 插件通过已有 extension/descriptor 注册机制注册 renderer runtime hook。
  - root scene renderer 只保存 descriptor/executor id、trait object 或 erased output packet。
  - GI/VG 具体 GPU resources、readback、completion parts、prepare outputs 均由插件私有持有。
- 清理 root renderer 中所有直接命名 GI/VG 类型的模块：
  - `SceneRendererAdvancedPluginResources` 改为插件资源注册表/执行表。
  - advanced outputs/readbacks/storage 改为按 feature key 的中立输出。
  - VG 专属 render pass 模块一并迁入 virtual_geometry 插件，root render path 只通过 hook 调用。
- 移除旧 runtime 路径：
  - 删除/断开 `graphics::scene::scene_renderer::{hybrid_gi, virtual_geometry}` re-export。
  - 删除/断开 `graphics::runtime::{hybrid_gi, virtual_geometry}` 残留引用。
  - 删除 `graphics::types` 中 GI/VG 类型出口，更新所有 imports。

## Docs And Tests

- 实现阶段每个迁移片只跑语法/类型检查：
  - `cargo check -p zircon_runtime --lib --locked --offline`
  - `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --offline`
- 全部迁移完成后再跑单元测试：
  - `cargo test -p zircon_runtime --lib --locked --offline`
  - `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --lib --locked --offline`
- 同步更新渲染/插件化文档，至少覆盖：
  - 插件拥有 GI/VG renderer runtime 的新边界。
  - `graphics/types` 中 GI/VG DTO 移除后的替代导入路径。
  - 最终验证命令与结果。

## Assumptions

- 继续直接在当前 `main` 工作树迁移，不新建 branch/worktree。
- `zircon_runtime` 不能依赖 `zircon_plugins`；依赖方向保持插件依赖 runtime。
- 不保留旧 API 兼容层，编译错误通过更新调用点解决。
- 用户选择的 `graphics/types` 策略为“全量高级 DTO 抽出”，所以 GI/VG 具体 prepare/readback/completion 类型都归插件。
- 执行期间不跑单元测试；只有所有目录、类型、引用和文档迁移完成后进入最终测试阶段。
