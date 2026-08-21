---
related_code:
  - zircon_reflect_derive/src/attributes.rs
  - zircon_reflect_derive/src/derive.rs
  - zircon_reflect_derive/src/fields.rs
  - zircon_runtime/reflection_macros/src/args.rs
  - zircon_runtime/reflection_macros/src/attrs.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/reflection_macros/src/function.rs
  - zircon_runtime/reflection_macros/src/module.rs
  - zircon_runtime_interface/src/reflect
  - zircon_runtime/src/core/framework/script
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/plugin/state_migration.rs
tests:
  - zircon_reflect_derive/src/tests.rs
  - zircon_runtime/reflection_macros/src/tests.rs
  - zircon_runtime/src/scene/reflect/derived/tests.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/engine-architecture/generated-code-boundary.md
reference_engines:
  - dev/bevy/crates/bevy_reflect/derive
  - dev/bevy/crates/bevy_reflect/compile_fail
  - dev/bevy/crates/bevy_reflect/src
  - dev/Fyrox/fyrox-core/src/reflect
  - dev/UnrealEngine/Engine/Source/Programs/Shared/EpicGames.UHT
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/ObjectMacros.h
  - dev/godot/core/object/class_db.h
  - dev/godot/core/object/class_db.cpp
  - dev/godot/core/object/property_info.h
  - dev/godot/core/object/method_info.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderGenerator
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/ShaderGenerator/ShaderGeneratorAttributes.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 04 · Reflection Derive、Script Host Macro、Schema 与 Codegen 工程化差距

## 1. 结论

当前反射并非空壳。`zircon_runtime_interface::reflect` 已有类型登记、字段元数据、读写错误、编辑器提示、序列化策略、脚本/远程可见性和多种 `ReflectedValue`；`ZrReflect` 派生已接入 15 个内建场景类型，derived component adapter 会先 clone、批量修改再一次性回写；VM registration 又对plugin identity、type path前缀和声明值类型做了额外校验。脚本宿主侧也已能从两个内建类型和两个函数生成descriptor/export wrapper。这些底座应保留。

但当前不能称为工程级统一反射或稳定代码生成系统。仓内实际有两套互不共享IR的proc macro：`zircon_reflect_derive`生成场景/编辑器登记和slot读写，`zircon_runtime_reflection_macros`另行生成脚本type/function/module描述；生产math module却又绕过`zircon_host_module`手工拼装descriptor。三条authority对同一概念使用不同type path、value kind、serialization、visibility和验证规则，无法证明它们彼此一致。

最直接的合同错误是`ZrReflect`把任意`Vec<T>`推断成裸`"List"`，而运行时`DeclaredValueType`只接受`List<T>`；显式`value_type_path`、脚本`type_name/value_kind`又都可与真实Rust类型无约束地分离。枚举在两套derive中都只产生`Enum + 空字段/空variant`；生产`Mobility`不得不手写虚拟`kind`读写，仍无法向Editor发布enum options。类型/字段/variant也没有稳定ID、schema version、definition hash、alias或通用migration，所以模块移动和重命名没有可验证兼容路径。

测试只能证明token拼接的少数happy path。两个proc-macro包分别有7和11个单元测试且当前均通过，但没有trybuild/compile-fail、依赖重命名、下游crate、schema golden或兼容diff。完整runtime consumer聚焦测试通过managed validator启动后，在执行测试前被当前runtime lib-test的326个无关编译错误阻断，不能据此宣称端到端宏合同通过或失败。

本轮记录4个P0、42个P1和8个P2。没有修改Rust、Cargo、反射schema、场景格式或脚本ABI；只新增审查记录并更新索引。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| `zircon_reflect_derive` | 5个Rust文件 / 870行，加Cargo manifest | E3：attribute parser、field model、所有expansion与7个测试逐文件读取 |
| `zircon_runtime/reflection_macros` | 8个Rust文件 / 848行，加Cargo manifest | E3：type/function/module三类宏与11个测试逐文件读取 |
| interface/runtime reflection与生产consumer | 合并clean scoped set共89 tracked文件 / 11,472行 | E2-E3：DTO、registry、derived adapter、dynamic scene、内建组件、script descriptor/migration/host module纵向追踪 |
| reference reflection/codegen | Bevy/Fyrox/UHT/Godot ClassDB/Unity ShaderGenerator | E2-E3：只采用本地源码可验证的责任与合同，不推断闭源Unity Editor对象系统 |

