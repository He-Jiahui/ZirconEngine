---
title: Runtime Product Build/Export Profile/Build Plan/Platform Host/Cross Compilation/Package/Launch Handoff 当前工作树复审
category: zircon_runtime
report_id: Runtime208
review_date: 2026-08-31
baseline_head: working-tree
observed_head: 18481bc218dc544d3232d7d8826ac5fb97f7cb0c
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
coordination_tracking: skipped_by_user_request
related_reports:
  - docs/plans/optimize/zircon_runtime/207-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-cook-package-incremental-build-worker-determinism-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
zircon_scope:
  - zircon_runtime/src/core/framework/project/export_profile.rs
  - zircon_runtime/src/asset/project/manifest/export_profiles.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan
  - zircon_runtime_interface/src/export
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/product_composition
  - zircon_app/src/entry/product_host_config
  - zircon_app/src/entry/engine_entry.rs
reference_scope:
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/BuildCookRun.Automation.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/ProjectParams.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/Platform.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Gauntlet/SelfTest/Unreal/Gauntlet.SelfTest.TestUnrealInstallAndRunDesktop.cs
  - dev/godot/editor/export/editor_export_platform.h
  - dev/godot/editor/export/editor_export_platform.cpp
  - dev/godot/editor/export/editor_export_preset.h
  - dev/godot/editor/export/editor_export_preset.cpp
  - dev/godot/platform/windows/export/export_plugin.cpp
  - dev/godot/platform/android/export/export_plugin.cpp
  - dev/godot/platform/web/export/export_plugin.cpp
  - dev/godot/tests/core/io/test_pck_packer.cpp
  - dev/bevy/crates/bevy_asset/src/processor/mod.rs
  - dev/bevy/crates/bevy_asset/src/processor/process.rs
  - dev/bevy/crates/bevy_asset/src/processor/log.rs
  - dev/bevy/crates/bevy_asset/src/processor/tests.rs
  - dev/Fyrox/fyrox-build-tools/src/export/mod.rs
  - dev/Fyrox/fyrox-build-tools/src/export/pc.rs
  - dev/Fyrox/fyrox-build-tools/src/export/android.rs
  - dev/Fyrox/fyrox-build-tools/src/export/wasm.rs
  - dev/Fyrox/fyrox-build-tools/src/export/asset.rs
  - dev/Fyrox/fyrox-build-tools/src/export/utils.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/BuildProcessors/CoreBuildData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/BuildProcessors/CorePreprocessBuild.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderStripping/ShaderPreprocessor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderStripping/ShaderStrippingReport.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/ShaderStripping/ShaderStrippingTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/ShaderStripping/ShaderStrippingReportTests.cs
---

# Runtime Product Build/Export Profile/Build Plan/Platform Host/Cross Compilation/Package/Launch Handoff 当前工作树复审

## 1. 结论

当前 Runtime 已经有一组可保留的局部底座：`ExportProfile`/`ExportTargetPlatform` 具备基础序列化和角色映射；`ExportBuildPlan` 能投影插件、Cargo manifest、平台 scaffold 和校验报告；生成文件有确定性排序、固定 ZIP 时间/权限；`ProductCompositionRequest` 能在 App 边界解析角色、Runtime profile、插件和平台 target；移动/浏览器 FFI 有启动/停止状态保护和 panic guard。这些能力说明工程并非空白，但它们还没有组成可发布的产品构建服务。

本轮最重要的判断是：当前代码把“导出配置被识别”误当成“目标平台产品已构建并可运行”。桌面/Headless 生成的 `main.rs` 只 bootstrap `ProductComposition` 后立即返回；移动和浏览器生成的 `lib.rs` 只持有 `ProductComposition`，输入、生命周期、viewport 和资源回调只是 `is_running` 探针；`LibraryEmbed` 与 `SourceTemplate` 的 Cargo 命令没有 `--target`、架构、SDK、linker 或工具链 receipt；移动/浏览器包脚本要求用户手工编译、复制 Rust 库和资源。因而当前成功路径无法证明有可启动、可渲染、可加载资源的目标产品。

当前账目为 **P0 5 Open / 0 Partial / 0 Closed，P1 44 Open / 4 Partial / 0 Closed，P2 12 Open / 0 Partial / 0 Closed**；资格门 **10 Fail / 0 Partial / 0 Pass**。这不是与 Unreal 的性能比较，也不是跨平台支持声明。没有在固定项目、固定 SDK/工具链和真实目标设备上完成 build/package/install/launch/first-frame/资源加载验证，不得声称性能或表现优于 Unreal。

