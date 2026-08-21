---
related_code:
  - tools/zircon_export/__main__.py
  - tools/zircon_export/cli.py
  - tools/zircon_export/cli_arguments.py
  - tools/zircon_export/preset_contract.py
  - tools/zircon_export/pipeline_stages.py
  - tools/zircon_export/stage_handoff.py
  - tools/zircon_export/compile_host.py
  - tools/zircon_export/cook_assets.py
  - tools/zircon_export/cook_assets_manifest.py
  - tools/zircon_export/cook_assets_project_fallback.py
  - tools/zircon_export/pack_stage.py
  - tools/zircon_export/platform_bundle.py
  - tools/zircon_export/export_template.py
  - tools/zircon_export/source_template.py
  - tools/zircon_export/source_template_generated_project.py
  - tools/zircon_export/source_template_plan_command.py
  - tools/zircon_export/pipeline_report.py
  - tools/zircon_export/pipeline_report_compile_host.py
  - tools/zircon_export/pipeline_report_compile_host_stage_schema.py
  - tools/zircon_export/export-templates/windows-x86_64-library_embed-debug/template.toml
  - tools/zircon_build.py
  - zircon_runtime/src/asset/pack/writer.rs
  - zircon_runtime/src/asset/pack/reader.rs
  - zircon_runtime/src/asset/pack/delta.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/plugin/export_build_plan/library_embed_compile_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs
  - zircon_editor/src/core/export/stages/compile_host.rs
  - zircon_editor/src/core/export/stages/platform_bundle.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/options.rs
tests:
  - tools/zircon_export/tests
  - zircon_runtime/src/asset/tests/pack.rs
  - zircon_editor/src/core/export/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/02-cargo-zircon-plugin-scaffold-manifest-validation-native-probe-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/BuildCookRun.Automation.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/ProjectParams.cs
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/CookOnTheFlyServer.cpp
  - dev/UnrealEngine/Engine/Source/Developer/IoStoreUtilities
  - dev/UnrealEngine/Engine/Source/Runtime/PakFile
  - dev/UnrealEngine/Engine/Source/Developer/TargetPlatform
  - dev/godot/editor/export/editor_export_platform.cpp
  - dev/godot/editor/export/editor_export_preset.cpp
  - dev/godot/platform/windows/export/export_plugin.cpp
  - dev/Fyrox/fyrox-build-tools/src/export/mod.rs
  - dev/Fyrox/fyrox-build-tools/src/export/android.rs
  - dev/Fyrox/fyrox-build-tools/src/export/wasm.rs
  - dev/bevy/crates/bevy_asset/src/processor/mod.rs
  - dev/bevy/crates/bevy_asset/src/processor/process.rs
  - dev/Graphics/.yamato/wrench/wrench_config.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 03 · Export Preset、Build/Cook/Pack、Platform Bundle 与 Release 工程化差距

## 1. 结论

当前导出目录不是空壳。Python 侧已经有 Validate、SourceTemplate、NativeDynamic、CompileHost、CookAssets、Pack、PlatformBundle、Report 八阶段，Rust 侧有版本化 Validate report、确定性 zrpack、内容哈希、全资产去重、依赖闭包和 delta apply verification，Editor 侧还有进程树取消、bounded output、preset/artifact fingerprint 与阶段进度。这些底座应保留。

但是当前链路不能被称为可发布游戏产品流水线，原因不是平台数量少，而是同名阶段没有形成一致的产品语义。`.zpreset` 中 target mode、entry scenes、keep/exclude、plugin subset、cook compression/binary assets 和 customized files 在解析后没有 consumer；CLI 的 CompileHost 仍执行 Validate 计划中的直接 Cargo 命令，而生产 Report schema 明确只接受 `tools/zircon_build.py`，所以 CLI 编译成功后最终报告必然 fatal。Editor 又走第三条语义：client export 构建 `hub,editor,runtime` 整套引擎分发，并把 `zircon_hub.exe` 当 launcher，而不是构建 Validate 已解析的项目 runtime/plugin linkage。

产品真实性还有更直接的阻断。生成的 desktop `main()` bootstrap 后立即返回并 drop runtime owner；移动端和浏览器 lifecycle/input/viewport/resource C ABI 大多只返回 `true`。仓内三个 export template 均把 host 声明为 `placeholder`，而 PlatformBundle 把该值列为合法状态。真实动态复现表明，一个内容仅为 `not-a-zircon-pack` 的文件加 Windows 文本占位 host 会得到 `exit_code=0`、`fatal=false`、空 diagnostics，并被复制为成功 bundle。

CookAssets 当前只是规范化 JSON manifest，或用正则从 UTF-8 文件寻找 `res://` 字符串；Pack 随后把原始 source bytes 整文件读入内存，一资产一 chunk 原样拼接。它没有调用 importer、shader/material compiler、平台纹理/音频/mesh 转码、DDC 或平台 cook writer，也没有兑现 preset 中的 zstd/lz4。把这一产物称为 cooked/shipping 会让后续工程在错误抽象上继续堆叠。

