---
related_code:
  - zircon_plugins/neural
  - zircon_plugins/neural/plugin.toml
  - zircon_plugins/neural/runtime
  - zircon_plugins/neural/editor
  - zircon_plugins/neural/features/post_process/runtime
  - zircon_plugins/neural/dist
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNERuntime.h
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNERuntimeCPU.h
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNERuntimeGPU.h
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNERuntimeRDG.h
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNERuntimeRunSync.h
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNEModelData.h
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNETypes.h
  - dev/UnrealEngine/Engine/Source/Editor/NNEEditor/Private/NNEEditorModelDataFactory.cpp
  - dev/UnrealEngine/Engine/Source/Editor/NNEEditor/Private/NNEEditorOnnxFileLoaderHelper.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/NNERuntimeBasicCpu/Source/NNERuntimeBasicCpu/Private/NNERuntimeBasicCpuModel.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/NNERuntimeRDG/Source/NNERuntimeRDG/Private/NNERuntimeRDGModel.cpp
  - dev/UnrealEngine/Engine/Plugins/NNE/NNEDenoiser/Source/NNEDenoiser/Private/NNEDenoiserModelInstanceRDG.cpp
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource_saver.cpp
  - dev/bevy/crates/bevy_asset/src/loader.rs
  - dev/bevy/crates/bevy_asset/src/render_asset.rs
  - dev/Fyrox/fyrox-resource/src/loader.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Upscaling/IUpscaler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/RenderPass/DLSSPass.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 02 · Neural 模型、ONNX 导入、CPU/GPU 推理、后处理与 Editor 产品链工程化差距

## 1. 结论

`zircon_plugins/neural` 不是空目录，也不应被简单删除。当前已有四块值得保留的真实基础：一个有显式 magic/version/table/weight alignment 的 `.znn` 二进制格式；一个能解析 ONNX protobuf 子集并输出结构化转换诊断的离线转换器；一个覆盖 Gemm、卷积、逐元素、池化、上采样和归一化子集的 CPU reference interpreter；以及一个能为相同子集生成 `ComputePassDescriptor`、WGSL、parameter bytes 和 weight upload plan 的 GPU 计划器。41 个物理文件中有 36 个 Rust 文件、7,016 行、39 个 test attributes，说明它已经超过纯 skeleton。

但这四块目前彼此没有形成可交付产品。`NeuralRuntimePlugin::register()` 直接返回 `Ok(())`，没有注册 asset loader、resource owner、model runtime、inference service、render feature或pass executor；可选 `neural.post_process` 的测试反而逐项固定所有 runtime extension 集合为空。Editor Host 默认编译 Neural runtime/editor catalog，Editor 插件能把 ONNX 覆盖写成 `.znn` 并将其登记为 `neural.model`，仓内却没有任何 Neural 包外的 production consumer 能加载、cook、实例化或执行这个资产。当前用户可见 authoring 入口因此生产的是死产物，不是可运行的 Neural 功能。

安全与正确性边界也没有闭合。Editor import 与 CLI 均 whole-file read/convert/serialize 后直接 `fs::write` 目标；现有文件失败时可能被截断，undo 又把完整旧文件保存在内存并同样直接覆盖。自制 ONNX parser 没有 bytes/field/string/tensor/node/dimension/allocation 预算，把有符号维度直接 `as u32`，忽略 opset/domain/external data。`.znn` loader 在证明 op table 能容纳记录前按不可信 `op_count` `Vec::with_capacity`，随后无上限复制 weight blob。模型 `validate()` 只验证结构引用与局部 attr encoding，不验证拓扑序、唯一生产者、tensor kind、算子 arity/shape/backend executable contract；CPU/GPU 执行器各自再做不一致的晚期检查，GPU `Reshape` 甚至不验证输入输出 element count就直接别名。

本轮登记 **5 项 P0、60 项 P1、12 项 P2**。修复顺序不是继续增加算子或生成更复杂 shader，而是先关闭假能力与非原子/无界输入面；再建立 source model、target runtime cook、artifact identity、resource loader、runtime provider和model instance合同；然后把CPU reference与真正的Render Graph执行后端接入；最后才做后处理产品、图融合、低精度、异步计算和与Unreal/其他后端同语义的性能竞争。没有真实scene image I/O、GPU submission、像素oracle和同硬件统计前，不能把descriptor planning称为GPU inference，也不能宣称表现或性能超过当前Unreal。

## 2. 审查边界与证据

### 2.1 物理范围

| 集合 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Neural package全量 | 41 / 7,016 / 239,770 | E3逐文件读取；fingerprint `df857c43c8f4b06f66c3d6aad77d410709807c4b2a0b1cb6721cb9cf4a9facc1` |
| Rust源码 | 36 / 6,886 / 235,676 | E3逐runtime/editor/post-process/dist、39个test attributes、0 ignored、6个`unsafe`出现 |
| Runtime model/ops/CPU/GPU | 24 Rust文件 | E3逐format/validate/op attrs/interpreter/graph plan/WGSL/layout/upload/tests |
| Editor importer/converter/CLI | 11 Rust文件 | E3逐registration/path authority/EditCommand/protobuf reader/converter/executable contract/tests |
| Product composition | App feature、两个first-party catalog | E3追踪Editor Host默认feature到runtime/editor registration |
| 包外consumer反向检索 | 非Neural production源码 | E3：0个`NnModelAsset`、`NnGraphExecutor`、`NnPostProcessSettings`或Neural import command业务consumer |