上述89文件的`git status --short -- <scoped files>`为0项，内容指纹为`b10d341b4efe4d70027fa5bd2ef63cb5a5778b25a1bcb23d418a28a07a6205a3`。实施前必须重取指纹并重读consumer，因为本轮只对clean source作稳定结论。

### 2.2 动态验证

两个独立proc-macro包测试：

```powershell
cargo test -p zircon_reflect_derive --locked
cargo test -p zircon_runtime_reflection_macros --locked
```

结果分别为7 passed、0 failed和11 passed、0 failed。初次运行使用的三个临时target目录已被Session Coordinator按managed artifact策略清理，不构成源码变更。

完整consumer通过Windows managed validator尝试：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 `
  -Package zircon_runtime `
  -SkipBuild `
  -LibTests `
  -TestFilter rust_reflection_macros_generate_type_function_and_module_descriptors
```

该命令耗时545.6秒、exit 101，测试未开始。编译阶段报告326 errors / 1,456 warnings，首批错误来自graphics缺失test module、text layout符号、cubemap irradiance helper、post-process settings和dynamic-resolution imports，均不在本轮clean scope。结论只能是“完整consumer验证被当前runtime基线阻断”，不能归因于宏修改。

### 2.3 参考边界

- Bevy：derive通过`BevyManifest`解析被重命名的crate路径；typed derive data保留实际field type、generics和enum variants；serialization data显式映射skip field；`bevy_reflect/compile_fail`有28个Rust入口/用例文件覆盖generics、bounds、lifetimes、remote wrapper、函数参数与返回失败。
- Fyrox：`TypeInfo`包含source/type/assembly/documentation/type UUID，`FieldMetadata`包含display/read-only/collection/min/max/step/precision；反射支持enum variant、集合、map与递归field path。这里借鉴稳定身份和property model，不宣称其拥有Zircon所需的全部热更迁移。
- Unreal：UHT是155文件的独立Tokenizer/Parser/Specifier/Table/Type/Exporter程序，生成definition/body hash和reload version info，并把deprecated alias、replication、SaveGame等specifier纳入集中语义。Zircon应借鉴typed IR与兼容治理，不照搬C++宏表面。
- Godot：ClassDB集中维护method/property/compatibility map，拒绝重复绑定，计算包含method/property/default/hint/signal的API hash，并按signature hash查compatibility method；PropertyInfo有storage/editor/internal/group/read-only等usage flags。
- Unity Graphics：`CSharpToHLSL`先排序generator，任一type生成失败则跳过整文件，显式拒绝unsupported field/HLSLArray，成功后写带不可手改标记的稳定HLSL。它只作为typed codegen、错误与产物治理参考，不作为通用对象反射参考。

## 3. 当前 P0

### TOOL-REFLECT-P0-001 · 三条reflection authority没有共享schema IR

`ZrReflect`在`derive.rs:82-166`生成registration和字段名/slot读写；`ZirconScriptType`在`derive_type.rs:56-85`重新生成另一份registration/projection；生产math module在`builtin_host_modules.rs:337-399`又手工组合type/function descriptor。两套宏没有共享parser、type model、validation或schema ID，`zircon_host_module`也没有production caller。Editor、scene、VM和host ABI因此无法从一个权威证明同名类型等价。

重构必须先建立手写`ReflectionSchemaIr` owner：Rust derive只把声明lower到IR，统一validator检查identity/type/access/policy，scene registration、script projection、docs/catalog和thin adapter分别从同一IR投影。生成物不得拥有registry lifecycle。

### TOOL-REFLECT-P0-002 · 声明值类型与真实Rust/wire类型可直接矛盾

`fields.rs:264-281`只看路径最后一段，把`Vec<T>`无条件写为`"List"`且丢弃T；`DeclaredValueType::parse`在`declared_value_type.rs:51-65`只接受`List<T>`，所以该derive结果无法通过VM schema parser。显式`value_type_path`不校验实际`ZrReflectValue`输出；`zircon_script(type_name/value_kind)`和host return override也可伪造type ref。普通native registration在`type_registry.rs:380-423`甚至不parse全部value type，只校验非空和default value。

需要从Rust `syn::Type`构造typed `ValueTypeIr`，容器递归保留item/key/value，所有override必须是受约束alias或通过显式converter证明。IR验证后才能发出字符串codec；禁止先生成自由字符串再由不同consumer猜测。

### TOOL-REFLECT-P0-003 · 没有schema身份、兼容hash与通用迁移合同

默认type path来自`module_path!() + ident`，字段身份就是显示/序列化名称，enum variant无模型；registration没有schema ID/version/hash/alias/deprecation。scene JSON虽然有document migration，VM state也另有手写`schema_version + renames`，但两者都没有绑定derive生成的type/field identity。模块移动、类型/字段重命名、插件热更和旧场景读取无法在登记阶段比较兼容性。

应增加稳定type UUID、field/variant ID、schema revision、definition hash、name aliases和兼容等级；catalog登记时执行old/new schema diff并选择compatible、migration-required或reject。scene document version与type schema version必须分层但可关联，不能互相替代。

### TOOL-REFLECT-P0-004 · 两套derive都把enum降成空壳

`ZrReflect`在`derive.rs:38-41`把所有enum变成`Enum + Vec::new()`；`ZirconScriptType`在`derive_type.rs:37-45`同样不生成variant。生产`Mobility`只能在type attribute中手写虚拟`kind`和read/write转换，registration仍没有enum options、variant fields、discriminant、alias或unknown-variant策略。任何新enum若未手工补洞都会“成功derive”但没有可用反射表面。

统一IR必须建模unit/tuple/struct variants、稳定variant ID、tagging/serialization策略、default/unknown/deprecated alias和当前variant访问；Editor、scene、script ABI再按能力投影，而不是让每个enum作者手工复制转换表。

## 4. Authority、生成边界与注册治理差距

### TOOL-REFLECT-P1-001 · 缺少parse → typed IR → validate → emit分层

当前两个crate都在attribute解析后直接拼`quote!`，类型语义散落在`fields.rs`、`tokens.rs`和consumer。应把诊断收集、canonicalization、capability matrix与emitter拆开，使多个错误可一次报告，emitter只能消费已验证IR。

### TOOL-REFLECT-P1-002 · `zircon_host_module`生成注册行为违反leaf codegen边界

`module.rs:79-85`直接生成调用`HostExportRegistry::register_module`的函数。仓内规范明确生成代码只能是data/table/manifest/schema/thin adapter，不能拥有plugin/module registration。宏应只生成静态descriptor/export table，手写module owner负责时序、capability admission、rollback与handle生命周期。

### TOOL-REFLECT-P1-003 · 两套宏硬编码依赖crate绝对名称

`ZrReflect`展开固定引用`::zircon_runtime_interface`，script宏固定引用`::zircon_runtime`。依赖重命名、re-export或SDK facade会使下游展开失败。应像Bevy manifest resolver一样解析Cargo依赖名，或允许受验证的`crate_path`参数并用downstream compile test锁定。

### TOOL-REFLECT-P1-004 · production host module没有采用module macro

全production只有两个`zircon_host_function`和两个`ZirconScriptType`使用点，math module仍手工`.with_type/.with_function`；`zircon_host_module`只在`reflection_docs`测试中使用。这意味着宏的module发现、注册和capability组合从未承担产品authority，应在收敛设计前禁止按“已接入”计完成度。

### TOOL-REFLECT-P1-005 · 宏crate和公共contract的owner命名不清

场景derive是顶层`zircon_reflect_derive`，脚本derive却位于`zircon_runtime/reflection_macros`并导出为`zircon_runtime_reflection_macros`。同属reflection schema的parser/model散落在runtime内部和workspace顶层。应把canonical IR/validator放在不依赖runtime实现的schema crate，proc-macro只依赖schema前端，runtime/interface各自消费稳定DTO。

### TOOL-REFLECT-P1-006 · 没有可审计schema catalog artifact

当前schema只在进程启动调用derive函数时构造，CI无法在不运行runtime时列出所有type/function/field、比较definition hash或发现删除/重命名。应产出确定性catalog manifest，包含producer/toolchain/schema版本与来源位置，并由CI执行diff policy。

### TOOL-REFLECT-P1-007 · scene/editor/script/remote capability矩阵没有单一来源

同一registration含serializable、editor_visible、remote_visible、script_visibility、component/resource/plugin flags，但不同宏只填其中子集，consumer又分别解释。IR validator必须定义合法组合，例如remote公开是否要求stable wire type、script public是否要求host projection、resource是否允许entity reference。

### TOOL-REFLECT-P1-008 · native和VM registration验证强度不一致

VM路径检查canonical text、plugin ownership、plugin ID一致性、prefix、全部field declared type和component-only；普通`register`只检查component/resource冲突、field name/type path非空/唯一与default type。应共享核心schema validator，再由VM/native policy追加约束，避免同一registration因入口不同得到相反结论。

### TOOL-REFLECT-P1-009 · derived adapter只验证component flag，不验证accessor/schema一致性

`finish_component_registration`只确认component-only，然后装配slot/name读写函数。应在adapter materialization前逐字段验证metadata type、read result、write conversion、editable/accessor、batch semantics，并在debug/test catalog中探测default instance或converter witness。

### TOOL-REFLECT-P1-010 · 内建登记和特殊字段继续手工重复

`builtin_reflection/registration.rs`手工列出每个derived component，Hierarchy/ActiveInHierarchy另装特殊adapter，Mobility再手写虚拟enum字段。需要显式手写owner表，但表项应消费生成的schema descriptor，并让catalog test验证所有声明恰好登记一次，而不是用source-string测试维持重复列表。

## 5. Schema、类型与字段模型差距

### TOOL-REFLECT-P1-011 · 默认type path与源码模块布局耦合

两套derive都使用`module_path!()`作为默认持久身份。重构module会静默产生新类型，旧场景/插件state成为unknown type。可持久、远程或插件类型必须显式稳定ID；Rust path只保留为debug/source identity。

### TOOL-REFLECT-P1-012 · `ReflectTypePath`只有字符串，不拥有alias/canonical revision

构造器只做非空验证，短名歧义由registry另行维护，没有old path alias、canonical ID、source owner或revision。应把display name、Rust path、wire name、stable ID和aliases拆成不同字段，禁止一个字符串同时承担所有语义。

### TOOL-REFLECT-P1-013 · `plugin_owned`可由derive生成但不能生成`plugin_id`

`ReflectTypeRegistration`和VM validator要求plugin ID，`ZrReflect`attribute却只有boolean `plugin_owned`。任何派生插件VM类型都会在登记时缺ID，迫使外部二次修改registration。插件identity应由package context注入并进入stable type ID，而不是让type作者写自由字符串。

### TOOL-REFLECT-P1-014 · script宏接受空白name/version/capability/documentation

host args parser只检查literal类型，不trim或拒绝空值；type/field attrs同样可产生空字段名和空type name。descriptor builder可能后续才失败或发布无意义metadata。canonical text、SemVer、capability ID和documentation budget应在macro span处诊断。

### TOOL-REFLECT-P1-015 · 重复attribute静默last-write-wins

多个`zr_reflect/zircon_script`属性会累积，重复`type_path/name/value_kind/doc`覆盖旧值；host function/module的重复key同样覆盖，capability才追加后dedup。应像Bevy container attribute parser一样区分repeatable与single-use key，并把重复/冲突同时指向原始和重复span。

### TOOL-REFLECT-P1-016 · 冲突组合验证不足

当前只显式拒绝component+resource、readonly+write和部分virtual accessor缺失；`skip`仍可与name/type/accessor混用，serializable可与serialization None矛盾，plugin/public/remote策略也未联动。需要声明式constraint table和compile-fail覆盖。

### TOOL-REFLECT-P1-017 · 所有generics被一刀切拒绝

两个derive都在发现任意generic parameter时失败，无法表达`Handle<T>`、typed resource、generic collection wrapper或const-sized类型。应像Bevy那样为active field types生成精确bounds，并显式区分支持的type/lifetime/const generic，而不是永久禁止整个类别。

### TOOL-REFLECT-P1-018 · 类型推断只看最后一个path segment

任意`foo::String`都会被当作标准String，任意同名`Vec3`也会被当作数学向量；generic arguments完全不参与判定。应基于resolved trait/type witness而非标识符拼写，宏无法解析类型时应要求typed adapter而非猜测。

### TOOL-REFLECT-P1-019 · derive、`ZrReflectValue`与`ReflectedValue`能力矩阵分裂

derive推断只覆盖bool、固定整数、f32、String、Vec2/3/4、EntityId、Vec；`ZrReflectValue`另有`Option<u64>`，DTO/parser还有Enum、Quaternion、Resource、Map、Json。新增variant必须由一张capability matrix生成converter、schema type、editor hint、serialization和测试，不能各文件手工补齐。

### TOOL-REFLECT-P1-020 · nested reflected object、array、map和typed resource没有组合模型

没有`Struct<T>`、`Option<T>`、`Array<T,N>`、`Map<String,T>`、resource/class reference与polymorphic handle的统一描述；用户只能把它们压成Json或手写virtual field。需要递归`ValueTypeIr`和明确ownership/nullability/collection mutability策略。

### TOOL-REFLECT-P1-021 · 自定义converter没有类型证明和错误边界metadata

`read/write`接收任意path，schema只存自由`value_type_path`。应生成converter descriptor，声明source Rust type、wire type、lossless/lossy、validation、thread/side-effect属性，并用compile-time trait约束签名；写入错误应携带稳定diagnostic code。

### TOOL-REFLECT-P1-022 · `ReflectFieldInfo`已有丰富字段但derive无法声明

DTO已有default value、numeric range、enum options、documentation，derive只发name/type/hint/editable/serializable/editor visible。Editor 05已证明consumer也未完整使用这些字段。attribute/IR必须支持并类型校验default/range/step/precision/options/docs/category/order/units/asset class，而不是继续加不受约束字符串。

### TOOL-REFLECT-P1-023 · field和container的serialization语义可能矛盾

字段默认`serializable=true`，container的serializable又由serialization strategy推断或覆写；registry不验证`strategy=None + serializable=true`、不可读字段被标serializable等组合。序列化能力应由codec存在性派生，field skip/default/rename与type strategy共同验证。

### TOOL-REFLECT-P1-024 · 可写字段隐式要求`PartialEq`

生成writer在`fields.rs:139/178`无条件执行`self.field == next`，但derive声明没有显式bound或attribute说明。合法converter类型若不实现`PartialEq`会在展开后得到难懂错误，大集合还被迫做昂贵比较。应使用显式change policy：always-write、typed equality、epsilon、identity或custom comparator。

### TOOL-REFLECT-P1-025 · dense slot是隐式源码顺序而无schema绑定

physical+virtual字段按当前顺序枚举成`u32` slot，derived batch adapter直接消费。dynamic scene会先按字段名映射到当前slot，所以不能夸大为现有持久文件必然损坏；但slot仍是热路径内部ABI，插入/skip/重排会改变profiling、staged request和跨generation假设。应由validated schema分配generation-bound slot table，禁止slot跨catalog generation存活。

## 6. Script Host type/function/module 宏差距

### TOOL-REFLECT-P1-026 · `ZirconScriptType`默认`Null`且override可脱离Rust类型

未指定type `value_kind`时默认Null；field和return kind可指定任意path，编译器不证明与`ScriptHostFromArgument/IntoValue`一致。应从统一wire type派生ABI kind，只有注册过的converter可覆写。

### TOOL-REFLECT-P1-027 · script type registration硬编码policy

所有类型固定serialization None、serializable false、editor hidden、script public，无法表达可持久脚本值、私有helper、remote-safe DTO或共享Editor schema。这些policy应来自统一schema capability，host projection只是consumer之一。

### TOOL-REFLECT-P1-028 · type path、type name与host prototype身份分裂

registration full path固定Rust module+ident，short/display name可由`name`重写，host type ref又采用display name和value kind。rename可能只改变部分身份，registry/VM/docs看到不同类型。必须以stable ID连接Rust source、wire name和display alias。

### TOOL-REFLECT-P1-029 · host function只支持同步精确arity自由函数

宏拒绝async、generic、method和destructuring，min/max arity都写成参数数目；没有optional/default/variadic、receiver、host context、cancellation或异步completion。应先定义稳定HostCallable ABI和执行policy，再让宏生成薄adapter。

### TOOL-REFLECT-P1-030 · host function不能自然返回typed domain error

生成调用把Rust返回值直接交给`ScriptHostIntoValue`，该trait返回plain value；`Result<T,E>`没有转换合同。业务函数要么不能用宏，要么把错误编码成普通值。应支持`Result<T, ScriptHostError/E>`、stable error code、stack/source context和panic containment。

### TOOL-REFLECT-P1-031 · float返回可直接发布NaN/Infinity

`ScriptHostIntoValue for f32/f64`不做finite验证，而反射向量转换已有finite检查。脚本ABI应统一numeric policy，在参数、返回、serialization和remote边界使用同一validator，并允许域明确选择IEEE special值策略。

### TOOL-REFLECT-P1-032 · 参数没有独立schema

参数名只能取Rust identifier，没有rename、documentation、default、range、units、nullability、direction、capability或deprecation；return metadata也只有type ref/docs。应把callable parameter/return纳入schema IR和signature hash。

### TOOL-REFLECT-P1-033 · `ScriptHostValue`自身的type ref固定为Null

`ScriptHostIntoValue for ScriptHostValue`返回Null type ref，不检查实际variant；使用动态返回值时descriptor会宣称Null，除非作者再手写override。动态/union value必须有显式Variant/Any wire kind与运行时tag policy，不能伪装成Null。

### TOOL-REFLECT-P1-034 · function缺少稳定ID、signature hash与compatibility overload

descriptor只有name、arity、type refs和capability，没有function ID、revision、deprecated aliases或兼容signature列表。脚本bytecode/远程调用/热更无法区分可兼容扩展与breaking change。应借鉴Godot compatibility hash建立确定性签名身份。

### TOOL-REFLECT-P1-035 · module scanner只看inline直接子项且会按末段拼写误认宏

scanner不处理nested module/re-export；任意attribute末段叫`zircon_host_function`或derive末段叫`ZirconScriptType`都会被视为本宏产物，随后调用并不存在的私有helper。应由宏自身生成显式inventory marker/type-level descriptor，不通过token拼写猜测另一个宏是否运行。

### TOOL-REFLECT-P1-036 · module identity/version/capability缺少验证与兼容策略

module name仅要求存在，version默认`0.1.0`但不验证SemVer，capability允许空字符串；没有engine API range、module stable ID、dependency、signature hash或upgrade policy。module descriptor必须进入package/catalog authority，由手写loader执行admission与rollback。

## 7. 诊断、测试与运维差距

### TOOL-REFLECT-P1-037 · 测试以token字符串shape为主，没有compile-pass/fail矩阵

两个macro crate共18个`#[test]`，主要断言展开字符串包含片段；没有trybuild或等价fixture。应覆盖每个合法/非法attribute组合、span、generic bound、enum形态、converter签名、module inventory和host return error。