测试规模也不能证明成熟度。完整 `tools/zircon_export/tests` 实际运行 1,642 项、耗时 373.192 秒，结果为 667 failures。大量 NativeDynamic 负向测试因共享 Validate fixture 缺少当前 `schema_version=2` 而先把 Validate 判 fatal，SourceTemplate 也有诊断合同漂移。当前没有可作为 pre-merge 基线的 export suite。

本轮记录 8 个 P0、46 个 P1 和 8 个 P2。没有修改 Python、Rust、Editor、template、preset、pack 格式或 CI；只新增本审查记录和索引。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| `tools/zircon_export` production | 246 Python 文件 / 42,172 行 | E3：八阶段入口、preset、handoff、template、native/sign、全部 report schema owner 按域逐文件读取 |
| Python export tests | 201 文件 / 71,091 行 / 1,568 个源码 `test_` method | E2-E3：测试结构逐域盘点，并完整运行发现 1,642 个展开后 test case |
| Rust build plan | 40 文件 / 5,930 行 | E3：profile、strategy、generated project、desktop/mobile/browser host 与 compile plan |
| zrpack/export bins | pack/reader/delta 与两个 bin owner | E3：格式、内存模型、写入、delta、报告和校验链 |
| Editor export | core stage、wizard plan/execution/session options | E3：产品入口、默认输入、进程执行、报告生成与 bundle layout |
| shipped templates | Windows x86_64、Linux x86_64、macOS aarch64 共 3 包 | E3：manifest、hash、host payload、resolution 与 materialization |

本轮 clean scoped set 共 338 个 tracked 文件，`git status --short -- <scoped files>` 为 0 项；内容指纹为 `dcc07a3876f8a52d9a5ea0d60a8b58f68df75afece04637eeb1a04d0c7c3fbb1`。实施前必须重取指纹，尤其要重读 Editor export 与 Python report owner，避免覆盖并行变化。

### 2.2 动态验证

完整 Python suite：

```powershell
python -m unittest discover -s tools/zircon_export/tests -p 'test_*.py'
```

结果：

```text
Ran 1642 tests in 373.192s
FAILED (failures=667)
```

代表性失败不是随机环境故障。`_write_validate_report_with_native_dynamic_exports` 仍不写 `schema_version`，于是几十个本应断言 `NativeDynamic` fatal 的测试实际只得到 `fatal_stages=['Validate']`；`test_source_template_rejects_plan_with_blank_command_entry` 也因实际诊断文本变化而失败。suite 运行期间没有 Cargo/rustc 子进程，分钟级耗时来自 Python 合同测试自身。

用生产 CompileHost 输出形状调用生产 stage schema：

```text
diagnostic_count=4
compile_host report unknown field link_plan
compile_host report unknown field validate_report
compile_host report staged_engine_root must be a string
compile_host report command must run tools/zircon_build.py through Python
```

这四项直接来自 `compile_host.py` 与 `pipeline_report_compile_host_stage_schema.py` 的当前生产合同，不依赖测试 fixture。

PlatformBundle 最小复现使用仓内 Windows template，并把 17 字节 `not-a-zircon-pack` 作为 `--pack-file`：

```text
exit_code=0
fatal=False
host_source_origin=template
host_artifact=placeholder
diagnostics=[]
```

输出 bundle 同时包含原样无效 pack 和 `zircon_runtime.host-placeholder` 文本 host。PlatformBundle 只证明路径、template hash 和复制结果，不证明输入是 zrpack，也不证明 host 是目标平台可执行文件。

### 2.3 参考责任边界

- Unreal `BuildCookRun` 明确串联 Build、Cook、Stage、Package、Archive、Deploy、Run；Cook owner 同时接入 Asset Registry、package dependency、DDC、shader compilation、target platform 与 cooked package writer。Zircon 不应复制其体量，但必须建立同等清晰的责任与产物证明。
- Godot Windows export 在选择 debug/release template 后检查 executable architecture，生成/嵌入 PCK，执行 code sign，并通过临时文件 rename 提交嵌入/签名产物。它证明轻量引擎同样需要平台 artifact preflight 与提交语义。
- Bevy AssetProcessor 把 processor/settings/meta、loader dependencies 和 full hash 纳入处理结果；只作为资产处理层参考，不据此推断 Bevy 拥有完整商业发布器。
- Fyrox export 显式区分 PC/Android/WASM，Android 传 `--target` 并用 `cargo-apk`，WASM 用 `wasm-pack`，还提供 convert assets 和 run-after-build。其实现仍较轻，但已证明 target triple 与可运行验证不能只停留在 profile 标签。
- Unity Graphics checkout 只用于 package CI、测试与发布治理参考，不包含 Unity Player 闭源构建实现；本报告不从该目录推断 Player pipeline 能力。

## 3. 当前 P0