指纹算法为：按相对路径排序，对41个文件逐个取SHA-256，再对`path + space + hash`的LF清单取SHA-256。成文时`git status --short -- zircon_plugins/neural`为空，故本篇`source_recheck_required: false`；任何后续源码变更仍必须重算指纹并按finding owner复核，不能用该字段宣称实现完成。

### 2.2 调用链复核

本轮实际追踪了以下纵向链，而不是只搜索关键词：

1. `zircon_app target-editor-host` -> advanced render runtime catalog + Neural editor catalog -> project selection -> runtime/editor registration；
2. `plugin.toml` -> runtime descriptor/package manifest -> native dist V3 entry -> registration manifest/behavior table；
3. Editor importer/menu/operation factory -> project/asset-root path authority -> `NeuralModelImportCommand` -> ONNX read/convert -> `.znn` write/undo；
4. ONNX protobuf field -> graph/tensor/node/attribute DTO -> tensor ID/weight layout -> executable-contract validation -> `NnModelAsset`；
5. `.znn` header/tensor/op/weight decode -> `validate()` -> CPU interpreter或GPU graph plan；
6. GPU plan -> WGSL/bindings/dispatch/parameter bytes/transient name/weight upload plan -> 包外Render Graph consumer；
7. optional post-process manifest/settings -> runtime feature registration -> render feature/pass executor/scene hook；
8. unit tests -> 是否存在malformed/fuzz、shader compile、real device、product load、cook/reopen、fault和performance gate。

第6、7条在包边界终止：没有生产consumer把计划声明变成resource allocation、pipeline compilation、render-graph insertion、submission、completion或post-process image output。

### 2.3 动态证据边界

本轮没有修改production或tests，也没有运行新的Cargo、GPU、Editor窗口或CLI动态测试。已有39个test attributes只作为静态测试库存，不被本文宣称通过。此前公共`zircon_editor --lib` test build在617.2秒后被239个既有错误和122个warning阻断，本轮没有重复同一不可达lane；该阻断也意味着Neural Editor product E2E当前没有可复用的绿色宿主证据。

静态E3足以确认空注册、直接写盘、无预算预分配、signed-to-unsigned cast、缺失consumer和验证不变量；它不能证明WGSL能由当前后端编译、GPU数值与CPU一致、post-process像素正确、device loss可恢复或性能优于任何参考引擎。

## 3. 可保留基础

1. `.znn`有固定little-endian header、magic/version、tensor/op table、256-byte weight alignment、reserved field和precision flag检查，比把任意`Vec`直接serde成资产更适合作为后续版本化artifact基础。
2. model format使用checked arithmetic验证table/blob边界，weight tensor验证offset/alignment/range，未知opcode和错误attrs会返回typed error。
3. converter对unsupported op、arity、attribute和shape输出结构化diagnostic，不静默近似不支持的算子；V1 executable contract已尝试保证CPU/GPU共有子集。
4. Editor operation对project root、asset root、绝对/相对路径、symlink canonicalization和输出父目录做了权限边界检查，这部分应保留并接入共享import job。
5. CPU interpreter在执行前调用model validation，对输入长度、F32 weight长度、arity与多数算子shape做检查，适合作为correctness oracle，而不是删除后只保留GPU。
6. GPU planner至少显式描述binding kind、weight offset、workgroup、dispatch和parameter buffer，shader没有隐藏在Editor里；这为后续compiled pipeline/runtime provider提供了可迁移起点。
7. weight upload和tensor layout有独立类型，错误使用checked element/storage size；应扩展为真正的resident resource owner与memory plan。
8. optional post-process默认disabled，`NnPostProcessSettings::validate`会拒绝enabled但无model和越界intensity。保持disabled是真实性基线，不能在无executor时打开。
9. native dist明确只有registration manifest、stateless、无command/event/state/bridge；它没有假造callback执行结果。后续要么实现行为等价，要么移除`native_dynamic`产品承诺。
10. 测试覆盖format roundtrip、bad magic/version、weight range/alignment、CPU基本算子、GPU descriptor布局、path authority和import undo happy path，为fault/fuzz/device/product gate提供了种子。

## 4. P0：产品真实性、数据完整性、输入资源与执行正确性阻断

### P0-01 · Editor可生产`neural.model`，Runtime却没有任何资产或推理provider