Runtime208 不重复 Runtime207 的 importer、derived-data、cook 与 pack 内部问题，也不重复 Editor269 的 preset UI、Editor job routing 和 legacy/wizard 双路径问题。这里只拥有 Runtime/App 的产品请求、目标资格、BuildPlan、materialize/publish、平台宿主和 launch handoff；Editor/CLI/CI 应通过同一个 Runtime-owned service 调用。Tooling/Python 按用户要求排除，只记录其当前被迫承担 orchestration 的事实。

## 2. 审查边界与冻结快照

### 2.1 证据等级与限制

| 等级 | 本轮使用方式 | 能证明什么 |
|---|---|---|
| E3 | 逐文件读取 Runtime export profile、BuildPlan、Cargo 计划、materialize、平台宿主模板、App composition/bootstrap 与 interface report | 当前字段、调用链、状态、写入边界和断链 |
| E2 | 读取 Unreal AutomationTool/Gauntlet、Godot 平台 exporter/preset/PCK tests、Bevy processor/WAL、Fyrox build tools、Unity Graphics build-scoped data/strip report | 可吸收的工程合同与明确的参考上限 |
| E1 | 读取 Zircon export/profile/platform tests 和生成字符串断言，本轮未运行 | 静态测试意图，不代表动态通过 |
| E0 | 未运行 Cargo、Android/iOS/Xcode/Gradle/wasm、真实 GPU/窗口、安装、first-frame、崩溃恢复、签名、跨机 reproducibility 或 benchmark | 不得宣称“可发布”“可运行”或性能领先 |

`rg.exe` 在本机被 Windows App package 拒绝访问，本轮使用 PowerShell 的定向文件读取和 `Select-String`。没有因此放宽范围，也没有等待外部协调器状态。

### 2.2 Zircon 选择集

本轮冻结 29 个 Runtime/App/Interface 生产 Rust 文件，5,441 行、4,943 非空行、227,645 bytes、16 个 test markers、1 个 ignored marker；按小写 workspace-relative path、每文件 SHA-256、`path + NUL + hash + LF` 排序串联计算 fingerprint：`4fdf22a3814f3e1ce62418448c94056c2f085aa4b151dc0e44fb64a69a33e5f3`。选择集包含 `export_build_plan` 的 profile/build/materialize/platform host 生产链和 App composition/config；排除独立 tests 目录，但保留生产文件内 test marker 计数。工作树存在并发修改，fingerprint 比提交号更能表达本报告所读内容。

参考选择集为 28 个文件、22,362 行、19,514 非空行、869,553 bytes、25 个 test markers，fingerprint：`52fa7d2c01f10903e55175a1d6a7fc7312a630eb84a43b757bc07ccc067d0c99`。Unreal 是 Build/Cook/Stage/Package/Archive/Deploy/Run 的主要上限；Godot 是平台能力、preset、模板/SDK/架构和失败清理对照；Fyrox 是 Rust 侧最低限度的 target/cancel/asset-copy 落点，不是目标上限；Bevy 只用于处理状态、依赖、WAL 和并发文件事务；Unity Graphics 只用于 build-scoped graphics context、扩展点生命周期和 stripping receipt。

## 3. 当前链路事实

### 3.1 Profile、policy 与 App role 只有元数据闭环

`zircon_runtime/src/core/framework/project/export_profile.rs:10-121,138-259` 将 Windows/Linux/macOS/Android/iOS/WebGPU/Wasm/Headless 映射到 host kind、resource strategy、plugin strategy 和 `supports_native_dynamic`。`ExportProfile` 实际字段只有 target mode、可选 runtime profile、platform、策略列表、Debug/Release、output name、插件、features 和 asset filter，没有 target triple/architecture/ABI、最低 OS/SDK、graphics backend/capability、toolchain/template identity、signing/package format、device/install policy 或 schema revision。

`zircon_app/src/entry/product_host_config/product_role_request.rs:49-72` 将平台映射到 Desktop/Server/Web/Mobile/Embedded 角色；`resolution.rs:198-285` 再映射为 `PlatformTarget`。这证明配置能够被识别，但 `ResolvedProductHostConfig.platform_target` 只进入 App platform config 和 provenance，未成为 Cargo target、linker、SDK 或 platform adapter 的输入。iOS 映射为 `Embedded`，代码注释也明确其 capability admission 尚未完成，这不应在导出矩阵中呈现为已支持平台。

