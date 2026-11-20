# 本地 Cargo 测试跨工作树复用了上一来源的可执行文件

## 实际故障

在同一 Git 仓库的两个独立工作树依次验证 [#239](https://github.com/He-Jiahui/ZirconEngine/pull/239) 与 [#242](https://github.com/He-Jiahui/ZirconEngine/pull/242)。两次都直接使用本地 Cargo 验证器，先前命令已退出，没有并发写者；Windows x64/MSVC、Rust 1.94.1、`zr_rhi_wgpu` 库、test profile、关闭默认特性、同一显式本地 target/build 池。两树同名 Rust 包的源码不同。

在 #242 工作树，请求执行 `production::surface::tests::`。命令回执的 `cwd` 和 `--manifest-path` 均正确指向 #242，Cargo 却直接运行与 #239 相同的 `zr_rhi_wgpu-567762abc626f386.exe`。终态 **0**、只运行了旧树的 **4** 个用例：#242 源码中的 6 个既有用例和 2 个新增用例共有 4 项未注册。不能用这个退出码为 #242 验收。

| 证据 | 原共享池中的旧结果 | 刷新当前工作树源码时间戳后 |
|---|---|---|
| 实际 Rust 测试 | 4 个旧用例通过，未编译本树 | 本树重新编译，8 项通过（含两条新增 Canvas 用例） |
| 可执行文件 SHA-256（路径不变） | `1a103ce4bacaccf2e31744592582cac5031f7579d0c355bb5d316600713c6090` | `aaf93fe6f4451748bb16ef2f319411a8547086da05f38548256bb3b1c3304c34` |
| 原始回执 SHA-256 | `763a9182769c857f3257416baf4f03f3d5606f7c0dfcad53f917b89728e0793a` | `9ee5bb873569a7852cff08644d2c581e76d4af28b49695c247271f7efb601776` |

旧可执行文件的写入时间为 `2026-11-20T23:43:15Z`，晚于 #242 格式实现及测试文件的时间戳（`23:39:12Z`、`23:39:33Z`）。只更新本任务工作树中 RHI 的三个相关文件的 mtime；源码、测试及锁文件 Git blob 不变，没有删除缓存，也没有修改上一工作树。Cargo 的完整 fingerprint 判定路径尚未采集，不从成功的临时处理推断所有缓存都应强制刷新。

实测本地验证器来自 `cb495bb1`。其中 [Python 入口](https://github.com/He-Jiahui/ZirconEngine/blob/b9489641af65ff5f27c869d57684190bec782d14/tools/dev/local_cargo.py) blob `32cc7334`、[PowerShell 入口](https://github.com/He-Jiahui/ZirconEngine/blob/b9489641af65ff5f27c869d57684190bec782d14/tools/dev/local-cargo.ps1) blob `e2a7092b` 与 [#136](https://github.com/He-Jiahui/ZirconEngine/pull/136) 同一版本，不声称旧主干验证器实测通过或复现。实际复现用显式池。新验证器的[默认池投影](https://github.com/He-Jiahui/ZirconEngine/blob/b9489641af65ff5f27c869d57684190bec782d14/tools/dev/local_cargo.py#L148-L168)也有意共享同一 Git common-dir；[现有测试](https://github.com/He-Jiahui/ZirconEngine/blob/b9489641af65ff5f27c869d57684190bec782d14/tools/tests/test_agent_local_cargo.py#L285-L299)只比较 dry-run 的目录和 Cargo home，未执行两个源码世代。普通回执已经明确 `sourceClosureVerified=false`，本案不把它冒充假造的正式验收；问题是实际测试进程在已知不同输入上执行了旧程序。

## 与 UE 对齐的边界

本地 UE 6.0.0 / UE5 / CL 0：UBT 在 `ActionGraph.IsIndividualActionOutdated` 先用产物路径和包含命令、参数、版本的生产属性检查历史，再检查依赖时间戳（`Engine/Source/Programs/UnrealBuildTool/Actions/Graph/ActionGraph.cs:711–779`）。`ActionHistoryLayer.UpdateProducingCommandLine` 以输出文件为键比较属性摘要（`Engine/Source/Programs/UnrealBuildTool/Actions/History/ActionHistoryLayer.cs:80–98`）。相邻原测 `TargetRulesTests.TestTargetRequiresUniqueEnvironmentFirstOrder` 用真实规则程序集证明：原规则允许共享，改变要求唯一环境的选项时应隔离（`Engine/Source/Programs/UnrealBuildTool.Tests/TargetRulesTests.cs:13–46`）。未找到 UBT 中与 Cargo 工作树复用同形的原测；也不声称 UBT 自动检查任意源码内容。等价点是先确认生产输入与产物的归属，再采用缓存结果。

ZE 保留 Rust/Cargo、共享的依赖缓存和原生 Cargo 锁，不移植 UBT，不恢复 Jenkins 或旧协调器。本案不改 RHI、WebGPU、产品 target 产物选择 [#161](https://github.com/He-Jiahui/ZirconEngine/pull/161) 或 #136 的退役流程。

## 独立回归与验收计划

1. 在本 Draft 后续使用与实测相同的验证入口和已声明的最小包，构造两个同一 common-dir 的真实 Git 工作树：A、B 各有独特的 Rust 测试见证且都可通过。先从 A 编译，再在非并发的 B 指向同一池，验证即使 B 文件早于 A 产物，B 不会输出 A 的见证。用退出码、实际测试名和可执行文件指纹一起判定，不用 dry run 代替执行。
2. 在独立验证器中验证源码所有权边界。不兼容时须触发安全编译或明确拒绝将旧二进制充作本次结果；方案要保留 A→A 兼容复用、只读预览及共享依赖缓存。对两个工作树的来回切换、失败构建和并发 writer 另加控制；不通过改时间戳、清共享缓存、全量复制 target 或签发虚假的 source attestation 来交付生产修复。
3. 先在本 PR 留下可重复的 RED，再做修复。受影响入口待 #136 的同源实现可用后按独立范围验收。本首发提交只记录问题和证据，不含实施或新的编译结果。
