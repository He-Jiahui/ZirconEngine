---
title: Zircon App Product Launch Request / CLI / Config Resolution / Provenance / Diagnostics / Exit Semantics 当前工作树复审
category: zircon_app
report_id: App09
review_date: 2026-08-31
baseline_head: working-tree
observed_head: f31fd06f69fdaedb70a0a56fe6d0268de1af83a6
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
coordination_tracking: skipped_by_user_request
related_reports:
  - docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
  - docs/plans/optimize/zircon_runtime/208-runtime-product-build-export-profile-build-plan-platform-host-cross-compilation-package-launch-handoff-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
owner_plans:
  - docs/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
zircon_scope:
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_app/src/entry/cli
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/product_host_config
  - zircon_app/src/entry/product_shutdown
  - zircon_app/src/entry/runtime_library/library_path.rs
  - zircon_app/src/entry/platform_preferences.rs
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_editor/src/core/commandlet
  - zircon_runtime/src/diagnostic_log
reference_scope:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/CommandLine.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/CommandLine.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/Launch.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/Windows/LaunchWindows.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/ConfigCacheIni.cpp
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Tests/DerivedDataConfigTest.cpp
  - dev/UnrealEngine/Engine/Source/Editor/EditorConfig/Private/Tests/EditorConfigTests.cpp
  - dev/UnrealEngine/Engine/Source/Editor/EditorConfig/Private/Tests/JsonConfigTests.cpp
  - dev/godot/main/main.cpp
  - dev/godot/main/main.h
  - dev/godot/core/config/project_settings.cpp
  - dev/godot/core/config/project_settings.h
  - dev/godot/tests/core/config/test_project_settings.cpp
  - dev/godot/tests/test_main.cpp
  - dev/godot/tests/test_main.h
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Fyrox/editor-standalone/src/main.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/BuildProcessors/CoreBuildData.cs
---

# App09 · Product Launch Request、CLI、配置解析、来源证明、诊断与退出语义

## 1. 结论

当前 `zircon_app` 有可用的局部参数解析器、`EntryConfig`/`ResolvedProductHostConfig`、产品失败 ledger、Runtime library 路径选择和 Editor commandlet JSON 报告，但这些部分仍是互相独立的“解析 helper”，不是工程级产品启动合同。argv、环境变量、project manifest、export profile、runtime profile、BuildSet、dynamic library、Hub/Play intent、日志配置和终态没有收敛成一份不可变、可签名、可复放的 `ResolvedProductLaunch`。

最严重的三项差距是：

1. Runtime binary 直接解析 `RuntimeSessionProfile` 并调用 `RuntimeSession::create_with_profile_and_project`，没有经过 Editor/export 使用的 `EntryConfig -> ResolvedProductHostConfig`，因此同一产品请求存在两套角色、profile、platform、plugin 和 host capability 真值。
2. `ProductHostConfigProvenance` 只保存字段对应的来源 enum，不能回答最终值、候选值、优先级、冲突、digest、环境快照或为什么选中；系统没有跨 argv/env/project/export/BuildSet 的 immutable launch receipt 和 canonical digest。
3. `ProductExitClass` 虽然区分 startup/runtime/shutdown/forced termination，但 `ProductProcessExitCode::from_class` 将所有失败压成 1，两个 binary 又分别自行映射结果，机器无法稳定区分失败阶段、primary cause 和 secondary teardown/report failure。

这不是“缺几个命令行选项”的问题。只要不同入口可以对同一个 project 产生不同 effective host，CI、Hub、Editor Play、发行包和 supervisor 就不能证明启动了同一产品。当前本轮判定为 **P0 3 Open；P1 37 Open、3 Partial；P2 10 Open；16 项资格门 14 Fail、2 Partial、0 Pass**。没有运行二进制、真实 env/argv matrix、跨平台 process spawn、非 UTF-8 参数、crash/restart、签名或性能测试；不得据此声称可发布或性能超过 Unreal。

App08 独立拥有 Winit 主循环、窗口/surface、DLL quiesce 和 Play child transport 的生命周期闭环。本文只记录它们在启动请求、配置身份、诊断和终态上的输入/输出缺口，不重复计算 App08 的 P0/P1。Runtime208 继续拥有 BuildPlan/cook/package/platform artifact 的底层资格；本文要求 App 只能消费其 qualified BuildSet，不能重新解释或旁路它。

## 2. 审查边界、统计与 currentness

### 2.1 统计口径

选择集按当前工作树物理文件统计：物理行、非空行、文件 bytes、源码中 `#[test]`/`#[tokio::test]`/参考测试宏、`#[ignore]`。fingerprint 按小写 workspace-relative path 排序，将 `path + NUL + lowercase(file SHA-256) + LF` 串联后再 SHA-256。工作树有大量其他 Session 的修改，因此本文以文件指纹而不是提交号作为证据身份。

