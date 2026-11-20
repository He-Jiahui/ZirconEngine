# 着色器缓存重复发布相同制品时会报告清单冲突

## 1. 用户可见的结果

在 Windows x64 上，同一个普通 `zircon_shader_prewarm` 程序、同一份内置 shader 输入和相同的六个 pass，写入空目录时全部成功。给它提供已成功生成的这六个制品，再执行同一预热操作，程序返回 2，六项都报告 `CorruptTarget("manifest rename conflict differs from requested identity")`。原来的缓存文件均可读取。[磁盘缓存写入入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/shader/variant_cache/disk.rs#L208-L242)

## 2. 复现输入与测量

程序来自公开 `main` 提交 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`，可执行文件 SHA-256 为 `5b16ed830a2c50d569a9bf29929643445ea8a740f4753a0fa7ffdf98d5cea12b`。用 `--builtin-fallback --quality-tier medium --geometry-source static` 导出六个内置材质 pass，保留首次写入的六份 `.manifest` 与六份 `.wgsl.zst`。稍后在这份缓存的副本和一个空目录上分别预热。两次使用同一项目目录、同一程序与参数，只指定各自的缓存、报告和注册表输出位置。

| 缓存初始内容 | 请求 | 写入 | 失败 | 退出码 |
|---|---:|---:|---:|---:|
| 空目录 | 6 | 6 | 0 | 0 |
| 上次成功的六个变体 | 6 | 0 | 6 | 2 |

两批六个分片位置和变体键完全相同。逐一比较压缩 WGSL，六份内容都逐字节相同。逐字段比较清单，唯一差异是 `created_unix_seconds`，从 `1795154155` 变为 `1795156891`。重复发布前后，原始副本的十二个文件均保留原 SHA-256。

## 3. ZE 冲突发生的位置

运行时用规范化的变体字符串和 shader 源身份计算内容键。每次写入都会把当时的 Unix 秒数装入清单，然后先确认或提交不可变的 WGSL，再提交清单。首次提交成功后，同键再次写入会在两个已有文件分支之一比较整份清单的序列化字节，即使只变了时间戳也返回冲突。[内容键](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/shader/variant_cache/disk.rs#L435-L452) · [清单的创建时间和写入顺序](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/shader/variant_cache/disk.rs#L208-L241) · [已有清单与竞争写入的两个分支](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/shader/variant_cache/disk.rs#L542-L584)

## 4. Unreal 的对应实现及原测试

本地参考树为 `6.0.0 / UE5 / CL 0`。`Engine/Source/Developer/DerivedDataCache/Private/FileSystemCacheStore.cpp:1927–1987` 的 `PutCacheRecord` 按 key 查找并加载既有记录，以记录中的 values 识别内容差异。`2489–2510` 行按 raw hash 定位压缩内容，已有内容可直接复用。

`Engine/Source/Developer/DerivedDataCache/Tests/DerivedDataCacheStoreTest.cpp:1096–1148` 的 `TestCacheDeterminism` 先写入两个值及原元数据，再用同一 key、相同值和新元数据执行第二次 `Put`。测试断言第二次状态是 `Ok`，返回的是第一次的元数据和两个原值。`1437–1456` 行的 `DerivedData::CacheStore / FileSystem` 在真实文件系统实现上调用这个测试。本次对照依据是 UE 测试源码。表中的退出码和制品来自 ZE 实际运行。

## 5. 现有测试与相关 PR

ZE 现有重启测试在首次写入后执行 `lookup`。现有冲突测试向低层函数两次传入完全相同的原始字节，再测试不同字节应拒绝。本次回归覆盖同一生产 writer 对同一变体的第二次写入。#253 处理坏材质首次失败后被错误跳过。#254 处理 Python 打包验收读取 v3 制品。这两个相关问题分别位于资产库存和打包工具。[重启读回测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/shader/variant_cache/tests/disk.rs#L77-L106) · [已有字节冲突测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/shader/variant_cache/tests/disk.rs#L204-L231) · [预热重试 #253](https://github.com/He-Jiahui/ZirconEngine/pull/253) · [打包验收 #254](https://github.com/He-Jiahui/ZirconEngine/pull/254)

## 6. 修复与验收

在正式 v3 写入路径识别已经提交的同一条目，保留首次发布的清单和创建时间。用同一批缓存验证两次命令均返回 0，六个 pass 的键、压缩内容和已发布清单均一致。另测错误的源码身份、变体键、工具链、payload 摘要、损坏清单及并发竞争，确保真实不一致仍能被发现并保留首份制品。

## 同一批磁盘制品的修复结果

同一个项目、六个内置 pass，在 Windows x64/MSVC 上用主干和修订后的普通 `zircon_shader_prewarm` 分别执行七组用例。表中三项数字依次表示请求、写入成功和失败的变体数。

| 初始缓存 | 主干 | 修复后 |
|---|---:|---:|
| 空目录 | 6 / 6 / 0，退出 0 | 6 / 6 / 0，退出 0 |
| 只改变六份清单的合法创建时间 | 6 / 0 / 6，退出 2 | 6 / 6 / 0，退出 0 |
| 在一个变体上改变工具链字段、增加未知字段、写入非法时间、重复时间字段或修改同长压缩载荷，分五次运行 | 每次 6 / 0 / 6，退出 2 | 每次 6 / 5 / 1，退出 2 |

两版程序对每个用例收到相同的预置字节。修复后，已有缓存的十二个文件在运行前后具有相同的 SHA-256。五组冲突中的其余五项仍返回原来的完整变体身份。源码改动位于运行时磁盘缓存写入函数，低层首次提交成功的清单继续保有原来的创建时间。此次直接运行普通 CLI；并发提交的 Rust 单元用例已随源码提交，执行情况另行记录。

[写入端修订](https://github.com/He-Jiahui/ZirconEngine/blob/09b0797793e18d80521a591c30fffbe3e7289429/zircon_runtime/src/graphics/shader/variant_cache/disk.rs#L538-L572) · [缓存回归](https://github.com/He-Jiahui/ZirconEngine/blob/09b0797793e18d80521a591c30fffbe3e7289429/zircon_runtime/src/graphics/shader/variant_cache/tests/disk.rs)