`target-editor-host`默认启用advanced-render runtime与Neural editor catalog；Editor插件注册`.onnx` importer、`neural.model.import` operation、menu和asset type，能成功生成`.znn`。然而`NeuralRuntimePlugin::register()`不向`RuntimeExtensionRegistry`注册任何内容，包外没有`NnModelAsset`consumer；post-process registration test还把modules/managers/shaders/importers/components/options/events/scene hooks/render features/pass executors/prepare collectors/providers等全部固定为空。Dist DLL也只有manifest，`is_stateless: true`、无command/event/bridge/host-ready。

这不是“partial功能少”，而是authoring success与runtime consumability相矛盾。M0必须让无loader/provider的import/open/apply/run入口Unavailable，或先补齐最小真实provider；capability admission必须验证asset loader、runtime backend与consumer generation，不能仅凭catalog registration返回Some。

### P0-02 · Import、undo与CLI直接覆盖目标，失败或崩溃可破坏last-good资产

`NeuralModelImportCommand`先把完整旧输出读入内存，再用`fs::write(output, converted)`覆盖；write失败时仍把`applied`置为true以允许undo，但截断、部分写、进程崩溃、磁盘满或断电可能已经破坏旧文件。undo对旧bytes再次`fs::write`，新文件则直接`remove_file`，同样没有staging、flush、atomic replace、journal或reopen recovery。`zr_onnx_convert`也直接写调用方目标。

必须统一到asset import transaction：preflight source/output revision与预算，写同卷临时文件，验证完整artifact与digest，flush file和必要目录元数据，atomic replace，记录receipt/previous artifact，然后更新asset index。undo只引用content-addressed previous artifact，不把任意大旧模型塞进Editor command history。fault injection必须覆盖每个写盘阶段。

### P0-03 · 自制ONNX parser与转换器对不可信内容无预算，并把负维度包装成巨大`u32`

parser以嵌套slice读取protobuf，但对文件bytes、length-delimited field、string、node、tensor、initializer、dimension、attribute列表、raw data、递归/嵌套总量和累计allocation均无上限。tensor/value-info维度使用`read_varint()? as u32`，ONNX有符号负值或大于`u32::MAX`的值会包装；字符串使用`from_utf8_lossy`改变identity。Editor command又在宿主同步执行whole-file read、graph build、weight复制和完整serialize，CLI同样如此。

项目内模型、下载包或第三方资产可据此触发巨量预分配/循环/weight输出和Editor主线程长时间阻塞。必须使用成熟ONNX protobuf实现或至少建立共享bounded reader，先验证size/count/depth/dimension/element/weight/output/time budget，再分配；有符号/动态维度必须typed表示并拒绝非法转换，超限返回稳定diagnostic且不改变输出。

### P0-04 · `.znn` loader在证明table容量前信任count并无界复制weight blob

header中的`op_count`来自不可信`u32`。边界检查只证明`op_table_size`切片在文件范围内，`decode_ops`随后立即`Vec::with_capacity(expected_count)`，但最小op record至少8 bytes，未先证明`op_count <= op_table_size / minimum_record_size`。构造很小的op table与巨大count即可先请求巨大容量，之后才因count不匹配返回`UnexpectedEnd`。合法边界内的weight blob又直接`to_vec()`，没有runtime asset byte budget；tensor count虽受32-byte table长度间接约束，也没有产品配额。

Runtime loader必须先用格式最小记录证明所有count可由section容纳，再应用per-asset和per-project bytes/items/ops/tensors/weights上限；使用fallible allocation或mapped/shared immutable artifact，解析过程中不复制整个weight blob。malformed corpus、property test与fuzzer要证明任意输入只产生bounded typed failure。

### P0-05 · `.znn validate()`接受语义非法图，CPU/GPU在不同阶段产生不一致错误、别名或越界访问面

`validate()`只要求tensor非空、rank/shape/weight range、op输入输出非空、引用存在和attrs可编码；它不验证图是DAG/拓扑序、tensor唯一生产者、输出不可覆盖输入/weight、每个tensor是否可达、算子arity、dtype、shape、broadcast、view element count或backend支持。CPU会顺序执行并把结果插回`BTreeMap`；GPU unary/binary dispatch只用output element count，不验证input/output shape，shader按output count读取input；`fold_reshape`甚至不比较element count就把output别名到source。结构合法但语义非法的`.znn`因此可通过admission，在CPU与GPU得到不同失败或GPU robust-access依赖下的错误结果。

必须建立一次性的`ValidatedNnGraph`/compiled IR：解析后完成producer/consumer、topology、kind、arity、dtype、shape、broadcast、alias、backend capability、resource budget与IO contract验证；CPU/GPU只能消费该不可变IR。任何未通过IR编译的模型不能进入asset cache、GPU resource allocation或product capability ready状态。

## 5. P1：工程化前必须补齐的合同

### 5.1 Package、capability、asset与生命周期