### TOOL-REFLECT-P1-038 · 没有真实下游crate与依赖重命名测试

当前测试直接调用内部implementation函数，无法发现hardcoded crate path、visibility、re-export、edition/no_std feature或多个macro组合后的编译错误。至少需要SDK consumer fixture和renamed dependency fixture。

### TOOL-REFLECT-P1-039 · parser没有重复/空白/畸形输入与fuzz防线

host key-value parser在逗号处只“可选消费”，duplicate key静默覆盖；attribute parser也无冲突矩阵。应增加proptest/fuzz或系统化table tests，限制attribute/documentation/field数量和嵌套深度，保证诊断而非panic/超时。

### TOOL-REFLECT-P1-040 · 没有schema golden、definition hash或breaking diff test

source token不等于产品schema。CI需要把全部first-party catalog导出为canonical JSON/binary，golden测试验证稳定排序/hash，breaking diff必须附migration或显式major revision。

### TOOL-REFLECT-P1-041 · 完整runtime consumer基线当前不可执行

managed聚焦测试在测试运行前被326个无关runtime编译错误阻断。应先恢复可编译的reflection consumer shard，使macro/IR/registry/scene/script不必等待整个graphics-heavy lib test；同时保留完整workspace/nightly作为跨域门。

### TOOL-REFLECT-P1-042 · 没有编译成本、catalog规模和安全预算

