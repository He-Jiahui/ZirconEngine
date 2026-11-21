# KTX1 两个测试把同名局部上下文误当作工厂函数

验证门维护，不是 KTX1 解码器行为报告。固定 ZE `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。初次提交只有定位和验证方案。

## 1. 哪个入口有问题？

纹理导入插件的 Rust libtest 要将整套测试编译成同一可执行文件，然后按名字筛选。[构造真实导入上下文的测试 helper](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/tests/ktx1_plugins07_ktx1_hotpath.rs#L5-L17)是函数。[metadata 有效/截断对照](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/tests/ktx1_plugins07_ktx1_hotpath.rs#L161-L178)和[mip 大小有效/截断对照](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/tests/ktx1_plugins07_ktx1_hotpath.rs#L180-L193)分别先用它建立一份有效输入，却把结果也命名为 `context`；随后建立第二份输入时，名字已经指向 `AssetImportContext` 的值，而非可调用函数。

## 2. 实际影响和证据

Windows x64/MSVC 1.94.1 上，`-p zircon_plugin_texture_importer_runtime --no-default-features --lib --no-run --profile test --locked` 的实际编译在上述两行给出 **2 个 E0618**，没有产生测试程序。另有 21 个来自另一 mip 测试叶的诊断，不由这两行造成。两个正例和两个负例都尚未由这次执行证明；仅按测试名过滤无助于绕过编译。关联 [#219](https://github.com/He-Jiahui/ZirconEngine/pull/219) 仅提供实际编译出处，并不拥有此测试错误。其[验证记录](https://github.com/He-Jiahui/ZirconEngine/blob/482351810f7dd48f627a476c3dc003fb00608359/docs/plans/unreal-alignment/texture-array-source-refresh.md#L65)区分原生数组回归与额外插件 libtest；此处没有把未执行的用例计成产品缺陷。

## 3. 这些断言原本在保护什么？

两份两字节样品分别直接进入[KTX1 生产校验器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/container/ktx/ktx1.rs#L105-L131)和[mip 范围校验器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/texture_importer/runtime/src/container/ktx/ktx1.rs#L227-L253)。原有断言要求具体的 metadata-size 或 mip-imageSize 错误；它们不是任意损坏文件“报错就算通过”，也不必为了触达这些内部校验器而构造一个伪容器。当前没有证据指向生产解析结果、GPU 或容器格式本身的错误。

## 4. UE 的哪些原测相邻？

`System.Engine.Texture2D.CreateTransient`（`Engine/Source/Runtime/Engine/Private/Tests/Texture2DTests.cpp:16–38`）对三个零尺寸变体同时检查预期诊断与空返回；`System.Engine.Texture2D.LockingWhenEmpty`（`Engine/Source/Runtime/Engine/Private/Tests/Texture2DTests.cpp:41–110`）区分可锁与不可锁的 mip。Interchange 的纹理结果断言 helper（`Engine/Plugins/Tests/InterchangeTests/Source/InterchangeTests/Private/ImportTestFunctions/TextureImportTestFunctions.cpp:16–105`）属于测试辅助函数，不是一个可单独运行的 KTX1 自动化测试。只借“正常样本+准确失败边界”的验证方法，没有找到覆盖本 Rust 名称遮蔽或 KTX1 同一解析分支的 UE 原测。

## 5. 最小修复与责任边界

仅重命名两个正例的局部值，或同等明确地指向原 helper。保留验证函数、输入字节、精确错误文字、现有正负断言与性能用例。不要为测试增公开 API 或改生产解析；不顺带修另一文件的 `mipgen` 返回类型。两类不相同的根因都归 Plugins07/G01 验证门，而不是两个纹理产品故障。

## 6. 如何验收？

在本 PR 的原基线和修后以相同参数重新编译、逐条对比主诊断文件及行。修这个问题只能消除本叶 **2 个 E0618**；若另一叶的 21 项仍挡住 libtest，行为用例数必须仍报 0。等原生目标真正能生成后，原样执行 `plugins07_ktx1_hotpath_lazy_metadata_labels_preserve_validation` 和 `plugins07_ktx1_hotpath_lazy_mip_labels_preserve_validation`；只认实际断言结果，不跳过、不复制别处的补丁来报绿。

本地参考为 6.0.0 / UE5 / CL 0，未知上游 revision；没有运行 UE。这些哈希固定参考文件而非发行版本：

- `TextureImportTestFunctions.cpp`：`01bccbd5b2caf13efdb75e6d336fc04f7f0f8dfdbd1cbe53c1eddf929568a86d`
- `Texture2DTests.cpp`：`6ee1de61a79f5cd6a816e7e628f3c30ca7c06ab96edc06fde6a41f00f41eb25b`

## 7. 原测试的独立修正和编译器实证

[同 PR 单一测试文件改动](https://github.com/He-Jiahui/ZirconEngine/blob/50d6834db0d58d827bec5edd0599bcac6f3bf04d/zircon_plugins/texture_importer/runtime/src/tests/ktx1_plugins07_ktx1_hotpath.rs)。Windows x64/MSVC、Rust 1.94.1，`test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_texture_importer_runtime --no-default-features --lib --no-run --profile test --locked --jobs 1 --message-format=json`。原版包树、Runtime、接口、工作区清单与锁均取固定 main；两项独立调查起点只有各自的 Markdown 文档不同，**共用同一次真实原版编译**，不是叠加更改后的分支。

KTX1 原测试的 2 项编译诊断已消失；同一插件 libtest 从 23 项降为 21 项，剩余均为独立 mipgen 测试的原诊断。仍未生成测试程序、0 行为用例执行；仅修测试，不改生产。

- 原始编译回执 `ae1f236932d914fda6c778db8ab10196847bf050d9380d492569936aa9d1144f`：2 个 KTX1 E0618、11 个 mipgen E0599、10 个 mipgen E0609。
- 本 PR 的实际编译回执 `400e0a06ba43f17fec5a0df2a2983e25f78c11abd5f23bc3caff918a32c40230`：逐个比对剩余的 `21` 个主错误（路径、行、code、message 均相同），其他未改测试的 Git blob 仍与 main 匹配，没有新增其它类别。
- 两处有效输入的局部名字不再遮蔽原 helper，第二份输入、验证函数、正例和具体负例断言未改。

这不是整套测试已绿，也不是生产纹理画面验证；未用跳过、临时 cfg、替代 runner 或从其它未合并 PR 拿代码。待两个独立改动合法进入同一基线后，仍须从原 Cargo 目标启动并运行原来的实际用例。