| 范围 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| App owner source | **39** | **9,051** | **8,202** | **333,816** | **106** | **0** | `a9a6fa37eebe3422269d0c6d34a589d0e6242c39705031954470c64d5dc4c274` |
| Focused direct tests | **11** | **3,217** | **2,896** | **110,599** | **111** | **0** | `9ec47d4218e9b6dd2da4e90f4a8c3e754b4df0daa552f1aa1232b9183073740c` |
| Editor/Runtime direct contracts | **8** | **2,407** | **2,158** | **78,470** | **13** | **0** | `1f877114fca581bc2c9a6e4af1bb87b045a1d057692d843fca63c4926b11616b` |
| Five-engine references | **21** | **31,629** | **27,121** | **1,103,326** | **71** | **0** | `c9ecf67f2b14f164ed31f6b4030032e67f6e382af6c5a98503d7d6993a1b5096` |

App owner source 39 文件覆盖两个 binary、CLI parser、runtime/editor runner、product host config、shutdown、library path 和 platform preference。Focused tests 是与本审查直接相关的 App/Editor commandlet/Runtime diagnostic tests，不把 App08 的 Winit/Play integration corpus 重新计入。参考选择集同时读取 Unreal 启动/命令行/config/test、Godot CLI/project settings/test、Bevy AppExit/Winit、Fyrox typed clap 入口和负面 silent fallback、Unity Graphics build-scoped context。

### 2.2 读取方法与限制

1. 从两个 binary 的 `std::env::args().skip(1)` 沿 diagnostic parser、Editor route、Runtime session parser 追踪 token 的消费、错误和剩余参数。
2. 从 Editor GUI、commandlet、Runtime、export bootstrap 追踪每个入口是否生成并消费同一个有效产品配置。
3. 从 `EntryConfig::resolve` 读取 role/profile/platform/render/window/plugin/provenance/capability 合同，并与 Runtime 独立 profile mapping 对比。
4. 从 library path、platform preferences、log env precedence 和 profiling/report output 追踪机器环境、路径和 secret 是否进入可审计记录。
5. 从 `ProductExitClass`、failure ledger、binary `ExitCode` 和 report writer 追踪 primary/secondary failure 是否被保留到进程终态。
6. 只将参考源码中已读取的实际字段和测试作为 E2 证据；Unity Graphics 没有 Player 进程 host，因此只用于说明 build-scoped context，不推断 Unity Player 生命周期。

本轮没有运行 Cargo、binary、dynamic DLL、真实项目、Hub/Editor 双进程、跨平台 argv/env、fault injection、crash/restart、非 UTF-8、压力/模糊测试或 benchmark。`rg.exe` 在本机不可用，使用 PowerShell 定向枚举与 `Select-String`，没有因此缩小选择集。

## 3. 当前启动链与权威分裂

```text
OS argv / environment
        |
        +--> diagnostic_log_args.rs ------> filter + remaining Vec<String>
        |                                      |
        |                                      +--> EditorLaunchRoute
        |                                      |      +--> commandlet parser (zircon_editor)
        |                                      |      +--> GUI hand parser (zircon_app)
        |                                      |
        |                                      +--> RuntimeSessionStartupArgs
        |
        +--> runtime log env / frame env / library env / preference env

Editor GUI / export helper ----------------> EntryConfig::resolve
                                                |
                                                +--> ResolvedProductHostConfig
                                                +--> plugin/module composition

Runtime binary ----------------------------> RuntimeSessionProfile
                                                |
                                                +--> RuntimeEntryAppConfig
                                                +--> RuntimeSession::create_with_profile_and_project
                                                +--> no ResolvedProductHostConfig

Editor commandlet -------------------------> CommandletRequest
                                                |
                                                +--> JSON CommandletReport + u8 code

terminal reason / failure ledger ----------> ProductExitClass
                                                |
                                                +--> ProductProcessExitCode (all failure = 1)
```

`EditorLaunchArgs::parse` 先剥离 log 参数，再由 `EditorLaunchRoute::parse` 在所有 token 中寻找 `--help`/`-h`；命令let 解析、GUI hand parser 和 Runtime parser 各自拥有 option grammar。`parse_diagnostic_log_startup_args` 支持 `--log-level[=]` 与 `--log-filter[=]`，但只返回 `remaining_args`，没有记录每个 token 的 origin/span/precedence。Runtime 再独立把 `remaining_args` 映射为 `RuntimeSessionProfile`、project、scene、report pipe 和 reference CPU presenter。

Editor GUI 会调用 `EntryConfig::new(EntryProfile::Editor).resolve()`；export bootstrap 也由 `EntryConfig` 投影 product role。Runtime `run_runtime_with_args` 则在校验剩余 token 后直接加载 library、创建 Winit event loop，并把独立枚举的 `as_bytes()` 传入 Runtime session。即使 argv、project、export profile 和 env 的语义相同，两条路径也没有共享一份 effective launch identity。

## 4. P0 阻断

