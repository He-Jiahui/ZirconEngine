# 动画无默认特性的库测试误编译宿主专用用例

固定公开基线 [ZirconEngine main `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`](https://github.com/He-Jiahui/ZirconEngine/tree/e89349ca51b4e9d024c194cd3bdf645bd25b6c37)。本调查只涉及动画插件库测试的编译边界。

## 1. 用户入口与工程合同

维护者可在插件工作区关闭 `zircon_plugin_animation_runtime` 的默认特性后执行 `--lib`，检验仍可用的动画代码。生产中，[Web App 的 `target-web`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/Cargo.toml#L155-L161)也选用[关闭默认特性的动画包依赖](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/Cargo.toml#L205-L213)。[浏览器引擎入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/mod.rs#L15-L35)传入公开的 `module_descriptor()` 与 `portable_runtime_extension_plan()`，[创建级别前安装该 plan](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/open.rs#L56-L86)。[便携 plan](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/runtime_system.rs#L55-L93)在 `PostUpdate` 接同一个动画求值器。

## 2. 基线中的编译错位

[库根](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/lib.rs#L20-L33)与[宿主导出](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/lib.rs#L52-L67)只在 `native-host` 下提供宿主注册、capability 与分发清单；[库根挂载测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/lib.rs#L85-L90)只要求 `test`。三项未加同一条件的测试仍引用已移出本配置的符号。

- [注册贡献检查](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/tests/cases.rs#L36-L121)调用 `plugin_registration()`，核对模块、系统、资源、事件与 capability。
- [调度顺序检查](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/tests/cases.rs#L123-L144)用同一原生注册报告对照 `PostUpdate` 与 `FixedPostUpdate` 物理同步阶段的顺序。
- [分发契约检查](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/tests/cases.rs#L161-L218)调用 `package_manifest()` 并检查 NativeDynamic、ABI v3、描述符和 capability。

Rust 在 libtest 筛选前编译整个目标。因此即使只选[原有的管理器激活用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/tests/cases.rs#L146-L159)，这三项仍挡住可执行文件。影响落在现有自动化覆盖，浏览器动画的功能结论另验。

## 3. 特性边界与已有实测

[动画包](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/Cargo.toml#L8-L22)默认启用自己的 `native-host`，显式关闭后仍保留 Runtime 的 `animation,physics-contracts`。测试另拉入[物理包](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/Cargo.toml#L8-L21)，该包与其 SDK 经[插件工作区依赖](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/Cargo.toml#L143-L169)给共享 `zircon_runtime` 加入 `native-host`。共享依赖的特性不会反向启用动画包自己的同名特性。

历史 Windows/MSVC v4 对照是 #165 候选树，关闭动画包默认特性编译 `--lib --no-run`，退出 101 且无 EXE。19 条错误中三项原测试占 14 条 `E0425`、4 条伴随 `E0282`。另一个 `ResourceLocator: FromStr` 属于当时新增夹具，[#165 随后的版本改用显式解析](https://github.com/He-Jiahui/ZirconEngine/blob/7bd76527b6b808747a7dc863a9172c112bd8f378/zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs#L448-L463)。这份历史试跑与固定基线的独立实跑分开计。#165 已公开的 [14 项集成执行和两项宿主库执行](https://github.com/He-Jiahui/ZirconEngine/blob/7bd76527b6b808747a7dc863a9172c112bd8f378/docs/plans/unreal-alignment/timeline-paused-binding.md#L218-L226)走了不同目标或特性。其[集成夹具](https://github.com/He-Jiahui/ZirconEngine/blob/7bd76527b6b808747a7dc863a9172c112bd8f378/zircon_plugins/animation/runtime/tests/sequence_tick.rs#L154-L183)在原生环境真正安装公开 plan，再进行关卡动画验证。

## 4. UE 的条件编译原测

本机参考源自报 `6.0.0 / UE5 / CL 0`，上游修订未核定。动画原测 `Ispc.Animation.ConstantKeyLerp.GetPoseRotations` 在 `WITH_DEV_AUTOMATION_TESTS && INTEL_ISPC` 下编译，连 include 与测试体一起限定。它通过真实 `GetPoseRotations` 比较 ISPC 与 C++ 两路结果并恢复开关。生产旋转解码将生成头及 ISPC 路径置于 `INTEL_ISPC` 下，同时保留 C++ 路径。这支持把专用测试与能力边界对齐。在所查本机源码中未找到 ZE 原生插件注册、排序及 ABI 清单的同形原测。本调查未运行 UE。

```text
Engine/Source/Runtime/Engine/Private/Animation/IspcTestAnimEncodingConstantKeyLerp.cpp:3–13,61–99
SHA-256 404c62415223706c9a2f939b490c956ba1c83fa1e3d33183cc7c97150ca05ce7
Engine/Source/Runtime/Engine/Private/Animation/AnimEncoding_ConstantKeyLerp.cpp:7–14,22–29,268–315
SHA-256 d1384c9d004b8091222566be2826696a69b0d6d11ff6ec7ff16294a0c014ec77
```

## 5. 当前 CI 与相邻问题

固定基线的[受影响插件规则](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/scripts/ci-scope.py#L354-L363)已注册 `plugin-zircon_plugin_animation_runtime`，[命令生成与执行](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/scripts/ci-scope.py#L574-L588)选择 `--lib tests:: --no-default-features --target x86_64-pc-windows-msvc`，并[要求测试有匹配项](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/scripts/ci-scope.py#L685-L708)。[主分支受信任 push 的 Rust 作业](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/workflows/ci.yml#L391-L398)对受影响范围[执行该规则](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/workflows/ci.yml#L503-L506)。这是已配置的匹配编译门，仍须看实际作业结果。[Wasm 核心 check](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/scripts/ci-scope.py#L389-L395)与[MVP 默认特性 workspace check](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/workflows/mvp-editor-windows.yml#L104-L110)各覆盖另一种选择。在固定 CI 配置中未见三项宿主断言的对称 `native-host` libtest 处方。

[#165](https://github.com/He-Jiahui/ZirconEngine/pull/165)追踪暂停时资产版本和目标绑定的真实帧行为。它与本项共用一个测试文件，固定差异只替换原第 221 行起的投影源检查；本项的五处条件均位于原第 161 行及以前。就固定 hunk 而言可独立合入。另有 [#214](https://github.com/He-Jiahui/ZirconEngine/pull/214) 的暂停过渡重复通知、[#216](https://github.com/He-Jiahui/ZirconEngine/pull/216) 的 SDK 默认测试模块安装，以及 [#233](https://github.com/He-Jiahui/ZirconEngine/pull/233) 的 Runtime UI 生产依赖，分别拥有不同的失败入口与修复位置。

## 6. 最小修复与回归门

给三项原测试和[两条宿主专用导入](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/tests/cases.rs#L1-L5)各加 `#[cfg(feature = "native-host")]`。原测试函数体、`CoreRuntime` 导入和其它测试留在原位。改动只落测试声明，沿用原 API、ABI、清单、制品与资产格式，迁移只需维护者按特性选相应库测试。

待按 Windows x64/MSVC、`test` profile、插件工作区唯一选包 `zircon_plugin_animation_runtime` 记录以下实际结果与源码、回执指纹。固定基线预检先跑 `--no-default-features --lib --no-run --profile test --locked --offline`。修后在同配置执行现有 `tests::`，确认正常测试有运行且三个宿主名称不在列表；再显式追加 `--features native-host`，执行原三名的确切过滤并保留实际匹配与结果。两组均用真实 libtest。关闭默认的这一组使用 Windows 原生进程，Wasm 与浏览器的验证保有自己的目标和回执。

## 7. 无默认特性的库测试编译结果

在公开主线的动画插件上关闭默认特性后，Windows 库测试编译失败。三项宿主测试产生 14 条名称解析错误和 4 条随后的类型推断错误，编译器未生成测试程序。

原文件中的三项测试分别检查原生注册、物理同步后的调度位置和分发清单。编译诊断的主位置位于第 38–203 行。使用 Windows x64/MSVC、`test` profile 和包 `zircon_plugin_animation_runtime --no-default-features --lib`。源码 `432c6efd72417695adcc621945f7c76ef665ae1c`，编译回执 `3bd2b970a12119c2122c347530109f4de5cd3954d768475f3e83b0729c138540`。

## 8. 两种特性配置下的原库测试

关闭动画包默认特性时，原先阻断库测试构建的 18 条诊断已消除。使用仓库现有的 `tests::` 筛选，实际通过 89 项。显式开启 `native-host` 后，注册、物理同步后的求值顺序和分发清单三项原测试全部通过。

两条宿主专用导入和三个测试的编译条件对齐了原有 API。关闭默认特性后，测试列表包含 `tests::animation_module_resolves_manager`。显式开启宿主特性后，在同一份源码中逐名执行三个原用例。

实跑源码 `2e041b050efd762d0fa6175d3cb33fd20cf75d73`。包为 `zircon_plugin_animation_runtime`，Windows x64/MSVC，`test` profile。无默认特性库套件回执 `193a00b54214a248a394d162d99b66379e90d90712e70db1e22f5bdcfb87a3b0`，显式 `native-host` 构建回执 `e35f9963e2e99187184ff61fd07ebe928ce2cb5ef65680ef70f62f4cbc5c5399`。