未测proc-macro expansion时间、incremental invalidation、catalog startup、10k type/field lookup、large enum、错误风暴或untrusted plugin schema。应建立时间/内存/诊断数量/字符串长度/字段数量预算，并让remote/script可见schema通过更严格安全profile。

## 8. P2 与长期完善项

### TOOL-REFLECT-P2-001 · Rust doc comment没有自动进入schema documentation

当前必须在自定义attribute里重复文档。IR frontend可受控提取`#[doc]`并保留source location，但应有长度、本地化和公开API过滤策略。

### TOOL-REFLECT-P2-002 · 缺少`zircon schema inspect`只读工具

需要按type/function/module查询stable ID、hash、fields、policy、source owner、aliases和consumer projection，供Editor、CI、plugin作者与支持人员共用。

### TOOL-REFLECT-P2-003 · 缺少可视化schema diff与迁移预览

工具应把added/removed/renamed/type-changed/policy-changed分类并解释scene、script、remote、Editor影响，而不是只显示JSON文本差异。

### TOOL-REFLECT-P2-004 · 缺少proc-macro expansion和增量编译趋势

应记录冷/热build、单字段修改fan-out、catalog generation与downstream monomorphization，防止反射能力扩展后拖垮编辑迭代。

### TOOL-REFLECT-P2-005 · 诊断没有稳定code和remediation链接