### 3.2 BuildPlan 是投影，不是执行服务或不可变 build set

`zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs`、`from_project_manifest.rs:73-420` 能选择 profile、解析插件、生成文件和可选 compile/source plan；`library_embed_compile_plan.rs:67-126` 按 target mode 选择 `zircon_app` binary/feature，`source_template_build_plan.rs:25-65` 生成 `cargo build`。两者的命令只包含 manifest、package/binary、feature、target-dir 和 `--release`，完全没有 `--target`、cargo/rustc 版本、toolchain channel、SDK/linker、locked/frozen/offline、环境白名单、source revision、feature closure 或 artifact receipt。

`cargo_manifest_template.rs:1-42` 生成指向 `../../zircon_app`、`../../zircon_runtime`、`../../zircon_plugins` 的相对 path dependency。SourceTemplate 因而依赖原仓库布局，不能作为独立、可复制、可审计的产品源包；不同机器/不同 workspace 的同一 profile 可能解析不同依赖。

### 3.3 Desktop/Headless launch 是立即退出

`zircon_runtime/src/plugin/export_build_plan/main_template.rs:3-11` 生成的 `main` 仅调用 `bootstrap_export_runtime[_with_native_plugins_from_export_root]`, 保存到局部变量后返回 `Ok(())`。没有 host window/surface、frame clock、tick、event pump、render submit、resource mount、shutdown drain、exit code 或 first-frame barrier。Headless 也没有 server loop、signal/shutdown 或 health endpoint。一个成功进程只证明 Core composition 构造成功，不证明产品启动。

### 3.4 Mobile/browser host 的 ABI 只证明“状态存在”

`platform_host_files.rs:46-286` 的 `ZirconProductCompositionState` 只有 Vacant/Starting/Running/Stopping；`zircon_export_handle_lifecycle`, touch, keyboard, viewport 和 `zircon_export_fetch_resource` 都调用 `zircon_export_is_running`，没有把事件送入 Runtime input/lifecycle/window/resource owner。Android JNI keyboard 丢弃 text；所有 Android lifecycle 数值最终被 Rust 忽略。

`platform_host_files/mobile.rs:187-335` 生成的是 Kotlin/Swift scaffold。Android 固定 compileSdk/minSdk/targetSdk 为 35/28/35，README 要求用户为每个 ABI 手工编译 `.so` 并复制到 `jniLibs`；release script 创建 Google Play edit 但没有上传 bundle/commit edit。iOS 只有 Swift Package 和 unsafe `-L`，脚本只是 `swift build`，没有 Xcode project/scheme、XCFramework、多架构 slice 或 archive 资格。

`platform_host_files/browser.rs:82-189` 生成的 npm manifest 使用 `latest` 依赖且无 lockfile；`package-export.mjs` 只复制 `zircon-project.toml`，不解析 canonical asset/cook/pack closure；Wasm host 的 `zircon_host_fetch_resource` 只打印“requires generated memory adapter”并返回 0，keyboard code/text 传入零。`#[allow(unreachable_code)]` 下还保留旧 host 字符串，说明实现尚未完成收敛。

### 3.5 Materialize/publish 不是事务

`materialize/generated.rs:15-42` 对每个文件直接 `fs::write`，没有 temp+fsync+rename、ownership manifest、stale output removal、rollback 或 cancel fence。`materialize/archive.rs:18-71` 在所有生成/原生包校验和写入完成前直接 `File::create(archive_path)`；后续错误会截断原有 archive。`write_native_package_entries:96-161` 对选中的但缺少 `plugin.toml` 的包只追加 diagnostic 并继续，报告仍可能没有 fatal。

生成文件写入会保留旧目录中未被本次 plan 声明的文件；native inventory 只保存路径，源文件在 inventory 后改变仍可被读取，存在 TOCTOU。`ExportMaterializeReport` 只有 archive/generated/copied paths 与字符串 diagnostics/fatals，没有 generation、request/profile/build identity、target/toolchain、digest/size、status、attempt、transaction/publish receipt。

### 3.6 Validate report 与 interface report 没有消费链

`export_validate_report.rs:16-229` 的 v2 report 能记录 profile/platform、generated file digest 和 plan summary，但 `zircon_export_validate` 只加载 manifest/build plan 并序列化 contents；它不验证目标工具链、SDK、真实 compile、materialize、bundle、install 或 run。

