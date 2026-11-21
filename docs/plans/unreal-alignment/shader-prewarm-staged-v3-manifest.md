# 发布验收把当前 v3 着色器缓存报为缺少元数据

## 1. 用户可见的失败

在 Windows x64 上，用公开主干 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 的普通 `zircon_shader_prewarm` 生成打包用的着色器缓存。程序退出码为 0，六个 pass 都写出了制品。把同一份报告、缓存和导出的资源注册表交给打包流程使用的 Python 验收函数，它把六个压缩 WGSL 文件全部列入 `missing metadata for WGSL artifacts`。这会阻止使用当前格式的制品通过着色器交付检查。[从预热到验收的调用路径](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build.py#L727-L749)

## 2. 复现与物理输出

在空的 stage 目录中，给普通预热程序指定标准 `ZirconEngine/cache/shader_variants`、报告和注册表路径，设置 `--builtin-fallback --quality-tier medium --geometry-source static`。`forward`、`gbuffer`、`depth_prepass`、`shadow`、`velocity`、`taa_reactive_mask` 各有一次请求，一次写入，零失败。每一项都有实际的 `.manifest` 和 `.wgsl.zst`。报告 SHA-256 为 `190fedd1cce4266a56bf09073fb855bd5d1e2da34ebc53e1adc2972473d63c1c`。再次读取三份来自真实项目材质的 forward 变体也复现了相同的文件配对错误。[正式 stage 目录、报告及校验顺序](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_shader_prewarm_acceptance.py#L37-L88)

## 3. 发布格式和读取错误的位置

Rust 运行时写出 `v3/<分片>/<hash>.wgsl.zst`，再用同目录 `<hash>.manifest` 提交这个条目，最后读回确认。正式清单携带源码身份、变体、工具链、压缩与解压长度、内容摘要。提交前单独出现的 payload 属于尚待发布的条目。提交后的 manifest 必须有对应的 payload。Python 验收器从每个 payload 反推 `.meta`，还要求 schema 为 1、目录为 v1。仅改变查找的文件后缀仍会在版本与路径检查处失败。[运行时写入及回读](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/shader/variant_cache/disk.rs#L217-L291) · [Python 查找旧元数据](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_shader_prewarm_cache_artifacts.py#L495-L539) · [版本与目录校验](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_shader_prewarm_cache_artifacts.py#L562-L657)

## 4. Unreal 的对应实现和现有测试

本地 Unreal 参考树版本为 `6.0.0 / UE5 / CL 0`。`Engine/Source/Runtime/RenderCore/Private/ShaderSerialization.cpp:187–214、318–338` 中，`BuildCacheRecord` 在一个 key 下保存 shader 对象和已编号的代码值，`ReadFromRecord` 按对应身份读取。

`Engine/Source/Developer/DerivedDataCache/Tests/DerivedDataCacheStoreTest.cpp:1420–1457` 的 `DerivedData::CacheStore / FileSystem` 使用真实文件系统存储运行完整和部分记录测试。其 `961–1008` 行验证缺失数据的记录在默认策略下返回 `Error`，值为空。显式允许部分记录时仍返回 `Error`，并提供可用描述。这里复核了 UE 原测源码。ZE 的六个 pass 和报错均来自本地实际运行。

## 5. 与现有 PR 的边界

#152 处理材质位域超出上限。#253 处理首次解析材质失败后下一次预热错误跳过。这里的普通预热已经成功，错误位于交付工具读取已完成制品的版本判断。#244 则核对运行时发行目录中资产的来源。三个问题可以分别用各自的生产入口验证。[材质位数 #152](https://github.com/He-Jiahui/ZirconEngine/pull/152) · [失败重试 #253](https://github.com/He-Jiahui/ZirconEngine/pull/253) · [发行资产 #244](https://github.com/He-Jiahui/ZirconEngine/pull/244)

## 6. 修改范围与验收

让正式 stage 按报告的每个变体 hash 查找当前 v3 清单及其同分片 payload，再核对变体身份、来源、工具链、格式和实际压缩长度。旧报告可继续使用 v1 诊断。用发布前后的同一批真实文件验证交付检查，同时覆盖缺清单、缺 payload、错误分片、过大清单、错误版本或来源、仅有 v1 制品等输入。原有六个 pass、画质、几何来源与注册表检查继续参与同一次验收。[验收的维度与来源](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_shader_prewarm_acceptance.py#L37-L82)

## 7. 同一批 v3 制品的修前、修后验收

正式发布验收现按报告逐项检查已提交的 v3 清单与同分片 WGSL。Windows 同一批六个材质 pass 的真实物料由拒绝变为通过，文件、报告和注册表的原始字节未变；11 个回归用例和 58 个既有用例全部通过。

在 Windows x64 使用固定基线已封存的普通 `zircon_shader_prewarm`（不重新编译），以默认内建着色器、`medium` 画质和 `static` 几何生成标准 stage。真实进程退出 0，`forward`、`gbuffer`、`depth_prepass`、`shadow`、`velocity`、`taa_reactive_mask` 各一次请求和写入，共 **6 请求、6 写入、0 失败**。以同一目录原有报告、资源注册表、6 个 `.manifest` 和 6 个 `.wgsl.zst` 两次调用正式 staged acceptance，没有替换或重生成物料。

| 对照 | 原验收 | 修订后 |
|---|---|---|
| 六个材质 pass 的真实暂存物料 | 把全部 6 对判为 `missing metadata` | 同一正式验收入口通过 |
| 合法项目材质的三个真实 forward 对 | 下级缓存检查拒绝全部 3 对 | 下级缓存检查通过（不是另一组六 pass） |
| Python 回归 | 仅 v2 拒绝正控通过；其余用例触发旧判定或旧诊断 | 11/11 |
| 相邻既有用例 | 58/58 | 58/58 |

正式 stage 只接受当前 v3，按报告 hash 读取提交清单、同分片的不可变 payload，并核对变体、来源 ID/摘要、工具链、声明预算、压缩字节数和 Zstd 帧头。没有清单的已报条目不算提交，已提交但缺 payload 不是成功；旧 v1 只保留历史诊断，不能证明当前运行时可读。另用独立只读核验解压上述九对并核对实际 BLAKE3，记录的物料 SHA-256 修前修后相同；Python 发布检查本身不重新做运行时的解压、BLAKE3 或 GPU 验证。

[仅回归提交](https://github.com/He-Jiahui/ZirconEngine/blob/d1689f0542564bf258e362a4ea7ac3a9a218e23b/tools/tests/test_zircon_build_shader_prewarm_v3_artifacts.py) · [正式验收入口](https://github.com/He-Jiahui/ZirconEngine/blob/71d5fff6500a89f6804f058fe20b0ec29d3885d4/tools/build/zircon_build_shader_prewarm_acceptance.py#L37-L84) · [v3 物料验证](https://github.com/He-Jiahui/ZirconEngine/blob/71d5fff6500a89f6804f058fe20b0ec29d3885d4/tools/build/zircon_build_shader_prewarm_v3_artifacts.py#L25-L68) · [运行时字节回读](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/shader/variant_cache/disk.rs#L302-L352)

## 8. 用真实 v3 制品复核错误放行

以 `ed4f902448be0f3d01f96ec4ba6d808256805af5` 为审查基线。用上述 6/6/0 stage、标准 `BuildConfig` 和正式 staged acceptance 入口复测。原样物料修前修后通过，报告、注册表和六对制品的 14 个 SHA-256 全部不变，另用独立 Zstd 解码核对解压长度与 UTF-8。只改临时副本中的 `forward` 对：

| 改动 | PR 审查基线 | 补丁后 |
|---|---|---|
| 在原有 `schema_version: 3` 前重复写入 `schema_version: 2` | 错误放行：Python 保留最后一个值，Serde 拒绝已知字段重复 | 拒绝，指出重复字段 |
| 清除真实 Zstd 数据块的末块位，保留文件长度和魔数 | 错误放行；独立 Zstd 解码报告流在终止标记前结束 | 拒绝，指出缺失块头 |

[Python 回归](../../../tools/tests/test_zircon_build_shader_prewarm_v3_artifacts.py)用 `python -B -m unittest tools.tests.test_zircon_build_shader_prewarm_v3_artifacts` 执行。修前 8 个断言/子断言发现错误放行，修后 16/16；排除下述两个既有入口错误后，相邻 Python 批次在 3.12 和 3.14 各通过 142/142。Python 仍只校验帧结构及可用的帧声明长度，不解码压缩块或验证其 BLAKE3；运行时及 GPU 验收边界不变。

144 个非编译用例的全批次在旧公开基线上有两个 CLI dry-run 用例提前被退役 Jenkins 入口拒绝（`functional_scope_required`），该入口的移除已在 [#136](https://github.com/He-Jiahui/ZirconEngine/pull/136) 跟踪。未恢复服务、未修改入口或 CI，未重跑完整打包/新构建。