当前都是英文自由字符串。长期应由validator产生稳定code、primary/secondary span、owner、兼容等级和修复建议，供IDE与CI结构化消费。

### TOOL-REFLECT-P2-006 · 多语言binding尚无统一schema源

未来C#/Lua/JS/remote inspector应从同一validated IR生成codec与stubs，不能各自重新解释Rust attribute；在核心schema稳定前不应提前铺多套generator。

### TOOL-REFLECT-P2-007 · 缺少reference engine schema regression corpus

可建立不含其实现代码的行为fixture：generic struct、data enum、deprecated rename、method overload/default、nested collection、read-only/range/property usage，用来防止Zircon IR长期退化。

### TOOL-REFLECT-P2-008 · 缺少长期反射热路径性能基线

应覆盖name/slot lookup、批量读写、scene capture/spawn、Editor 10k属性、VM catalog reload和schema diff，并将slot优化与正确性身份分开测量。

## 9. 重构路线

### M0 · 修复合同错误并建立可运行consumer shard

1. 删除`Vec<T> -> "List"`推断，先以compile error阻断，再实现递归`List<T>`。
2. 普通和VM registration共享declared type parser与核心validator。
3. 增加trybuild：Vec、lying override、duplicate/blank/conflict、enum和renamed dependency。
4. 建立不依赖graphics全量lib-test的reflection integration target，恢复端到端可执行门。

