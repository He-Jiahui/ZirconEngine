# mip 生成测试没有按当前结果合同读取纹理和诊断

这是对既有验证目标的维护，非 mip 算法产出错误或 GPU 问题。固定 ZE `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。首提交不改代码。

## 1. 从哪个正常入口可见？

[既有纹理管线的验收表](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_runtime/render/13-texture-pipeline.md#L462-L476)已列出离线颜色、法线和 mip 链的产品验证项。旧测试调用[生产处理函数的签名](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/importers.rs#L81-L96)；成功值是 `(TextureAsset, Vec<ResourceDiagnostic>)`，而不是裸纹理。[纹理和非致命诊断分别进入导入结果](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/importers.rs#L104-L129)仍区分“带可报告警告的成功”与产生 `Err` 的失败。

## 2. 实编译的影响是什么？

七处调用只用 `expect` 除去 `Result`，没有解开二元组，随后直接读取 `texture_descriptor()`、`rgba` 或 `payload`。Windows x64/MSVC 1.94.1 对无默认特性的插件 `--lib --no-run --profile test --locked` 给出 **11 个 E0599、10 个 E0609**；23 个总诊断中另 2 个来自独立 KTX1 测试。没有测试 EXE，因此这些断言（也包括同目标里按过滤器请求的邻项）在该命令下均为零执行。关联 [#219](https://github.com/He-Jiahui/ZirconEngine/pull/219) 仅提供实际编译出处，并不拥有此测试错误。其[验证记录](https://github.com/He-Jiahui/ZirconEngine/blob/482351810f7dd48f627a476c3dc003fb00608359/docs/plans/unreal-alignment/texture-array-source-refresh.md#L65)区分原生数组回归与额外插件 libtest；此处没有把未执行的用例计成产品缺陷。

## 3. 为什么不能只让它“编过”？

[sRGB 及默认链的原断言](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/tests/mipgen.rs#L14-L56)核 mip 数、filter、长度及线性域平均后的具体 RGBA；[法线链及 BC5 容器断言](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/tests/mipgen.rs#L145-L208)核法线归一、容器和 payload。成功可伴随诊断，不应回退正式返回类型，或通过抹去诊断/数值断言让测试“通过”。[Plugins07 的原生产 kernel 记录](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md#L202-L208)追踪的是生产 Box/Kaiser 参数个数，不是这里的七处测试调用。

## 4. UE 参照及局限

`ITextureCompressorModule::GenerateMipChain`（`Engine/Source/Developer/TextureCompressor/Private/TextureCompressorModule.cpp:1799–1837`）是 UE 的生产过滤路径，输入是线性 RGBA32F，会按设置选择滤波核，不是测试。`System.Engine.Texture2D.LockingWhenEmpty`（`Engine/Source/Runtime/Engine/Private/Tests/Texture2DTests.cpp:41–110`）的真实测试逐级检查有效源 mip 能被访问；`System.Engine.Texture2D.CreateTransient`（`Engine/Source/Runtime/Engine/Private/Tests/Texture2DTests.cpp:16–38`）同时验证特定无效输入的结果及诊断。没有把它们当作 ZE 的 Kaiser 数值、BC5 字节或 Rust 二元组合同的一对一测试。

## 5. 修复属于哪里？

只改这个测试文件的七个成功调用：显式分开纹理与诊断，照常对纹理做全部旧断言，并对本组正常输入应有／不应有的诊断作清楚检查。不要修改生产返回类型、压缩/mip 算法、GPU 上传或资产格式，不混入 KTX1 的变量遮蔽。

## 6. 怎样核实结果？

以 `zircon_plugin_texture_importer_runtime --no-default-features --lib` 在固定入口对照。若此特定调用失配修好，仅本叶的 **21 项**应消失。独立 KTX1 两项若还存在则编译目标仍失败，七个 mip 案没有执行，不能汇报为通过。等原目标可执行后，逐一核实 `render_mipgen_` 六例及 `render_normal_import_transcodes_default_bc5_target` 一例，保留实际颜色、长度、法线、容器字节和诊断断言；不得靠替身 runner 或合并未审分支完成验收。

本地参考为 6.0.0 / UE5 / CL 0，未知上游 revision；没有运行 UE。这些哈希固定参考文件而非发行版本：

- `TextureCompressorModule.cpp`：`f6226f8c32941aec1d94402825392151fa524ecf9ec11d041b32eb5f2b6c983f`
- `Texture2DTests.cpp`：`6ee1de61a79f5cd6a816e7e628f3c30ca7c06ab96edc06fde6a41f00f41eb25b`

## 7. 原测试的独立修正和编译器实证

[同 PR 单一测试文件改动](https://github.com/He-Jiahui/ZirconEngine/blob/0d37909af9584aadcf3dddc29c06bf738ca39de1/zircon_plugins/texture_importer/runtime/src/tests/mipgen.rs)。Windows x64/MSVC、Rust 1.94.1，`test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --no-default-features --lib --no-run --profile test --locked --jobs 1 --message-format=json`。原版包树、Runtime、接口、工作区清单与锁均取固定 main；两项独立调查起点只有各自的 Markdown 文档不同，**共用同一次真实原版编译**，不是叠加更改后的分支。

mipgen 原测试的 21 项编译诊断已消失；同一插件 libtest 从 23 项降为 2 项，剩余均为独立 KTX1 测试的原诊断。仍未生成测试程序、0 行为用例执行；仅修测试，不改生产。

- 原始编译回执 `ae1f236932d914fda6c778db8ab10196847bf050d9380d492569936aa9d1144f`：2 个 KTX1 E0618、11 个 mipgen E0599、10 个 mipgen E0609。
- 本 PR 的实际编译回执 `1a0e5558cf45f5044d4de259766bb2a2b603d8369ef98b04baa77c8624bb7d9c`：逐个比对剩余的 `2` 个主错误（路径、行、code、message 均相同），其他未改测试的 Git blob 仍与 main 匹配，没有新增其它类别。
- 七处成功值显式解构纹理和诊断；原纹理、法线、mip 字节和容器断言不改，并对这些正常输入核实没有未预期的诊断。

这不是整套测试已绿，也不是生产纹理画面验证；未用跳过、临时 cfg、替代 runner 或从其它未合并 PR 拿代码。待两个独立改动合法进入同一基线后，仍须从原 Cargo 目标启动并运行原来的实际用例。
