# 纹理数组：全量重扫须读取已更改的层源图

固定 ZE `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。本 Draft 是对既有 [Plugins07 的 IMP-P1-016/017](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md#L225-L238)、[TEX-P1-29](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming.md#L188-L203) 的具体验证，不是首次提出这项需求。初次提交只有源码调查及回归设计；下文的预期颜色不是已运行的测试结果。

## 1. 哪个正常操作受影响？

启用且**真正链接**首方 Texture Importer 的宿主打开项目。作者使用 `.zarray` 的两种已支持写法之一：有序 PNG 列表，或一张按行切片的 PNG。只改其中一层的颜色，不改路径、尺寸、层序或数组清单，再执行全量重导。此时独立 PNG 可以更新，数组却可能继续读出旧色。[作者合同](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/crates/zircon_runtime/asset/assets/texture/cubemap-array-authoring.md#L34-L82)、[array@2 清单](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/plugin.toml#L89-L96)、[真实 handler 注册](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/plugin.rs#L72-L79)、[显式链接的报告生成](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs#L34-L56)。仅有内置元数据、DLL 或渲染用 Texture 插件并不等于该导入器已安装；不声称 stock 桌面编辑器总是走这条路径。

## 2. 成品应当是什么？

数组不是“将来再去取原图”的两个 locator。[解码](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/manifest_source.rs#L17-L37)实际读取 PNG 字节，[按清单顺序取图](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/array.rs#L48-L71)、[合并分层 RGBA](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/texture/array_asset.rs#L128-L137)，最后写入 canonical Texture。正常结果是改动的层变色、其它层和数组 UUID 保持；不能以 `Ready`、时间戳、URI 或预览缓存替代[从磁盘成品读取](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/artifact_access.rs#L10-L37)。本问题在 GPU 之前，区别于 [#180](https://github.com/He-Jiahui/ZirconEngine/pull/180) 的“CPU 已发布新纹理、渲染端误命中旧预备缓存”。

## 3. 当前为什么会保留旧色？

[正式项目启动](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/open_project.rs#L19-L35)先装已选注册表，随后调用与[公开全量重导](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/manager/service_contracts/asset_manager_contract.rs#L157-L179)相同的项目代际准备。数组是[单文件来源](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs#L238-L252)。[辅助输入发现](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs#L502-L565)现只对 glTF / 复合 shader 等取得额外源字节，没有把普通数组清单中的 PNG 纳入[父输入摘要与动作键](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L172-L222)。[恢复条件](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L552-L605)满足后[读取既有磁盘成品并跳过 handler](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L227-L307)，并非重新解码 PNG。[导入输出](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/importers.rs#L150-L158)与[事后运行时依赖提取](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/registry/dependency_extractors/mod.rs#L44-L70)也不能让已经返回的判定失效。

## 4. Unreal 哪一段可对照？

本地 `Build.version` 是 6.0.0 / UE5 / CL 0，未知上游提交；以下路径和哈希仅固定本次参考树。`Engine/Source/Editor/UnrealEd/Private/Factories/EditorFactories.cpp:4278–4295`：旧纹理导入工厂在源纹理覆盖后查找引用它的**已加载** Texture2DArray；`Engine/Source/Runtime/Engine/Classes/Engine/Texture2DArray.cpp:448–495`：按顺序从各源 mip 0 拷入对应切片并更新资源。这不是对未加载资产、ZE 的清单格式或冷启动行为的等价保证。

| 既有 UE 原测（只读，未运行） | 借用的真实判断与限制 |
|---|---|
| `Editor.Auto Reimport.Simple Modify`，`Engine/Plugins/Tests/EditorTests/Source/EditorTests/Private/UnrealEd/AutoReimportTests.cpp:220–292` | 同路径替换正常 PNG，文件缓存报告修改并提交。它不读取数组成品。 |
| `Plugins.PCG.Texture.Array.Readback`，`Engine/Plugins/PCG/Source/PCG/Private/Tests/Data/PCGTexture2DArrayReadbackTest.cpp:44–129` | 需要非空 RHI，用不同纯色区分切片并核对读回内容和顺序。它测试临时纹理，不测源文件重导；ZE 的磁盘 RGBA 测试也不宣称验证显卡。 |
| `DerivedData::Cache::Hierarchy::MemoryCacheServesGetThroughHierarchy`，`Engine/Source/Developer/DerivedDataCache/Tests/DerivedDataCacheStoreHierarchyTest.cpp:488–561` | 缓存层级按同一内容读回，作为冷、热对照；不是数组源变更测试。 |

factory `a8e302ed7c83e2639862831f32d31b255efacda188b1e26404e3fcbf8ab879e2`；array `a6fed8902e798fc580fdbfd7170a15cc9f8ad8c10a232558acce3dadd30f6705`；watch `c059fd611676ae2ca09efaa6bdf37c5e23be09d0ec5969a1207e4d7ebacfa3f5`；readback `05f11de69083f5aa0067ad28950435cc1a056751e8c7057da30844bb25e5e0f8`；ddc `c43152a70d03e17b441a1e3ece47f27f5aa814db26cd9c9c361491b7428c86e6`。

## 5. 怎样证明错误，而不是猜测？

只在真实插件和资产服务都已安装的隔离 CPU 项目中，创建两层 1×1、颜色可区分的 PNG 列表与 `.zarray`。先确认子图和数组均能经 `res://` 与实际 `lib://` 读取，描述符是两层 D2；未改再扫应稳定。只替换一层的同路径 PNG，另一层不动，用相同服务做全量重导，不靠后台 watcher 触发。要求独立 PNG 就是新色、未改层未变、主清单 SHA 和同根 UUID 不变，再核数组磁盘字节与父动作键。用从未有制品、但已放入新 PNG 的第二个工程证明正确层色和次序，不跨工程比较 UUID。旧根新开是持久恢复，不冒称空缓存。可另补原有[单图纵切用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/tests/cubemap.rs#L121-L142)和旧根重开对照，但它们不能替代此生产入口。