`zircon_runtime_interface/src/export/artifact.rs` 的 `ExportArtifactRef` 只有 key/locator/可选 digest；`report.rs` 的 `ExportStageStatus` 只有 Passed/Skipped/Failed，`ExportPipelineReport::record` 只是 first-match 查询。没有 digest algorithm/size/type/producer、Pending/Running/Cancelled/Aborted/Blocked、attempt/time/exit code/host/toolchain/environment/source revision，也没有唯一 stage/order/graph 校验。Interface DTO 因而不能成为 Runtime service 的事实来源，Editor/Python 自己维持 orchestration 时必然出现第二套状态语义。

## 4. P0 阻断

| ID | 状态 | 当前证据 | 必须完成的退出条件 |
|---|---|---|---|
| **RT-EXPORT-P0-001** | Open | 桌面/Headless `main_template.rs:3-11` bootstrap 后立即返回；移动/browser 只有 composition owner，没有 frame/session/surface/input/resource loop。 | Runtime-owned `ProductSession` 提供 start/admit/mount/tick/submit/present/stop；desktop/headless 有真实 loop，mobile/browser 有 surface/frame callback；first-frame、resource-load、graceful-shutdown smoke 全部通过后才能报告成功。 |
| **RT-EXPORT-P0-002** | Open | `library_embed_compile_plan.rs:102-126` 与 `source_template_build_plan.rs:51-65` 没有 `--target` 或 target-specific linker/SDK/toolchain；profile platform 只影响字符串 policy。 | 平台 registry 返回 immutable target triple/arch/ABI/SDK/toolchain/template capability receipt；命令、Cargo config、generated host 和 artifact 全部绑定同一 qualified target，unsupported/missing capability fail-closed。 |
| **RT-EXPORT-P0-003** | Open | Runtime `ExportBuildPlan`/materialize 与 `zircon_runtime_interface::export` DTO 没有 service/consumer 连接；Editor/Python 承担 orchestration，App 只 bootstrap。 | 建立唯一 `RuntimeExportService`：Editor/CLI/CI/App 都提交同一 typed request，服务拥有 DAG、stage state、journal、artifact/publish receipt；禁止旁路旧 command report 成功。 |
| **RT-EXPORT-P0-004** | Open | `materialize/archive.rs:31-54` 先 truncate archive；`generated.rs:35-42` 原地写；缺 native package 仅 diagnostic。 | 采用 per-generation staging namespace、temp file+durable rename、ownership/stale cleanup、rollback/quarantine；缺输入、写入错误、取消、digest mismatch 一律不发布且不可产生 Passed。 |
| **RT-EXPORT-P0-005** | Open | `generated_files.rs:60-64` 和 browser/mobile scripts 只复制 `zircon-project.toml`；没有 canonical cooked/packed resource closure 或 runtime mount receipt。 | BuildPlan 从 Runtime207 canonical source/cook/package graph 获取 immutable content closure；每个 bundle 记录 mount roots、artifact digests、size、target variant；launch 前验证并实际加载入口资源。 |

## 5. P1 工程化差距

### 5.1 Request/Profile/Target/Capability（RT-EXPORT-P1-001..012）

| ID | 状态 | 差距与重构方向 |
|---|---|---|
| RT-EXPORT-P1-001 | Partial | `ExportProfile` 与 `ExportPreset` 分离且只用字符串/profile_ref；引入 versioned immutable profile identity、revision/digest 和 migration。 |
| RT-EXPORT-P1-002 | Open | target mode 在 profile、runtime profile、App role 三处重复；构造 `EffectiveExportRequest`，只保留一个 qualified mode。 |
| RT-EXPORT-P1-003 | Open | platform policy 是 enum match，不是可发现 owner；建立 `PlatformAdapterRegistry`、capability probe、generation lease 和 unsupported reason。 |
| RT-EXPORT-P1-004 | Open | 缺 target triple、architecture、ABI、CPU feature、minimum OS、graphics backend；这些字段必须进入 request/key/receipt。 |
| RT-EXPORT-P1-005 | Open | 缺 rustc/cargo/toolchain/SDK/NDK/Xcode/Gradle/template/linker 版本与路径身份；建立 host capability snapshot，路径只作为受控 locator。 |
| RT-EXPORT-P1-006 | Open | Debug/Release 只有两态；增加 Development/Shipping/Checked、symbols/strip/LTO/Panic/UBSan/sanitizer、reproducible policy。 |
| RT-EXPORT-P1-007 | Open | `strategies` 可组合但没有冲突矩阵和唯一 packaging owner；验证 SourceTemplate/LibraryEmbed/NativeDynamic 与平台/role 的闭合性。 |
| RT-EXPORT-P1-008 | Open | output name 仅 trim/sanitize，缺 case-fold、reserved names、collision、path length 和 package identifier 合同。 |
| RT-EXPORT-P1-009 | Open | selected plugins/features 是字符串集合，未绑定 catalog/provider/package/artifact digest；冻结 provider generation 和 feature closure。 |
| RT-EXPORT-P1-010 | Open | asset_filter 无 schema、entry/keep/custom file/conflict/budget 语义；改为 typed content selection query。 |
| RT-EXPORT-P1-011 | Open | manifest 只有项目格式版本，没有独立 export schema、compatibility domain、migration/downgrade refusal。 |
| RT-EXPORT-P1-012 | Partial | App 能记录 platform provenance，但未把 provenance 传给 build/launch；形成跨层 `QualifiedTargetContext`。 |