### TOOL-EXPORT-P0-001 · Export preset 大部分字段只解析不执行

`cli_arguments.py:345-353` 把 preset 的 target mode、debug、include/exclude、entry scenes、keep list、plugin subset、cook 和 customized files挂到 `args`。全 production Python 搜索显示这些 `preset_*` 名称各只出现一次，即赋值位置；`binary_assets` 和 `compression` 只在 `preset_contract.py` 被类型校验。Validate 也只读取 project manifest/profile，不接收 preset。用户在 Editor/CLI 选择的场景、裁剪、插件、压缩和定制文件不会改变产物，这是静默错误而非未提供高级选项。

必须建立一个版本化 `ResolvedExportRequest`，把 project/profile/preset/CLI override 归一化为唯一 IR；每个字段必须有 owner stage、默认来源、冲突规则和 receipt projection。未消费字段应在 Validate 阶段 fatal，禁止“解析即支持”。

### TOOL-EXPORT-P0-002 · CLI CompileHost 与最终 Report 是互斥的生产协议

`compile_host.py` 消费 Validate 的 direct Cargo command，并输出 `validate_report`、`link_plan`，不输出 `staged_engine_root`。`pipeline_report_compile_host_stage_schema.py` 反而把前两项视为未知字段，要求 `staged_engine_root`，且命令必须通过 Python 运行 `tools/zircon_build.py`；`pipeline_report_compile_host.py` 还把 `-p/--package/--bin/--features/--target-dir/--release` 定义为 removed Cargo options。

这不是 legacy 文件未删除，因为 `cli.py` 主流水线仍真实 dispatch `run_compile_host(args)`。必须先选择唯一 CompileHost 产品合同，再删除另一条 authority；修复门必须运行 production CLI 从 Validate 到 Report，不能继续用彼此独立的 stage fixture 证明正确。

### TOOL-EXPORT-P0-003 · Editor 默认 CookAssets 输入指向不会被任何阶段生成的文件

`export_wizard_options` 把 `source_asset_manifest` 固定为新 output root 下的 `assets/assets.json`。全 `tools/zircon_build*.py` 没有 `assets.json`、`source_asset_manifest` 或 `asset_manifest` writer，wizard 在 CookAssets 前也没有生成该文件的 stage。默认 export 因而依赖用户预先在一个新输出目录手工放置隐藏输入；否则 CookAssets 必然报告文件不存在。

应由 Asset Registry/Cook Planner 直接发布版本化 source asset graph artifact，wizard 只引用上游 receipt。临时方案也必须把输入指向 project-owned manifest 并在 Validate 显示阻断原因，不能指向不存在的输出。

### TOOL-EXPORT-P0-004 · Editor client export 构建并发布 Hub/Editor 分发体，不是项目游戏 host

`CompileHostStage::command` 对 client 固定 `--targets hub,editor,runtime` 和 `target-client`；`PlatformBundleLayout` 再把 `ZirconEngine/zircon_hub(.exe)` 选为 launcher，并要求 editor、Hub、runtime DLL 同时存在。命令没有使用 Validate 的 linked runtime crates、selected plugin features 或项目生成 host。结果既可能漏掉项目插件，也把开发工具、Tauri/web assets 和 Editor 攻击面带进“游戏导出”。

必须拆分 `EngineDistributionBuild` 与 `ProductHostBuild`。导出只允许后者消费 ResolvedExportRequest/Build Set，产出项目专属 client/server executable 或受控通用 player 加项目 receipt；Hub/Editor 不得作为 client shipping launcher。

### TOOL-EXPORT-P0-005 · 生成 host 启动后立即退出，平台回调是假成功

`main_template.rs` 中两种 `main()` 都只创建局部 `_core/_bootstrap` 后返回；没有 event loop、window/surface、frame pump、server loop 或 shutdown ownership。`platform_host_files.rs` 生成的 `zircon_export_start` 同样在局部 bootstrap drop 后返回 bool；lifecycle、touch、keyboard、viewport 和 resource fetch callback 忽略参数并返回 `true`。Android/iOS/Web shell 虽会调用这些符号，调用不会驱动 runtime。

必须先定义跨平台 `ExportRuntimeInstance` opaque handle 与 start/tick/event/resize/suspend/resume/stop 生命周期，host 持有到进程/scene 终止。所有 callback 需要 typed error、线程约束和真实桥接测试；无实现平台必须 Validate fatal，禁止 true stub。

### TOOL-EXPORT-P0-006 · 占位 host 与任意现存文件可被发布为成功 bundle

三个 shipped template 的 host payload 都是文本 placeholder；Windows manifest明确写 `host_artifact="placeholder"`，而 `EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS` 把 `placeholder` 与 `precompiled` 并列为合法值。PlatformBundle 对 pack 只检查路径存在并复制，不调用 `ZrPackReader`。本轮真实复现已用无效 pack 得到 exit 0/fatal false。

