# 反射序列化：DynamicScene 无法恢复带已声明默认值的非持久 VM 组件字段

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。第 1–6 节保留初次调查及测试计划；当时未运行 UE/ZE 测试。ZE 原实现的原生结果见第 7 节，修复和配对结果见第 8 节；未执行 UE 测试。

**当前状态：已在原 Draft 修复。原 6 条回归和 1 条补充控制均通过；3 个相邻用例在原实现与候选上仍以相同断言失败，不是全库通过。** 首次调查结论如下： 是既有 [Runtime63 RSR-P1-037](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md#L254-L264) / [Runtime99l](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor.md#L245-L254) “完整字段集合与默认值”下的可复核子项：**同 schema、字段明确不入存档且有合法默认值**，不要求新增 optional/redirect/unknown 字段版本政策。和脚本 `onStart` 不是同一问题。

ZE 本地 `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；与 Draft 固定 blob 基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 在 5 个引擎目录无差异。UE 为本机 [Build.version:1–10](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L10>) 的 `6.0.0 / BranchName UE5`；下列 UE 定位为该检出，SHA-256 见末表。

## 1. 不一致：导出有权省略，恢复却把省略视为无效对象

- **ZE 允许合法输入。**
 [field_info.rs:20–41,111–133](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime_interface/src/reflect/field_info.rs#L20-L41) 的字段元数据分别记录 `serializable`、`editable`、`default_value`；[type_registration.rs:50–79](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime_interface/src/reflect/type_registration.rs#L50-L79) 的 `Json` 组件可在类型级参与持久化。
实际 ZrVM `stateSchema` 字符串由 [instance.rs:119–136](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs#L119-L136) 解码（有反射 schema 的 JSON 协议；[BytesV1 明确不支持](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/zr_vm_language/runtime/src/real_backend/state_encoding.rs#L106-L114)），公开组件经 [schema.rs:15–32](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/reflection/schema.rs#L15-L32) 的 `VmReflectionSchema::from_state_schema`、[hot_reload_coordinator.rs:198–234](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs#L198-L234) 的 catalog 预检/发布，进入既有及新 World（[catalog.rs:280–303,318–341](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/reflection/catalog.rs#L280-L341)）。
[type_registry.rs:669–733](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/reflect/type_registry.rs#L669-L733) 会验证已声明默认值的预算和类型，不禁止这种标志组合。
设公开插件组件 `probe.Component.Scratch` 的两个小整数是 `saved=7`（持久）及 `scratch=99`（`serializable=false`，schema 默认 `42`）；合法组件可由 [gameplay `set_component_json`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/script/vm/gameplay_host/components.rs#L64-L79) 等受检入口附加。
- **ZE 捕获。** [RuntimeSessionSlot::from_world_with_metadata:17–28](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/session/slot/capture.rs#L17-L28) → [DynamicScene::from_world:6–13](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/scene/world_operations.rs#L6-L13) → [capture.rs:104–128,163–178](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/scene/capture.rs#L104-L178) 的 `reflected_components_for_entity/serializable_fields`，快照只包含 `saved=7`。这是正确的省略。Editor 正常菜单 [menu_action.rs:293–314](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/editor_event_execution/menu_action.rs#L293-L314) 经 [PlaySceneSource::from_world:19–23](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/play/snapshot/source.rs#L19-L23) 亦用这个捕获/编码入口。
- **ZE 恢复（目标已加载同一 VM schema）。**
 [RuntimeSessionArchive::apply_slot_to_level:18–25](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/session/facade/restore/apply/named.rs#L18-L25) → [RuntimeSessionSlot::apply_to_level/apply_to_world:8–14,41–46](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/session/slot/restore.rs#L8-L46) → `DynamicScene::spawn_into`。
插件分支 [spawn/transaction.rs:463–503](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/scene/spawn/transaction.rs#L463-L503) 用目标注册把存档的稳定字段 ID 变回名称；[value/json.rs:13–56](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/value/json.rs#L13-L56) 只转换 payload 中存在的键，产物是 `{saved: 7}`，不读 `scratch.default_value`。
预检 [world/transaction.rs:499–530](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/transaction.rs#L499-L530) 保留目标完整反射注册和 VM 路径标记；[spawn/transaction.rs:685–689](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/scene/spawn/transaction.rs#L685-L689) 调用 [World::set_dynamic_component:224–240](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/dynamic_components.rs#L224-L240)，最终 [validate_dynamic_component_value_against_registration:738–777](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/dynamic_components.rs#L738-L777) 逐字段要求全套键，预测返回 `UnknownField(scratch)`（经 `SceneError` 到动态场景错误），尚未发布。
Play 读端 [project.rs:280–324](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/project.rs#L280-L324) 同样调用 `spawn_into`；[construction.rs:274–289](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/construction.rs#L274-L289) 在创建 Play level 前加载启动脚本，[level_manager_lifecycle.rs:125–138](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/module/level_manager_lifecycle.rs#L125-L138) 建 World 前先应用其 runtime extension（上列 catalog）。
Play 是否走到该条件仍取决于源 World 数据和启动脚本实际配置。
- **UE 对照。** [CoreUObject.Build.cs:26–34](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/CoreUObject.Build.cs#L26-L34>) 依赖 `Core`（archive 基础）。[UObject::SerializeScriptProperties:2034–2059,2095–2099](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/Obj.cpp#L2034-L2099>) 在 tagged 路径将已存在的对象存储和 archetype 交给 `UStruct::SerializeTaggedProperties`。普通持久实例不写 transient property，[FProperty::ShouldSerializeValue:1043–1067](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/Property.cpp#L1043-L1067>)；[Class.cpp:1992–2019](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/Class.cpp#L1992-L2019>) 在获取属性地址前就应用此谓词，读时也不要求每个声明字段在流里有一项。语义输出是持久字段还原、非持久字段保持目标初始化值，而不是“缺键导致整个实例恢复失败”。

## 2. 有界实际风险

触发条件只需当前合法组件、一个规模正常且有明确默认的非持久字段、同一 VM 运行时 schema、一次捕获/生成快照再向已注册 World 应用。目标风险是增量场景/槽位整个应用被拒绝（Play 配置符合条件时也是同一读路径），即使没有旧版本、损坏文件、插件卸载、超预算或并发；不能靠存档泄露 `scratch=99` 修复。反例不等价于一般项目场景文件、无 schema 的 restore 或所有已经发行的插件；未看到一份一方产品配置实际导出此标志组合，故不虚报已观察玩家故障。目标原 World 不会发布预检失败的实体，见 [apply_compiled_scene_spawn:180–189](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/scene/spawn/transaction.rs#L180-L189)，不宣称覆盖预检之外的所有事务错误。

## 3. 根因与 UE 真正处理

ZE 把“字段稀疏的持久表示”当作“字段完整的 VM 实例 JSON”直接提交。问题不是完整实例校验过严，而是恢复构建完整实例缺了一步。相比之下，UE 对象构造与 sparse archive 覆盖是两件事：[UObjectGlobals.cpp:4258–4271,4563–4596](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/UObjectGlobals.cpp#L4258-L4271>) 在通常从 archetype 初始化的路径调用 `FObjectInitializer::InitProperties`，根据构造类型和 flags 初始化目标存储；[Class.cpp:1627–1669,1786–1822](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/Class.cpp#L1627-L1669>) 的二进制 tagged load 遍历**已存** tag，遇终止名结束，给已存在且获准的属性读值；[Class.cpp:1357–1367](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/Class.cpp#L1357-L1367>) 文本路径也先确认属性存在与准入。UE 并没有在这里为“缺 tag”发明一个归档值；未覆盖值来自对象已有初始化。此处只对照普通持久实例 tagged 路径，不把 PIE duplication、CDO/`IsSerializingDefaults`（`CPF_Transient` 的例外）、unversioned layout 扩成等价承诺。

## 4. Rust 做法

在当前 DynamicScene VM plugin restore 的构造候选阶段，从目标 schema 补齐“**缺席且 `serializable=false` 且有显式已验证 `default_value`**”的字段，再用原有严格 writer 原子预检。`World::dynamic_component_creation_template` 已读取真实声明默认值并用 `json_value_from_reflected` 转换（[creation.rs:95–134](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/dynamic_components/creation.rs#L95-L134)）；复用其底层转换/元数据，不借用 Inspector 编辑可见性或创建请求 ABI。缺失持久字段仍应由原有规则拒绝（这个子项不隐式迁移它）；无明确默认的非持久字段不猜测 `null` 或零。现有 stable ID 解析、报错、实体重映射、值预算与 world-generation 原子门槛保留。

## 5. ABI / ECS 差异

UE 的实体是已分配/初始化的 UObject/UStruct 存储，`FProperty` 给 serializer 定位成员；ZE 的 World 动态组件以 JSON 建行，`ReflectTypeRegistration`/稳定字段 ID 与 `[当前槽位]` 分开，VM 需要显式形成完整键集。只复制“目标默认已初始化 → 持久值覆盖”语义，不复制 UObject 字节布局、FArchive tag、序列化策略类型、默认对象装置或插件 ABI。此条无需给 DynamicScene 文档加新字段，也不改变普通 `World::set_dynamic_component` 直接写入严格性；Prefab 仍由 Unity 方向独立审查。

## 6. ZE 具体改法草案（先等价单测，再收敛实现）

1. **先补一个经生产入口的回归。** 参考下表的所属 Runtime 现有夹具，在源、目标 World 注册同一公开 VM schema；源 `saved=7`、`scratch=99`，`scratch` 默认 `42`。从 `DynamicScene::from_world` → `to_versioned_json_pretty` → `from_versioned_json` → `spawn_into`；先断言解码后的 `DynamicComponent.fields` 含 `saved` 的稳定 ID 且不含 `scratch` 的 ID（不要按整段文本搜索，schema 描述仍可含名称），再断言目标映射出的新实体 JSON 是 `saved=7, scratch=42`，目标其他行不动。当前预期 RED 在 `UnknownField(scratch)`；还需实际执行。加入小控制：直接交 `{saved:7}` 给 World 普通 VM writer 仍拒绝，存档缺持久字段也不能乱补。此条是 UE 源码推导的本地行为回归，不冒称直接移植测试。
2. **仅在 RED 后**，从 [compile_component_write:471–503](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/scene/spawn/transaction.rs#L471-L503) 的“目标存在 authoritative VM schema”分支组装完整候选，使用现有 [JSON 转换:80–122](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/reflect/dynamic_json.rs#L80-L122) 填充上述缺席默认值，然后进入原预检和 `set_dynamic_component`。方案不触碰 descriptor-only 的未知 VM schema 路径、不改变存档字段投影、不泛化默认引用的 remap 政策；复杂默认引用另做契约对照。当前只报告，未新增或运行单测，未改引擎。

### UE 原有测试迁移表

| UE 测试名 / 路径行 | UE 输入与期望 | ZE 既有测试位置；迁移边界 |
| --- | --- | --- |
| `CoreUObject::Serialization::TrackInitializedProperties` (`FTrackInitializedPropertiesTest`)；[PropertyStateTrackingTest.cpp:317–397](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Tests/UObject/PropertyStateTrackingTest.cpp#L317-L397>)、[文本分支:407–440](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Tests/UObject/PropertyStateTrackingTest.cpp#L407-L440>) | persistent tagged 存档后把目标属性设为 `7`；读取恢复 A[0]、A[2]、B，未写入值的 C/D 仍为 `7`。文本分支 C/D 仍有断言，部分数组断言被禁用。测试是 initialized/delta 省略，不是 `CPF_Transient` 直接断言。 | [hash_membership.rs:65–87](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/scene/capture/tests/hash_membership.rs#L65-L87) 的 `optimization_batch_20260826ai_runtime52_dense_field_filter_preserves_schema_order` 仅验证 `hidden` 投影被过滤、顺序保留；[creation.rs:46–79](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/tests/ecs_typed_api/component_registry/creation.rs#L46-L79) 的 `dynamic_creation_uses_only_real_declared_defaults_and_requires_remaining_values` 验证显式默认可作为创建值、无默认键不造值；二者未覆盖已有动态场景捕获→恢复。 |
| **未找到直接匹配；源码推导。** `FProperty::ShouldSerializeValue` + `UStruct::SerializeTaggedProperties` + 对象初始化，实现见第 1、3 段。 | 一普通实例有持久字段及瞬态字段；存储不含瞬态实际值，加载后持久值覆盖、瞬态保有正常构造值，不应只因瞬态项缺席而失败。 | [transaction/tests/cases.rs:407–480](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/scene/spawn/transaction/tests/cases.rs#L407-L480) 的 `compiled_spawn_projects_only_affected_target_schema_into_preflight` 证实已注册 VM 标志进入预检，但夹具无字段；[vm_type_backing.rs:16–55,99–134](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/reflect/tests/vm_type_backing.rs#L16-L134) 验证实时 VM 反射读写和类型不匹配拒绝，并非场景往返。第 6 段给出缺口。 |

查找范围：UE `Runtime/CoreUObject/Tests/{UObject,Serialization}` 和上一级对象/属性测试、`Runtime/Core/Tests/Serialization`、`Runtime/Engine/Private/Tests` 与 `Plugins/Tests` 的 C++ 测试；搜索 `ShouldSkipProperty`、`CPF_Transient`、`CPF_SkipSerialization`、`IsPersistent`、`SerializeTaggedProperties` 和测试宏。`SubobjectInstancingTest` 的 transient 构造与 duplication 不冒充持久往返。ZE/UE 编译、运行测试均 **0**；无 Cargo、引擎编辑、Git 写入或远端访问。**去重：** 本地 [PR 快照（仅本地审计记录，不随 PR 发布）](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>) #131–145 的标题与正文无此根因；[#133](https://github.com/He-Jiahui/ZirconEngine/pull/133) 的归档/对象对应是 Prefab 容器及元数据保留，不是 VM 瞬态缺省字段。对 capture、JSON 转换、spawn 事务、World 动态写入四个 owner 文件的 `git log --branches --not e893… --` 无未合提交；不是远端全量查重，不接管未提交工作。

### 模块与待核实清单

| UE 模块 | ZE owner | 读到的测试覆盖 | 待核实（不是候选） |
| --- | --- | --- | --- |
| `CoreUObject`：`FProperty` / `UStruct` / `UObject`、tagged serialization；`Core`：Archive 基础，[Core.Build.cs:8–20](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Core.Build.cs#L8-L20>) | `zircon_runtime_interface::{reflect,serialization}`；`zircon_runtime::scene::{reflect,dynamic_scene,world}` | tagged 读写、属性初始化测试，以及 ZE capture、schema fixture、事务预检 | 产品配置端到端、默认实体/资源引用语义、无默认处理、未知/重命名字段及 unversioned / binary / editor-only policy 不在本条。 |
| `CoreUObject` 对象初始化 / `Engine` 与编辑器调用者 | `zircon_editor::core::play`、`zircon_runtime::dynamic_api`、VM catalog | 已读 Play 捕获/装载接线；slot capture/apply 为同一持久链；未运行 | 不把所有 Editor Play 模式或缺失 VM 注册的纯空 World 恢复认作等价。数据表、SaveGame、网络复制及 Prefab 不在本条。 |

### UE 本地源文件 SHA-256

| 路径（相对本地 UE 根） | SHA-256 |
| --- | --- |
| [Engine/Build/Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| [Engine/Source/Runtime/Core/Core.Build.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Core.Build.cs>) | `2bae34201d978e95d778674f5293b80683c8844ba8963184cf306c20ab06e558` |
| [Engine/Source/Runtime/CoreUObject/CoreUObject.Build.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/CoreUObject.Build.cs>) | `cb5fab6f1b508e0ed8c2d712e24d097b91d317ed18a72f8cd36074bc763b20fe` |
| [Engine/Source/Runtime/CoreUObject/Private/UObject/Obj.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/Obj.cpp>) | `85d878573091575ce5e1709c9d66efec077bb085d7fcaba5f300b3a9c916f8c4` |
| [Engine/Source/Runtime/CoreUObject/Private/UObject/Class.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/Class.cpp>) | `ca56058fe946141abe03f4fa239ae603908e8f7d5b35a18b2887a8ef606edbdf` |
| [Engine/Source/Runtime/CoreUObject/Private/UObject/Property.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/Property.cpp>) | `356ae0aa811b501525132186b0dd7fad03a771fd8abbc89748da8f6ff4997631` |
| [Engine/Source/Runtime/CoreUObject/Private/UObject/UObjectGlobals.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/UObjectGlobals.cpp>) | `5cea26e1bb14ceb29edd6017e58b9c133f61eae07ac3058e1d054991fa4020da` |
| [Engine/Source/Runtime/CoreUObject/Tests/UObject/PropertyStateTrackingTest.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Tests/UObject/PropertyStateTrackingTest.cpp>) | `f740941e51b82994f4915742fd583bd70b987adf9bec492667080b1829bc769f` |

## 7. Windows 原生基线：存档已正确省略字段，恢复却拒绝整个实例

[测试提交](https://github.com/He-Jiahui/ZirconEngine/commit/ce0d141f1cdd173b56ca6198fe515082244fe9dc)只在[现有 VM 反射库测试](../../../zircon_runtime/src/scene/reflect/tests/vm_type_backing.rs#L16-L17)挂载了 [6 个用例](../../../zircon_runtime/src/scene/reflect/tests/vm_transient.rs#L14-L162)。原引擎与原错误类型未改。聚焦运行 **4 通过、2 失败、0 忽略、7775 过滤**，退出 101。两处失败不是同一种证据：

| 观察 | 原实现实测 |
|---|---|
| [版本化场景往返与恢复](../../../zircon_runtime/src/scene/reflect/tests/vm_transient.rs#L15-L50) | 源实例 `{saved:7,scratch:99}` 已写入；解码后，组件实例只含 `saved` 的稳定 ID 和值 `7`，描述符仍有 `scratch`。在同一 schema 已注册、另有哨兵实体的目标上，`spawn_into` 报 `WorldMutation(Reflect(UnknownField { type_path: "probe.Component.Scratch", field_name: "scratch" }))`。未到达目标值应为 `42` 的断言。**这就是本 PR 的行为缺陷。** |
| [补值必须计入有限目标预算](../../../zircon_runtime/src/scene/reflect/tests/vm_transient.rs#L114-L162) | 同一个目标、同样两个源实体，版本化文档分别缺 0、1、2 个 `scratch`；在相同 1 MiB 有限额度下，目标快照估算都是 **5814 字节**，递增断言失败。当前引擎还没有展开默认值；此案约束拟议修复不能免费展开，不是第二个 UE 缺陷，也不是内存耗尽复现。 |
| [严格写入及事务失败](../../../zircon_runtime/src/scene/reflect/tests/vm_transient.rs#L52-L96) | 直接给 World 写残缺 VM 对象仍报 `scratch` 缺失；第二行缺**有声明默认**的持久 `saved`，仍报 `saved` 缺失，两行均未发布；非持久字段无默认也仍拒绝。各自的原值哨兵保持不变。三案通过。 |
| [只有存档描述符的目标](../../../zircon_runtime/src/scene/reflect/tests/vm_transient.rs#L98-L112) | 没有注册 VM schema 时，同一类型的兼容路径可以写入 `{saved:7}`，没有推断或补出 `scratch`。通过。 |

**取得基线后的修复约束**（实现与结果见第 8 节）。 归档值继续由稳定 ID 定位并使用原实体映射。仅在隔离预检、[原严格 setter](../../../zircon_runtime/src/scene/world/dynamic_components.rs#L224-L240)之前，依据目标 World 的 VM backing 和实际字段表，给“缺席、非持久、有显式默认”三者同时成立的字段赋目标默认；不要改导出、普通写入或未注册描述符的行为。需先在[目标快照计量](../../../zircon_runtime/src/scene/dynamic/scene/spawn/transaction.rs#L264-L288)按每个实例计入要展开的字段，再交给现有[异步预约](../../../zircon_runtime/src/scene/dynamic/asset_reload/stage_task.rs#L98-L110)。这是既有估算口径，不是字节精确的堆或 RSS 上限。

测试选择命令（本地 wrapper 另指定工作树 manifest 与 F 盘产物目录）：

```text
cargo test -p zircon_runtime --no-default-features --features core-min,native-host --lib reflect163_ --locked --offline --jobs 1 -- --test-threads 1
```

Windows x64/MSVC，Rust 1.94.1，test profile `debug=0, incremental=false`；native-host 含 asset-types，未启用 Script/Graphics/UI。命令回执 SHA-256：`dc9879542650a3799ff58963cf70ea53ac76ad6a110b10763e85d7f56f541da7`。运行前后核对 34 项选定输入，不代表完整源码或进程封印。UE `TrackInitializedProperties` 的已有测试是 initialized/delta 条件下对**已有目标值**的观察，不是 `CPF_Transient` 或 ZE VM restore 的直接移植。未运行真实脚本后端、Play、全套 Runtime、UE 或跨平台产品验收。

## 8. 修复结果：存档仍然稀疏，目标实例恢复完整

[修复提交](https://github.com/He-Jiahui/ZirconEngine/commit/cab70397ffb6d8012ca1158a493aa23534a43cc4)只改了场景恢复的两处实现。源实例的 `scratch=99` 仍然不入档；[原来的同一组测试](../../../zircon_runtime/src/scene/reflect/tests/vm_transient.rs#L15-L50)现在读到新实体的 `{saved:7,scratch:42}`，源值和目标原实体的哨兵值均未改变。

- **在哪补值。** [隔离预检写入](../../../zircon_runtime/src/scene/dynamic/scene/spawn/transaction.rs#L715-L766)先确认规范类型路径确属目标 World 的 VM 动态组件。只给实际缺席、声明为非持久、且有显式默认的键补值，再交给原严格 writer。已存字段不覆盖，源实体引用仍按原路径重映射；目标默认不经源实体 remap。普通插件描述符不推断 VM 默认。没有修改导出、格式、ABI、ECS 发布顺序或模块 features。
- **何时计费。** [目标快照准备](../../../zircon_runtime/src/scene/dynamic/scene/spawn/transaction.rs#L274-L318)按每个待写实例使用同一筛选规则；[有界字节计数器](../../../zircon_runtime/src/scene/world/staging_snapshot.rs#L44-L78)以借用的字段名与声明值计算紧凑序列化字节数 ×2，不为计量先克隆/展开默认值。该增量进入已有目标快照估算，先通过预算检查，再在预检中构造实际 JSON。不同的充分额度（包括 `usize::MAX`）不会改变相同输入的估算。这是可执行的估算准入，不是对堆分配或 RSS 的精确界限。
- **第 7 条控制在另一个含 4 案的 Prepared 过滤组中运行。** [Prepared 测试](../../../zircon_runtime/src/scene/dynamic/spawn_task/tests/prepared.rs#L127-L219)使用字符串默认 `"target"`：缺省时必须有非零增量且 MAX 与 1 MiB 估算相同；若 Prepared 输入显式携带 `"stored"`，按无默认值的额度也能暂存并提交，实际读回仍是 `"stored"`（此补充案不再做版本化文本往返）。它并非原 `reflect163_` 六案之一。在含全部 7 案、仍为原引擎的[对照提交](https://github.com/He-Jiahui/ZirconEngine/commit/62808619ad8cc263d7cf19544168d09a5ace567c)中，该用例实际败于“重建必须占用目标预算”；候选通过。曾考虑按 MAX 略过计量，但审查时已去掉，该中间方案没有运行过。

| 原生库测试过滤范围 | 原实现（过 / 败 / 忽略） | 候选（过 / 败 / 忽略） |
|---|---:|---:|
| `reflect163_` 原 6 个用例 | 4 / 2 / 0 | **6 / 0 / 0** |
| `scene::dynamic_scene::spawn_task::prepared::tests::`（含第 7 案） | 2 / 2 / 0 | **3 / 1 / 0** |
| `scene::dynamic_scene::scene::spawn::transaction::tests::` | 23 / 1 / 1 | **23 / 1 / 1** |
| `scene::tests::dynamic_scene_asset_reload::byte_budgets::` | 3 / 1 / 1 | **3 / 1 / 1** |
| `scene::world::staging_snapshot::tests::` | 未单独复跑 | **1 / 0 / 0** |

使用第 7 节同一包、features、test profile、工具链、工作树和 F 盘目标目录，表中每个过滤器分别运行，不按筛选范围冒充完整 Runtime。候选与原实现的 3 个非零相邻组共 **34 个同名用例**；逐名对照，唯一状态变化是上述第 7 案失败转为通过。2 个既有忽略均同名、同原因。下列 3 个失败也分别在原实现重现，位置和断言内容相同：

1. [准备态过期目标](../../../zircon_runtime/src/scene/dynamic/spawn_task/tests/prepared.rs#L30-L47)与[编译后目标代际变化](../../../zircon_runtime/src/scene/dynamic/scene/spawn/transaction/tests/cases.rs#L109-L138)都拒绝了过期操作；实际错误是组件目录代际 `0 → 7`，断言要求 World 代际 `0 → 1`。两条都未改。
2. [热重载帧预算](../../../zircon_runtime/src/scene/tests/dynamic_scene_asset_reload/byte_budgets.rs#L222-L248)期望 `first_commit.apply.apply_budget_exhausted` 为真，两边都在第 239 行失败。同组[暂存预约按实际目标字节收缩并归零](../../../zircon_runtime/src/scene/tests/dynamic_scene_asset_reload/byte_budgets.rs#L253-L301)的独立控制在两边均通过；这并不证明所有异步调度都正确。

聚焦候选回执 SHA-256：`ef040496e8d1d8882cdb8e869d35e644add49d97ea4ed88f76307edc78a3c8c2`。3 组配对回执前缀（原实现 → 候选）：Prepared `af3703249215 → 6328032e9d8b`；事务 `46ad5268dce7 → b4a8b824060d`；队列 `6fd2909b417d → 01f60025bd3d`。目标修复和负控可独立重现，未屏蔽或修补相邻错误，仍保留 Draft；不据此外推整个引擎、UE 原测、真实 VM/Play、各平台及性能验收。

发布前核对了当时全部 65 个 open PR 的文件列表（含分页）。[#199](https://github.com/He-Jiahui/ZirconEngine/pull/199) 处理同一 World 内固定步物理写回与变换修订的关系，不是本问题的版本化存档恢复。相邻测试的 3 次相同失败也不自动算作 3 个新引擎缺陷；需先另行确认错误优先级和异步就绪约定。