| ID | 状态 | 当前证据 | 必须重构为 |
|---|---|---|---|
| **APP-LAUNCH-P0-001** | **Open** | `zircon_app/src/entry/entry_runner/runtime.rs` 先解析 `RuntimeSessionProfile`，随后直接 `LoadedRuntime::load_default` 和 `RuntimeSession::create_with_profile_and_project`；没有 `EntryConfig`/`ResolvedProductHostConfig`/qualified BuildSet 消费。Editor 和 export 使用另一套 role/profile/platform/plugin resolver。 | 唯一 `ProductLaunchRequest -> ResolvedProductLaunch` transaction。Runtime、Editor、commandlet、export、Hub handoff 只能提交 request 并消费同一个 immutable resolved plan；plan 中必须含 role/profile/platform/target/linkage/plugin/module/capability/window/input/render/library/build-set identity。任何旁路 profile parser 在迁移完成后删除。 |
| **APP-LAUNCH-P0-002** | **Open** | `ProductHostConfigProvenance` 每个字段只保留 `ProductConfigSource`，合并插件只保留 `ProductConfigSourceSet(u8)`；没有 argv/env/project/export/BuildSet 版本、值 digest、优先级、winner、冲突、环境快照或 canonical replay receipt。 | 建立 versioned `ResolvedProductLaunch` 和 durable launch receipt：保存规范化输入、来源链、候选/选中值、schema/build-set/toolchain/platform identity、secret redacted value/hash、input digest、生成时间、attempt/generation，并能以 receipt 复放或明确拒绝。 |
| **APP-LAUNCH-P0-003** | **Open** | `ProductExitClass::from_class` 将 Startup/Runtime/Shutdown/Forced 全部转换为 `Failure(1)`；`editor.rs` 和 `runtime_preview.rs` 又分别把 `Result`、commandlet u8、log shutdown 状态映射到 `ExitCode`，没有统一 terminal receipt/code registry。 | 统一 `ProductTerminalOutcome`：primary class/reason/code、secondary ledger、shutdown durability、report/IPC write status、attempt/generation 和 machine-readable receipt。保留跨平台 0-255 code registry，至少为 usage/config/capability/startup/runtime/shutdown/forced 分配稳定类别；不能让 binary 自行覆盖语义。 |

## 5. P1 工程化差距

### 5.1 参数语法、解析顺序与输入边界（APP-LAUNCH-P1-001..012）

| ID | 状态 | 差距与重构方向 |
|---|---|---|
| APP-LAUNCH-P1-001 | **Open** | diagnostic、Editor route/GUI、Runtime session、Editor commandlet 是四套 hand parser；同一 flag 的 equals、help、duplicate、unknown 和 missing-value 语义会漂移。定义单一 typed option schema，所有入口只提供 role-specific capability view。 |
| APP-LAUNCH-P1-002 | **Open** | parser 用 `S: Into<String>` 和 binary `std::env::args()`，非 UTF-8 OS 参数在进程入口就无法保真；Windows verbatim path、Unix byte path 和原始 argv 无法进入 receipt。使用 `args_os` 到边界 token，再按 option contract 选择 byte-preserving path/string。 |
| APP-LAUNCH-P1-003 | **Open** | 没有正式 `--` 用户参数分隔符；未知参数只能累积后由 Runtime 失败，无法把 engine options 与 project/script user args 分层。引入 engine/user token stream 和每个 role 的 `--` 合同。 |
| APP-LAUNCH-P1-004 | **Open** | `--project`、`--scene`、`--layout` 等 space-separated value 会无条件消费下一个 token；若下一个 token 是另一个 flag，错误只在后续阶段显现。值 token 必须检查 flag boundary、允许显式空值并附 token index/span。 |
| APP-LAUNCH-P1-005 | **Open** | GUI parser 只支持分离形式；Runtime/diagnostic 同时支持 equals，用户脚本和 Hub 生成命令无法得到一致 canonical re-emit。schema 应统一两种书写并输出唯一规范形式。 |
| APP-LAUNCH-P1-006 | **Open** | `--help` 在 Editor route 中以任意位置优先，`--run plugin-list --help` 被全局 Help 截获，commandlet 自己的 help/invalid argument 语义不可达。先解析 mode，再把 help 绑定到 root 或 commandlet subcommand。 |
| APP-LAUNCH-P1-007 | **Open** | Runtime help 在 unknown rejection 前返回成功，而 diagnostic env/CLI 先解析；`--help --unknown` 与非法 log env 的结果不一致。帮助必须是无副作用、明确优先级的 parse outcome，并定义冲突 token 行为。 |
| APP-LAUNCH-P1-008 | **Open** | `EditorLaunchRoute::parse` 将 commandlet 解析错误转换成 `CommandletRejected`，GUI unknown 则再包装成 editor diagnostic；错误类型、code、source span 和 recovery 不统一。建立结构化 `LaunchDiagnostic`，CLI/stdout/stderr 只是 projection。 |
| APP-LAUNCH-P1-009 | **Open** | 参数没有总 token 数、单值 bytes、JSON bytes/depth、重复列表、response-file 深度或展开总量预算。所有 parser 和 response file（若引入）需要 bounded admission、cycle detection、拒绝原因和计量。 |
| APP-LAUNCH-P1-010 | **Open** | Hub session token 与 `--project-launch-intent` JSON 直接出现在 argv；当前 redaction 只作用于格式化诊断，不能防 OS process list、crash dump、shell history 或 supervisor 记录泄漏。敏感 capability 应使用受控 handle/pipe/file descriptor，argv 只传短-lived reference。 |
| APP-LAUNCH-P1-011 | **Partial** | GUI 对 duplicate、project/template/scene/layout、Hub protocol 有显式拒绝测试，diagnostic parser 也覆盖 equals/duplicate；仍缺跨 parser grammar corpus、边界预算、非 UTF-8、response/canonicalization 和 process-level assertions。 |
| APP-LAUNCH-P1-012 | **Open** | operation id 由进程内 `OnceLock<ProjectActivationOperationIdGenerator>` 生成，跨进程重启、Hub retry 和 receipt replay 没有持久 lineage。operation/request/launch/attempt/generation 应由统一 identity service 发放并可验证重放。 |

