# 目录式 Project 资源抽象优先全链路替换计划

## Summary
- 这个计划替换当前“目录式 Project 资产渲染首里程碑计划”，把 `资源抽象` 提升为前置基础层，再继续推进 `project/import -> scene -> render -> editor`。
- 设计基线固定为：Bevy 的 `locator/path -> typed handle -> render prepare` 分层，加上 Fyrox 的 `resource io -> manager -> state -> renderer cache` 分层。
- 选型固定为新建 `zircon_resource` crate，并采用硬切换；`zircon_scene`、`zircon_graphics`、`zircon_editor` 不再继续直接依赖 `AssetId` / `AssetUri` 直通路径。
- 资源来源统一为 `res://`、`lib://`、`builtin://`、`mem://` 四类一等公民；不允许在 scene/render/editor 中继续散落 builtin 或 runtime-generated 资源特判。

## Public Interfaces
- 新增 `zircon_resource`，作为跨 crate 的资源基础层，公开：
- `ResourceLocator`、`ResourceScheme`、`ResourceId`
- `UntypedResourceHandle`、`ResourceHandle<TMarker>`
- 资源标记类型：`Model`、`Material`、`Texture`、`Shader`、`Scene` 对应的 marker types；typed handle 绑定 marker，不绑定 importer DTO
- `ResourceData` 擦除型 payload trait，用于承载 ready 资源数据并支持下游按类型读取
- `ResourceState`，固定为 `Pending`、`Ready`、`Error`、`Reloading`
- `ResourceRecord`，至少包含 `id`、`kind`、`primary_locator`、`artifact_locator`、`revision`、`state`、`dependency_ids`、`diagnostics`
- `ResourceEvent`，至少覆盖 add/update/remove/rename/reload-failed
- `ResourceRegistry`、`ResourceIo`、`ResourceManager`
- `ResourceLocator` 语法从第一版就支持可选子资源标签，统一使用 `#label`；glTF/FBX/atlas 等子资源以后不得再额外发明一套引用语法。
- `res://`、`lib://`、`builtin://` 的 `ResourceId` 由规范化 locator 稳定派生；`mem://` 由运行时分配，仅进程内有效，不跨会话稳定。
- `zircon_server` 新增 `ResourceServer` façade，提供 locator 解析、record/status 查询、revision 查询、资源事件订阅；现有 `AssetServer` 收窄为 project/import/watch 生命周期 façade。
- `zircon_asset` 保留 `TextureAsset`、`ShaderAsset`、`MaterialAsset`、`SceneAsset`、`ModelAsset` 作为具体资源数据类型，但它们不再充当跨 crate 的通用引用类型；它们通过 `ResourceData` 进入资源系统。
- `AssetUri`、`AssetId`、`AssetRegistry` 从 `scene/graphics/editor` 的公开依赖面移除，只允许作为 `zircon_asset` 内部迁移期实现细节存在，迁移完成后删除。

## Implementation Changes
1. `zircon_resource` 基础层先落地。
- 建立统一 locator 规范、typed/untyped handle、state machine、registry、event、diagnostic 和 `ResourceIo` 协议。
- `ResourceRecord` 是权威索引；scene、render、editor 都只能通过 handle 或 locator 间接访问资源，不能自行缓存源路径语义。
- `mem://` 资源允许 runtime/editor 临时创建并进入同一 registry，但它不可写入 `ProjectManifest`、`SceneAsset`、`MaterialAsset`。

2. `zircon_asset` 重建为 project/import 层，而不是通用资源层。
- `ProjectManager` 只负责目录项目、资产扫描、importer、artifact store、watcher 和 library 失效规则。
- `AssetManager` 保留为高层编排器，但内部拆成 `ProjectManager + ResourceManager` 组合；资源注册、状态、事件由 `ResourceManager` 负责，项目扫描和导入由 `ProjectManager` 驱动。
- importer 输出统一注册为 `ResourceRecord + Arc<dyn ResourceData>`；`builtin://` 和 `mem://` 也通过相同注册入口进入 registry，不再靠 `builtin_asset()` 一类分支散落在 manager 内。
- watch/reimport 规则改成 revision 驱动；成功重载递增 revision，失败重载保留上一个 `Ready` payload，同时把 record 状态置为 `Error` 或 `Reloading+diagnostic`，上层能继续工作但必须看见错误。