发布模式必须只接受经过 Build Receipt 认证的 executable/library 和经过格式 reader 完整校验的 pack。placeholder 只能存在于显式 test fixture namespace，并且 production template resolver 必须拒绝。复制后还要验证目标文件 hash、format、architecture、依赖与可启动性。

### TOOL-EXPORT-P0-007 · Cook/Pack 名称掩盖了“原始文件清单加拼接容器”

Cook fallback 以正则扫描 UTF-8 文本中的 `res://`，binary source直接跳过；manifest 归一化后把 source 变成绝对路径。Pack bin 对每个 included asset调用 `std::fs::read`，再由 writer 把整资产 bytes 作为单一 chunk原样 append。没有 importer artifact、平台派生格式、shader/material compile、texture/audio/mesh transcode、scene serialization、platform strip 或压缩。

该链可以保留为 `RawAssetBundlePrototype`，但不能作为 shipping Cook。重构必须把 import artifact、cook transform、platform variant、dependency graph、chunk assignment 与 container write 分离，并让 runtime 只消费 cooked schema。完成前 release profile必须明确 fatal。

### TOOL-EXPORT-P0-008 · Export 测试基线已有 667 个失败

完整 suite 667/1,642 failure，失败集中暴露共享 fixture/schema authority漂移，而不是单个平台环境缺失。当前 CI 即使接入该 suite也只能长期红灯或被忽略；继续增加 shape test会扩大维护债务。

先建立测试恢复 freeze：按 stage 收敛 canonical fixture builder，修复当前 baseline 到 0 failure，再加入 production CLI E2E。任何协议迁移必须同提交更新 schema、generator、fixture与consumer conformance；未恢复前不得宣称 export validation complete。

## 4. Pipeline、Authority 与 Receipt 差距

### TOOL-EXPORT-P1-001 · 没有贯穿全链的 Build Set ID

stage report主要用 stage/profile/path相认，没有把project、preset、source revision、toolchain、dependency graph与输入artifact hash收敛成同一Build Set。应以内容寻址 request ID贯穿所有report和文件名。

### TOOL-EXPORT-P1-002 · Stage report 缺少 attempt、producer 与输入 receipt 身份

除Validate有`schema_version=2`外，其余stage没有统一schema version、attempt ID、producer version、input receipt IDs和created-at monotonic ordering。旧文件可以在同profile下冒充当前运行结果。

### TOOL-EXPORT-P1-003 · Resume 在 Validate report 不可用时退回固定宽阶段链

`pipeline_stages_from_resume` 遇到无效Validate report时使用 `FALLBACK_RESUME_STAGES`，没有证明请求实际选择了SourceTemplate/NativeDynamic，也没有证明上游artifact新鲜。恢复必须从持久DAG checkpoint和receipt dependency重建，而不是阶段名切片。

### TOOL-EXPORT-P1-004 · 没有 export writer lease 或并发仲裁

两个Editor/CLI进程可以同时删除、覆写同一stage/output/bundle。应有 output-root lease、owner PID/session、heartbeat、stale lease recovery 与显式 read-only inspection mode。

### TOOL-EXPORT-P1-005 · 阶段与全流水线都不是事务提交

SourceTemplate先 `rmtree` 再逐文件写，Pack用 `fs::write` 覆盖，PlatformBundle清理live目录后复制。应写入同卷staging，完成内容/hash/launch gate后原子promote，并保留上一已知良好receipt供回滚。

### TOOL-EXPORT-P1-006 · Python 子进程没有 timeout、process tree cancel 或资源预算

八处 `subprocess.run/call` 没有timeout；CLI中断也没有跨平台job/process-group owner。Editor新wizard已有可取消runner，可作为底座；所有外部命令必须统一进程树、deadline、stdout/stderr预算与typed termination。

### TOOL-EXPORT-P1-007 · 104 个 report 模块仍是手写 schema authority

`pipeline_report*.py` 已达104文件/21,979行，却只在Validate共享schema version，CompileHost漂移已证明模块数量没有换来一致性。应从版本化stage schema/IR生成Python/Rust codecs、validator和fixture builder。

### TOOL-EXPORT-P1-008 · Report 缺少工具链、时间与资源证据

最终receipt不记录rustc/cargo/python版本、target triple、SDK、Cargo.lock hash、wall time、CPU/RSS、cache hit、source revision或环境白名单。无法比较性能、复现构建或定位供应链差异。

### TOOL-EXPORT-P1-009 · 成功、可打包与可发布三种状态混在 fatal/diagnostics

SourceTemplate未请求build时stage可exit 0并带“skipped”diagnostic，最终Report又把它视为不可发布。应定义 `planned/materialized/verified/publishable/promoted` 状态机，diagnostic严重度不能代替产品状态。

### TOOL-EXPORT-P1-010 · 没有 Publish/Install/Run/Promote/Rollback 终端阶段

PlatformBundle后直接Report；不存在安装目标、启动smoke、首帧/ready handshake、崩溃捕获、渠道上传、promotion policy和rollback。至少要把“生成目录”和“可发布artifact”拆成不同命令与receipt。