### 5.2 Product role、config resolution 与 provenance（APP-LAUNCH-P1-013..024）

| ID | 状态 | 差距与重构方向 |
|---|---|---|
| APP-LAUNCH-P1-013 | **Open** | `EntryConfig` 是 programmatic builder，未序列化、无 schema/version/migration，不能作为 Editor/CLI/Hub/CI 共同交换的 launch document。定义 versioned request schema 和 forward-incompatibility policy。 |
| APP-LAUNCH-P1-014 | **Open** | `ProductConfigSource` 只有 ProductRole/RuntimeProfile/EntryRequest/ExportProfile；argv、environment、project manifest、Hub、BuildSet、platform probe、library manifest 等真实来源没有被建模。扩展 source domain 并记录 source locator/digest。 |
| APP-LAUNCH-P1-015 | **Open** | provenance 只给每个字段一个 winner enum，没有候选列表、优先级、覆盖原因、冲突/被拒值、normalization 和 value digest。每个 resolved field 需要 `ValueOrigin {source, raw, normalized, digest, precedence, reason}`。 |
| APP-LAUNCH-P1-016 | **Open** | project plugins merge 只留下来源集合，丢失顺序、provider generation、去重前后列表和 winning definition。改为有序 immutable overlay receipt，冲突默认 fail-closed。 |
| APP-LAUNCH-P1-017 | **Open** | `ProductConfigSourceSet(u8)` 是固定四位 bitset，扩展新来源会改变语义且无法保留历史未知 source。用 versioned extensible source IDs/ordered vector，并拒绝未识别 schema。 |
| APP-LAUNCH-P1-018 | **Open** | `resolve()` 主要验证 role/profile/target/render/window，不验证 `ProductCapabilityRequirement::HostProvided` 的 admission；input 也没有和 host capability 形成有效闭环。resolve 必须消费 Runtime capability snapshot 并对 Required/Forbidden/HostProvided 给出明确结果。 |
| APP-LAUNCH-P1-019 | **Open** | `ProductPlatformClass::HostProvided` 总是 `true`，`default_platform_target` 对 current desktop、Wasm、Android 做默认猜测；主机不具备对应 adapter/SDK/ABI 时仍能产生 resolved config。无能力证明就拒绝，不用默认 enum 冒充支持。 |
| APP-LAUNCH-P1-020 | **Open** | Runtime profile vocabulary 与产品 role vocabulary 分裂：`RuntimeSessionProfile` 含 RuntimePipelined/Editor/Dev/Minimal/Headless，`RuntimeProfileId`/ProductRole 又有另一组值。建立 registry，profile id、role、artifact、runner、capability 和 deprecation alias 由同一 catalog 提供。 |
| APP-LAUNCH-P1-021 | **Open** | resolved config 不包含 library selection、ABI/API table identity、plugin artifact digest、target triple、toolchain、environment allow-list 或 BuildSet digest；加载后无法证明它就是被解析的产品。将这些纳入 qualified launch context。 |
| APP-LAUNCH-P1-022 | **Open** | `EntryConfig` 与 `ResolvedProductHostConfig` 没有 canonical serialize/hash/equality contract；相同请求中 Vec order、path spelling、default insertion 和 plugin merge 顺序可能产生不同物理结果。定义 deterministic normalization 和 hash golden。 |
| APP-LAUNCH-P1-023 | **Open** | config error 主要是 typed enum + display string，但没有 source span、field path、candidate values、machine code、secret class 和 remediation payload；上层只能把 error 转成自由文本。统一 structured diagnostic envelope。 |
| APP-LAUNCH-P1-024 | **Partial** | role/profile conflict、render/window conflict、plugin overlay、provenance 和 artifact descriptor 已有约 20 个 focused tests；没有序列化 round-trip、schema migration、capability probe、environment snapshot、conflict explain 或 cross-entry same-receipt tests。 |

### 5.3 Environment、library、preferences 与 diagnostics（APP-LAUNCH-P1-025..036）