3. `zircon_scene` 改成 handle-runtime 模型。
- `MeshRenderer` 改为持有 `ResourceHandle<ModelMarker>` 和 `ResourceHandle<MaterialMarker>`；render extract 也持 handle，不持 `AssetId`。
- `SceneAsset` 仍序列化 locator，但加载时统一解析成 typed handle；scene runtime 不接触 importer DTO，也不接触 artifact 路径。
- 删除当前 `builtin://cube <-> res://models/cube.obj`、`builtin material <-> project material` 这类隐式重写；序列化时写出真实 locator。
- 默认项目模板如果需要 project-local 支撑资源，应在项目创建时显式生成 `res://` 资源并让场景直接引用它们，而不是在 save/load 时偷偷做 locator 映射。

4. `zircon_graphics` 改成 prepare/cache 分层。
- 新增 graphics 内部的 `PrepareRenderResource` 流程，等价于 Bevy 的 `RenderAsset::prepare_asset`；它从 `ResourceManager` 的 ready payload 派生 `PreparedModel`、`PreparedTexture`、`PreparedShader`、`PreparedMaterial`。
- GPU cache 统一按 `ResourceId + revision` 失效，不再直接把 `TextureAsset`、`ModelAsset`、`ShaderAsset` DTO 长期塞在 renderer 的业务缓存里。
- pipeline 选择、材质绑定、shader 编译缓存都建立在 prepared/render resource 上，而不是建立在 importer DTO 上。
- gizmo、grid、selection 继续保持 render-only overlay，不进入资源系统。

5. `zircon_editor` 和 `zircon_server` 最后迁移。
- editor 的资产面板、viewport bridge、项目打开/保存流程改为查询 `ResourceServer + AssetServer` 两层接口，而不是把项目状态、资源状态和导入状态混在一个 manager 上。
- 打开项目时先建立资源世界，再实例化默认 scene；资源变化先更新 `ResourceRecord.revision/state`，再由 scene/render/editor 按需刷新。
- 所有需要列资源、显示错误、显示来源的 UI 都直接读 `ResourceRecord` 和 `diagnostics`；不再自己拼接 URI/ID/导入状态。

## Test Plan
- `zircon_resource` 单元测试覆盖：locator 规范化、`#label` 解析、scheme 边界、stable id 规则、`mem://` 非稳定性、typed/untyped handle 转换、state 转换、diagnostic 保留、event 顺序。
- `zircon_asset` 单元与集成测试覆盖：`res/lib/builtin/mem` 四类资源统一注册、project 扫描生成 records、reimport revision bump、rename/delete、重载失败保留 last-good payload、watcher 事件折叠。
- `zircon_scene` 测试覆盖：scene locator roundtrip、typed handle 恢复、`mem://` 序列化拒绝、`builtin://` 原样持久化、缺资源 placeholder 行为、默认项目模板生成显式 `res://` 支撑资源。
- `zircon_graphics` 测试覆盖：prepared cache 命中与失效、`revision` 驱动的 shader/material/texture 重建、headless project render、shader 颜色回归、gizmo overlay 与资源刷新并存。
- `zircon_editor` 测试覆盖：目录项目打开后资源树可见、默认 scene 从 locator 实例化、资源错误诊断可见、watcher 触发后 viewport 依据 revision 刷新。
- 验收命令固定为：`cargo test -p zircon_resource -p zircon_asset -p zircon_scene -p zircon_graphics -p zircon_editor` 和 `cargo test --workspace --locked`。

## Assumptions
- 这是有意的 breaking change；不保留 `AssetId/AssetUri` 在 scene/editor/graphics 的兼容通路。
- `zircon_resource` 是唯一资源基础层；以后新增资源种类先扩 marker/data/prepare 协议，不允许在上层新加字符串特判。
- `builtin://` 是可序列化、可查询、可渲染的一等 locator；`mem://` 是可注册、可引用、不可持久化的一等 locator。
- 热重载失败时默认保留最后一个成功的 `Ready`/prepared 资源，错误通过 `ResourceRecord.diagnostics` 和事件上报，不采用“失败即清空资源”的策略。
- 后续 FBX、ASTC、PVRTexTool 接入只允许扩展 importer 和具体 `ResourceData`，不得破坏 locator/handle/state/prepare 四层契约。