### M1 · 建立canonical `ReflectionSchemaIr`

1. 定义Type/Field/Variant/Callable/Module/ValueType IR与source spans。
2. 两套attribute frontend只负责lowering；共享validator返回多诊断。
3. scene、Editor、script、remote projection从同一validated IR生成leaf descriptor/table。
4. 手写registry/module owner消费table，删除宏生成注册生命周期行为。

### M2 · 稳定身份、hash与兼容治理

1. 引入type UUID、field/variant/function ID、schema revision和definition/signature hash。
2. 分离Rust path、wire name、display name、aliases和plugin/package owner。
3. 生成canonical catalog artifact和CI schema diff。
4. 将scene type migration、VM state rename和hot reload admission连接到同一identity，但保留各自数据格式版本。

### M3 · 完整类型/property模型

1. 支持data enum、nested reflected type、Option/List/Array/Map/resource/reference和受控generic bounds。
2. 从trait witness生成wire type，override只能选择注册converter。
3. 将default/range/step/precision/options/docs/usage/category/order纳入typed field IR。
4. 定义change comparison、collection mutation与unknown value/variant策略。

### M4 · Script Host ABI收敛

1. 定义HostCallable ABI：typed params/return/error/context/cancel/async/optional/default/variadic。
2. 为function/module生成stable ID、signature hash、compatibility aliases和capability policy。
3. module macro只生成inventory table，loader负责admission/register/rollback/handle。
4. 迁移所有builtin module，删除manual descriptor与macro descriptor双轨。

