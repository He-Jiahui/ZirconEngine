# 生成的 Model 子网格引用没有绑定最终 Mesh UUID

状态：问题 Draft，仅定位、对照和验收设计。无生产或测试改动，不是 #228 的 RED，不是变换或切线问题。

## 用户可见问题

用项目导入的 Model 作为网格源时，模型记录及其带标签子 Mesh 都可能显示已导入，但渲染资源入场拒绝 `Model mesh reference UUID/locator mismatch`。根 Model 通过目录的依赖列表能查到 Mesh，不代表 Model 载荷中的 `AssetReference` 实际指向该 Mesh。真实直接 `Mesh` 绑定不经过 Model 的 primitive 引用，能作为独立对照，不是修复方案。普通渲染也使用这一解析入口；目前原生执行证据仅来自三个归档 VG 用例，不外推为所有格式或提交路线均已实测。

## 已核实的证据

在保留的 `e27e84e` 归档里，两个 `.model.toml`（单驻留页／两页）和一个内建 OBJ 对照实际启动 RTX 3060 Laptop DX12，均 exit 101，且都先在上述资产检查失败，未经过 #228 所指的执行器、未捕获画面。对两份仍在的 `.model.toml` 项目只读校验 manifest、压缩块 BLAKE3 和长度；按 v7 positional 布局解出 Model 首个 primitive 的 UUID+locator、Mesh 的载荷 locator，同时复算 UUID 域及其 `ResourceId`：

| locator | Model 工件内引用 UUID | 子 Mesh 工件／注册身份的资产 UUID |
| --- | --- | --- |
| `res://models/vg_triangle.model.toml#Mesh0/Primitive0` | `50d9e213-38a5-8bff-a433-26a9eaff98f8` | `f7e8a1da-6cc0-4ab7-960f-86c25a136f0b` |
| `res://models/vg_two_pages.model.toml#Mesh0/Primitive0` | `022a2eb1-4cb6-85ec-92c7-e7c662843ebc` | `a557c562-a240-498a-9ca1-48b29329dcb8` |