## 6. 修复边界与不做什么

由被选择的导入器声明它真正消费的辅源，宿主只应把这些**同一次读取**的来源与内容纳入恢复前的动作键；解码器必须消费同一快照。保持首扫、未改命中、增删或重排层、已有旧缓存升级的语义。不在 Runtime 中硬编码可选插件的 TOML 格式，不用无条件重导、删缓存或事后多写一条依赖来“修好”。如果需要扩展跨包导入器合同，应先定其兼容边界；本 Draft 不引入公开 ABI。TOML `.zcube` 的外图分支为同类待验证项，不据此拆第二个 PR，也不把它的二进制格式、GPU、Cook 或导出列为已验收。

## 7. 原版：真实公开入口的行为记录

[array_layer_refresh 的测试原文](https://github.com/He-Jiahui/ZirconEngine/blob/eb023fb1f421f3c339680f54448e538dc901f813/zircon_plugins/texture_importer/runtime/tests/array_layer_refresh.rs)。测试经首方 `plugin_registration` 安装 image/array handler，在独占目录新建暖、冷两个工程，通过 `AssetManager::open_project` 和显式 `reimport_all` 执行。初扫与未更改再扫、输入 PNG、原图重新烘出新色、持久 UUID、未改层、顺序、实际 `res://` 与 `lib://` 内容、冷工程均在主断言之前通过。只改 `a.png` 红→蓝，清单 `sources = ["b.png", "a.png"]` 保持原字节。旧工程内数组 `00ff00ff ff0000ff`，初始就有新 PNG 的独立工程 `00ff00ff 0000ffff`。不使用后台 watcher 作为触发条件，也没有把在旧根 reopen 冒称本次冷对照。源码审查没有找到在恢复前执行的插件源发现钩子；不在 Runtime 写死本插件格式，不删除缓存或移植整套 DDC。本提交不改生产。

**0 过、1 败。** 只编译 `zircon_plugin_texture_importer_runtime` 对应集成目标，关闭默认特性、无额外 feature，`test` profile，Windows x64/MSVC 1.94.1。用同一编译的测试程序实际执行 `array_layer_source_ --ignored --test-threads 1 --show-output`，非 `--no-run` 通过数。编译回执 `073aa29460f2f128cc9948388a693df7d5ea860d8c0892db08aea60d0e88c4df`，可执行文件 SHA-256 `2dc39e2c4a37fe300dd49a697710b8922e7facd1d28b238cfb7a6349f4e5c49c`，实际测试回执 `866c662ce77b307b9233822989928992a33a8abf292f84aaee4d28dfad59198d`，stdout `eae103e2a0c5dce0e93d5979a47438056d756a14cc31719591f372ae5951a96f`。UE 原测未运行。

## 8. 同一动作键对应了两份不同的层像素

测试退出后保留了暖工程与独立冷工程的六个真实容器及其 sidecar。复核了同一执行的原始 stdout 与盘上文件：两个工程最终的两张 PNG 和数组清单内容逐一相同；各自拥有自己的项目/资产 UUID，不靠跨工程 UUID 判定。

| 真实数组记录 | 源指纹 | 动作键（BLAKE3，省略前缀） | 读出的内容摘要 | 两层 RGBA（绿在前） |
|---|---|---|---|---|
| 初始暖扫、无变更再扫及替换 PNG 后的暖扫 | `afcc013aa509c9f0` | `f94d446e36680144ecbf3f28c4deae2968913c5eeec0e8d1de42f266a219771c` | `9eb34ac8507f00ce2f4e9136bb07add9744ca9c38becd6720d06d30a28412673` | `00ff00ff ff0000ff` |
| 先放入蓝色 PNG、从无制品开始的新工程 | `afcc013aa509c9f0` | `f94d446e36680144ecbf3f28c4deae2968913c5eeec0e8d1de42f266a219771c` | `f3f4a3c5c9f2cf9d2bc262d77f168f5c88787e39caf1fbed113c2045af227609` | `00ff00ff 0000ffff` |

暖工程改色 PNG 自己已经变蓝，且源指纹、动作键和制品摘要均改变。**父数组的动作键没变，但冷工程按新 PNG 生成了不同内容**，这才是缺失输入的证据，而不仅仅是预览没刷新。表中摘要来自真实 artifact inventory，并非容器文件的通用 SHA-256。没有测试 GPU 或修改引擎。