- **P1-01**：`runtime.plugin.neural`与`runtime.asset.neural_model`标为Partial，却没有机器可判定的sub-capability，例如`import-only`、`cpu-reference`、`gpu-plan-only`、`runtime-executable`。拆分并让admission fail-closed。
- **P1-02**：Editor Host feature无条件编译advanced-render runtime与Neural editor catalog，实际selection只决定返回registration，不能表达平台/设备/backend不可用。product composer应结合project、target、RHI capability和artifact backend。
- **P1-03**：runtime catalog以plugin id返回空registration report，没有`CanCreateModel`/reason/status/selected backend合同。建立runtime provider registry与可诊断的capability probe。
- **P1-04**：native dist声称neural model与compute services“remain hosted by runtime module”，自身无行为，也没有验证静态runtime module确实存在。没有static/native parity前移除`native_dynamic`默认包装或明确metadata-only形态。
- **P1-05**：optional post-process module只返回manifest，空extension被单测当正确。测试应改为在未实现时断言Unavailable，实现后断言唯一render feature/pass owner和teardown。
- **P1-06**：`NnPostProcessSettings`按值持有`Option<NnModelAsset>`，复制settings会复制graph与weight bytes。改用stable asset handle、revision和resident model instance handle。
- **P1-07**：没有model asset identity、source revision、artifact digest、runtime/provider id、device generation或instance generation；热重载与陈旧句柄无法拒绝。
- **P1-08**：没有load/unload/cancel/drain/device-loss/plugin-reload生命周期，weight/pipeline/in-flight dispatch也没有retirement owner。接入Runtime lifecycle与GPU completion gate。

### 5.2 Source、cook、cache与artifact

- **P1-09**：`.znn`只有执行数据，没有ONNX source identity、原始file id、import settings、converter/version或diagnostic provenance，无法判断产物陈旧。
- **P1-10**：artifact key不含BuildSet、target platform、RHI/backend、precision、operator-set、optimization profile和dependency digest；同一文件名可遮蔽不兼容产物。
- **P1-11**：Runtime resource/asset registry没有`neural.model` loader、type schema或异步load状态，Editor asset type贡献与Runtime asset authority断裂。
- **P1-12**：没有cook/export阶段按目标平台和runtime provider产生变体；Editor生成的通用`.znn`被隐含当作所有目标可执行。
- **P1-13**：没有DDC/derived cache、cache miss编译、corruption验证、remote cache或last-good；每次转换只能直接覆盖项目文件。
- **P1-14**：不支持ONNX external tensor data与additional buffers，source dependency graph也不记录附属文件；重导入和打包会漏依赖。
- **P1-15**：artifact install没有staging/digest/atomic publication/generation swap/in-flight retirement，不能安全hot reload model。
- **P1-16**：`.znn`版本只有等值V1判断，没有schema migration、forward-compatible section、minimum reader或explicit hard-cut policy；未来加operator/layout只能全文件拒绝。

### 5.3 `.znn` schema与graph invariant

- **P1-17**：tensor表没有name、semantic、layout、quantization、color/image mapping或input/output binding metadata，业务consumer只能依赖易错的`u16`位置。
- **P1-18**：shape硬限制rank 1..4并左填充为`[u32;4]`，原rank与storage shape混合；不支持rank 0、5-8、symbolic/dynamic dimension或layout stride。
- **P1-19**：format暴露F16并用全局flag限制混合precision，CPU和GPU又都拒绝F16执行；当前F16是可序列化但不可执行的假能力。
- **P1-20**：没有dynamic shape constraint、shape inference program、profiled shape set或input shape preparation；不同分辨率只能重新生成静态模型。
- **P1-21**：header/table/weight section没有checksum或digest，任意bit corruption只在恰好破坏结构时被发现，数值权重损坏会静默执行。
- **P1-22**：op records没有独立schema version/domain/provider requirement；opcode numeric新增与attr变化只能绑死整个file version。
- **P1-23**：不验证graph input/output数量、producer唯一性、topology、unused tensor、cycle、in-place legality和output reachability；这些应进入compiled graph invariant。
- **P1-24**：weight range允许重叠、未检查padding为零、未检查NaN/Inf/denormal policy，也没有per-tensor digest或compression/streaming block表。

### 5.4 ONNX读取、转换与诊断

- **P1-25**：手写protobuf只识别少数字段，未知field静默skip；合法但未理解的模型可能被误判为缺省语义，而非报告unsupported feature path。
- **P1-26**：`from_utf8_lossy`会把不同非法byte序列归并为同一名称，可能造成tensor/node/attribute identity collision；必须严格UTF-8或保留原始bytes并诊断。
- **P1-27**：有符号dimension和attribute varint直接cast，缺range/negative检查；除了P0资源面，也会产生错误shape与误导diagnostic。
- **P1-28**：reader不解析IR version、opset import、operator domain、producer/version、model metadata或function definition，converter无法证明按哪个ONNX语义解释operator。
- **P1-29**：不支持external data、sparse initializer、typed tensor字段、string tensor、sequence/map/optional或control-flow；必须明确capability matrix并fail closed，不能静默skip。
- **P1-30**：graph/node/tensor/attribute创建大量`String`、`Vec`与`BTreeMap`复制，没有arena、interning、streaming或累计allocation account；大模型峰值内存远超源文件。
- **P1-31**：read error只有`UnexpectedEnd/InvalidVarint/UnsupportedWireType`等粗粒度枚举，没有protobuf field path、byte offset、node/tensor identity、opset和repair hint。
- **P1-32**：converter只支持F32/static rank<=4，支持表与`NnOpCode`枚举并不一致；Concat/Slice存在opcode却明确拒绝，必须由生成的backend matrix统一。
- **P1-33**：tensor id由`BTreeMap`名称字典序分配；无名/改名会重排整个artifact，降低diff/cache稳定性，也没有duplicate name/empty identity的专门合同。
- **P1-34**：Editor与CLI都whole-file读取，graph、initializer F32 vector、weight byte vector、serialized output和undo previous bytes可同时驻留；没有峰值内存预算或spill策略。

