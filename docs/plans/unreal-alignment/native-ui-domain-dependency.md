# Windows 的 Runtime UI 独立构建缺少实际编入的外部依赖

原 GitHub `main`：`e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。本问题在 [#232](https://github.com/He-Jiahui/ZirconEngine/pull/232) 的普通滚动回归首次编译时暴露；与滚动框的尺寸、输入或布局实现无关。本 PR 的首个提交只记录独立调查与等价回归方案，不修改代码、Cargo 或 CI。

## 1. 产品承诺与用户入口

开发者在 Windows 只编译 Runtime UI 时，应能使用当前仓库声明的域，而不是隐式启用整个桌面宿主。[Windows x64 域编译作业](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/workflows/ci.yml#L339-L346) 在受影响的受信任主分支 push 上选择 [domain-ui / domain-graphics 生成规则](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/scripts/ci-scope.py#L400-L407)；[实际命令参数](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/scripts/ci-scope.py#L574-L586)为 `check -p zircon_runtime --no-default-features --locked --lib --features ui --target x86_64-pc-windows-msvc`。[UI、Graphics 的真实 owner-test 规则](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/scripts/ci-scope.py#L157-L176)另有单独测试入口。这里只指出仓库规定的工作，不是说人已运行 GitHub CI，也不要求恢复历史调度器。

## 2. 实际故障与最小证据

在 tests-only `25569db598005184535f0c7b075607056bee7d76`（[与故障无关的 UI 回归](https://github.com/He-Jiahui/ZirconEngine/blob/25569db598005184535f0c7b075607056bee7d76/zircon_runtime/tests/scroll_padding.rs)），真实 Windows x64/MSVC Rust 1.94.1、默认特性关闭，仅 `ui`，编译一个集成可执行文件的 `test --no-run --profile test --locked`。Cargo 以 **101** 结束；首要且唯一的编译错误是 `zircon_runtime/src/asset/artifact/render_manifest/loader/decode.rs:20` 的 `E0433`（找不到 `zstd`），**没有测试可执行文件，没有运行一个布局用例**。该次回执 SHA-256 `2418936fddad7a46640eb67fa7c018695986ab2689bbda2eead405727f96a160`。与故障有关的 Runtime 源、manifest、锁文件和 CI 处方均逐一比对为原 `main` 的 Git blob。检验的是 production `zircon_runtime` lib，而不是测试文件导入私有 crate。这不是已运行的 `domain-graphics`、Web 或发布产品验收。

## 3. ZE 源码的生产者与消费者

[UI → graphics](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/Cargo.toml#L210-L216), [graphics / text → cooked-assets](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/Cargo.toml#L177-L190), [cooked-assets → asset-types](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/Cargo.toml#L145-L152)。没有 `native-host` 时 [Runtime 选择 portable 资产模块](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/lib.rs#L31-L46)，其中 [cooked-assets 装配 artifact](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/portable.rs#L42-L51) → [共享 render_manifest](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/portable_artifact.rs#L9-L13) → [loader 模块](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/artifact/render_manifest/loader/mod.rs#L4-L13)。在非 wasm 目标 [实际使用 zstd crate](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/artifact/render_manifest/loader/decode.rs#L9-L21)；然而同一原 manifest 仅由 [native-host 的现有直接依赖](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/Cargo.toml#L113-L138) 启用这项可选库。[单一非 wasm 的包声明](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/Cargo.toml#L361-L375) 本身并不会启用它。这条链与测试中有没有数据、是否调用布局、是否在运行时打开工程均无关。

## 4. UE 怎样表达依赖，以及真实的邻近原测

`Unsync.Build.cs`（`Engine/Source/Programs/Unsync/Unsync.Build.cs:31–51`） 将平台选项和消费模块的私有库依赖分别声明；`UnsyncCompression.cpp`（`Engine/Source/Programs/Unsync/Private/UnsyncCompression.cpp:1–12`） 是真实消费者。`UEBuildModule 解析私有模块`（`Engine/Source/Programs/UnrealBuildTool/Build/UEBuildModule.cs:1277–1286`） 把声明纳入构建图。类比点是**哪个模块的代码会被编译，哪个依赖就须进入那个目标的图**，不是 UE 的 Slate、Unsync 或 SDK 具有 Cargo 的同名 feature。

可核的 UBT 原始测试 `TargetRulesTests.TestTargetRequiresUniqueEnvironmentFirstOrder`（`Engine/Source/Programs/UnrealBuildTool.Tests/TargetRulesTests.cs:12–46`） 用实际规则程序集构造 Win64 Development `TargetRules`，比较修改选项前后的独立环境判定。它并不检查 ZE 的 codec 边，也不是 UE 已有“UI 单 feature 编译”同形测试；本地未发现同形原测，没有运行 UE。UE 参考为 6.0.0 / UE5 / CL0 源码，未取得可靠上游提交，不分发实现。

## 5. Rust / Cargo 的最窄责任

确定要编入共享 cooked-artifact 模块时，应让这条平台限定的现有依赖可见。最小候选是在 `cooked-assets` 声明复用现成的 `dep:zstd`，保留 `native-host` 原直接边和 `cfg(not(target_arch = "wasm32"))` 的依赖声明。不要将整个 `native-host` 加入 `ui`/`graphics`、把库改成所有平台无条件依赖、移除正式域检查、关闭加载器或改滚动测试。wasm 路由有独立条件编译和既有 Raw 工件政策，不能以 Windows 客户端过了就宣称 Web 通过。若既有依赖政策/架构文档与实际使用不一致，应在同一个独立问题内核实并更新真实约束，不能放宽所有 guard。

## 6. 去重、等价回归和退出条件

已查全部状态 232 个 PR 的标题及正文、全部开放文件，复核相关历史 CI/构建改动。[#136](https://github.com/He-Jiahui/ZirconEngine/pull/136) 管退役流程和命令接线；[#161](https://github.com/He-Jiahui/ZirconEngine/pull/161) 管构建产物的显式 target 身份；[#174](https://github.com/He-Jiahui/ZirconEngine/pull/174) 的早期 UI 测试曾受 libtest 的 Scene/Project 等配置约束。这些均不是这次 **production lib 的直接依赖未启用**。本缺陷取证不依赖 [#232](https://github.com/He-Jiahui/ZirconEngine/pull/232) 的生产改动，也不会把两个 PR 合并验证。

优先在这个 doc-only 原分支直接跑原 CI 所规定的 Windows 单包 `ui`、`--lib`、`check`、dev profile，观察是否复现同一诊断；再在仅改责任边的分支以完全相同参数重验。另验 `graphics` 的最小域和非 wasm 依赖/wasm Raw 边界，说明是原生编译、解析 cargo graph 还是实际 wasm 构建，各自独立报告。原有 CI 处方本身是回归门，不新增永远为真的字符串断言，也不用另一个 Draft 的测试或整 workspace 的 feature 合并遮蔽问题。

本地 UE 参照 SHA-256：

- `Engine/Source/Programs/Unsync/Unsync.Build.cs`：`2707263ecaf19819d3508d54dd164a3f4a56627ebbda43c214acfa13b8ed61a3`
- `Engine/Source/Programs/Unsync/Private/UnsyncCompression.cpp`：`9119c7a81ed7aedae0a477909d1362bf3b88c42acb12839d454507876ff55b4c`
- `Engine/Source/Programs/UnrealBuildTool/Build/UEBuildModule.cs`：`d88199f14b2f439eed1e1a6960ea277a3fb6666a51a231b981664929f96266ed`
- `Engine/Source/Programs/UnrealBuildTool.Tests/TargetRulesTests.cs`：`89a438e2e92e494c5e870290b36ca19493a127102d4e6a027fdc73f7ea50c345`