## 5. Build 与 Target Toolchain 差距

### TOOL-EXPORT-P1-011 · target platform 只是标签，不进入 Cargo target

`library_embed_compile_host_plan` 没有 `--target`，SourceTemplate contract还明确禁止 `--target`。Editor的`zircon_build.py --target`只是`--targets`别名，不是Rust target triple。非本机profile不会产生对应平台二进制。

### TOOL-EXPORT-P1-012 · executable 名与后缀由构建主机推断

CompileHost根据当前Python/Rust host选择`.exe`等路径，而不是从resolved target descriptor读取。交叉构建或远程构建receipt无法可靠定位artifact。

### TOOL-EXPORT-P1-013 · 缺少版本化 TargetPlatform descriptor

没有统一owner描述triple、ABI、CPU baseline、SDK/NDK/Xcode、linker、runner、binary format、resource rules、signing与template compatibility。应由platform plugin提供typed descriptor和preflight。

### TOOL-EXPORT-P1-014 · 只有 Cargo debug/release，没有 shipping/profile policy

没有统一LTO、codegen-units、panic、strip、debug symbols split、incremental、sanitizer/PGO和reproducibility策略。profile名称也没有映射到经审计的Cargo profile/target feature set。

### TOOL-EXPORT-P1-015 · Build没有证明 Validate 的插件链接计划

CLI虽报告`link_plan`，最终schema拒绝它；Editor report则完全不带link plan，Report只在双方都存在时才比较。应由linker input receipt列出crate/features/native packages并与ResolvedExportRequest逐项相等。

### TOOL-EXPORT-P1-016 · 平台SDK与工具发现没有可复用诊断

Android、iOS、WASM、签名/公证只在生成README或外部command中出现，没有preflight SDK version、license、credentials capability、emulator/device和remote builder。缺失必须在Build前一次性列全。

### TOOL-EXPORT-P1-017 · SourceTemplate只允许debug/release且绑定本地workspace

生成Cargo.toml随后用正则把`zircon_*` path dependency重写成当前repo绝对路径。它不能移动到另一机器/CI，也没有SDK version/source archive/vendor mode。需要可解析的engine SDK/registry/git receipt，不得依赖原checkout绝对路径。

## 6. Cook 与资产图差距

### TOOL-EXPORT-P1-018 · fallback依赖图来自文本正则而非序列化/registry authority

`RES_ASSET_REFERENCE_RE` 扫描字符串，无法识别binary、import metadata、subresource、typed soft reference、code-generated reference或alias。应由资产类型serializer/importer报告dependency edge。

### TOOL-EXPORT-P1-019 · 只从 default_scene 起图，entry scene/keep rule 不参与

fallback root固定project `default_scene`；preset entry scenes、keep list、exclude/filter没有进入closure。Cook Planner必须同时处理hard root、soft root、always-cook、never-cook、DLC/optional group与冲突诊断。

### TOOL-EXPORT-P1-020 · 没有 importer/processor 版本化输出

Cook未消费Editor import artifact或runtime importer schema，也不记录processor version/settings/source hash。应定义 `ImportedArtifact -> CookedVariant`，并把processor/settings/dependency hash纳入cache key。

### TOOL-EXPORT-P1-021 · 没有 DDC 与增量失效图

每次都重新读manifest/source，没有本地/共享缓存、negative cache、dependency invalidation、platform key或cache provenance。应先建立correctness-first content-addressed DDC，再做并行和远程缓存。

### TOOL-EXPORT-P1-022 · Shader/material/pipeline 未进入 cook

没有shader permutation discovery、platform compiler、material flatten、PSO/driver cache seed或编译失败聚合。Graphics报告中的runtime fallback不能替代shipping离线编译门。

### TOOL-EXPORT-P1-023 · Texture/mesh/audio/font/scene 没有平台派生格式

source bytes原样进pack；没有GPU texture format/mip/virtual tile、mesh LOD/cluster、audio codec/stream chunk、font atlas/glyph range或scene binary schema。每类资产需要owned cooker和runtime compatibility version。

### TOOL-EXPORT-P1-024 · Cook manifest 泄露并固化绝对源路径

相对source会被resolve为本机绝对路径后写入cooked manifest，破坏可搬运性与隐私。receipt应使用workspace-relative logical source ID和content hash，调试映射另存受控provenance。

### TOOL-EXPORT-P1-025 · 没有规模、循环、预算与并行调度模型

队列用list `pop(0)`，没有显式cycle SCC、fanout budget、worker pool、memory/backpressure、progress或cancel checkpoint。大型项目会在真正转码前先遇到控制面瓶颈。

## 7. Pack、Patch 与 Runtime Delivery 差距

### TOOL-EXPORT-P1-026 · 一整个资产等于一个 chunk

writer按完整asset hash建chunk；任意大资产必须小于`u32`，且微小变化会重发整资产。应引入固定/内容定义block、logical bulk segments与asset-to-many-chunks映射。