### M5 · Codegen与开发者工具

1. 提供schema inspect/diff/lint和IDE diagnostics。
2. 生成确定性docs、catalog与必要language bindings，全部带producer/schema/toolchain fingerprint。
3. 建立incremental generation cache和stale artifact gate。
4. 用reference behavior corpus和first-party golden catalog锁定兼容性。

### M6 · 规模、安全与运维

1. 建立10k type/field、large enum、scene spawn、VM reload和Editor property性能预算。
2. 对untrusted plugin/remote schema做长度、深度、数量、capability和wire type限制。
3. 增加fuzz、fault injection、catalog corruption、hash collision simulation和rollback测试。
4. 将schema/API hash、migration与兼容拒绝纳入release receipt和crash diagnostics。

## 10. 验收门

1. 同一Rust声明只lower一次，scene/editor/script/remote投影共享相同type/field stable ID。
2. 任意`Vec<T>`生成递归`List<T>`，nested list/map/option的parser、converter和schema完全一致。
3. 显式wire type/value kind与真实converter不一致时在声明span编译失败。
4. unit/tuple/struct enum variants均有stable ID、字段schema、序列化tag和Editor options。
5. 移动Rust module不改变显式stable type ID；旧path alias可读取但新写只用canonical name。
6. 字段/variant重命名必须提供alias或migration；未声明breaking change在CI schema diff失败。
7. plugin-owned类型从package context获得plugin ID，type registration与type path identity始终一致。
8. native和VM registration共享核心validation，同一非法schema不会因入口不同而通过。
9. `serialization=None + serializable=true`、editable无writer、remote不安全type等冲突全部前置失败。
10. generics生成最小精确bounds；支持/不支持用例均有compile-pass/fail fixture。
11. dependency重命名、SDK re-export和外部plugin fixture能成功使用derive。
12. enum、container、converter和attribute重复/冲突的trybuild diagnostics稳定且指向正确span。
13. schema catalog输出确定性；同source/toolchain两次生成字节和definition hash相同。
14. first-party catalog breaking diff必须附migration receipt或显式major revision。
15. dense slot只在同catalog generation内有效；跨generation request被拒绝或按stable field ID重映射。
16. host function支持typed `Result`，panic被隔离，NaN/Infinity按统一numeric policy处理。
17. optional/default/variadic/async/cancel/context的ABI语义有真实VM调用测试。
18. function signature hash覆盖参数顺序/type/default/return/capability，兼容alias按policy解析。
19. module generator不调用registry；手写owner能对部分register失败执行完整rollback。
20. 所有builtin host module迁移到唯一descriptor authority，catalog与实际export集合逐项相等。
21. reflection integration shard在60秒预算内执行，不依赖完整graphics lib-test编译。
22. 10k type/100k field catalog在固定startup、lookup和内存预算内完成。
23. untrusted plugin schema超过字符串/字段/深度/文档预算时得到bounded typed diagnostic。
24. Editor Inspector、dynamic scene round-trip、VM state migration和remote schema各有至少一个跨版本golden E2E。

## 11. 保留项

- 保留`ReflectTypeRegistration/ReflectFieldInfo/ReflectedValue/ReflectError`的typed DTO方向，补齐身份和验证而不是退回`serde_json::Value`中心。
- 保留derived component adapter的clone-then-commit和dense batch write优化，但将slot绑定catalog generation。
- 保留`ZrReflectValue`整数范围与向量finite检查，扩展为统一numeric policy。
- 保留TypeRegistry的deterministic map、short-path ambiguity和catalog generation基础，让generation承接schema hash。
- 保留VM registration现有plugin ID/prefix/declared type严格门，并下沉共享核心validator。
- 保留`ScriptHostTypeDescriptor::from_reflect_registration`对projection字段集合一致性的检查，将其改为IR投影后的防御性断言。
- 保留VM state migration已有field rename验证，但由stable field ID/schema diff生成迁移计划，不再手工复制类型事实。
- 参考引擎只提供责任和验证样板；Zircon目标应是更强的typed IR、确定性catalog、可审计兼容diff和低开销generation-bound访问。