| ID | 状态 | 差距与重构方向 |
|---|---|---|
| APP-LAUNCH-P1-025 | **Open** | `ZIRCON_LOG_FILTER > ZIRCON_LOG > RUST_LOG` 的 precedence 在 Runtime 中实现，但 Editor help 没有完整记录 alias，invalid env 值只 `eprintln!` 后回退默认。环境值应进入 resolved input receipt；非法值默认 fail-closed 或由显式 compatibility policy 决定。 |
| APP-LAUNCH-P1-026 | **Open** | `ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME` 非 `1/true/yes` 时静默 false；同一进程对 invalid boolean、empty、non-UTF-8 没有统一诊断。采用 typed boolean parser，invalid/ambiguous 值必须带 code 和 source。 |
| APP-LAUNCH-P1-027 | **Open** | first-frame=true 会优先返回 frame limit，因而可能隐藏 invalid `ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES`；两个互相冲突的环境输入不应靠短路决定是否校验。先完整解析并记录 conflict，再按 policy 选 winner。 |
| APP-LAUNCH-P1-028 | **Open** | 空 `ZIRCON_RUNTIME_LIBRARY` 被当作 unset，空白字符串却报错；selection provenance 只有私有测试访问器，未并入 product launch receipt。统一 env tri-state、path identity、artifact manifest/hash/API/target 验证和可公开的 selection receipt。 |
| APP-LAUNCH-P1-029 | **Open** | library default 依赖 executable sibling/`deps` 猜路径，环境 override 可为绝对路径；没有允许根目录、symlink/junction policy、签名/信任、TOCTOU snapshot 和 target/build identity。由 BuildSet manifest 提供受控 locator。 |
| APP-LAUNCH-P1-030 | **Open** | platform preferences 直接读取 `LOCALAPPDATA`/`XDG_DATA_HOME`/`HOME`，并落到 engine-global `ZirconEngine/preferences`；这些环境和路径不是 launch identity，移动/browser/headless 也没有明确 backend。preferences root 应由 product role/portable policy resolver 生成并记录。 |
| APP-LAUNCH-P1-031 | **Open** | Runtime 在 resolve project、frame env、exit env、library load 等早期阶段可能尚未初始化正式 log；失败通过 stderr/字符串返回，无法进入同一 bootstrap ring。进程入口应立即启用 bounded early diagnostics，正式 sink 初始化后 replay。 |
| APP-LAUNCH-P1-032 | **Open** | `RuntimeStartupExecutionError`、`EditorLaunchArgumentError`、config error 和 commandlet report 各自 Display；缺少稳定 error code/domain/source chain/field path/secret classification。实现可机器消费、可本地化但不依赖文本匹配的 diagnostic model。 |
| APP-LAUNCH-P1-033 | **Open** | 一般启动错误没有 JSON receipt；只有 commandlet 有 `CommandletReport`，Runtime/GUI 仍输出自由文本。为 supervisor/Hub/CI 提供 bounded structured terminal/startup report，human log 是另一个 sink。 |
| APP-LAUNCH-P1-034 | **Partial** | Editor GUI 对 project-intent payload 和路径有格式化 redaction，Runtime project diagnostic 也有 display path；没有全局 privacy registry，library/env/scene/layout/profile 和 child token 的脱敏策略不一致。 |
| APP-LAUNCH-P1-035 | **Open** | profiling stop/export failure、capture path failure、log shutdown failure 与 product failure ledger 的归属不一致；部分只 `eprintln!`，部分覆盖 exit，部分作为 `?` 返回。所有 observability side effect 必须是 secondary ledger item，且 terminal receipt 声明 durability。 |
| APP-LAUNCH-P1-036 | **Open** | `runtime_process_teardown_complete` 在 log shutdown 完成前写入，runtime/editor binary 的 top-level teardown 诊断格式也不一致。只能在 evidence flush 成功后发布 completed；失败需保留 primary cause 和 emergency sink 记录。 |

### 5.4 退出、commandlet、跨入口与验证（APP-LAUNCH-P1-037..040）

为避免与 App08 的主循环/Play child finding 重复，本组只列启动合同对这些边界的影响；App08 仍拥有 transport/lifecycle 实现。

| ID | 状态 | 差距与重构方向 |
|---|---|---|
| APP-LAUNCH-P1-037 | **Open** | Editor commandlet 使用 `--run` hand parser 和稳定 u8 code，App 先全局检查 help；GUI/Runtime/commandlet 的 root mode、subcommand、version/capability discovery 没有统一 catalog。用一个 role-aware command tree 生成 parse/help/version/diagnostic。 |
| APP-LAUNCH-P1-038 | **Open** | commandlet report 是 stdout JSON，但一般 Editor startup/help 和 Runtime Play report 也可能写 stdout；没有正式 stdout protocol ownership，混入日志会破坏 CI parser。定义 stdout/stderr/typed control channel 的独占合同；Play transport 细节由 App08 实现。 |
| APP-LAUNCH-P1-039 | **Open** | 只有 parser/helper tests 和少量 process log lifecycle test，没有 spawn `zircon_editor` 与 `zircon_runtime` 的真实 integration matrix，无法断言 exit code、stdout/stderr、env precedence、library selection、help ordering 和 report durability。增加跨 binary process contract tests。 |
| APP-LAUNCH-P1-040 | **Open** | 没有 parser fuzz/property/corpus、response-file cycle、long path、reserved name、non-UTF-8 OS arg、oversized JSON、duplicate alias、random env combination 的资格测试。把 grammar/schema/budget/receipt 作为可生成的测试输入。 |