### TOOL-EXPORT-P1-027 · Writer/reader/delta 全量驻留内存

输入asset是`Vec<u8>`，输出pack也是`Vec<u8>`；reader拥有整pack，delta apply再重建全部target assets与完整pack。需要streaming file writer、bounded buffer、mmap/async range reader和磁盘staging。

### TOOL-EXPORT-P1-028 · preset compression 完全没有进入容器

zstd/lz4/none只在preset parser验证，chunk metadata没有codec/uncompressed size/dictionary。应按asset class/chunk policy压缩，记录codec/version/dictionary ID并有解压炸弹预算。

### TOOL-EXPORT-P1-029 · 容器没有加密、签名或可信根

BLAKE3内容哈希可发现损坏但不能证明发布者，也不保护内容。需要manifest签名、key ID/rotation、可选chunk encryption、authenticated metadata和offline verification tool。

### TOOL-EXPORT-P1-030 · 没有页/扇区/IO 对齐与 streaming tier

payload紧密拼接，没有alignment、install chunk、startup/level/optional tier、bulk data、read amplification预算或平台文件系统策略。应让Cook Planner产出chunk assignment，再由container writer执行layout。

### TOOL-EXPORT-P1-031 · Delta 是整资产集合差，不是块级patch

delta只复用已有完整chunk hash；资产任一字节变化会复制全文件。需要block/chunk级复用、base Build Set约束、patch chain上限、空间预检和回滚策略。

### TOOL-EXPORT-P1-032 · determinism check 默认关闭且preset deterministic未消费

CLI/Editor默认不二次写，preset deterministic也不控制它。shipping应默认强制deterministic build或至少做跨worker/sample gate，并记录非确定来源diff。

### TOOL-EXPORT-P1-033 · Pack/Delta/Report 直接覆盖，没有 durable commit

`fs::write`不提供temp+flush+fsync+rename事务；进程崩溃可能留下截断文件或新旧receipt混合。需要同卷临时文件、directory sync和commit journal。

### TOOL-EXPORT-P1-034 · Runtime mount 与 patch activation 没有同一 receipt 协议

export writer、runtime reader、installer/promotion尚未共享Build Set、mount order、base compatibility与activation marker。格式reader通过不等于安装可安全切换。

## 8. Platform Bundle、Source Product 与发布差距

### TOOL-EXPORT-P1-035 · shipped template 覆盖仅三个desktop占位组合

只有Windows x86_64、Linux x86_64、macOS aarch64的library-embed debug命名模板；没有release预编译host、macOS x86_64/universal、Windows ARM64、Android、iOS或Web product template。unsupported组合应显式不可发布。

### TOOL-EXPORT-P1-036 · 不验证 executable format、architecture 与动态依赖

template hash正确并不证明PE/ELF/Mach-O/WASM合法。应使用object parser检查format/machine/subsystem/min OS/imports/rpath，并对DLL/so/dylib closure做allowlist与缺失诊断。

### TOOL-EXPORT-P1-037 · 没有平台签名、公证、entitlement 与身份合同

NativeDynamic允许调用任意外部signer，但最终product host没有Windows Authenticode、macOS codesign/notarization、iOS provisioning/entitlement或Android signing verification receipt。平台owner必须提供typed step和post-verify。

### TOOL-EXPORT-P1-038 · 没有 installer/package metadata 与升级语义

bundle只有目录和JSON；没有app ID、semantic/build version、icon/resource metadata、installer、uninstall、file association、runtime prerequisite、upgrade/downgrade和side-by-side策略。

### TOOL-EXPORT-P1-039 · 不验证Unix executable bit、sandbox与启动结果

复制不恢复mode/ACL/quarantine metadata，也不在目标环境运行。最低门应启动产物、等待runtime ready/首帧或server ready、执行健康命令后受控退出。

### TOOL-EXPORT-P1-040 · 缺少 symbols、source map、crash、SBOM、license 与 provenance

发布目录没有分离符号、WASM source map、crash symbol upload receipt、SPDX/CycloneDX、第三方notice、SLSA式provenance或可复现验证。Release promotion必须消费这些artifact。

### TOOL-EXPORT-P1-041 · Android/iOS生成工程仍要求人工构建和复制Rust库

README要求开发者先为ABI/architecture构建 `.so/.a` 并放到指定目录，package script只运行Gradle/Swift/Xcode。Build graph没有拥有Rust cross compile、lipo/XCFramework、ABI矩阵或artifact hash交接。

### TOOL-EXPORT-P1-042 · Android Play脚本只创建edit，没有上传bundle或commit

脚本POST edits endpoint后就打印release bundle ready；没有读取edit ID、上传AAB、分配track、commit、poll processing或rollback。不能把本地AAB存在投影为Play publish完成。

### TOOL-EXPORT-P1-043 · Web scaffold依赖未锁定，WASM与资源仍靠人工补齐