### 5.5 CPU runtime与数值合同

- **P1-35**：每次`run_cpu`都复制全部input、从weight bytes解码复制全部F32权重，并为每个中间tensor新建Vec；共享模型权重与per-instance workspace没有分离。
- **P1-36**：没有`Model`/`ModelInstance`两层、prepared shape、persistent workspace、caller-owned binding或instance pool，重复推理无法复用任何准备成本。
- **P1-37**：API同步返回嵌套Vec，没有deadline、cancel、priority、thread affinity、async job、output buffer size或partial/error telemetry，不能接实时帧预算。
- **P1-38**：Conv/Gemm/Norm为朴素Rust循环，没有SIMD、线程池、tiling、cache-aware packing、operator kernel registry或外部成熟runtime backend，定位只能是reference oracle。
- **P1-39**：除部分epsilon检查外没有统一NaN/Inf/div-zero/overflow/denormal/rounding policy；CPU与WGSL fast-math语义可能分叉。
- **P1-40**：CPU/GPU共有子集只在converter入口做shape contract，直接构造/加载`.znn`绕过；没有逐算子随机/edge/golden parity与tolerance policy。

### 5.6 GPU backend、Render Graph与性能

- **P1-41**：`NnGraphExecutor`只返回descriptor/parameter bytes/name，没有创建buffer、编译pipeline、导入resource、插入builder、提交、同步或回收；名称不等于可执行图。
- **P1-42**：`transient_outputs`只有字符串，没有size/alignment/usage/lifetime/alias interval；Render Graph无法据此分配或证明读写资源存在。
- **P1-43**：每个plan携带inline WGSL字符串，未接shader artifact/compiler reflection/pipeline cache/PSO key；错误只会推迟到未存在的consumer阶段。
- **P1-44**：stage固定PostProcess、queue固定AsyncCompute，没有caller选项、dependency/queue capability、graphics fallback或调度成本模型。
- **P1-45**：没有device limits、max storage binding、uniform alignment、max dispatch、shader feature、subgroup、precision或vendor workaround probe；model admission与实际GPU能力无关。
- **P1-46**：除Reshape alias外无constant folding、operator fusion、layout propagation、transpose elimination、Conv/Norm folding或backend graph optimization。
- **P1-47**：没有liveness/memory planner、workspace budget、in-place eligibility、buffer reuse或persistent weight residency；每个intermediate使用永久化字符串身份。
- **P1-48**：WGSL按operator重复生成并clone String，parameter Vec逐pass分配；没有compiled plan cache、specialization cache或source/artifact digest复用。
- **P1-49**：shader全部F32/NCHW/标量路径，没有FP16/BF16/INT8、packed vector、tensor core/cooperative matrix、subgroup或布局特化；不能承担“优于Unreal”的性能目标。
- **P1-50**：Conv/Pool/Upsample Z dispatch用`saturating_mul(batch, channels)`，overflow会静默变成`u32::MAX`而非admission error；dispatch与GPU limit也未比较。
- **P1-51**：没有resource barrier、queue ownership、external input/output state、history validity、device-loss generation、timestamp query或per-op diagnostics合同。
- **P1-52**：GPU测试只比较descriptor字段和WGSL marker，不编译shader、不绑定真实buffer、不运行adapter、不读回数值，也无pixel/post-process E2E。

### 5.7 Post Process、Editor产品与验证

- **P1-53**：post-process没有scene color/depth/motion/exposure/history输入映射、output format、colorspace、alpha、dynamic resolution或tile/overlap合同。
- **P1-54**：没有temporal state、camera cut/resize/history reset、multi-view/VR、frame generation或in-flight model swap语义；`inference_scale`只是枚举factor。
- **P1-55**：model asset没有业务用途、expected input/output semantic、normalization、transfer function、range、channel order或compatible post-process contract，任意`.znn`都可被settings接受。
- **P1-56**：Editor没有model inspector、graph/operator/shape/weight视图、runtime/backend compatibility、memory estimate、preview、CPU/GPU diff或profile结果。
- **P1-57**：转换和I/O在`EditCommand::apply`同步执行，没有Background Job admission、cancel、progress、deadline、source revision recheck或shutdown drain。
- **P1-58**：undo把完整旧output bytes保存在command对象中，没有history byte budget、dedup、spill、compression或content-addressed reference；多个模型可耗尽Editor内存。
- **P1-59**：CLI接受任意input/output路径并绕过project asset authority、artifact registry、recipe、transaction与receipt；它应成为共享import service的thin client，而不是第二套writer。
- **P1-60**：39个单测没有hostile ONNX/`.znn` corpus、fuzz/property、OOM budget、fault injection、shader compile、real GPU、product load/cook/reopen、device loss、soak或benchmark lane。