P1 共 **40 项：37 Open、3 Partial、0 Closed**。Partial 仅指已有局部 duplicate/grammar tests、config/provenance tests 和路径/payload redaction；这些底座不能替代统一 launch authority 或真实跨进程资格。

## 6. P2 治理与长期质量

| ID | 状态 | 收敛方向 |
|---|---|---|
| APP-LAUNCH-P2-001 | **Open** | profile/role/platform/commandlet 名称统一 stable schema id，显示名称和 deprecated alias 分开。 |
| APP-LAUNCH-P2-002 | **Open** | help/version/capability 输出从同一 option catalog 生成，避免手写字符串漂移；同时保留 human-readable 与 machine-readable projection。 |
| APP-LAUNCH-P2-003 | **Open** | 所有 path、URI、JSON、token、filter 使用 typed bounded value，禁止把 raw `String` 当作跨层身份。 |
| APP-LAUNCH-P2-004 | **Open** | launch receipt、diagnostic event、failure ledger、BuildSet/artifact receipt 共享 trace/request/attempt/generation identity。 |
| APP-LAUNCH-P2-005 | **Open** | 提供 explain-config/diff/replay projection，能够回答每个字段由哪个输入覆盖以及被拒绝的候选是什么。 |
| APP-LAUNCH-P2-006 | **Open** | 统一 env/argv deprecation、版本兼容、unknown-field 和 forward/backward migration policy，避免隐式 alias 永久存在。 |
| APP-LAUNCH-P2-007 | **Open** | 记录启动阶段耗时、argv parse、config resolve、library load、composition、first-ready、shutdown flush 等 stage timing，但不把 timing 当作功能完成证据。 |
| APP-LAUNCH-P2-008 | **Open** | 为 secret/path/env 建 privacy class、stable hash 和 redaction registry，所有 sink 复用同一策略。 |
| APP-LAUNCH-P2-009 | **Open** | cold/warm launch、help/commandlet、invalid input、library fallback、failure shutdown 和 replay 具备跨 OS 的 golden receipt。 |
| APP-LAUNCH-P2-010 | **Open** | 对小型 parser 与 receipt projection 优化 clone/格式化，但必须先满足预算、确定性和安全边界；不能用微优化替代统一权威。 |

## 7. 参考引擎对照

| 参考 | 已读取的工程事实 | Zircon 应吸收 | 不应照搬/当前反例 |
|---|---|---|---|
| Unreal | `FCommandLine` 保留 original/filtered/logging command line，有固定 `MaxCommandLineSize`、registered args、allow-list 和 subprocess inheritance；`LaunchEngineLoop` 有多阶段 PreInit/Init/Exit、`-CmdLineFile=` 展开、重复文件追踪、错误级别和大量 boot timing；`ConfigCacheIni` 有层级 config、command-line override、dynamic layer、NoSave/write policy。 | 一次初始化的 canonical command line、bounded expansion/cycle detection、来源优先级、临时 override 不回写、阶段化启动和显式错误级别。 | 不复制 Unreal 全局单例、宏或 INI 具体格式；目标是约束和证据模型。 |
| Godot | help 中明确 `--`/`++` user-args separator、`--version`、editor/headless/path/mode availability；`Main::setup/setup2/start/cleanup` 分阶段，project settings 支持 `project.godot`/binary、override.cfg、feature override、resource pack 与测试。源码也承认 setup/start 重复解析是历史问题。 | engine/user arg 分界、完整 mode/help/version catalog、project/resource/config override 的有序层和 cleanup phase。 | 不复制其已知 duplicated parsing；它正好说明 App09 必须保持单一 parser schema。 |
| Bevy | `AppExit` 以 Success/Error(nonzero u8) 建模，runner 返回正确 code，`should_exit` 选择第一个 error；Winit runner 在 event loop 无 exit code 时记录 bug 并返回 error。 | 退出结果是 typed value，不是字符串；runner 必须证明 event loop 的终态，否则失败。 | Bevy 的 u8 错误域仍不足以表达本产品的 startup/runtime/shutdown ledger；需在 Zircon 上层保留语义 class。 |
| Fyrox | `editor-standalone` 用 clap derive、typed `Option`/`Vec`/bool、version/description；但 `fyrox-impl::executor` 的 `Args::try_parse().unwrap_or_default()` 会把非法 CLI 静默变成默认参数。 | typed parser、生成 help/version 和明确 option ownership。 | **不吸收 silent fallback**；Zircon 的 invalid option 必须 fail-closed 并进入 receipt。 |
| Unity Graphics | `CoreBuildData` 是 build-scoped singleton，按 active build target 解析 render pipeline assets、managed code variant、GPU-resident drawer 支持并在 Dispose 清理缓存。它属于 graphics build context，不是 Player process launcher。 | 将 target/build variant/capability snapshot 绑定到一次 build/launch attempt，并有明确 dispose。 | 不从 Graphics corpus 推断 Unity Player/Editor CLI；App09 的进程合同仍以 Unreal/Godot/Bevy 级别的 host 源码为依据。 |