两份源文档都未显式指定 `mesh`，位置与索引与 Model 工件前缀相符；侧车、资产注册和根依赖的 UUID 一致。归档中 joint index 槽位的变化是 [VG 顶点 ordinal](../../../zircon_runtime/src/asset/assets/model/primitive.rs#L30-L57)，不是本问题。错误中的 ID 与工件里的 locator 派生 UUID 一致，而不是 WGPU/Runtime 将正确 ID 改坏。OBJ 归档对照未保存项目，只有设备与原生错误记录。以上是旧原生执行及保留字节的对账，不是本 Draft 的新冻结 RED，也未解码完整 Mesh/VG 载荷。

## ZE 所有权与源锚点

- **Runtime 导入生成关系**：[作者 Model TOML](../../../zircon_runtime/src/asset/importer/ingest/import_model.rs#L10-L24) 和 [内建 OBJ](../../../zircon_runtime/src/asset/importer/ingest/import_obj.rs#L68-L96) 汇入[子资产生成](../../../zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs#L12-L29)。已经解析的显式引用保留；否则生成 `#MeshN/Primitive0` Mesh 及 `from_locator` 占位引用。[interface](../../../zircon_runtime_interface/src/resource/asset_reference.rs#L25-L52) 明确派生操作不查侧车；[ResourceId](../../../zircon_runtime_interface/src/resource/resource_id.rs#L83-L101) 又是最终资产 UUID 的独立派生域，不等于原 UUID。
- **Project 是发布身份的所有者**：[侧车](../../../zircon_runtime/src/asset/project/manager/scan_and_import/metadata.rs#L156-L168) 为标签沿用旧 UUID 或生成新 UUID；[全量发布](../../../zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L680-L742) 用最终 entry 身份写记录，却直接将 importer 的 `entry.asset` 写工件。也要核对[提前捕获的直接引用](../../../zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L326-L353)。[定向发布](../../../zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs#L625-L675) 在 [registry 标签碰撞归一化](../../../zircon_runtime/src/asset/registry/targeted.rs#L115-L159) 后取 ID，同样把未重新绑定的载荷交给工件及 ready 资源。[全量缓存命中条件](../../../zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L552-L604) 没有检查该引用等式；只修新导入不能自动证明既有缓存会修正。
- **已有 wire 和消费校验应该保留**：[Model 库工件](../../../zircon_runtime/src/asset/artifact/cache_payload/model.rs#L64-L118) 保存、恢复既有 Mesh 引用；[AssetReference 二进制](../../../zircon_runtime_interface/src/resource/asset_reference.rs#L75-L96) 明确写 UUID 和 locator。Runtime [冷依赖入场](../../../zircon_runtime/src/graphics/scene/resource_streamer/model_geometry_resolution.rs#L119-L160) 要求两者都命中当前投影，[验证](../../../zircon_runtime/src/graphics/scene/resource_streamer/model_geometry_resolution.rs#L165-L214) 对确定性兼容地址只收精确成对的 ID；错 UUID、同路径不是可修复的通配符。[负对照](../../../zircon_runtime/src/graphics/scene/resources/tests/model_geometry_resolution.rs#L209-L247) 应继续拒绝它。ECS 保留 `MeshRenderer` 的现有 Model／Mesh 选择，不在 World 或渲染入口重写项目身份。
- **其他 producer 只做范围说明**：源码中的[内建 glTF](../../../zircon_runtime/src/asset/importer/ingest/import_gltf.rs#L132-L135)、[链接 glTF 标签 Model](../../../zircon_plugins/gltf_importer/runtime/src/subassets.rs#L93-L143)、[链接 OBJ](../../../zircon_plugins/obj_importer/runtime/src/lib.rs#L69-L101) 和 [可选静态模型导入器](../../../zircon_plugins/asset_importers/model/runtime/src/mesh_importer.rs#L181-L213) 也含 locator 派生的生成关系，但本轮没有运行它们。任何触及这些路径的后续设计先协调 #141 等相邻 Draft，不包含 MikkTSpace、morph、Gltf scene binding、SDF 或 VG 内容改动。#219 在全量／定向同文件当前改的是输入快照捕获，不是子资产发布；#137、#139 已拥有 Runtime 解析器夹具同文件，新增测试应独立，不把其 fixture 改动塞入本 Draft。修改前仍复核精确行和 live diff。

## 对照物理 Unreal 源码

Interchange 的 [StaticMesh 工厂](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Mesh/InterchangeStaticMeshFactory.cpp#L213-L287>) 优先用重导 `ReimportObject`，否则尝试已绑定软对象路径，无对象才创建；重导保持已选 `UStaticMesh`，单独卸载其渲染状态与资源。[导入任务](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Interchange/Engine/Private/Tasks/InterchangeTaskImportObject.cpp#L597-L636>)把工厂返回的**实际产物**写回 `FSoftObjectPath`，没有把 import node UID 当 UObject 身份。[Actor 工厂](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Import/Private/Scene/InterchangeStaticMeshActorFactory.cpp#L140-L185>)从节点找到目标、加载并验证 `UStaticMesh` 后才绑组件；[Nanite 代理](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Rendering/NaniteResources.cpp#L892-L923>)消费选定 Mesh 的 render data，不纠正导入身份。借鉴的是「先确定实际产物、后绑定，再发布给渲染器」，不复制 UObject 身份、路径规则、生命周期或 UE `FTransform`。本轮是物理源码对照，没有运行 UE。

## 冻结和判定计划

1. 用真实项目导入，区分首次全量发布、内容已改的定向重导、关闭重开后的工件缓存命中。读取最终 `.zmeta`、registry、typed ready snapshot 和持久工件；同一个生成 primitive 的 UUID 和 locator 必须与其 Mesh 一致，并关联精确 revision。覆盖一次故意触发标签碰撞重铸，旧有效 UUID 重导保持不变，已解析的显式外部 Mesh、精确 locator 错 UUID 的负对照，不能只断言目录依赖边。当前[旧测试](../../../zircon_runtime/src/asset/tests/pipeline/manager/model_import.rs#L59-L87)明写派生 UUID 为预期，不能把它当完整身份一致性验收。
2. 原生用例通过真实 ECS/Level 只给 Model handle、不设置直接 Mesh；同口径直接 Mesh 为正控制。先普通渲染，再按原生可用路线核对 VG，不用 #228 未修的自动提交当唯一验收路径。检查同代前景和背景像素；只把 UUID/locator 精确失配且正控制成功的失败记为本 Draft 的 RED。
3. 保持当前严格 UUID 校验和已有 durable 事务。只讨论具有明确「本次 importer 生成」来源的引用在确定最终 entry 身份后如何同时发布到载荷、目录和缓存。不能凭版本 8、相同 locator、空 inline 或标签拼法重写显式引用；对既存坏缓存先证明失效或有来源保证的迁移策略，不清缓存冒充通过。冻结并执行未改生产真实 RED，发布证据后才改，复跑同用例；不启用 CI、不跳过检查、不用 Jenkins、不合并未验收。