## 6. P2：可维护性、诊断与长期演进

- **P2-01**：从一个operator schema生成opcode、attrs codec、ONNX mapping、CPU/GPU support matrix、shader binding与文档，消除当前多处手写switch漂移。
- **P2-02**：为每个supported/unsupported operator维护最小合法fixture、边界fixture和来源/license元数据，避免测试内手拼protobuf成为唯一语义样本。
- **P2-03**：统一diagnostic code、severity、field/node/tensor span、backend、build/artifact/correlation id与JSON/text projection。
- **P2-04**：为`.znn`提供确定性inspect/dump/diff工具，输出section digest、graph topology、IO和resource estimate，不要求人工读bytes。
- **P2-05**：建立版本化模型corpus与CPU/GPU/reference backend golden，分别跟踪正确性、导入时延、峰值RSS、cook体积和运行时性能。
- **P2-06**：补per-model/per-instance/per-op CPU/GPU time、workspace、resident weights、compile/cache hit、fallback和failure reason观测。
- **P2-07**：记录ONNX与第三方runtime/parser的版本、license、security update和模型来源信任策略，接入package/release SBOM。
- **P2-08**：source dependency与artifact lineage进入Asset Inspector和build report，支持“为何重导入/为何未命中cache”解释。
- **P2-09**：为capability、provider和backend状态提供稳定的human-readable reason，Editor只投影该状态，不自行拼成功字符串。
- **P2-10**：错误与UI文本接入本地化，内部operator/path/build identity与面向用户说明分离。
- **P2-11**：定义V1 hard cut、deprecated operator/schema window和批量reimport工具，不长期保留不可验证的legacy reader分支。
- **P2-12**：把本篇finding、owner、source fingerprint、implementation evidence和gate接入全局机器可读manifest，源码变化自动标记recheck。

## 7. 目标架构与Owner边界

### 7.1 目标产品链

```text
ONNX source + external data + import recipe + source revision
    -> bounded parser / trusted ONNX frontend
    -> typed source graph + diagnostics
    -> backend-neutral validated IR
    -> target/runtime/provider compiler
    -> immutable NeuralArtifact { digest, schema, BuildSet, backend variants }
    -> AssetManager loader + generation-qualified Model
    -> CPU/RDG/GPU ModelInstance { prepared shape, workspace, bindings }
    -> Render Graph / async job execution + terminal receipt
    -> Neural PostProcess provider + Editor preview/inspector/profile
```

任何箭头失败都必须返回typed diagnostic且不发布下一层对象。Editor、CLI、cook和reimport共享同一个compiler service；CPU与GPU共享validated IR和operator contract，但各自拥有kernel/pipeline/workspace。Post-process只能消费已通过image-I/O contract和selected backend admission的model instance。

### 7.2 Canonical owner

| Owner | 建议物理归属 | 必须拥有 | 不应拥有 |
|---|---|---|---|
| Neural source frontend | `zircon_plugins/neural/editor`或独立tooling crate | ONNX parse、external data、opset/domain、source diagnostic | Runtime GPU resource |
| Neural artifact schema | `zircon_plugins/neural/runtime/model`，稳定wire类型可下沉interface | versioned sections、validated IR serialization、digest、limits | Editor command/history |
| Neural runtime service | `zircon_plugins/neural/runtime` | provider probe、model/model instance、CPU/GPU backend、lifecycle | Project文件直接写盘 |
| Asset/cook owner | Runtime asset + Tooling cook | recipe、dependency、BuildSet、target variant、DDC、atomic install | Operator kernel实现 |
| GPU execution owner | Neural runtime adapter + Runtime Render Graph/RHI | buffer/pipeline/workspace、enqueue、barrier、completion、device generation | Editor preview state |
| Post-process owner | `features/post_process/runtime` | image semantic、history、render feature/pass、settings adapter | 模型bytes按值持有 |
| Editor owner | Neural editor extension | job/operation projection、inspector、preview、diagnostic、transaction receipt | 第二套parser/writer/runtime truth |

映射到全局owner家族：O00 Capability Truth、O01 BuildSet、O02 Lifecycle/Job、O03 Schema/Identity、O04 Source/Artifact/Cook、O05 Transaction、O07 Budget、O09 GPU/Render、O11 Evidence、O13 Authoring、O14 Delivery、O15 Trust。