参考证据中的共同工程约束是：输入有界且可分层，配置覆盖有顺序和可解释来源，启动分阶段并可返回错误，退出由 typed result 驱动，测试覆盖实际边界。当前 Zircon 只拥有这些约束的局部命名和 helper，尚未把它们收敛为一个 product launch authority。

## 8. 目标架构

### 8.1 Immutable launch request 与 resolution transaction

定义：

```text
RawOsLaunchInput
  -> TokenStream (argv/env/response/user separator, bounded)
  -> ProductLaunchRequest (typed, versioned, secret-aware)
  -> CapabilitySnapshot + ProjectSnapshot + BuildSetSnapshot
  -> ResolvedProductLaunch (immutable, canonical, hashed)
  -> ProductComposition / ProductSession / Commandlet / Export service
```

`ProductLaunchRequest` 只表示用户/Hub/Editor/CI 的意图；`ResolvedProductLaunch` 才拥有最终 role、profile、target、platform adapter、render/window/input requirements、library/artifact/build-set identity、plugin/module graph、preference/log policy 和 shutdown policy。任何入口都不得再次从 raw argv 选择 profile。

### 8.2 Provenance 与 replay

每个字段记录 raw source、normalized value、source kind、locator、precedence、candidate set、winner reason、digest、secret class 和 schema revision。整个 receipt 记录 canonical request digest、project manifest/resource snapshot、environment allow-list/hash、BuildSet/artifact/library identity、toolchain/platform capability snapshot、attempt/generation、stage timings 和 terminal outcome。对不可复放的 machine-local 值必须记录 reason 并显式标为 non-replayable，而不是省略。

### 8.3 Diagnostics 与 output channels

定义 bounded `LaunchDiagnostic`：`code/domain/severity/stage/field_path/token_span/source/recovery/secret_class/related_ids`。early ring 在正式 log sink 前收集同一结构；human stderr、structured stdout、Hub/Play control channel 由不同 writer 投影。stdout 协议必须有 schema/version/length/sequence，并禁止一般 log 混入机器 channel。App08 拥有 Play handshake 的状态机和 transport owner，App09 提供 launch receipt/diagnostic identity。

### 8.4 Terminal contract

```text
Resolve -> Admit -> Compose -> Start -> Running/Commandlet
  -> RequestStop -> Drain/Flush
  -> ProductTerminalOutcome(primary, secondary[], durable)
  -> stable process code
```

`ProductTerminalOutcome` 只产生一次，保留 first primary failure、ordered secondary ledger、report/IPC/log/profiling write result、shutdown phase 和 forced/emergency flag。`ExitCode` 是最后的 portability projection，不能反过来成为语义真值。

## 9. 分阶段重构计划

### M0：冻结 launch schema 与 ownership

- 冻结 role/profile/platform/target/linkage/capability catalog 和稳定错误/退出 code。
- 定义 bounded `RawOsLaunchInput`、typed `ProductLaunchRequest`、`ResolvedProductLaunch`、provenance receipt、diagnostic envelope 和 terminal outcome。
- 禁止新增第二个 parser、裸 `RuntimeSessionProfile` 选择、`Box<dyn Error>` 作为跨边界合同或 ad-hoc env product switch。

### M1：统一 Parser / Help / Env

- 将 diagnostic、Editor GUI、commandlet、Runtime session 合并为 role-aware option schema。
- 支持 args_os、`--` user args、canonical equals/separated forms、token span、response-file bounded expansion 和 non-UTF path policy。
- help/version/capability/error 全部从 schema 投影；invalid/duplicate/missing/unknown 不静默回退。

### M2：统一 Config / BuildSet / Capability

- Runtime binary 改为消费 `ResolvedProductLaunch`，删除独立 profile-to-bytes authority。
- resolver 同时读取 project/export/Hub/env/library/BuildSet/platform probe，校验 HostProvided/input/render/window capability 并 fail-closed。
- 生成 canonical digest 和 durable receipt；Editor/export/Hub/CI 必须验证同一 receipt。

### M3：Early diagnostics 与 terminal outcome

- 进程入口启用 early ring，正式 log sink replay；所有错误结构化并按 privacy registry 脱敏。
- failure ledger、profiling、report/IPC、log shutdown 统一为 secondary effects；primary cause 永不被 `?` 或 stdout 写失败覆盖。
- 建立跨平台 stable exit code registry 和 durable terminal receipt。

### M4：真实跨入口合同资格

- spawn `zircon_editor`、`zircon_runtime`、commandlet 和 packaged child，断言 argv/env/config/library/BuildSet/receipt 相等或明确不同。
- 覆盖 help precedence、unknown/duplicate/missing value、`--` separator、non-UTF-8、long path、oversized JSON/response、invalid env、library mismatch、crash/restart 和 stdout contamination。
- 与 App08 的 Winit/Play lifecycle fixture 组合，但不把 spawn 成功或文本 Ready 当作产品 ready。

