# [Scene / Prefab] 场景可以打开，但一条缺省值的 Prefab 覆盖会阻断普通保存

**阶段：独立调查，未改引擎、未运行 ZE 或 UE 回归。** ZE 固定 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。覆盖指实例上既有的属性记录，不是 Apply/Revert 动作；Prefab 仍保持 Unity 风格，不引入 Blueprint 对象模型。

## 1. 差异及其生产入口

[ZE 的 Prefab override DTO](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/authoring.rs#L242-L267) 要求 `entity_path` 和 `property_path`；单行 `value` 带默认值，整个覆盖集合也可以为空。这意味着“没有这行”与“这行存在、读入值为 JSON Null”并不是一件事。项目的[Prefab 元数据契约](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/prefab_tools/README.md#L16-L31)要求保留已有记录。[现有查询诊断](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/prefab_tools/editor/src/authoring/snapshot.rs#L334-L392)检查无源、空路径与重复键，不把 Null 当作删除。可用以下合法 TOML 片段，嵌入有真实 Prefab 源引用的场景；**不要写无效的 `value = null`**。

```toml
[[entities.prefab_instance.overrides]]
entity_path = "Root/Weapon"
property_path = "Joint.connected_entity"
# 不写 value
```

[正常 Open Scene](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/project/scene_document.rs#L206-L232)读取[物理场景源与其 revision](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/project_io/source_document.rs#L103-L129)，[正式 reader](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/project_document/scene.rs#L226-L260)保留整行，缺项在反序列化后成为 JSON Null。[World 安装](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/project_io/scene_asset.rs#L435-L452)与[作者快照回收](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/project_io/prefab.rs#L7-L23)均保存这份元数据。普通 File Save 的[动作入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/editor_event_execution/menu_action.rs#L75-L100)、[文档 owner](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/project/editor_project_document_save.rs#L14-L43)和[场景持久化事务](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/editor_document_transaction.rs#L255-L284)最终走同一正式文本 writer；不是用旁路序列化器构造故障。

## 2. 对用户的影响以及不应误报的后果

一个已经可以打开的现有工程，只要 Scene 保有上面的行，就可能连不相关的普通作者编辑也无法随 Scene 保存。源码预测：保存返回 `ProjectDocumentError::Serialize` / `unsupported unit type`；错误发生在准备文件写之前，**不是磁盘旧场景被擦掉或写坏**。对照是相同 Prefab UUID、实体、两条路径和变换，仅改变覆盖：无行、数值 `100`、字面字符串 `"null"` 和缺省 `value`。前三种应保存成功并各自保持语义；最后一种也应保有一行，不能退化为空集或字符串。

## 3. 根因、UE 处理与真实原测边界

[ZE 场景 writer](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/project_document/scene.rs#L263-L284)显式保护普通 component 的 JSON payload，却把 Prefab override 的 JSON Null 原样交给[第一次 TOML 值转换](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/project_document/codec.rs#L37-L52)。锁定的[serde_json 1.0.151](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/Cargo.lock#L6118-L6129) 中 `src/value/mod.rs:921–925` 给缺项 `Null`，`src/value/ser.rs:11–29` 将它提交为 `serialize_unit`；[toml 1.1.4+spec-1.1.0](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/Cargo.lock#L7427-L7440) 的 `src/value.rs:936–942` 拒绝 unit，`src/table.rs:397–429` 只会省掉 `UnsupportedNone`，不会吞掉此错误。这解释了为何正常读入与写回不对称。

UE 是本机提供的 `6.0.0 / UE5 / CL 0` 参考树，无可证的上游提交号。`FOptionalProperty::SerializeItem`（`Engine/Source/Runtime/CoreUObject/Private/UObject/PropertyOptional.cpp:174–221`，及 `Engine/Source/Runtime/CoreUObject/Private/UObject/PropertyOptional.cpp:307–311`）以“是否有值槽”决定读取/写入内值或恢复 unset。既有 `CoreUObject::NativeInstancedSubobjects` 的 `Unset optional default subobject initialized from template`（`Engine/Source/Runtime/CoreUObject/Tests/UObject/SubobjectInstancingTest.cpp:1656–1682`，注册 `Engine/Source/Runtime/CoreUObject/Tests/UObject/SubobjectInstancingTest.cpp:3201–3225`）实际分别断言**未设置**和**已设置但内部指针为 null**。这可作记录存在性判据，原测走对象实例化，不是磁盘 TOML。`FOptionalPropertyTestIdentical`（`Engine/Source/Runtime/CoreUObject/Tests/OptionalPropertyTest.cpp:157–196`）为邻近状态控制；没有找到相同的 UE 文件保存测试，本 PR 不称运行或直接移植过 UE 原测。

## 4. 可借用的 Rust 做法

将修复限定到项目 Scene 文本输出：已有 override 行顶层值为空时，保留行及两条路径，只省该行的 `value` 键；读回沿用已有默认规则。非空值仍用原来的原生 TOML 表示。可以使用私有 writer 行投影，不要在第一次 TOML 转换失败之后才清理，也不要在共享 JSON 类型上设置全局规则。即使 JSON 恰好将 `None` 和 unit 都写成 null，其他人类可读格式也未必如此；不应为此改变公开 Serde wire。

## 5. ZE 的接口、ECS 与 Prefab 边界

正常[项目引用持久化](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/persisted_reference.rs#L13-L103)将源 UUID 与路径提示关联；本例不更改源身份、实例变换、覆盖排序、World sidecar、插件 ABI 或组件删除语义。Null 是已加载的 opaque 值，不据此判定应清原属性、解除引用或恢复模板值。UE 的 UObject/CDO/归档器不进入 ZE。这里也不承诺所有嵌套 JSON Null 都能由 TOML 原样表示。

## 6. 同一 PR 上的等价回归与去重

先用[现有 Windows 项目 IO 夹具](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/project_io/tests/source_document.rs#L1-L69)建立注册的真实 `Hero` Prefab、`Root/Weapon` 与有连接目标的 Joint，四种变体均从原 Scene 源打开。依次验证源 UUID、行数/字段类型、路径和变换；对每项进行正常 Save、durable receipt 和两次重开，比较实体数据。未修版须在强前提通过后，只有缺省值案出现上述 typed error 且原文件未变，才算行为 RED。最终另用 Editor 正常动作验证脏状态、历史与 workspace；不能用 Runtime 通过替代 GUI 验收。

[既有正式 Scene codec 正控](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/project_document/tests/scene.rs#L119-L194)仅含非空值；[既有 World 保留测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/tests/asset_scene/hierarchy_sources.rs#L207-L287)和[二进制制品测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/tests/assets/artifact_store/prefab_overrides.rs#L32-L119)（后者已含嵌套 Null）均不走这一文本写入断点。已合 [#133](https://github.com/He-Jiahui/ZirconEngine/pull/133) 不重开，已有 [#157](https://github.com/He-Jiahui/ZirconEngine/pull/157) 场景后缀、[#163](https://github.com/He-Jiahui/ZirconEngine/pull/163) Script 默认值与 [#208](https://github.com/He-Jiahui/ZirconEngine/pull/208) 层级拖放也不同。下一阶段只在本 PR 加回归；未证之前不改生产。

本机参考指纹（SHA-256，仅引用路径与判据，不发布 Epic 源码）：

- `Engine/Source/Runtime/CoreUObject/Private/UObject/PropertyOptional.cpp`：`da60a3a406d12ff98867e4974ebfe5ab82568de3ec3fdd6c4934202087a42a90`。
- `Engine/Source/Runtime/CoreUObject/Tests/UObject/SubobjectInstancingTest.cpp`：`d1e32adb99792a360d10e5031747791cf4a148e2b39ddb8a6fbcaaa55cb7dd02`。
- `Engine/Source/Runtime/CoreUObject/Tests/OptionalPropertyTest.cpp`：`cee11a35f4d0e7756de2a1e1af9604d16f77abe7d1012bf085ca29d4c83e574d`。

## 7. 原生注册 Prefab、正式 Scene 源保存与重复重开

[仅测试 `e0709b78aa4c24cd0d885435d845bc8f01ca8d1d`](https://github.com/He-Jiahui/ZirconEngine/blob/e0709b78aa4c24cd0d885435d845bc8f01ca8d1d/zircon_runtime/src/scene/world/project_io/tests/prefab_save.rs)。真实注册 `Hero` 源 Prefab，验证 `Root/Weapon`、`Joint.connected_entity=100` 及项目解析的 UUID/path；普通 `World::load_scene_document` 读取小写 `.scene.toml` 的物理字节和 revision；`World::save_scene_to_project` 使用相同正式 writer 及受保护的文件事务。所有输入在测试专有临时目录，不改用户工程。Windows x64/MSVC，Rust 1.94.1，`test -p zircon_runtime --no-default-features --features native-host --lib prefab_override_save_ --locked --offline --jobs 1 -- --test-threads 1 --show-output`；**2 例，0 过 2 败**，7,775 过滤。

- **同一 Prefab，四种源表示。** 无覆盖行、数值 `100`、字符串 `"null"` 均通过真实的两次 durable 保存、两次从源重开及完整 Scene 比较。缺 `value` 的输入能打开，World 中仍有 1 行、值 `Some(Null)`，原 revision 匹配。到普通 Save 才返回 `project document TOML serialization failed: unsupported unit type`；在报告失败前已逐字节核实源文件与提交前相同。
- **混排的多个实例。** 三个实例分别有 `[3,0,2]` 条覆盖。读取并逐行查清数字、Null、字符串、另一个 Null 和 bool 的具体来源/顺序后，原版 Save 也在此报相同序列化错误；没有把一条 `None` fixture 直接塞入被测 writer。

这证明阻断保存，不是提交了损坏的新文件。未触发 Editor 菜单或测 dirty history，没有展开模板、应用覆盖、运行 UE 或声称所有 JSON 值都有 TOML 表示。

30 个选定源/清单；原生回执 `3e32e4037d52eb4a4dd6e2ddfe130ec41df4e138a9b4eb3521d478e6039df669`、stdout `521b9a195aa16521cdeb06c57c316c539cec3ba798a1ac4f4769e06f005a0a12`、stderr `9eb88004f54b5d0899ae2726321d145ef8c4019846fc8e2e04834dae7ace0025`。

## 8. 仅修项目 Scene 文本 writer：保行、保顺序，只省缺省值的键

[修复 `318b4b3ca9521a9c4fb228e7733f2ab520a56686`](https://github.com/He-Jiahui/ZirconEngine/blob/318b4b3ca9521a9c4fb228e7733f2ab520a56686/zircon_runtime/src/asset/assets/project_document/scene.rs) 相对修前证据只改一个生产文件；不改变 `PrefabPropertyOverrideAsset` 的公共 Serde wire。先让 Scene 的其它字段沿用既有编码，将含顶层 Null 的实例覆盖按原实体位置放回：仍是一条完整的路径记录，只省去该行的 `value`；非空行仍用原编码。不写字符串哨兵，不重新解释为 Apply、Clear 或删除，不改变持久引用、ECS sidecar 或文件事务。

原样复跑 §7 的两个用例：**0 过 2 败 → 2 过 0 败**，修前修后 `prefab_override_save_` 命令一致。实际读写/两轮重开四种源表示皆保持原义；目标场景落盘确实有该行且无 `value` 键，源 revision 对应新文件，不可能靠 no-op Save 过例。三个实例的 `[3,0,2]` 条混合记录在磁盘上按原值有无、路径与顺序核过，重开后完整 DTO 相同。

测试叶工作字节 SHA-256 `44e7984287131145f0e3d624a54c227f236d34e49fbbd7a150a9f6752b0746b2` 未变。旧回执 `3e32e4037d52eb4a4dd6e2ddfe130ec41df4e138a9b4eb3521d478e6039df669`。这不是 Editor 窗口、交互/dirty、完整 Prefab 展开、嵌套数组内 Null 或 UE 运行验收。

30 个选定源/清单；原生回执 `e4e8ca23b7f6d66e15fcddf7a367de5b4d12309a8ac58cd0fbb864e206c8650f`、stdout `cf34da152e4c55757c86e809bb1cad372e6751acb01afa42e54a56719ce3944a`、stderr `7d22b4d32d79969cd470d1942f37e76bd0e530e00ad4378fab631789d1595999`。