## 8. 参考引擎对照与使用边界

### 8.1 Unreal NNE

Unreal把源模型与runtime-specific data分开：`INNERuntime`提供`CanCreateModelData`、`CreateModelData`和包含file id/target platform/runtime version的identifier；`UNNEModelData`保留源file/additional data、target runtimes、runtime settings和每个runtime的cooked/cached model data。CPU/GPU/RDG各自提供`CanCreateModel*`与`CreateModel*`，`IModel`创建可复用的`IModelInstance`；实例暴露input/output descriptor、动态shape准备和caller-owned binding，RDG实例直接`EnqueueRDG(FRDGBuilder...)`。BasicCPU和RDG又是独立provider，不把reference interpreter伪装成唯一高性能后端。

Zircon不需要复制UObject或DDC实现，但必须保留这些边界：源模型、target/provider artifact、shared model、per-call/per-instance workspace、shape preparation、caller binding、backend probe和真正的render graph enqueue不能合并为一个`NnModelAsset + Vec` helper。

NNEDenoiser进一步表明后处理不是`Option<Model> + intensity`：它有model instance、IO mapping、resource manager、history、transfer function、tiling、auto exposure、view extension和CPU/GPU/RDG路径。Zircon可以设计更紧凑、更快的接口，但scene resource semantic、temporal state、tiling和frame lifecycle不能省略。

### 8.2 Godot、Bevy与Fyrox

本地Godot、Bevy、Fyrox源码没有可与Unreal NNE同级的通用Neural inference subsystem，本篇不虚构其operator/backend能力。它们只用于asset工程边界：Godot ResourceLoader/Saver的typed format/resource path/cache；Bevy AssetLoader/RenderAsset的source load与render-world prepared asset分层；Fyrox ResourceManager/Loader的typed resource、state与异步管理。这些共同反证“Editor写出一个扩展名文件”就等于runtime asset。

### 8.3 Unity Graphics

本地镜像是Unity Graphics packages，不含Sentis/Barracuda等完整ML runtime源码，因此不能用于ONNX推理能力对照。可用的事实仅是Graphics中的`IUpscaler`、DLSS/FSR/STP option与HDRP pass把provider、support probe、camera/dynamic-resolution options和render-pipeline execution分层。Zircon Neural post-process可借鉴provider/admission/pass边界，但不能据此宣称支持Unity的Neural模型产品。

## 9. 重构里程碑

### M0 · Truth Freeze与风险封口

- 无runtime loader/provider时，Neural import/open/run/post-process入口统一Unavailable；
- 修复`.znn` count/weight budget与semantic admission，ONNX解析加hard budget和严格dimension；
- Import/CLI改为共享atomic artifact transaction，保留last-good；
- 删除“post-process空extension即成功”的测试合同。

### M1 · Source、Artifact与Identity

- 定义`NeuralSourceId/RecipeId/ArtifactId/ProviderId/ModelGeneration`；
- 建立source+external data依赖、BuildSet/target/backend key、digest与DDC；
- Runtime asset registry可异步加载、验证和generation swap Neural artifact。

### M2 · Bounded ONNX Frontend

- 采用成熟ONNX/protobuf实现或完整bounded frontend；
- 解析opset/domain/external data/dynamic shape和严格UTF-8/identity；
- 产出field/node/tensor path diagnostic与malformed/fuzz corpus。

### M3 · Validated IR与Versioned `.znn`

- operator schema生成arity/type/shape/attrs/backend matrix；
- graph compiler验证topology/producer/IO/alias/budget并产出不可变IR；
- `.znn`采用可校验section、digest、limits和明确migration/hard-cut策略。

### M4 · Runtime Provider与Model Instance

- 实现provider probe、CanCreate/CreateModel、Model/ModelInstance、shape prepare和caller bindings；
- shared immutable weights与per-instance workspace分离；
- 生命周期接入cancel/drain/reload/device generation。

### M5 · CPU Reference与生产Backend

- 保留CPU interpreter作为oracle，移出实时性能承诺；
- 接入至少一个成熟CPU runtime或工程化kernel registry；
- 完成threading/SIMD/workspace reuse/numeric policy与reference parity。

### M6 · 真正的GPU/RDG执行

- compiled shader/pipeline artifact、device capability和cache；
- memory/liveness plan创建transient/persistent buffers；
- 真实Render Graph enqueue、barrier、async queue、timestamp、completion与device loss；
- real adapter数值readback与fault gate。

### M7 · GPU优化与低精度

- operator/layout fusion、constant folding、weight packing、workspace reuse；
- FP16/BF16/INT8只在provider capability、calibration和accuracy gate后启用；
- 针对device/vendor选择kernel，不用单一naive WGSL冒充高性能backend。

### M8 · Neural Post Process产品

- 定义scene image IO、colorspace/range/channel、history、camera cut、dynamic resolution与tiling；
- 注册真实render feature/pass executor并支持fallback；
- model metadata与post-process contract不匹配时fail closed。

### M9 · Editor Authoring产品

