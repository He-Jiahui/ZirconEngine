# M01 / Draft 问题定义：材质 option 超出 u32 位预算仍被接收，枚举值在真实 shader 变体入口被截断

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

状态：**仅调查、问题定义稿**。顺序保持「独立 Draft → 同一问题的等价/契约单测 → 可行修复」；本稿不是 PR 创建、修复、测试通过或引擎验收回执。

- 源码读取基线：ZE 本机 HEAD `cb495bb1c952a24c86f4513b8099bc4942c3a87d`。lead 更正的远端基线为 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`；本地只读 `git diff --name-only e893… cb495… -- zircon_runtime zircon_plugins zircon_runtime_interface zircon_editor zircon_app` 输出为空。因此本文 ZE 链接固定为 **e893 blob**，不把流程退役提交夹入引擎问题。
- UE 本地只读版本由 [Build.version:1–10][ue-version] 确认为 `6.0.0 / BranchName UE5`。UE 子目录不是本稿已确认的独立 Git checkout，不把其向上继承的 ZE HEAD 当成 UE 提交号；以下以精确路径、符号、行号与文件 SHA256 定位。
- 已读规范 [ZE AGENTS][ze-agents]、[任务树 AGENTS][task-agents] 及工程/dev 相关规则；未触碰外来 skill/catalog 脏文件。**未改源码、未 Cargo、未运行 UE、未 Git 写、未访问远端。**

## 1. 差异、精确位置及真实可达性

**ZE 在作者输入阶段接收可超过 32 位的 option 列表，编码仍固定返回 u32。**

- [zshader.rs:55–98、216–232、439–447][ze-zshader] 的 `ZShaderDocumentV2::from_toml_str` 检查文档形状/字段，surface 的 `options` 是 `Vec<ZShaderOptionDocument>`，转换逐条保留输入；此处没有累计位预算检查。
- [import_shader_package.rs:49–50、105–151][ze-import] 的 `import_shader_package` 将这些 option 直接交给 `generate_material_artifact`，并发布生成的 `material_option_table`。这是实际资源导入入口，不是孤立辅助函数。
- [property_layout.rs:170–194、228–232][ze-pack] 的 `material_option_table` 为 bool 分配 1 位、enum 按值数计算位宽，只用 `u8::saturating_add` 增加 offset，没有验证 `offset + width <= 32`。
- [material_property_layout.rs:197–249、252–264、291–306][ze-bits] 的 `MaterialOptionTable::{default_bits,bits_for_values}`、`apply_material_option_value`、`MaterialOptionRef::definition_value_for_bits` 都对 **u32** 移位。`total_bits` 没有成为 admission 条件。

### 可判定的最小输入（源码推导，未执行）

同一个 shader 声明 31 个默认 false 的 bool，随后声明 `detail_mode` 三值 enum，`editor.values = "off,uv,triplanar"`、默认 `off`。两个材质除 `detail_mode` 外完全相同：A=`off`，B=`triplanar`。31 个 bool 均不覆写。

| 步骤 | 精确推导 |
|---|---|
| enum 布局 | `bit_offset=31, bit_width=2, total_bits=33` |
| 输入类型验证 | `triplanar` 是列表中合法字符串，`value_bits` 返回 `Some(2)`；[validation.rs:161–178][ze-validation] 只验证名称和值，不验证总位数 |
| 写入 | u32 中 `(2 & 3) << 31 = 0`；高位被截掉，**不是依赖 debug 溢出开关才触发** |
| 结果 | A、B 的 `material_option_bits` 都是 0 |
| 反解 | `(0 >> 31) & 3 = 0`，得到 `ZR_OPT_DETAIL_MODE = uint(0)`，不是作者的 2 |

### 已追到实际消费者，不只是理论 key 冲突

1. [MaterialAsset::material_option_bits_for_shader:472–475][ze-material] 调用上述编码；[ensure_material.rs:255–262、636–641][ze-ensure] 在 readiness 检查前求 bits，并把结果装入真实 `MaterialRuntime.pipeline_key`。
2. [PipelineKey:9–18、76–93][ze-pipeline-key] 的 Eq/Hash 与 `shader_variant_key_for_geometry` 都携带这一个 u32；材质作者 option map 不在后续 key 中。
3. [variant_registry.rs:120–149][ze-registry] 的 `MeshPipelineVariantRegistry::resolve_variant_for_geometry` 对该 key 执行 HashMap 查找、命中则返回原 variant id。因此同 shader/revision、同其余 PSO 条件的 A/B 会被登记成同一变体。
4. [accessors.rs:737–750][ze-def-consumer] 的 `shader_material_option_defines` 从这个 bits **反解实际编译 define**；[shader_source.rs:692–713][ze-source-consumer] 将其传入普通 material/deferred 模板请求，非诊断专用路径。
5. [ensure_pipeline.rs:235–270][ze-pipeline-consumer] 组装这些源后再计算 source hash、读取 shader module cache。内容寻址不能恢复已在此前变成 0 的 enum；即使不命中缓存，也只会编译错误的 define=0。

**UE 对应的是命名静态参数的保真与有界 permutation 输入，不是要求 ZE 增加 UE 材质系统。**

- [StaticParameterSet.h:395–428][ue-static-set] 的 `FStaticParameterSetRuntimeData::StaticSwitchParameters` 按独立条目保存静态 bool，而不是把所有命名参数压到一个 u32。
- [MaterialShader.cpp:956–1006、1092–1105][ue-material-shader] 的比较与 `SetStaticSwitchParameterValue` 保留条目；[MaterialInstance.cpp:4859–4912][ue-material-instance] 的 `UMaterialInstance::UpdateStaticPermutation` 比较完整集合、更新编译资源。
- 传统链：[MaterialExpressions.cpp:8968–8972][ue-material-expressions] 的 `UMaterialExpressionStaticBoolParameter::Compile` → [HLSLMaterialTranslator.cpp:8947–8965][ue-material-translator] 的 `StaticBoolParameter` 按 `ParameterInfo` 找值。现代 MIR 在 [MaterialExpressionsToMIR.cpp:759–788][ue-material-mir] 也从完整静态参数列表读对应 bool。

**UE permutation 上限另行区分：**[ShaderPermutation.h:96–138、451–492、540–549][ue-permutation] 的 `TShaderPermutationInt` 检查各维 value 范围，`TShaderPermutationDomain` 的总基数是 `constexpr int32` 乘积、构造函数检查 ID 范围。这不是“最多 32 个材质参数”，也不是无限维支持。31 个 bool × 3 值的全排列基数为 6,442,450,944，超过正 int32；不能把本稿 ZE witness 声称为该 UE 模板可运行的等量 domain。其常量溢出编译诊断本轮未实际验证。本问题允许 ZE **明确拒绝**超预算声明，所要求的是不静默接受后损失作者值，而非必须支持该组合。

## 2. 有界风险

- 仅针对累计材质 option 位宽超过现有 u32 容量的 shader 声明。32 位内健康布局、本稿未检查的内置 shader、普通浮点/颜色 uniform 不因此被判定有错。
- 上述 33 位 witness 在值验证成功后将合法 enum=2 变成 0，可导致错误分支、材质间错误共用 variant；没有证据称所有材质受影响、存在内存破坏或 GPU 驱动问题。
- 更多 bool 导致 offset≥32 的移位还可能触发受检查构建的 panic，但不是本稿成立所必需的证据，也不把未运行的 release/debug 行为写成实测。
- 未盘点产品资产中实际有多少超预算 shader，因此频率未知；这是一条已接收输入的确定性编码/诊断缺陷，不是“ZE 尚无 UE 高级材质”的能力差距。

## 3. 根因与 UE 实际处理

ZE 自身既有 [32 位上限/超限 import 诊断合同:165](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md#L165>) 已明确禁止隐式截断；这只是原合同佐证，不把计划文字当成运行时检查已经实现。

根因是**输入空间与表示空间脱节**：动态作者 `Vec` 的位数未经验证就生成固定 u32 编码；`saturating_add(u8)` 仅避免 u8 算术溢出，既不证明 u32 可装下，也不保证每个 enum round-trip。现有 material 类型校验只问“是否是已声明枚举值”，不问“布局能否表示它”。

非法组合被接收的时机是 [zshader.rs 的反序列化入口][ze-zshader] → [import_shader_package 的直接 artifact 生成][ze-import]；到 [ensure_material 的 bits 求值][ze-ensure] 时没有 budget 错误可供 readiness 拒绝。UE 的静态材质参数保留完整集合；UE 编译期 permutation domain 则分别约束可表达维值及 ID 范围。可借鉴原则是**在表示边界验证，不把不可表示值折成另一合法变体**，不能宣称 UE 已有本 witness 的专门测试或完整等量 domain。

## 4. Rust 可直接借鉴，不新建架构

保持已有 u32 variant ABI/缓存格式：在唯一 option 布局生成点用足够宽的计数器和 `checked_add` 验证 `width>0`、`offset<32`、`width<=32-offset`；超过预算产生具名错误或已有导入诊断，不截断、不饱和后继续、不用 mask/`wrapping_shl` 伪装成功。

错误在发布 shader artifact 前走现有 `AssetImportError` / shader readiness 通路；从缓存重建/程序化再生成也必须经过同一检查。后续编码可用 checked shift 作防御，但防御失败应明确失败，不能返回成功的 bits=0。无需 TArray、模板 permutation 系统、更多 ECS 服务或全新材质架构。

## 5. wgpu / ABI / ECS 差异

- ZE 的 option 在 CPU 上生成 `RenderShaderDefinitionValue`，再进入 WGSL 模板/变体编译；不是 WGSL uniform 的对齐问题，也不是 GPU 32 位限制要求所有作者参数只有 32 个。修复 admission 应位于 CPU contract。
- UE 材质静态参数集合、UE `int32` permutation ID、ZE u32 option bits 是三种不同表示。不能把 UE 的 31/32 位整数容量直接套成 ZE 的维数规则。
- 不扩大 `PipelineKey.material_option_bits`，不改动态运行时函数表，不跨 ABI 传 UE 对象或 Rust trait/wgpu 对象；`Result`/诊断传播是进程内 Rust 实现选择，仍需复核缓存反序列化再生成调用。
- ECS 只消费准备后的材质句柄与 draw snapshot，本问题无需更改组件生命周期、World 调度、mesh affine、TBN 或 draw ABI。

## 6. ZE 具体修改草案（本轮未实施）

| 现有落点 | 最小职责 |
|---|---|
| [property_layout.rs::material_option_table / generate_material_artifact][ze-pack] | 分配前检查总预算，记录越界 option 名、所需位数与剩余容量；不生成可误用的成功 table |
| [import_shader_package.rs::import_shader_package][ze-import] | 传播布局失败到既有导入错误/diagnostic，在错误 artifact 发布前停下 |
| [shader_asset.rs::regenerate_material_artifact:125–138][ze-regenerate] | 程序化/缓存再生成同样拒绝非法 table；不清空错误并当作默认成功 |
| [material_property_layout.rs 的 bits/definition 编码][ze-bits] | 校验已构造 table 的边界；合法 32 位内行为不变；拒绝而非默认 0 |
| [material/validation.rs][ze-validation]、[ensure_material.rs][ze-ensure] | 将超预算声明判为不可用，并复核先算 bits 后 readiness 的时序，防止检查太晚 |

应先冻结 witness、完成 Draft 审核，再添加下表中的测试；不能为“绿色”把跨界输入自动截断/丢弃 enum，或把现有小表回归改为只比较错误实现的镜像结果。

## UE 既有测试迁移表与 ZE 落点

**没有找到本问题的 UE 既有等量测试；以下不是迁移回执。** 搜索范围：UE Engine 的 Private/Materials、Private/Tests；RenderCore 的测试命名文件；Plugins/Tests 的静态参数/StaticSwitch 查询；Source 下 `*ShaderPermutation*Test*` 路径；另读 MaterialBaking 既有测试。未搜索/执行所有二进制功能资产与所有平台测试，不声称 UE 仓库绝无相关测试。

| UE 既有测试名 / 路径行 | 输入 | 期望 | ZE 现成落点 / 迁移判定 |
|---|---|---|---|
| `FExportMaterialProxyAnisotropyTest` / `MaterialBaking.ExportMaterialProxy.ShaderDependencyConsistency`，[ExportMaterialProxyTest.cpp:24–66][ue-baking-test] | 灰色 0.5、DefaultLit、同步编译后创建 BaseColor baking proxy | shader 依赖一致、无 assertion | 与 option 预算无关，**不迁移、不借其名声称覆盖** |
| 本次未找到 named-option 编码预算/33 位拒绝的既有 UE 测试 | — | — | 本稿后续用**源码推导契约**，不是“UE 单测迁移” |

| 拟测契约（未新增、未执行） | 输入 / 期望 | ZE 已存在测试落点 |
|---|---|---|
| M-T1，合法边界 | 30 bool + 3 值 enum=2：恰好 32 位；编码为 bit31，反解仍为 2，生成 define=2 | [property_layout 测试:106–127][ze-pack-tests] 与 [option_lookup 测试][ze-option-tests] |
| M-T2，非法跨界 | 31 bool + 3 值 enum；现源码推导 2→0。修复契约为 artifact/admission 明确拒绝，名称/位预算诊断完整，不进入 variant registry | 同上；导入整链放 [compound_shader.rs:5–75][ze-import-tests] 现有复合 shader harness |
| M-T3，消费者见证 | 对合法边界生成的 0/2 两值，核对真实 `shader_material_option_defines` 与 registry variant id 不混同；非法声明不能进入该阶段 | [renderer/mesh/tests/material_options.rs:5–55][ze-consumer-tests]，并复用其既有 variant registry，不造新渲染器 |
| M-T4，再生成 | 缓存/程序构造非法 option schema 应受控失败；小表、bool/enum 默认位不变 | [property_layout 测试][ze-pack-tests]；[shader_asset.rs 的再生成入口][ze-regenerate] |

现有覆盖注意：`render_shader_option_table_packs_bool_and_enum_bits` 只覆盖 3 位；[option_lookup.rs:48–65][ze-option-tests] 的忽略性能例虽有 512 条，显式用 `index % 32` 复用 bit，并非 512 个独立 option 的正确性证明；[material_options.rs][ze-consumer-tests] 覆盖预置 bits=0/1，未从越界作者 schema 出发。本轮所有这些测试均**只读、未运行**。

## 模块 / owner / 覆盖与未知

| 模块 | 真实 owner / 依据 | 本轮覆盖 | 未知 / 排除 |
|---|---|---|---|
| 材质作者/实例、属性值 | runtime asset material/shader；[MaterialAsset][ze-material]、[ShaderAsset][ze-regenerate] | 参数到 artifact、校验、实际 mesh 消费链 | 未渲染材质球预览，不断言完整材质球 UI 功能 |
| shader 变体/管线 | runtime graphics ResourceStreamer / MeshPipelineVariantRegistry；[ensure_material][ze-ensure]、[registry][ze-registry] | bits → key → define → 源组装/缓存入口 | GPU/产品帧与磁盘 cache 运行状态未测 |
| 材质编辑器 | 独立插件 workspace 的 [material_editor/editor/Cargo.toml][ze-editor-cargo]；图/实例作者 helper [shader_compilation.rs][ze-editor-compile]、[instance_projection.rs][ze-editor-instance] | 已核真实路径及其 Runtime contract 委托，不把插件误当 runtime 内置 | UI 点击/撤销/保存/预览 owner 联调未测，不另报第二问题 |
| 构建边界 | [plugins workspace:66–81、164–170][ze-plugins-cargo]、[runtime graphics feature:177–183][ze-runtime-cargo]；UE [Engine.Build.cs:73–109][ue-engine-build]、[RenderCore.Build.cs:18–26][ue-rendercore-build] | 确认 crate/模块依赖，UE Engine 依赖 RenderCore/RHI | Cargo/UBT/编译均未执行；人工 CODEOWNER 未查明，不虚构人名 |

去重：已读 lead 提供的 [PR snapshot][pr-snapshot]（SHA256 `B374EE7808D92536D2680A200DE47479F5A458015F568562465B209C0AB0EF30`）。open #135/#144 为普通 mesh/editor affine，#138 为 CPU side-plane，#140/#141/#143 为 TBN/Mikk/fixture，#137/#139/#142 为几何测试编译/载荷/插件 API，#145 为物理写回，#136 为流程。M01 是材质 option 的表示/admission/define，不修改这些问题文件，不借 mesh/TBN 作第二 issue。snapshot 只涵盖其所含 #131–145，不能声称已遍历全部历史 PR。

## UE 源文件指纹（仅短说明，不复制实现）

| 文件 | SHA256 |
|---|---|
| [Build.version][ue-version] | `3DCD8B6654872D013F2B00AA5F0EF1E0DB89D4E8627CE820554675A27F228EA6` |
| [Engine.Build.cs][ue-engine-build] | `1C6A698C731CA203FC9648A53C13BCC3768FAE1FB91CD9EC658A3ABFA962E212` |
| [StaticParameterSet.h][ue-static-set] | `5F26DEFCD61BE6ECA75944895BCADB102087550FB9BF117AF325029F9C905DE7` |
| [MaterialShader.cpp][ue-material-shader] | `03BFB2BA34C5F95B6966DA548C0EDD013B79F857926225A6D6665F0E46FA572E` |
| [MaterialInstance.cpp][ue-material-instance] | `D3EE5244AFF7F059D5CDB0EC4CCB9F9CFEF56BA996D624B8CA2F3CE00CF5EDD6` |
| [HLSLMaterialTranslator.cpp][ue-material-translator] | `41C8ED07B7B5EE4E27BB2174E8AC6EEE139F0C34C4D41B1A5A35A6B9E3B5637A` |
| [MaterialExpressions.cpp][ue-material-expressions] | `54AE761435C1347F4E53DBE38C77FE887D23B98D65C06DB075E78B6C9CF28DD8` |
| [MaterialExpressionsToMIR.cpp][ue-material-mir] | `C96D4C27CC36F8C7BC84C6A0929014B23E465754B345A6EAE7A03AE765CCF694` |
| [ShaderPermutation.h][ue-permutation] | `EE3AB3AB6B424CC388F25D8EFCFF9F6723B565292042D74E23A4A30B20FD52FC` |
| [RenderCore.Build.cs][ue-rendercore-build] | `52AB046FD064FFEB48C111E60A1D7E3F63E5C529AF3CABC539F0455B56C3D13C` |
| [MaterialBaking.Build.cs][ue-baking-build] | `2DCC2255FFE58322AD76EE5B0ADACD7936101041B24809F97AEC97DEFD388885` |
| [ExportMaterialProxyTest.cpp][ue-baking-test] | `18085E2472B9122C5A349222E6F99AB2232AEE84A7CE784B85BE936E88788F06` |

测试模块补充：[MaterialBaking.Build.cs:9–24][ue-baking-build] 依赖 Engine/RenderCore/RHI；上表既有测试不是独立无渲染单测，因此未移植其运行条件，也没有据其源码声称本机 UE 已通过。

[ze-agents]: <E:/Git/ZirconEngine/AGENTS.md>
[task-agents]: <D:/CodexData/worktrees/ze-ue-audit-20261117/AGENTS.md>
[pr-snapshot]: <D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>
[ze-zshader]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/shader/zshader.rs#L55-L98>
[ze-import]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/ingest/import_shader_package.rs#L105-L155>
[ze-pack]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/shader/property_layout.rs#L170-L249>
[ze-bits]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/render/shader/material_property_layout.rs#L197-L306>
[ze-validation]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/material/validation.rs#L161-L178>
[ze-material]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/material/material_asset.rs#L468-L476>
[ze-ensure]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/resources/resource_streamer/ensure_material.rs#L255-L276>
[ze-pipeline-key]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs#L9-L93>
[ze-registry]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/mesh/mesh_pipeline_cache/variant_registry.rs#L120-L149>
[ze-def-consumer]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/resources/resource_streamer/accessors.rs#L737-L750>
[ze-source-consumer]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/mesh/mesh_pipeline_cache/shader_source.rs#L692-L713>
[ze-pipeline-consumer]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/mesh/mesh_pipeline_cache/ensure_pipeline.rs#L235-L270>
[ze-regenerate]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/shader/shader_asset.rs#L125-L138>
[ze-pack-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/shader/tests/property_layout.rs#L106-L127>
[ze-option-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/render/shader/tests/option_lookup.rs#L7-L65>
[ze-import-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/tests/project/zmeta/compound_shader.rs#L5-L75>
[ze-consumer-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/mesh/tests/material_options.rs#L5-L55>
[ze-editor-cargo]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/material_editor/editor/Cargo.toml#L1-L13>
[ze-editor-compile]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/material_editor/editor/src/shader_compilation.rs#L63-L85>
[ze-editor-instance]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/material_editor/editor/src/instance_projection.rs#L219-L265>
[ze-plugins-cargo]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/Cargo.toml#L66-L81>
[ze-runtime-cargo]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/Cargo.toml#L177-L183>
[ue-version]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L10>
[ue-engine-build]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Engine.Build.cs#L73-L109>
[ue-static-set]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/StaticParameterSet.h#L395-L428>
[ue-material-shader]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Materials/MaterialShader.cpp#L956-L1105>
[ue-material-instance]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Materials/MaterialInstance.cpp#L4859-L4912>
[ue-material-translator]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Materials/HLSLMaterialTranslator.cpp#L8947-L8965>
[ue-material-expressions]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Materials/MaterialExpressions.cpp#L8968-L8972>
[ue-material-mir]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Materials/MaterialExpressionsToMIR.cpp#L759-L788>
[ue-permutation]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ShaderPermutation.h#L451-L549>
[ue-rendercore-build]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/RenderCore/RenderCore.Build.cs#L18-L26>
[ue-baking-build]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MaterialBaking/MaterialBaking.Build.cs#L9-L24>
[ue-baking-test]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/MaterialBaking/Private/Tests/ExportMaterialProxyTest.cpp#L24-L66>

## 7. 真实注册导入器、磁盘载荷与重开：原版 2 过 3 败

[仅测试提交 `658212a20bf1d7e41813a3dd916baeae90e7e83f`](https://github.com/He-Jiahui/ZirconEngine/blob/658212a20bf1d7e41813a3dd916baeae90e7e83f/zircon_runtime/src/asset/tests/project/zmeta.rs) 与[测试实现](https://github.com/He-Jiahui/ZirconEngine/blob/658212a20bf1d7e41813a3dd916baeae90e7e83f/zircon_runtime/src/asset/tests/project/zmeta/option_budget.rs)。真实文件系统工程，内置 `zircon.builtin.shader.zshader_package` 导入合法 Surface WGSL；实际 `ProjectManager::scan_and_import`、`ArtifactStore::read`、材质值编码及 `definition_values_for_bits`。Windows x64/MSVC、Rust 1.94.1、test profile，`test -p zircon_runtime --no-default-features --features target-server --lib material_budget_ --locked --offline --jobs 1 -- --test-threads 1 --show-output`。5 个用例：**2 过、3 败**，0 忽略，7,889 过滤。

| 真实输入/操作 | 原版结果 |
|---|---|
| 30 个 bool + 3 值 enum，总宽 32 | `off/uv/triplanar` → bits `0 / 0x40000000 / 0x80000000` → `ZR_OPT_DETAIL_MODE=0/1/2`，通过。Compute 与 Fullscreen 作者文档各自接受 33 个 bool 的解析正控也通过。 |
| 31 个 bool + 同 enum，总宽 33 | Source 和预览均 `Ready`，有制品、诊断空；bits `0 / 0x80000000 / 0`，实际定义 **0/1/0**，不是 0/1/2。 |
| 编码一个真实 cache：保留旧的 32 位派生表，只给原始 `options` 加回一个 bool | 当前二进制 Shader chunk 可完整读取；解码再生出 33 位表并再次把 2 读成 0。不是损坏压缩头或凭空模拟 cache miss。 |
| 保持磁盘 `.zshader` 为可表达的 32 位版本，仅换入上述旧式载荷，另开一个 ProjectManager 并全扫 | 根仍是 `Ready`，table 仍为33，chunk 内容散列不变；未用合法源修复。 |

两例各生成的失配载荷均为 3,359 原始字节、1 个 chunk、内容散列 `ec112b8f85b657de98d825e7936592ff257c34ad1daa41de4756ac0d488d5616`。未构造 32 位移位 panic，不跑整个渲染器、预热 CLI/GPU、UE；验到的是实际要交给变体的值，而不是着色后的屏幕颜色。该三项行为失败与 RenderCore 的编译期排列数量上限无等量迁移关系。


38 项选定输入；实际回执 SHA-256 `d5594c3d3c190c39d6aea29900dfab28158c6c25278666cd306a1074547c21e2`，stdout `9bf54256b00510afa16152fd5967aef5345ca1facd5316d3b024359e91b9e8a7`，stderr `4d5149857669912a653dfa1a63468b6db541e0b0f5d277e49bd650c82278f1ae`。

## 8. 修复：在 Surface 作者入口和持久载荷读回时验证同一位预算

[生产修复 `02b6afc0e0e84924e4a0cb3902146b45fcde13ec`](https://github.com/He-Jiahui/ZirconEngine/blob/02b6afc0e0e84924e4a0cb3902146b45fcde13ec/zircon_runtime/src/asset/assets/shader/zshader.rs) 只在会失败并能给出诊断的入口检查原始 Surface `options`。校验与布局生成共用原有 option kind、枚举拆分及位宽计算；超过 `u32::BITS` 报首个越界名字、累计位数和上限。缓存读回先检查声明，再生成材质数据，不能信任序列化里看似合法的 `total_bits`。未扩大变体键、改变缓存格式或向 GPU ABI 加字段。

复跑 §7 的相同五案，测试文件和命令不变，**2 过 3 败 → 5 过 0 败**。恰好 32 位保留 `0x80000000 → enum 2`；不参与 Surface 材质变体的 Compute/Fullscreen 原文仍可解析。33 位 Surface 返回 `Error`，不发布根制品，诊断命名 `detail_mode` 及所需 33 / 上限 32；恢复同一份 3,359 B `ec112b8f85b657de98d825e7936592ff257c34ad1daa41de4756ac0d488d5616` 磁盘载荷时，直接读取得到对应的预算错误。保持作者源合法后重开工程，同一 locator 的错误载荷被重导，新的 root 内容散列 `d24823352d4efa71d238bd7c1d39e96a257c15bb1f7baea66fc13f2d2f914bd2`，32 位与 3 个值均复原。

本修复不把公开的 infallible Rust 构造器或直接赋值改为新 ABI；绕开正常导入与持久读回自行捏造的表不在保证内。冷 prewarm 从同一作者解析入口可获得诊断是源码结论；本轮没有运行 CLI、warm inventory、设备端 shader 或 UE，不称整个材质系统已验收。

38 项选定输入；实际回执 SHA-256 `7fefbc54bd75ee12a9d227b9b9a2f98d02cd7d2afbe6f0bb5a4904a985e3e130`，stdout `6a30f40fb25468141080568348a72198b4489979fdb5edfdd6819c51eb6053b8`，stderr `82385cb84b3d685fdda503246c3a5738b75b619c9468d21621fd76b93f8a5b78`。