### 5.2 Build graph、stage、receipt、resume（RT-EXPORT-P1-013..024）

| ID | 状态 | 差距与重构方向 |
|---|---|---|
| RT-EXPORT-P1-013 | Open | BuildPlan 是一次性 projection，没有 request/build-set identity；建立 immutable `ProductBuildDefinition`。 |
| RT-EXPORT-P1-014 | Open | Interface 八阶段没有 Runtime consumer；由 Runtime 注册唯一 DAG，Editor 只投影。 |
| RT-EXPORT-P1-015 | Open | stage status 缺 Pending/Running/Cancelled/Aborted/Blocked、attempt、timestamps、exit code、resource metrics；扩展 typed receipt。 |
| RT-EXPORT-P1-016 | Open | `ExportPipelineReport::record` 可存在重复 stage、乱序和隐式覆盖；校验唯一性、拓扑、input/output ownership。 |
| RT-EXPORT-P1-017 | Open | fingerprint 没有 source revision、recipe、toolchain、environment、target、provider generation；统一 cryptographic build key。 |
| RT-EXPORT-P1-018 | Open | validate 只验证静态生成内容；加入 toolchain probe、compile/link、bundle schema、mount closure 和 run policy。 |
| RT-EXPORT-P1-019 | Open | 没有 WAL/append-only journal；启动时必须恢复未完成 action，损坏/未来版本要 quarantine 而不是静默 fresh run。 |
| RT-EXPORT-P1-020 | Open | resume 没有 lease/fence/attempt lineage；同一 output 不能被旧 generation 覆盖，skip 必须重新验证 receipt。 |
| RT-EXPORT-P1-021 | Open | planned/generated/copied/published 路径混在 `ExportMaterializeReport`；分离 declared/materialized/verified/published artifact。 |
| RT-EXPORT-P1-022 | Open | 没有 progress/log stream 的 bounded tail、structured event、redaction、backpressure 和 cancellation receipt。 |
| RT-EXPORT-P1-023 | Open | 阶段失败后无 cleanup/quarantine policy；定义每个 stage 的 reversible/irreversible boundary 和 recovery action。 |
| RT-EXPORT-P1-024 | Partial | ZIP 排序、固定 timestamp/permission 是确定性底座，但没有 source snapshot/TOCTOU fence、完整 input digest 和 cross-machine golden。 |

### 5.3 Compile、package、platform host、launch（RT-EXPORT-P1-025..040）