### M5：可复放与性能证据

- 固定 OS/hardware/build/project，比较 cold/warm launch、config resolve、library load、commandlet、failure shutdown 和 replay receipt 的 P50/P95/P99/RSS/IO。
- 性能优化只在 receipt/diagnostic correctness、bounded parsing 和 deterministic normalization 通过后进行；不以 parser 微基准替代产品启动资格。

## 10. 产品资格 Gate

| Gate | 状态 | 关闭要求 |
|---|---|---|
| G01 single launch authority | **Fail** | Editor/Runtime/export/commandlet/Hub 对同一输入消费同一 `ResolvedProductLaunch` |
| G02 bounded token/JSON input | **Fail** | argv/env/response/user args 有 count/bytes/depth/cycle/encoding budget |
| G03 canonical config digest | **Fail** | normalized request、project/env/BuildSet 生成稳定 digest 和 durable receipt |
| G04 complete provenance | **Fail** | 每个字段有 raw/normalized/winner/precedence/reason/digest/secret class |
| G05 capability admission | **Fail** | Required/Optional/Forbidden/HostProvided/input/render/window 在 resolve 前闭合 |
| G06 parser/help/version contract | **Fail** | 单一 schema 生成 root/subcommand/help/version，help 不劫持 commandlet |
| G07 invalid input fail-closed | **Partial** | duplicate/invalid parser 已有局部测试；仍缺跨入口和 non-UTF/budget corpus |
| G08 secret/path privacy | **Fail** | argv/process list、stderr/stdout/log/receipt 对 token/path/env 使用统一 privacy registry |
| G09 early diagnostic durability | **Partial** | sink/flush 基础存在；早期错误与正式 receipt 尚未统一 replay/durability |
| G10 structured error envelope | **Fail** | code/domain/stage/field/span/recovery 可机器读取，文本不是合同 |
| G11 terminal primary preservation | **Fail** | report/log/profiling failure 只能 secondary，primary 永不被覆盖 |
| G12 stable exit code registry | **Fail** | semantic class/reason 到跨平台 code 的 mapping 可版本化且由单一 binary adapter 使用 |
| G13 commandlet stdout isolation | **Fail** | JSON report、human log、help 和 diagnostics 有独占 channel/协议 |
| G14 cross-binary process tests | **Fail** | 真实 spawn/env/argv/exit/stdout/stderr/library/receipt matrix 通过 |
| G15 replay/migration | **Fail** | receipt 可复放、版本迁移可审计，未知未来 schema 明确拒绝 |
| G16 launch performance baseline | **Fail** | cold/warm/error/replay P50/P95/P99、RSS/IO 和同项目基线可复现 |

## 11. 相邻责任边界

- `zircon_runtime/208` 拥有 qualified export BuildPlan、platform artifact、package/cook closure 和真实 launch handoff；App09 只解析并消费其 BuildSet，不复制 target/toolchain resolver。
- `zircon_app/08` 拥有 Winit loop、window/surface generation、dynamic DLL quiesce 和 Play child transport/lifecycle；App09 提供 launch identity、startup diagnostic、terminal outcome 所需的 request/receipt 字段。
- `zircon_editor/269` 拥有 Editor export pipeline/preset/job/publish；Editor 必须通过统一 `ProductLaunchRequest` 调用 Runtime export service，不能再私有解释 role/profile。
- `zircon_editor` commandlet owner 继续实现 commandlet domain capability/report，但 root parser、help/exit/receipt 由 App launch authority 统一。
- `zircon_runtime` diagnostic log owner 继续实现 sink/backpressure/durability；App 负责把 early launch diagnostics 和 terminal outcome 接入同一 sink，不修改日志内部性能合同。
- `zircon_runtime_interface` 拥有 ABI/table/version DTO；App 负责把 validated library/API/BuildSet identity 纳入 launch receipt。
- Hub/Tooling 可提交 request、读取 receipt 和 terminal report，但不拥有第二套 parser、config resolver 或成功判定。

## 12. 本轮完成定义

本轮完成了 App09 选择集的逐文件静态审查、直接 Editor/Runtime 合同读取、Unreal/Godot/Bevy/Fyrox/Unity Graphics 参考对照、P0/P1/P2 复构计划和 16 道资格门定义。没有修改 production Rust/Cargo/ABI/tests/UI，也没有运行动态资格、跨平台构建或性能比较；`implementation_status` 保持 `pending`。

App09 是新增的 launch/config/diagnostics/exit 责任域，不替代 App08 的 current-source refresh；两份报告不能把同一主循环、Play transport 或窗口问题重复计数。实现前必须重新计算本文四组 fingerprint 并重做工作树 currentness 检查。只有统一 launch receipt 被真实 Editor/Runtime/commandlet/export process matrix 验证，且 terminal/diagnostic/privacy/replay gate 通过后，才能把 Open/Partial 改为 Closed。