- importer改为Background Job + transaction receipt；
- model inspector、compatibility、memory、graph、diagnostic、preview和CPU/GPU diff；
- reimport、undo、save/reopen、external conflict、crash recovery与cook状态闭环。

### M10 · 安全、可靠性与发行

- malicious corpus、fuzz/property、allocation/fault/soak/device-loss测试；
- static/source/native distribution行为等价或移除不真实形态；
- package signature/trust、SBOM、model provenance和runtime admission贯通。

### M11 · 竞争性性能与表现资格

- 冻结模型/输入/输出/画质/backend/hardware/workload corpus；
- 报告import/cook/RSS、CPU latency/throughput、GPU time/VRAM/compile/cache、frame impact与accuracy；
- 同语义、同硬件、同失败条件和统计协议通过后，才允许与Unreal NNE或其他成熟runtime比较。

## 10. 验收门

| Gate | 验收内容 |
|---|---|
| G01 | 无asset loader/provider/backend时，Editor与Runtime所有Neural入口均Unavailable且给出typed reason |
| G02 | catalog registration不能单独使`runtime.asset.neural_model`进入Ready |
| G03 | 任意import写盘阶段故障后，旧artifact仍可按digest加载；无半文件被asset index接受 |
| G04 | undo不按值保存无界旧model bytes，history byte budget与spill可观测 |
| G05 | ONNX parser在bytes/items/depth/string/tensor/weight/time上均先admit后分配 |
| G06 | 负维度、超范围varint、非法UTF-8、巨大length与external path均typed拒绝且零输出变更 |
| G07 | `.znn`任何count必须由section最小record和产品limit双重证明后才分配 |
| G08 | malformed/fuzz输入不panic、不OOM、不长时间挂起、不产生可接受partial artifact |
| G09 | Validated IR拒绝cycle、forward reference、重复producer、非法kind、arity/type/shape/alias |
| G10 | CPU与GPU只能消费Validated IR，不能各自补一套互相漂移的admission |
| G11 | source/recipe/dependency/converter/BuildSet/target/provider完整决定artifact key |
| G12 | external ONNX data进入依赖图、cook、digest、package与reimport判断 |
| G13 | Neural asset支持async load、cancel、last-good、generation swap与in-flight retirement |
| G14 | Model与ModelInstance分层，weight共享，workspace按instance/shape复用并有budget |
| G15 | dynamic shape prepare返回resolved output与workspace需求，非法shape在执行前拒绝 |
| G16 | CPU caller-owned bindings、deadline/cancel和numeric policy通过edge corpus |
| G17 | GPU shader由当前backend真实编译，reflection/binding/layout与descriptor一致 |
| G18 | GPU memory plan声明每个buffer size/usage/lifetime，峰值VRAM在admission前可计算 |
| G19 | Render Graph真实enqueue、barrier、queue ownership、submission和completion证据闭合 |
| G20 | resize/camera cut/device loss/model reload不会使用旧history、旧pipeline或旧buffer generation |
| G21 | 每个operator CPU/GPU/reference golden按明确tolerance通过随机与边界shape |
| G22 | FP16/BF16/INT8只有在accuracy、device capability和fallback gate后广告Ready |
| G23 | post-process输入输出semantic、format、colorspace、range、channel和resolution明确校验 |
| G24 | multi-view、dynamic resolution、temporal history、tiling和fallback通过GPU pixel golden |
| G25 | Editor import为可取消job，progress、source revision、terminal receipt与shutdown drain可见 |
| G26 | inspector显示graph/IO/opset/backend compatibility/memory/artifact lineage而非静态成功文本 |
| G27 | reimport/save/reopen/cook/play路径消费同一artifact与compiler，不存在CLI第二authority |
| G28 | native dist若继续广告Neural capability，必须与静态provider通过行为、state与teardown parity |
| G29 | fuzz、fault、OOM、soak、device loss、shader compile、real GPU与product E2E进入required test plan |
| G30 | benchmark绑定source/build/model/input/backend/device/driver/quality和统计置信信息 |
| G31 | 与Unreal比较时输入输出语义、模型、精度、画质、硬件和失败策略一致，禁止proxy workload |
| G32 | `git diff --check`、LF/BOM/trailing-space、frontmatter路径、索引、coverage和finding计数全通过 |

## 11. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Neural 41文件逐文件静态审查 | review_complete | 2026-08-16 | 7,016行、239,770 bytes、fingerprint `df857c43...9facc1` |
| Product/capability反向consumer核对 | review_complete | 2026-08-16 | 包外0个Neural model/executor/settings业务consumer；runtime/post-process registration为空 |
| 差距与重构路线 | review_complete | 2026-08-16 | 5 P0 / 60 P1 / 12 P2；M0-M11；G01-G32 |
| Production修复 | pending | - | 本篇未修改production或tests |
| 动态验证 | blocked_by_existing_build | 2026-08-16 | 公共Editor test build既有239 errors/122 warnings；本轮未重复不可达lane |