| ID | 状态 | 差距与重构方向 |
|---|---|---|
| RT-EXPORT-P1-025 | Open | Cargo 命令未传 `--target`、`--locked`/`--frozen`、config、toolchain；由 adapter 生成受控 command/env，记录完整 argv。 |
| RT-EXPORT-P1-026 | Open | generated Cargo path dependencies 指向仓库相对路径；改为 source snapshot/ vendored package / immutable dependency manifest。 |
| RT-EXPORT-P1-027 | Open | 无 compile artifact receipt、binary format/arch/ABI/exports/symbols/size/digest；生成后必须 inspect 并签发 receipt。 |
| RT-EXPORT-P1-028 | Open | 无 native dynamic/static/VM plugin closure、ABI compatibility、load test、quarantine；插件加入 target-qualified artifact graph。 |
| RT-EXPORT-P1-029 | Open | Android 只生成 Gradle scaffold，固定 SDK 版本且手工复制 `.so`；平台 adapter 必须完成 ABI matrix、NDK/SDK admission、assemble/install/smoke。 |
| RT-EXPORT-P1-030 | Open | iOS `swift build` 不是 Xcode archive；生成 Xcode project/scheme、XCFramework、多 slice、codesign/provision receipt 和 device launch test。 |
| RT-EXPORT-P1-031 | Open | Browser npm `latest`、无 lockfile、package 只复制项目 manifest；锁定依赖、产生 WASM/asset closure、COOP/COEP、MIME/SRI receipt。 |
| RT-EXPORT-P1-032 | Open | CDN deploy 以 shell string + `shell:true` 上传，缺 argv boundary、dry-run、remote receipt、rollback/version path；迁至受控 publish adapter。 |
| RT-EXPORT-P1-033 | Open | desktop 没有 window/surface/backend selection、DPI/input mapping、frame pacing、exit semantics；由 `ProductSessionHost` 所有。 |
| RT-EXPORT-P1-034 | Open | mobile/browser 事件只过 FFI 探针，keyboard/text/lifecycle/resource 未入 Runtime；定义 versioned ABI、queue/backpressure、memory adapter、error/sequence receipt。 |
| RT-EXPORT-P1-035 | Open | `ProductComposition` 只有 Core/plugin owner，没有 mount/session/tick/render/shutdown API；拆分 composition、session、surface、resource mount 的生命周期。 |
| RT-EXPORT-P1-036 | Open | `zircon_export_fetch_resource` 不验证 URI、mount、length、stream completion；实现 bounded async callback/stream and capability-scoped resource resolver。 |
| RT-EXPORT-P1-037 | Open | host scaffold 中硬编码 Android/iOS/Web 版本、package id、application version；移入 profile schema 并由 platform capability 校验。 |
| RT-EXPORT-P1-038 | Open | publish/install/run 没有 target device discovery、clean install、health/first-frame/exit code；定义 launch handoff protocol。 |
| RT-EXPORT-P1-039 | Open | 无 SBOM/license/third-party runtime dependency closure、signing/notarization/attestation；作为 final artifact policy，而非 README 示例。 |
| RT-EXPORT-P1-040 | Open | 没有多架构/universal/fat binary、fallback backend、optional capability policy；矩阵必须逐 slice 构建并验证。 |

### 5.4 资源、可观测性、测试（RT-EXPORT-P1-041..048）

| ID | 状态 | 差距与重构方向 |
|---|---|---|
| RT-EXPORT-P1-041 | Open | build/launch 未消费 Runtime207 canonical cook/package graph；root 必须解析 qualified closure，不接受手写 raw path 作为事实。 |
| RT-EXPORT-P1-042 | Open | artifact/pack locator 没有 algorithm、size、type、producer、mount group、variant；扩展 `ExportArtifactRef`。 |
| RT-EXPORT-P1-043 | Open | 没有 artifact GC/refcount/pin/quota 和 staged namespace scavenger；发布事务必须可回收。 |
| RT-EXPORT-P1-044 | Open | diagnostics 仍是 `Vec<String>`；采用 code/severity/domain/source span/action/provider/target/secret-redaction。 |
| RT-EXPORT-P1-045 | Partial | native package inventory 和 generated file capacity 有局部测试，但无 fault injection、crash/restart、stale cleanup、TOCTOU 和 cancellation E2E。 |
| RT-EXPORT-P1-046 | Open | 无 target matrix tests 覆盖 Windows/Linux/macOS/Android/iOS/WebGPU/Wasm/Headless 的 unsupported、compile、bundle、launch。 |
| RT-EXPORT-P1-047 | Open | 无 install/first-frame/resource callback/keyboard/lifecycle/frame pacing golden；必须由 host adapter contract tests 和真实设备 smoke 覆盖。 |
| RT-EXPORT-P1-048 | Open | 无与 Unreal 同内容、同工具链、冷暖缓存、RSS/CPU/I/O/p95 和 correctness gate 的 benchmark；在声称性能前建立公开 corpus。 |

## 6. P2 治理项

