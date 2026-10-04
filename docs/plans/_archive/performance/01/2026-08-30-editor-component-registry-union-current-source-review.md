---
related_code:
  - zircon_editor/src/ui/component_registry/mod.rs
  - zircon_editor/src/ui/component_registry/registry.rs
  - zircon_editor/src/ui/component_registry/tests.rs
  - zircon_runtime/src/ui/component/catalog/registry.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/v2/compiler.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/Palette/SPaletteView.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/Library/SLibraryViewModel.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/Library/SLibraryViewModel.h
  - dev/Fyrox/fyrox-ui/src/node/constructor.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/editor/src/lib.rs
related_plans:
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
status: source_implemented_dynamic_acceptance_pending
---

# Editor component registry union当前源码性能复核（2026-08-30）

## 结论

`zircon_editor/src/ui/component_registry/**`不是帧循环热点：保留注册表由`OnceLock`只构造一次，后续节点投影只做借用的`BTreeMap`查询。当前可直接修复的问题位于编辑器冷启动：旧构造把71项showcase表和256项material表的全部327个描述符深克隆后逐项校验/注册，而最终只有258个唯一ID，69个重叠ID又被后到的material描述符覆盖。

本轮保留现有material-wins语义，改为克隆已经验证的256项共享material表，再借用扫描71个showcase ID，仅克隆并注册两个缺失项。最终258个ID及其完整描述符由测试与旧算法逐项比较。该改动减少69个重描述符深克隆，`327 -> 258`，即21.1%；完整注册/校验调用从`327 -> 2`，减少325次，即99.4%，另保留71次借用ID查询。以上是当前源码可精确推导的工作量，不是产品CPU采样数据。

## 稳定切片

本次聚焦3个Rust文件，修改后共85行、73行非空、2,946字节、3个测试、0 ignored、0 include，按路径排序拼接原始内容SHA256为`9f2cd327ad68bacdf7b633ad4e4fb15cb09fd49983c4544df6c151e6e5a2d143`。

- `mod.rs`：6/4行，94字节，SHA256 `cfa679479de4913ac63932e6629cce0bfca659bace4c76999268e3b809bd3975`。
- `registry.rs`：20/18行，776字节，SHA256 `a68dfc4c03a518201c0609b9ed2d442be38cf3a3306f578df4631989d21cd72b`。
- `tests.rs`：59/51行，2,076字节，3个测试，SHA256 `727914a079d122b38052e5271445337d7f03532e2858e0dd94395980958fbc18`。

`UiComponentDescriptor`不是小型Copy记录：它拥有ID、显示名、role、多个`Vec`、两个`BTreeSet`、palette和默认节点模板。这里的69次减少是真实的嵌套容器深克隆削减。material注册表克隆仍会复制256个描述符，因此本轮不是最终所有权模型。

## 当前结构问题

当前至少存在三种component catalog选择规则：编辑器asset palette和默认document compiler只看showcase；UI v2 compiler先查material再回退showcase；retained host维护material覆盖showcase的第三张合并表。相同ID的优先级由调用路径和插入顺序决定，而不是一个显式、可验证的profile/provider generation。

这会带来三类风险：同一组件在编译、palette和retained host中可能解析为不同描述符；catalog变更需要多个消费者分别记住失效规则；第三张表即使只初始化一次，仍复制完整material负载。Runtime 74中的component catalog注入问题和Runtime 11a中的dense compiled descriptor handle方案应拥有最终收敛，不能把本轮冷启动优化当作架构完成。

`retained_component_registry().revision()`当前没有生产消费者。旧revision记录的是插入历史，新revision记录material基表加两个补充项；测试刻意比较全部ID和描述符，而不把内部构造次数误作不可变catalog身份。后续必须使用显式catalog generation，而不是复用该revision作为跨路径兼容键。

## 参考引擎约束

Unreal UMG的`SPaletteView`从`FWidgetBlueprintEditor`取得一个共享`PaletteViewModel`，filter/tree直接消费该view model持有的`TSharedPtr<FWidgetViewModel>`；`FLibraryViewModel`按类别投影时继续传递`TSharedPtr<FWidgetTemplate>`，没有为palette、filter和selection再建立相互覆盖的完整模板注册表。这支持“一个目录所有者，多种借用投影”，不证明Zircon应复制Unreal的Slate/Content Browser ABI。

Fyrox由`new_widget_constructor_container`构造一次widget constructor容器，`Engine`持有`Arc<WidgetConstructorContainer>`，编辑器启动把同一个Arc注入engine和后续消费者。这支持进程/设备代际共享一个不可变构造目录，不证明Zircon应采用Fyrox的UUID或锁实现。

## 架构硬切

M0：增加catalog source count、重叠ID、冲突描述符、克隆字节和初始化次数的精确测试/计数；不同描述符的重复ID必须返回typed conflict，不能靠插入顺序静默覆盖。

M1：编译一个不可变`UiComponentCatalogGeneration`，携带profile、provider/plugin、schema/toolchain generation、稳定descriptor ID和完整来源receipt。showcase、material和plugin贡献只作为候选输入。

M2：编译阶段一次验证并分配dense descriptor/prop/state/slot/event handles。document compiler、UI v2、asset palette和retained host借用同一个Arc generation；稳定代际不再克隆描述符或重建BTreeMap。

M3：catalog替换采用candidate/accepted发布。验证失败保留上一代；模板、palette、retained projection和诊断必须由同一accepted generation失效，不能各自推断。

M4：Diagnostics Disabled不构造管理投影；Counters只记录稳定ID和计数；显式Full导出按行数/字节/deadline预准入并返回Arc-backed generation。

## 验收矩阵

- source表0/1/2/16，描述符0/1/71/256/1k/cap+1，ID唯一/相同等价/相同冲突，provider/profile/schema代际稳定与变化。
- compiler、UI v2、palette、retained host对每个ID必须解析到同一descriptor generation；缺失、冲突、plugin卸载和失败重载均有typed结果。
- 冷启动报告source visits、descriptor clones及字节、验证次数、BTreeMap节点分配、catalog发布时间；稳定调用报告lookup次数且描述符克隆/重建为零。
- 1/4/16编辑器窗口和1/64/1k模板节点下记录p50/p95/p99初始化、编译和投影时间；动态证据必须使用当前源码构建。

硬门槛：当前源码可构建；现有258项material-wins内容完全保持；稳定注册表只初始化一次；同一accepted catalog generation服务全部消费者；重复冲突不能由顺序决定；失败候选不改变已发布目录；稳定代际深克隆为零；诊断计数与实际工作一致。

## 当前验证状态

精确`rustfmt --check --edition 2024 --config skip_children=true`和scoped `git diff --check`通过。源码形状检查确认material共享表只克隆一次、showcase仅在ID缺失时克隆注册，并保留旧算法全量描述符等价测试。

Cargo和产品动态验证仍受当前共享工作树既知阻塞影响：UI/text生产编译错误、SDF `cfg(test)` Result/Option不匹配、compiled-scene/OIT陈旧source guard及graphics feature reexport问题。没有可接受的当前源码可执行文件，因此没有Cargo test、WPR、GPU、功耗或与Unreal耗时接近程度的声明；本记录继续处于dynamic acceptance pending，不能进入`review.md`。
