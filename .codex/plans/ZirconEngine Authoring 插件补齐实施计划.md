**ZirconEngine Authoring 插件补齐实施计划**

**Summary**
- 覆盖计划中的六个 Authoring 插件：`terrain`、`tilemap_2d`、`material_editor`、`prefab_tools`、`timeline_sequence`、`animation_graph`。
- 以 `dev/UnrealEngine` 为第一参照：Landscape、Paper2D、MaterialEditor、Sequencer、AnimGraph、LevelInstance/BlueprintGraph；实现仍以当前 Zircon 插件体系、`plugin.toml`、catalog、capability gate 和 extension registry 为准。
- 执行时直接在当前 `main` 工作区推进，保留现有脏改动，不回滚无关文件；采用硬切换，不新增旧路径 shim、兼容 re-export 或桥接层。

**Key Interface Changes**
- 新增 runtime-backed Authoring 包：`terrain`、`tilemap_2d`、`prefab_tools`；为它们补 `RuntimePluginId`、runtime/editor crate、`plugin.toml`、manifest/catalog/workspace/README 登记。
- 新增 editor-only Authoring 包：`material_editor`、`timeline_sequence`、`animation_graph`；它们只进入 editor catalog 和插件工作区，不进入 runtime export 链路。
- 扩展 editor extension registry，增加通用 authoring 描述符：viewport/tool mode、graph editor/node palette、timeline editor/track type、asset creation template；所有描述符都要支持 capability gate、重复 ID 校验、manifest/catalog 合并测试。
- 扩展 runtime asset 数据契约：新增 `TerrainAsset`/layer 数据、`TileSetAsset`、`TileMapAsset`、`PrefabAsset`、`MaterialGraphAsset`；复用现有 `AnimationSequenceAsset`、`AnimationGraphAsset`、`AnimationStateMachineAsset`。
- 统一 capability 命名：
  `runtime.plugin.terrain`、`runtime.plugin.tilemap_2d`、`runtime.plugin.prefab_tools`；
  `editor.extension.<plugin_id>_authoring`。

**Milestones**
1. **Foundation**
   - 固化六个插件 ID、asset kind、operation path、capability 和 manifest 规则。
   - 扩展 `zircon_editor` 的通用 authoring descriptors，并让 `editor_support` 能批量注册 view、drawer、template、operation、importer、asset editor、tool mode、graph/timeline 描述符。
   - 测试 registry 合并、重复 ID、capability gate、禁用插件后扩展消失。

2. **Runtime Data Contracts**
   - 为 `terrain`、`tilemap_2d`、`prefab_tools` 添加 runtime asset/component DTO、序列化、引用收集、manifest contribution。
   - 为 `MaterialGraphAsset` 添加 authoring-source 数据结构和编译目标 `MaterialAsset` 的最小契约。
   - 测试 TOML/二进制 roundtrip、direct references、无效 ID、manifest/catalog 一致性。

3. **Package Skeletons**
   - 新建六个插件包结构：三个 runtime+editor，三个 editor-only。
   - 更新 `zircon_plugins/Cargo.toml`、插件 README、runtime/editor builtin catalogs、root/plugin workspace 测试期望。
   - 每个 editor crate 先注册最小 view、drawer、template、menu item、operation，确保开关能力可观测。

4. **Terrain Authoring**
   - 参考 Unreal Landscape：实现 heightfield/layer stack、terrain component、heightmap/weightmap importer 描述符、terrain viewport tool mode、组件 drawer 和基础操作。
   - 支持导入扩展默认：`raw`、`r16`、`png`；输出 kind 为 `terrain.heightfield`、`terrain.layer_stack`。
   - 测试 importer 扩展冲突、component prefix、无效尺寸、capability gate、runtime target loading。

5. **Tilemap 2D Authoring**
   - 参考 Unreal Paper2D：实现 tileset/tilemap/layer/projection 数据，projection 默认支持 orthogonal、isometric_diamond、isometric_staggered、hexagonal_staggered。
   - 注册 tilemap asset editor、grid/layer/pivot/stats 操作、Tiled 导入器；默认导入扩展 `tmx`、`tsx`、`json`。
   - 测试层尺寸校验、tileset 引用、projection roundtrip、importer 登记、禁用插件后的 UI/operation 移除。

6. **Material Editor**
   - 参考 Unreal MaterialEditor：实现 `MaterialGraphAsset` 节点 palette，v1 节点为 output、texture_sample、scalar_parameter、vector_parameter、add、multiply。
   - 注册 material graph editor、material asset editor、compile/preview/validate operations；compile 产物写入现有 `MaterialAsset` 契约。
   - 测试缺少 output、重复节点、断开的必需输入、参数默认值、asset editor capability gate。

7. **Prefab Tools**
   - 参考 Unreal LevelInstance：实现 `PrefabAsset`、prefab instance component、override DTO。
   - 注册 create-from-selection、open prefab、apply overrides、revert overrides、break instance operations。
   - 测试 prefab 引用、override precedence、缺失源 prefab、break 后不再保留 prefab link、editor-only 状态不泄漏到 runtime DTO。

8. **Timeline Sequence**
   - 参考 Unreal Sequencer：基于现有 `AnimationSequenceAsset` 注册 timeline editor。
   - v1 track type：transform、component_property、event_marker；注册 keyframe/track create/delete/move/validate operations。
   - 测试时间范围、track 排序、keyframe 边界、event payload 校验、依赖 animation 插件的 gate 行为。

9. **Animation Graph**
   - 参考 Unreal AnimGraph：基于现有 animation graph/state machine assets 注册 graph editor。
   - v1 node palette：clip、blend、output、state、transition、condition；注册 validate/compile/open operations 和 animation player component drawers。
   - 测试缺少 output、重复 node ID、缺失 clip/state 引用、非法 transition、禁用 animation 依赖时插件不可用。

10. **Hardening, Docs, Validation**
   - 确认 editor workbench 通过通用 descriptors 消费六个插件，不为单个插件写特殊分支。
   - 确认 runtime export：editor-only 插件不进入 runtime link plan；runtime-backed 插件只在被选中时进入。
   - 更新 `docs/` 中相关插件、runtime asset、editor extension registry 文档，并记录 Unreal 参照来源、测试命令、实现文件。
   - 最终运行 root workspace build/test、plugin workspace build/test、manifest/catalog/export plan 测试；所有失败必须在本里程碑内修正后再收口。

**Test Plan**
- 每个里程碑结束运行对应 package 级测试；最终运行 CI 对齐命令：root `cargo build --workspace --locked --verbose`、`cargo test --workspace --locked --verbose`，以及 `zircon_plugins` workspace 的 build/test。
- 使用仓库验证脚本 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1` 覆盖 root 与插件工作区。
- 必测场景：manifest/catalog/workspace 一致性、插件 enable/disable gate、runtime target loading、editor-only export 排除、asset 序列化 roundtrip、importer/operation/descriptor 重复 ID、graph/timeline 校验错误路径。

**Assumptions**
- 六个 Authoring 插件全部纳入本轮计划。
- Unreal 只作为结构和生命周期参照，不复制源码；Zircon 当前插件/manifest/registry 体系优先。
- `material_editor`、`timeline_sequence`、`animation_graph` 默认 editor-only；`terrain`、`tilemap_2d`、`prefab_tools` 默认 runtime-backed。
- v1 UI 以可注册、可打开、可测试的 authoring surface 为完成标准；复杂视觉交互可在后续视觉打磨里程碑深化。