| ID | 状态 | 收敛方向 |
|---|---|---|
| RT-EXPORT-P2-001 | Open | `ExportTargetPlatform`/strategy/build mode 用 stable schema id 和 compatibility matrix，拒绝未知/降级反序列化。 |
| RT-EXPORT-P2-002 | Open | 所有 duration/size/exit/status 使用 typed units，明确 unknown/omitted，而非可选字符串。 |
| RT-EXPORT-P2-003 | Open | 生成路径统一 separator、case-fold、Unicode、reserved name、max length 和 canonical root policy。 |
| RT-EXPORT-P2-004 | Open | tool/environment receipt 对 secret、绝对路径和 machine-local 值做稳定 redaction 与 hash。 |
| RT-EXPORT-P2-005 | Open | stage event、artifact receipt 和 launch event 共享 trace/request/generation/attempt identity。 |
| RT-EXPORT-P2-006 | Open | 输出清单支持 schema evolution、dual-read/single-write、forward incompatibility 和 rollback window。 |
| RT-EXPORT-P2-007 | Open | 产物签名、SRI、SBOM、license、provenance 元数据以可验证 envelope 发布，不只生成 README。 |
| RT-EXPORT-P2-008 | Open | 结构化日志支持采样、保留、隐私、远端上传和离线重放；禁止 shell command 拼接泄漏。 |
| RT-EXPORT-P2-009 | Open | 平台版本、ABI、capability 与 runtime feature catalog 采用可查询矩阵和 explain API。 |
| RT-EXPORT-P2-010 | Open | 失败/取消/恢复状态机做 model-based tests，覆盖每个不可逆 publish 边界。 |
| RT-EXPORT-P2-011 | Open | 生成 host 模板通过 golden/snapshot、编译 lint、ABI symbol inventory 和 schema validation。 |
| RT-EXPORT-P2-012 | Open | 发布代表性项目、设备矩阵、基线和长期 p50/p95/p99/RSS/IO/correctness 结果。 |

## 7. 参考引擎对照与可吸收边界

| 参考 | 事实 | Zircon 应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal AutomationTool | `BuildCookRun.Automation.cs:251-347` 明确 Build/Cook/CopyBuildToStagingDirectory/Package/Archive/Deploy/Run；`ProjectParams.cs` 的 `Validate()` 拒绝不一致的 stage graph、target/config/cook/package 参数；平台实现通过注册的 setup/package/deploy/run owner 承担能力；Gauntlet desktop self-test 覆盖 clean cache、install 和 copy config。 | 以不可变 request + target descriptor + stage DAG + artifact/installation/run receipt 作为唯一成功事实；每阶段可跳过也必须带 reuse proof。 | 不复制其巨型参数面、全局静态状态和继承层级。 |
| Godot export | `EditorExportPlatform` 抽象定义 `has_valid_export_configuration`、project config、`export_project`、pack/zip/patch、messages 和 notifier；Windows exporter 校验模板架构并清理临时导出文件；Android exporter 检查 Gradle/template/SDK/Java/apksigner；Web exporter 先导出到 temp，再启动 HTTP server/browser；PCK tests 验证空包、非法 alignment/key、覆盖和最终尺寸。 | 平台 adapter 必须拥有 capability admission、template/toolchain/arch validation、临时 staging、错误清理、pack/launch 语义和真实 tests。 | 不复制 String/Dictionary 弱类型、同步 Editor global state 或把平台逻辑塞进 UI。 |
| Bevy AssetProcessor | `processor/mod.rs:82-97` 明确 source+steps 决定输出；`log_begin_processing`/`log_end_processing` 用 WAL 识别 unfinished transaction；`ProcessStatus` 有 Processed/Failed/NonExistent，`ProcessorAssetInfo` 保存 dependents、status 和共享 file transaction lock；`wait_until_processed` 提供可等待终态。 | Runtime export stage 需要 action status、dependency snapshot、WAL/recovery、逐资源 transaction lock 和 wait/receipt API。 | Bevy 只解决资产处理，不足以替代平台 compile/package/sign/install/run。 |
| Fyrox build tools | `export/mod.rs` 的 `ExportOptions` 明确 target platform/build target、取消、资源复制、build、binary copy、run；Android/wasm 将 target 传给 `cargo-apk`/`wasm-pack`；`utils.rs` 会清理旧 build dir。 | target 必须真正到达 build command，取消和 clean staging 必须贯穿 child process。 | 轮询 child、字符串错误、先删整个目标目录和简单 copy 只能作为下限，不能作为 Zircon 事务模型。 |
| Unity Graphics | `CoreBuildData.cs` 按 BuildTarget 和 standalone server subtarget 建立 build-scoped snapshot，pre/post callback 管理生命周期；ShaderPreprocessor 支持可发现 stripper scope，ShaderStrippingReport 记录 input/output variants 和 strip time，并有 JSON tests。 | graphics cook/strip、pipeline settings 和 shader report 必须绑定 qualified build context，扩展点有 begin/end 和可归因 receipt。 | 该仓库不是完整 Player exporter，不能从 shader processor 推断完整发布系统。 |