生成`vite`和plugin版本均为`latest`，没有lockfile；package script只复制`zircon-project.toml`，README要求人工编译/复制WASM。resource import还明确打印缺memory adapter。需要锁定toolchain、自动wasm-bindgen/component步骤、完整asset staging、integrity manifest和浏览器E2E。

## 9. Editor 与测试架构差距

### TOOL-EXPORT-P1-044 · Editor默认关闭SourceTemplate build与determinism gate

`ExportWizardPipelineOptions::default` 的两项均为false，UI没有根据shipping policy强制开启。默认值应来自ResolvedExportRequest policy，release不能静默降级为只生成或单次写。

### TOOL-EXPORT-P1-045 · Editor没有暴露/计划 NativeDynamic build、sign、notarize

Python CLI有相关参数，wizard options/plan没有等价能力，导致同preset从CLI与Editor得到不同payload。平台安全设置应是版本化preset/credential reference，不应只存在于命令行。

### TOOL-EXPORT-P1-046 · 测试是巨型 shape suite，缺少分层、超时与真实 E2E

201文件运行6分13秒、默认无进度，667失败又主要来自fixture drift。应分为秒级schema conformance、stage integration、target toolchain、fixture template、nightly scale/fault和少量production CLI golden path；每层有timeout、shard与owner。

## 10. P2 与长期完善项

### TOOL-EXPORT-P2-001 · CLI表面仍把stage做成参数而非稳定子命令

帮助文本容易让调用方误以为存在 `platform-bundle` 子命令；应提供稳定 `zircon export validate/build/cook/package/publish/inspect` 命令模型与兼容迁移。

### TOOL-EXPORT-P2-002 · Stage key、显示名与文件路径大小写规则分散

`compile_host`、`CompileHost`与`ExportStage`映射分布在多语言实现。应从schema生成canonical ID、display name与path segment。

### TOOL-EXPORT-P2-003 · Template manifest默认bundle路径不一致

Windows依赖空值默认，Linux/macOS显式声明更多path；虽然当前resolver可处理，长期会放大平台差异。应由schema生成完整canonical manifest。

### TOOL-EXPORT-P2-004 · 没有输出大小、预计耗时与容量预检

Editor应在执行前显示source/cooked/pack/install体积估算、剩余空间、cache命中和平台工具准备状态，但这些必须来自plan，不得用演示值。

### TOOL-EXPORT-P2-005 · 缺少 dry-run DAG/receipt diff

当前dry-run主要打印命令。应能比较两个ResolvedExportRequest/Build Set，解释会重建哪些资产、插件、平台artifact与原因。

### TOOL-EXPORT-P2-006 · 没有标准 artifact inspect 工具

需要一个只读inspect命令展示pack版本、chunk/codec/signature、bundle target、Build Set、SBOM和可运行门结果，供Editor、CI和支持团队共用。

### TOOL-EXPORT-P2-007 · 发布诊断没有稳定 code 与 remediation URL

大量字符串诊断难以本地化、聚合或自动修复。生成schema应包含code、severity、owner、artifact、help与safe fix action。

### TOOL-EXPORT-P2-008 · 缺少长期 cook/pack/package 性能基线

应覆盖小项目冷/热构建、大项目增量、1GB单资产、百万小文件、patch apply、网络/远程cache与启动read amplification，并存历史趋势和回归预算。

## 11. 重构路线

### M0 · 恢复真实性与测试基线

1. 禁止production resolver选择placeholder template，PlatformBundle完整解析zrpack并验证host object format。
2. 选择唯一CompileHost合同，让CLI和Editor都消费它；加一条Validate→Report production E2E。
3. 修复667项失败并把完整suite拆出秒级blocking shard；未恢复前freeze新增shape-only test。
4. Editor默认输入改为真实project asset graph，缺失时在Validate阻断。

### M1 · 建立 ResolvedExportRequest 与 Build Set

1. 合并project/profile/preset/CLI override，拒绝未消费字段。
2. 版本化stage schema，生成Python/Rust codec、validator与fixture。
3. 所有stage receipt携带Build Set、attempt、producer、inputs/outputs hash和toolchain identity。
4. 加output lease、checkpoint DAG、staging/promotion/rollback。

### M2 · 建立真实 Product Host

1. 分离engine distribution与product build，client/server不再打包Hub/Editor。
2. 定义持久runtime instance handle、event/frame/server loop和shutdown owner。
3. 让Validate plugin linkage进入linker input receipt并做逐项相等验证。
4. 建立desktop ready/first-frame与server ready/health smoke。

### M3 · TargetPlatform 与交叉构建

1. 平台plugin提供triple、SDK、binary、resource、sign、runner和template descriptor。
2. Windows/Linux/macOS先形成release host与object/dependency/sign验证。
3. Android/iOS/Web把Rust target build纳入同DAG，删除人工copy步骤。
4. 增加remote builder/emulator/device能力，但不让远程控制面替代本地receipt。

### M4 · Import/Cook/DDC

1. 资产registry发布typed dependency graph和import artifacts。
2. 每类资产拥有版本化Cooker与platform variant schema。
3. 引入content-addressed DDC、依赖失效与remote cache provenance。
4. 将shader/material/PSO和startup scene closure纳入release gate。

### M5 · Container、Streaming 与 Patch

1. zrpack vNext支持many-chunk asset、codec、alignment、tier、signature/encryption metadata。
2. writer/reader改为bounded streaming IO，运行时接入range/mmap异步读取。
3. patch改为block级复用，定义base identity、chain、空间预检、atomic activation和rollback。
4. 保留现有sorted order、BLAKE3、dedup和delta apply verification作为回归基线。

### M6 · Platform Package 与 Release Promotion

1. 生成安装包/app bundle、identity/version/icon/entitlement并完成post-sign verify。
2. 产出symbols/source maps、SBOM/license/provenance和reproducibility report。
3. Install→Run→Health→Stop→Promote形成typed终端链。
4. 渠道上传必须有upload/commit/poll/rollback receipt，禁止打印成功代替远端确认。

### M7 · 规模与运维

1. 建立cook/pack/build/package历史性能基线和budget regression gate。
2. 增加故障注入：磁盘满、进程崩溃、锁争用、cache损坏、签名失败、断网和patch中断。
3. Editor提供DAG、cache、artifact/receipt diff与稳定diagnostic code。
4. 长期保持production CLI golden path和每个平台最小真实应用nightly。

## 12. 验收门

1. 每个preset字段在ResolvedExportRequest中有唯一consumer；随机删除consumer会使schema conformance失败。
2. 同一project/preset从CLI和Editor生成相同Build Set ID与等价DAG。
3. production Validate→CompileHost→Cook→Pack→Bundle→Report E2E为0 fatal，CompileHost report能被最终schema接受。
4. client bundle不含Hub/Editor及其web/Tauri assets；server bundle不含window/input/editor依赖。
5. Product host启动后runtime owner持续存活，client报告首帧，server报告health，受控stop后资源释放。
6. lifecycle/touch/keyboard/viewport/resource callback进入真实runtime test double；stub `true`会被测试拒绝。
7. production template resolver对任何`host_artifact=placeholder` fatal。
8. PlatformBundle对随机字节pack、错误magic、错误hash、截断manifest和目标架构不匹配全部fatal。
9. Windows PE、Linux ELF、macOS Mach-O/WASM分别完成format/machine/dependency preflight。
10. Editor新输出目录无需手工文件即可完成source graph交接；缺资产只产生明确typed diagnostic。
11. entry scenes、keep/exclude、plugin subset、compression和customized files都能通过golden artifact差异证明生效。
12. Cook使用importer/cooker artifact，不从source文本正则推断正式dependency closure。
13. 修改processor/settings/dependency任一输入只失效正确DDC节点；冷/热构建receipt可解释命中来源。
14. texture/mesh/audio/font/scene/shader至少各有一个真实平台变体golden test。
15. 10GB source set在固定内存预算内完成cook/pack，writer/reader不持有完整pack Vec。
16. 单字节修改100MB资产的patch不重发完整资产，且apply后target Build Set完全一致。
17. pack compression字段、codec、uncompressed size和dictionary ID可inspect，zstd/lz4 preset确实改变容器。
18. 签名pack/bundle可离线验证；篡改payload、manifest或key ID均被拒绝。
19. 任一stage崩溃不会破坏上一已知良好bundle；重启可从journal恢复或安全清理staging。
20. 两个进程竞争同一output时只有一个writer，另一个得到owner/lease诊断。
21. 外部命令均有timeout、process-tree cancel、输出/内存预算；取消后不遗留child。
22. 完整Python export suite恢复0 failure，并拆出小于60秒的blocking contract shard。
23. 每个平台至少一个真实最小应用完成Build→Install→Run→Ready→Stop nightly，不能使用placeholder host。
24. Release receipt含source/dependency/toolchain/target hashes、symbols、SBOM、license、signature和promotion结果，可由inspect工具重放验证。

## 13. 保留项

- 保留 Validate 的版本化report方向、profile/runtime plugin availability与fatal diagnostics分离。
- 保留 zrpack 的排序确定性、BLAKE3内容哈希、完整资产去重、依赖/重复项前置门和delta apply verification。
- 保留 template manifest 的safe relative path、declared file SHA-256和content hash校验，但移出placeholder产品语义。
- 保留 Editor wizard 的进程树取消、bounded output、stage progress和core preset/artifact fingerprint；将其下沉为共享executor能力。
- 保留 NativeDynamic package materialization/sign operation audit中可验证的hash/selection基础，继续按Plugin 01/Tooling 02收敛ABI与安全边界。
- 保留参考引擎的责任分层，不照搬其具体格式：Zircon目标应是更严格的typed receipt、可重放Build Set和更低的增量/运行时IO成本。