## 8. 目标 Runtime 架构与重构顺序

```text
Editor / CLI / CI / App
          |
          v
RuntimeExportService
  -> EffectiveExportRequest (schema/profile/project/provider snapshot)
  -> TargetCapabilityRegistry
       -> triple/arch/ABI/SDK/toolchain/template/device receipt
  -> ProductBuildDefinition
       -> source/import/cook/package graph (consume Runtime207 closure)
  -> ExportScheduler + WAL
       -> CompileHost / SourceTemplate / NativePlugin / Cook / Pack / Bundle
       -> ArtifactRepository (digest/size/type/producer/mount/variant)
  -> AtomicPublishTransaction
       -> staged generation -> verify -> publish -> quarantine/rollback
  -> ProductLaunchHandoff
       -> mount -> session/surface -> frame/input/resource callbacks
       -> first-frame/health/shutdown receipt
```

建议按以下依赖顺序重构：

1. 先定义 `EffectiveExportRequest`、`QualifiedTargetContext`、platform capability receipt 和 schema/migration；没有它们，任何 Cargo 或 host 模板修补都无法证明目标正确。
2. 将 Interface DTO、Runtime BuildPlan、materialize 和 Editor/CLI/CI 统一到 `RuntimeExportService`，建立 stage DAG、typed diagnostics、WAL、attempt/generation fence 和 artifact receipt。
3. 将 SourceTemplate/LibraryEmbed 的所有输入冻结为 source snapshot/dependency manifest；取消仓库相对 path dependency，改为可重放 source/dependency closure。
4. 以 staging namespace + atomic publish + ownership manifest 重写生成文件、ZIP/native package 和 bundle；missing input、TOCTOU、取消、崩溃和 stale output 必须 fail-close。
5. 把 `ProductComposition` 拆为 Composition/Session/Surface/ResourceMount/FrameLoop owner，先完成 desktop/headless first-frame，再接 Android/iOS/Web 的实际 ABI、memory adapter 和 callback queue。
6. 最后接签名、安装、设备运行、首帧、资源加载、关闭和跨机 deterministic/benchmark 门；没有这些动态门，文档只能保持 pending。

## 9. 资格门与后续实现入口

| Gate | 当前 | 通过条件 |
|---|---|---|
| G1 Target qualification | Fail | 每个 profile 解析到 target/triple/arch/ABI/SDK/toolchain/template receipt，unsupported 明确拒绝。 |
| G2 Build identity | Fail | request/profile/source/provider/toolchain/environment/target 形成 cryptographic immutable key。 |
| G3 Stage truth | Fail | Runtime service 持有唯一 DAG、WAL、typed terminal receipt；Editor/CLI/Python 不再拥有成功权威。 |
| G4 Atomic publish | Fail | generation staging、durable rename、ownership cleanup、rollback/quarantine 和 crash restart 测试通过。 |
| G5 Content closure | Fail | cook/pack/bundle 消费 canonical closure，artifact mount manifest 可验证且入口资源可加载。 |
| G6 Desktop/headless launch | Fail | 真实 process loop、surface/server loop、first frame/health/shutdown/exit code smoke 通过。 |
| G7 Mobile/browser launch | Fail | Android/iOS/Web 真实 ABI/memory/input/lifecycle/resource callback、package/install/run smoke 通过。 |
| G8 Reproducibility | Fail | 同输入同工具链跨两台机器得到相同 stage/artifact/package digest，或所有差异有显式 policy。 |
| G9 Failure/recovery | Fail | compile/materialize/package/sign/install/launch 任一点失败均不发布；restart/resume/cancel/fault 注入通过。 |
| G10 Performance | Fail | 固定 corpus、冷暖缓存、p50/p95/p99、CPU/RSS/I/O 和 correctness 与参考引擎同口径，之后才能讨论性能。 |

本报告只完成 review 和重构计划，未修改 Rust/Cargo/ABI/测试/平台实现；后续实现应从 `RT-EXPORT-P0-002` 和 `RT-EXPORT-P0-003` 的请求/能力/服务边界开始，不应继续在 scaffold 字符串上堆叠临时功能。
