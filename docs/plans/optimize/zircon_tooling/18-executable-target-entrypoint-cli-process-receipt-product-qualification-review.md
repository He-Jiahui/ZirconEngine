---
related_code:
  - Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_app/build.rs
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/bin/zircon_host_reflection_docs.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs/args.rs
  - zircon_runtime/src/bin/zircon_export_validate/main.rs
  - zircon_runtime/src/bin/zircon_export_pack/main.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - zircon_runtime/src/bin/zircon_shader_ide_env/main.rs
  - zircon_runtime/src/bin/zircon_font_sdf_bake/main.rs
  - zircon_hub/Cargo.toml
  - zircon_hub/src/main.rs
  - zircon_hub/src/build/command.rs
  - zircon_hub/src/build/runner.rs
  - zircon_hub/src/process/editor_launch.rs
  - tools/cargo-zircon/Cargo.toml
  - tools/cargo-zircon/src/main.rs
  - tools/session_tray/Cargo.toml
  - tools/session_tray/src/main.rs
  - tools/session_tray/src/lib.rs
  - tools/mvp/MvpProductInputManifest.psm1
  - examples/woc/native/Cargo.toml
  - examples/woc/native/apps/woc_bot/Cargo.toml
  - examples/woc/native/apps/woc_bot/src/main.rs
  - examples/woc/native/apps/woc_client/Cargo.toml
  - examples/woc/native/apps/woc_client/src/main.rs
  - examples/woc/native/apps/woc_headless/Cargo.toml
  - examples/woc/native/apps/woc_headless/src/main.rs
  - examples/woc/native/apps/woc_server/Cargo.toml
  - examples/woc/native/apps/woc_server/src/main.rs
  - examples/woc/native/crates/woc_contract_codegen/Cargo.toml
  - examples/woc/native/crates/woc_contract_codegen/src/bin/woc_contract_codegen.rs
  - zircon_plugins/neural/editor/Cargo.toml
  - zircon_plugins/neural/editor/src/bin/zr_onnx_convert.rs
tests:
  - tools/cargo-zircon/tests/manifest_sync.rs
  - tools/cargo-zircon/tests/plugin_commands.rs
  - tools/tests/test_zircon_export_cli_owner_boundaries.py
  - tools/tests/test_zircon_export_validate_schema_test_owner_boundaries.py
  - tools/tests/test_zircon_build_shader_prewarm_command_contract.py
  - tools/tests/test_zircon_build_font_sdf.py
  - examples/woc/native/apps/woc_client/tests/application.rs
  - examples/woc/native/apps/woc_server/tests/fixed_tick_driver.rs
  - examples/woc/native/crates/woc_contract_codegen/tests/contract_generation.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_app/02-pbr-viewer-tool-runtime-evidence-renderdoc-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_plugins/02-neural-model-onnx-inference-post-process-editor-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/02-cargo-zircon-plugin-scaffold-manifest-validation-native-probe-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/04-reflection-derive-script-host-macros-schema-codegen-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
  - docs/plans/optimize/zircon_tooling/16-capability-truth-placeholder-noop-fallback-degraded-qualification-control-plane-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Configuration/Descriptors/TargetDescriptor.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Configuration/Rules/TargetRules.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/System/TargetReceipt.cs
  - dev/bevy/tools/example-showcase/src/main.rs
  - dev/bevy/examples/README.md
  - dev/bevy/Cargo.toml
  - dev/godot/main/main.cpp
  - dev/godot/main/main.h
  - dev/Fyrox/fyrox-build-tools/src/lib.rs
  - dev/Fyrox/fyrox-build-tools/src/build.rs
  - dev/Fyrox/fyrox-build-tools/src/export/mod.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Tooling 18：Executable Target、Entrypoint、CLI、Process Receipt 与产品资格审查

## 1. 结论

本轮逐个审查当前仓库18个真实Rust binary target及其Cargo声明、入口边界、参数与输出、退出行为、构建选择器、启动consumer和直接测试。当前可分为8个产品入口与10个工具/生成器/控制程序；没有证据表明还存在第19个Rust产品入口。问题不是入口数量少，而是target identity没有形成单一控制面：12个target由`[[bin]]`显式声明，6个依赖Cargo目录自动发现；只有3个manifest关闭`autobins`；MVP、Hub、测试和人工命令分别以package/bin/path/artifact-name字符串重新描述其中一小部分目标。

现有入口并非全部是一次性脚本。`zircon_editor`和`zircon_runtime`经`EntryRunner`进入共享host并处理日志清退，PBR viewer具有独立参数、work path、RenderDoc和event loop边界，Hub与session tray将薄`main`委托给library，export/prewarm/reflection等工具多数也把业务逻辑拆到模块。`zircon_app`的3个target和`zircon_runtime`的4个工具target已有`required-features`，MVP还显式绑定runtime/editor executable及对应runtime DLL。这些机制应保留并升级为canonical target catalog，而不是重写所有`main.rs`。

工程级缺口在声明、产物、进程和资格之间断链。当前没有稳定`TargetId`、角色taxonomy、platform/profile/feature closure、artifact receipt、install location、launch protocol或health/qualification contract。一个binary被Cargo成功编译、进程成功创建或以0退出，不能证明它启动了正确代次的产品、进入Ready、执行了预期能力、没有走placeholder/fallback，或产生了可由CI/Hub读取的同代证据。四个WOC产品入口目前只打印identity JSON便退出，这一产品实现缺陷已由App03拥有；本篇只登记“控制面为何会把这种程序当作成功target”的跨入口问题。

CLI同样缺乏统一automation contract。当前同时存在结构化JSON、自由文本、Rust `Result`终止、显式`exit(1)`、参数错误`exit(2)`，以及`cargo-zircon`的2/3/4业务退出码；stdout/stderr用途、schema version、稳定error code、取消、signal、超时、输出artifact和向后兼容没有共同约束。报告不要求GUI产品伪装为CLI，也不要求所有工具采用同一个parser crate；要求每个可自动化入口声明自己采用的protocol并由consumer按receipt验证。

本篇登记 **2项P0、50项P1、12项P2**。它只拥有ExecutableTargetManifest、TargetArtifactReceipt、LaunchContract、ProcessInstanceReceipt和TargetQualificationReceipt；入口内部的runtime/editor/viewer行为由App01/02拥有，WOC角色由App03-05拥有，Hub进程与持久化由Hub01拥有，ONNX转换由Plugins02拥有，具体工具语义由Tooling02-07拥有，测试选择与结果聚合由Tooling10拥有，MVP产品资格由Tooling15/16拥有。本轮没有修改任何manifest、入口、生产代码或测试。

## 2. 物理清单与调用面

### 2.1 8个产品入口

| TargetId候选 | Cargo package / bin | 声明方式 | 当前入口与角色 | 当前主要consumer |
|---|---|---|---|---|
| `product.woc.bot` | `woc_bot` / `woc_bot` | auto bin | 打印bot identity JSON后退出 | WOC脚本/人工运行 |
| `product.woc.client` | `woc_client` / `woc_client` | auto bin | 打印client identity JSON后退出 | WOC脚本/测试 |
| `product.woc.headless` | `woc_headless` / `woc_headless` | auto bin | 打印headless identity JSON后退出 | WOC脚本/人工运行 |
| `product.woc.server` | `woc_server` / `woc_server` | auto bin | 打印server identity JSON后退出 | WOC脚本/测试 |
| `product.editor` | `zircon_app` / `zircon_editor` | explicit，`target-editor-host` | `EntryRunner`、panic log flush、teardown diagnostic | MVP、Hub、人工运行 |
| `product.runtime.client` | `zircon_app` / `zircon_runtime` | explicit，`target-client` | `EntryRunner`、failure teardown、process exit | MVP、人工运行 |
| `product.viewer.pbr` | `zircon_app` / `zircon_shader_pbr_viewer` | explicit，`target-client` | GPU viewer/event loop/capture入口 | graphics开发与证据脚本 |
| `product.hub` | `zircon_hub` / `zircon_hub` | explicit，`autobins=false` | Tauri desktop launcher | 用户桌面入口 |

WOC四入口的内部语义不在本篇重新计数。控制面事实是它们与editor/runtime/Hub同样是Cargo可执行target，却没有role/ready/qualification声明；所以“编译和0退出”无法区分产品循环、一次性identity probe与占位程序。

### 2.2 10个工具、生成器与控制程序

| TargetId候选 | Cargo package / bin | 声明方式 | 当前职责 | 已有owner |
|---|---|---|---|---|
| `generator.woc.contract` | `woc_contract_codegen` | auto bin | contract projection生成/校验 | Tooling05 |
| `tool.cargo-zircon` | `cargo-zircon` | explicit | plugin scaffold/sync/check/validate | Tooling02 |
| `control.session-tray` | `zircon_session_tray` / `zircon-session-tray` | explicit，`autobins=false` | Coordinator Windows tray supervisor | Tooling06 |
| `tool.neural.onnx-convert` | `zircon_plugin_neural_editor` / `zr_onnx_convert` | auto bin | ONNX到模型artifact转换 | Plugins02 |
| `generator.host-reflection-docs` | `zircon_runtime` / `zircon_host_reflection_docs` | explicit，`script` | host reflection文档生成 | Tooling04 |
| `tool.export-validate` | `zircon_runtime` / `zircon_export_validate` | explicit | export preset/closure验证 | Tooling03 |
| `tool.export-pack` | `zircon_runtime` / `zircon_export_pack` | explicit | export package生成 | Tooling03 |
| `tool.shader-prewarm` | `zircon_runtime` / `zircon_shader_prewarm` | explicit，`dynamic-api` | shader预热 | Tooling07/Runtime09C |
| `tool.shader-ide-env` | `zircon_runtime` / `zircon_shader_ide_env` | explicit，`graphics` | shader IDE环境导出 | Runtime09B/Tooling01 |
| `tool.font-sdf-bake` | `zircon_runtime` / `zircon_font_sdf_bake` | explicit，`font-sdf-build-tool` | font SDF artifact生成 | Runtime11B |

`zircon_runtime`的export validate/pack没有`required-features`，其余4个工具有；是否合理不能只看是否能编译，而应由TargetDescriptor声明输入能力和最小feature closure。`zr_onnx_convert`作为`src/bin/*.rs`自动出现，未来新增同目录文件也可能成为可发布binary而没有catalog review。

### 2.3 声明与消费差异

| 检查 | 结果 | 工程含义 |
|---|---:|---|
| Rust executable target | 18 | 8 product + 10 tool/generator/control |
| 显式`[[bin]]` | 12 | name/path可见，但仍无统一role/protocol/receipt |
| Cargo auto-discovered bin | 6 | 4 WOC、WOC codegen、ONNX converter |
| `autobins=false` manifest | 3 | `zircon_app`、`zircon_hub`、session tray |
| 带`required-features`的bin | 7 | App 3个、runtime工具4个 |
| `default-run` | 0 | 多bin package的裸`cargo run`意图未声明 |
| bin级`test`/`bench` policy | 0 | 全部使用Cargo默认，没有产品taxonomy驱动的策略 |
| MVP显式产品target | 2 executable + 2 DLL变体 | 仅覆盖editor/runtime，不是仓库target catalog |

Cargo metadata可以枚举当前target，却不能表达“这是发布产品、内部生成器还是fixture”“哪个产品/安装profile允许携带它”“成功启动的定义是什么”。反向地，MVP manifest已有`logical_id/package/bin/features/output_group/artifact_name`，但只覆盖MVP四项且由PowerShell私有schema拥有，不能成为18个target的source truth。

### 2.4 参数、输出与退出行为

1. 四个WOC product用`Result` main打印一行identity JSON；进程0退出并不表示长期产品loop已Ready或执行过tick。
2. editor/runtime处理顶层错误和diagnostic log；editor保留内部`u8`退出码，runtime把错误折叠为1，但没有跨产品ExitCode registry。
3. PBR viewer、Hub、tray用Rust `Result` main；错误展示和终止码依赖Rust runtime/GUI subsystem，automation没有稳定machine result。
4. WOC codegen向stdout打印fingerprint；font bake打印人类可读摘要；ONNX converter混用JSON diagnostic与文本；生成器没有共同artifact receipt envelope。
5. reflection/export/prewarm/shader IDE工具在error时显式退出1；font/ONNX参数或执行错误退出2；`cargo-zircon`使用2表示CLI error、3表示drift、4表示validation failure。
6. 当前没有统一`--version`、target capability query、protocol negotiation、JSON schema version、stdout/stderr policy、signal/cancel/timeout或partial artifact清理合同。

这些差异不是要求“所有退出码必须相同”。工程要求是每个target声明ExitDomain和稳定meaning，consumer不得凭`code == 0`推断超过该target承诺的语义。

## 3. P0：先建立target与process资格真实性

### TOOL-EXECTARGET-P0-001 · 可执行目标没有canonical identity/catalog，构建、打包和启动只能依赖易漂移字符串

18个binary分散在11个package家族，声明方式、feature guard和consumer各异。Cargo package/bin name、文件路径、MVP `logical_id`、output group、artifact filename、Hub查找路径和脚本选择器没有共同`TargetId`或schema。自动发现还允许一个新增`src/main.rs`/`src/bin/*.rs`在未经过产品角色、发布范围、security和qualification review时成为target。当前无法机器证明“构建的是哪个角色、采用什么profile/platform/features、应携带哪些runtime dependency、能否进入release或由谁启动”。

硬切建立versioned `ExecutableTargetManifest`，所有workspace binary必须显式登记稳定TargetId、kind、owner、Cargo package/bin、source entry、platform/profile/feature constraints、artifact class、install group、launch protocol、security principal和qualification profile；Cargo metadata与manifest双向校验。未登记target、重复identity、auto-discovered target、consumer裸字符串或artifact closure不完整时fail closed。MVP现有四项迁移为该catalog的profile projection，不建立第二份真相。

### TOOL-EXECTARGET-P0-002 · 进程创建/0退出没有绑定artifact、Ready、执行观察与qualification，placeholder可满足产品成功协议

当前调用链最多证明Cargo命令成功、文件存在、`Command::spawn`成功或process exit code为0。它不绑定TargetArtifactReceipt，不证明启动的是预期BuildSet/target generation，不区分one-shot tool与long-running product，不要求Ready/health handshake，不记录实际执行能力，也不把fallback/degraded/placeholder状态带入qualification。四个WOC identity进程是直接反例：它们可编译、运行、输出结构化JSON并成功退出，但未构成长生命周期client/server/bot/headless产品。

硬切建立`TargetArtifactReceipt -> LaunchContract -> ProcessInstanceReceipt -> HealthObservation -> TargetQualificationReceipt`链。Product target必须在deadline内以authenticated/versioned handshake报告TargetId、BuildSetId、artifact digest、instance/generation、role、capabilities和Ready状态；Tool/Generator必须提交schema-versioned terminal receipt、input/output digests和atomicity结果。spawn成功、窗口出现、日志关键字或0退出只是一条observation，不能单独产生Qualified。placeholder/no-op/fallback进入Tooling16的负向状态并阻断相应profile晋级。

## 4. P1：Executable Target控制面重构

### 4.1 Target catalog、身份与角色

1. **TOOL-EXECTARGET-P1-001**：定义稳定`TargetId`，不得从可改名的exe文件名、Cargo package或source path临时推导。
2. **TOOL-EXECTARGET-P1-002**：定义`TargetKind::{Product, Service, Tool, Generator, ControlPlane, Probe, Fixture, TestHarness}`，每个kind采用不同qualification语义。
3. **TOOL-EXECTARGET-P1-003**：每个descriptor记录owner、support tier、shipping visibility、security boundary和deprecation/replacement policy。
4. **TOOL-EXECTARGET-P1-004**：禁止workspace binary依赖Cargo autobin进入catalog；manifest validator比较Cargo metadata，新增或消失target必须显式review。
5. **TOOL-EXECTARGET-P1-005**：为多bin package声明`default-run`或明确禁止裸运行，避免本地/CI选择依赖Cargo提示和人工记忆。
6. **TOOL-EXECTARGET-P1-006**：target role与runtime profile分开；同一package不因共享dependency就默认拥有client/editor/server全部能力。
7. **TOOL-EXECTARGET-P1-007**：platform、architecture、configuration、feature set和toolchain形成`TargetVariantId`，不可覆盖稳定TargetId。
8. **TOOL-EXECTARGET-P1-008**：WOC bot/client/headless/server声明为不同Product role；不得用同一identity probe结果冒充四种qualification。
9. **TOOL-EXECTARGET-P1-009**：generator/probe/fixture默认不进入shipping product；需要携带时由install profile显式纳入并记录reason。
10. **TOOL-EXECTARGET-P1-010**：catalog schema支持unknown-field、version upgrade和removed target tombstone，历史receipt仍可解释。

### 4.2 构建、artifact、package与install

11. **TOOL-EXECTARGET-P1-011**：`TargetArtifactReceipt`绑定TargetVariantId、BuildSetId、compiler/linker、dependency closure、features和artifact digest。
12. **TOOL-EXECTARGET-P1-012**：记录exe、DLL、symbols、assets、config、runtime dependency和notice为typed build products，不只记录一个filename。
13. **TOOL-EXECTARGET-P1-013**：MVP的`logical_id/package/bin/features/output_group/artifact_name`由catalog projection生成并做roundtrip drift gate。
14. **TOOL-EXECTARGET-P1-014**：Hub/build/export/CI按TargetId查询artifact，不各自拼接`target/{profile}/name.exe`。
15. **TOOL-EXECTARGET-P1-015**：required feature closure由resolver验证最小性、冲突、平台合法性和实际Cargo invocation一致性。
16. **TOOL-EXECTARGET-P1-016**：无`required-features`的export target也必须声明feature policy；“默认能编译”不是完整descriptor。
17. **TOOL-EXECTARGET-P1-017**：install manifest绑定目标路径、mode/ACL、side-by-side generation、runtime search path和rollback group。
18. **TOOL-EXECTARGET-P1-018**：package profile拒绝意外tool/debug/probe，开发SDK profile则显式包含对应artifact和documentation。
19. **TOOL-EXECTARGET-P1-019**：artifact discovery只接受签名/哈希验证的receipt，不以目录中同名文件或最新mtime选择。
20. **TOOL-EXECTARGET-P1-020**：TargetReceipt接入Tooling09 release、Tooling17 source/notice closure和O01/O04 identity，不创造孤立digest体系。

### 4.3 启动、监督与生命周期

21. **TOOL-EXECTARGET-P1-021**：`LaunchContract`声明工作目录、argv schema、env allowlist、stdio mode、principal、resource budget和startup deadline。
22. **TOOL-EXECTARGET-P1-022**：process argv使用结构化argument list；日志/receipt保留redaction后的原始值和canonical form，避免命令字符串二次解析。
23. **TOOL-EXECTARGET-P1-023**：`ProcessInstanceId`与OS pid、start time、TargetArtifactId和generation绑定，pid复用不能冒充旧实例。
24. **TOOL-EXECTARGET-P1-024**：long-running Product/Service通过versioned handshake区分Spawned、Starting、Ready、Degraded、Draining、Exited、Failed。
25. **TOOL-EXECTARGET-P1-025**：one-shot Tool/Generator通过terminal receipt区分Succeeded、NoChange、Drift、Rejected、Cancelled、Failed和Partial。
26. **TOOL-EXECTARGET-P1-026**：Ready handshake验证TargetId、BuildSetId、protocol version和expected nonce，禁止仅扫描stdout关键字。
27. **TOOL-EXECTARGET-P1-027**：supervisor拥有Child/process tree、bounded stdout/stderr、deadline、graceful stop、forced kill和terminal collection。
28. **TOOL-EXECTARGET-P1-028**：GUI subsystem的错误仍写入durable diagnostic/receipt；没有console不能丢失startup failure。
29. **TOOL-EXECTARGET-P1-029**：crash、panic、signal、OOM、device loss和operator termination映射为不同terminal reason，保留原始platform status。
30. **TOOL-EXECTARGET-P1-030**：restart/upgrade只在旧generation drain与新generation Ready完成后原子切换；失败恢复last-good。

### 4.4 CLI、输出、错误与自动化

31. **TOOL-EXECTARGET-P1-031**：每个automation target声明CLI protocol id/version；不支持machine mode的GUI target显式标记而非假装兼容。
32. **TOOL-EXECTARGET-P1-032**：machine mode向stdout输出单一schema-versioned event/receipt流，人类日志与progress进入stderr或指定log sink。
33. **TOOL-EXECTARGET-P1-033**：定义跨工具ExitDomain registry，保留目标私有code但必须映射stable category和retryability。
34. **TOOL-EXECTARGET-P1-034**：usage error、invalid input、drift、validation rejection、environment failure、internal fault和partial mutation不可都折叠为1/2。
35. **TOOL-EXECTARGET-P1-035**：`--help`、`--version`、`--protocol-version`和`--capabilities`具有稳定、无副作用、可测试语义。
36. **TOOL-EXECTARGET-P1-036**：路径、target、profile、platform等参数使用typed parser与canonicalization，未知flag和多余位置参数fail closed。
37. **TOOL-EXECTARGET-P1-037**：生成器terminal receipt记录input/output digest、写入模式、atomic commit、skipped/rejected项和diagnostic artifact。
38. **TOOL-EXECTARGET-P1-038**：取消与Ctrl-C/console close映射到Operation cancellation；不得留下可被consumer接受的半成品。
39. **TOOL-EXECTARGET-P1-039**：stdout/stderr、receipt和artifact均有bytes/items/time预算；supervisor不能因子进程持续输出而死锁或耗尽内存。
40. **TOOL-EXECTARGET-P1-040**：兼容窗口、schema negotiation和removed field策略写入CLI contract；脚本不得解析自由文本措辞作为业务状态。

### 4.5 测试、证据与currentness

41. **TOOL-EXECTARGET-P1-041**：catalog validator检查18个当前target恰好一一登记、无重复TargetId、无orphan consumer和无意外autobin。
42. **TOOL-EXECTARGET-P1-042**：每个Product target至少有bad args、startup failure、Ready、graceful drain、crash和stale artifact测试。
43. **TOOL-EXECTARGET-P1-043**：每个Tool/Generator至少有help/version、bad args、success、typed failure、cancel和partial-write恢复测试。
44. **TOOL-EXECTARGET-P1-044**：测试从catalog生成target matrix，不依赖手写bin清单；任何新增target自动要求owner和qualification profile。
45. **TOOL-EXECTARGET-P1-045**：运行证据绑定source、TargetArtifactReceipt、host/platform、argv/env digest、instance和terminal status。
46. **TOOL-EXECTARGET-P1-046**：WOC product test必须证明role-specific tick/authority/state行为；identity JSON只可作为probe observation。
47. **TOOL-EXECTARGET-P1-047**：editor/runtime/Hub真实窗口或headless smoke验证Ready protocol，而非仅验证entry helper返回值。
48. **TOOL-EXECTARGET-P1-048**：CLI golden覆盖JSON schema、stderr discipline和exit mapping；人类文案变化不破坏automation。
49. **TOOL-EXECTARGET-P1-049**：fault injection覆盖缺DLL/asset、wrong generation、端口占用、permission、timeout、signal和磁盘写失败。
50. **TOOL-EXECTARGET-P1-050**：qualification receipt进入Tooling10/15结果聚合；partial、omitted、unsupported和fallback不得被计作全绿。

## 5. P2：工程成熟度与运营体验

1. **TOOL-EXECTARGET-P2-001**：生成target catalog文档，按Product/Tool/Generator/ControlPlane展示owner、support tier和调用示例。
2. **TOOL-EXECTARGET-P2-002**：提供`cargo zircon target list/describe/build/run/qualify`作为catalog projection，而非建立新source truth。
3. **TOOL-EXECTARGET-P2-003**：shell completion从argv schema生成，避免target/profile/feature字符串漂移。
4. **TOOL-EXECTARGET-P2-004**：为每次运行生成可检索的短RunId，同时保留全局唯一ProcessInstanceId。
5. **TOOL-EXECTARGET-P2-005**：GUI产品提供“诊断包”动作，导出redacted launch/health/terminal receipt和相关日志。
6. **TOOL-EXECTARGET-P2-006**：target catalog支持support owner、runbook、known limitation和sunset date链接。
7. **TOOL-EXECTARGET-P2-007**：CI展示新增/删除target与shipping profile差异，避免manifest review被Cargo噪声淹没。
8. **TOOL-EXECTARGET-P2-008**：提供按TargetId聚合的startup latency、failure class和crash-free session趋势，指标不替代qualification。
9. **TOOL-EXECTARGET-P2-009**：支持response file和超长argv，在Windows/Linux编码、quote和路径语义上做跨平台golden。
10. **TOOL-EXECTARGET-P2-010**：machine receipt可选写入指定artifact路径并原子提交，stdout仍只输出locator/envelope。
11. **TOOL-EXECTARGET-P2-011**：为target alias提供显式兼容期与warning，禁止静默把旧名称重定向到不同role。
12. **TOOL-EXECTARGET-P2-012**：生成产品安装后的self-check，验证receipt、依赖、权限、协议和最小Ready，不执行破坏性业务操作。

## 6. 目标架构与状态机

### 6.1 Canonical数据模型

| 对象 | 最小字段 | 不得替代为 |
|---|---|---|
| `ExecutableTargetDescriptor` | TargetId、kind、owner、package/bin、entry、variant constraints、artifact/install/launch/qualification profile | exe名、source path、Cargo auto-discovery |
| `TargetArtifactReceipt` | TargetVariantId、BuildSetId、toolchain、build products、dependency/notice closure、digests | `target/debug`中文件存在 |
| `LaunchContract` | protocol、argv schema、env、cwd、principal、stdio、budget、deadline、health endpoint | 拼接command line或README示例 |
| `ProcessInstanceReceipt` | instance、pid/start、artifact、nonce、state transitions、terminal reason | `Child`存在或pid整数 |
| `HealthObservation` | reporter、timestamp/clock、protocol、role/capability、Ready/Degraded evidence | 窗口出现、sleep、日志关键字 |
| `TargetQualificationReceipt` | profile、artifact/instance、required checks、executed observations、negative states、verdict | compile success、spawn success、exit 0 |

### 6.2 分类型状态机

所有target共享声明与artifact前缀：

`Declared -> Resolved -> Built -> Packaged -> Installed`

长期Product/Service继续：

`Launched -> Starting -> Ready -> Active -> Draining -> Exited -> Qualified`

一次性Tool/Generator继续：

`Invoked -> InputsValidated -> Executing -> OutputsCommitted -> TerminalReceipt -> Qualified`

任一阶段可进入`Rejected`、`Failed`、`Cancelled`或`Degraded`；`Qualified`不是自然终态，只能由qualification evaluator对required observations、负向状态和同代receipt求值产生。

### 6.3 消费链

1. Cargo/workspace只负责解析和构建，不决定shipping role或产品完成度。
2. Target catalog解析TargetVariant并生成构建请求；builder产出TargetArtifactReceipt。
3. Package/install owner验证artifact closure并生成可回滚install generation。
4. Hub、MVP、CI或开发工具按LaunchContract启动并持有ProcessInstance。
5. Product通过health handshake，tool通过terminal receipt提供执行观察。
6. Qualification evaluator联合Tooling16 capability truth和Tooling10 test result判定目标是否可晋级。

## 7. 与已有报告的责任边界

| 现象 | 本篇拥有 | 既有owner拥有 |
|---|---|---|
| WOC四入口打印JSON即退出 | target kind、Ready/qualification不能被0退出满足 | App03-05的client/server/bot/headless实际产品loop与ZrVM语义 |
| editor/runtime/viewer启动 | descriptor、artifact/process/health receipt | App01/02的host、window、render和teardown实现 |
| Hub拼接并spawn editor | TargetId查询与LaunchContract | Hub01的project/install/process persistence和UX |
| ONNX converter参数/输出 | CLI/terminal receipt envelope | Plugins02的模型格式、转换正确性、atomic artifact |
| export/prewarm/font/reflection工具 | catalog、CLI protocol、exit/receipt | Tooling03/04/07与Runtime09/11的domain正确性 |
| test/CI认为target通过 | target qualification输入与identity | Tooling10的selection/result completeness、Tooling15的MVP gate |
| placeholder/fallback | 作为负向状态进入target receipt | Tooling16的全产品capability/no-op/fallback判定 |

## 8. 参考实现差异

| 参考 | 观察到的具体机制 | 对Zircon的约束 | 不外推的内容 |
|---|---|---|---|
| Unreal UBT | `TargetDescriptor`/`TargetRules`表达target配置，`TargetReceipt`记录name/type/platform/architecture/configuration/build products/runtime dependencies与version | target identity、variant、build product和runtime dependency必须能形成receipt | 不复制UBT/C#或Unreal庞大的target类型层级 |
| Godot | `main/main.cpp`集中解析editor/project-manager/headless/display/audio/rendering driver等模式并执行兼容性检查 | 多模式产品需要单一可审计routing、driver/profile validation和一致错误边界 | 不要求Zircon把所有产品合并成一个binary |
| Bevy | example metadata包含technical name/category/type/required features；example-showcase按catalog构建运行并记录成功/失败、日志和截图 | 大量可执行样例/target应由catalog驱动选择与证据，不靠目录猜测 | example showcase不是shipping target receipt或产品qualification |
| Fyrox | build tools定义可序列化`CommandDescriptor`、`BuildProfile`、env与build/run command queue | target运行命令、profile、env和build/run顺序应可序列化、验证与重放 | 不把其命令队列直接当作进程监督或安全模型 |

Unity Graphics本地镜像主要是package/render测试矩阵，不含Unity完整Editor/Hub/Player target控制面，因此本篇不以它证明可执行target架构完成度。

## 9. 重构里程碑

### M0 · Inventory Freeze

- 固定18个target清单、8/10角色划分和owner；
- validator报告12 explicit、6 auto-discovered及所有consumer裸字符串；
- 只生成diagnostic，不改变现有build/run行为。

### M1 · Descriptor 与 Cargo双向校验

- 引入ExecutableTargetManifest、TargetId/Kind/Variant schema；
- 全部binary显式声明并关闭非预期autobin；
- Cargo metadata、manifest、MVP projection和文档清单零漂移。

### M2 · Target Artifact Receipt

- builder记录exe/DLL/symbol/assets/config/runtime dependency/notice closure；
- Hub、MVP、export按receipt查找artifact；
- wrong generation、同名陈旧文件和缺依赖fail closed。

### M3 · CLI 与 Terminal Receipt

- 先迁移one-shot工具，定义machine output、ExitDomain和atomic output receipt；
- 保留现有人类CLI兼容窗口；
- bad args/cancel/partial write/unknown schema golden齐备。

### M4 · Product Launch 与 Health

- editor/runtime/Hub/viewer/WOC接入LaunchContract、nonce handshake和ProcessInstanceReceipt；
- supervisor拥有deadline、bounded output、drain/kill与terminal status；
- identity probe、窗口出现和0退出不再生成Ready。

### M5 · Qualification 与发布门

- Tooling10/15/16聚合TargetQualificationReceipt；
- Product证明role-specific execution，Tool/Generator证明input/output和atomicity；
- shipping profile只接受同BuildSet、无placeholder/fallback、完整receipt的目标。

### M6 · Hard Cutover

- 删除MVP/Hub/脚本中的target/filename私有真相和compat alias；
- 未登记target、旧CLI schema、无receipt artifact与裸spawn全部由CI阻断；
- 旧receipt只读解释，不能晋级新release。

## 10. 验收门

| Gate | 验收条件 |
|---|---|
| G01 | Cargo metadata枚举的workspace binary与catalog恰好18项、一一对应、0 auto-discovered orphan |
| G02 | 每项有唯一TargetId、kind、owner、variant constraints、artifact/install/launch/qualification profile |
| G03 | MVP、Hub、CI、export和开发命令只通过TargetId消费，不重写package/bin/path/filename映射 |
| G04 | 同名陈旧exe、wrong BuildSet、缺DLL/asset/config/notice均不能获得TargetArtifactReceipt |
| G05 | long-running Product必须完成authenticated Ready handshake，spawn/窗口/log/exit 0均不能单独通过 |
| G06 | one-shot Tool/Generator必须提交schema-versioned terminal receipt并证明output atomicity |
| G07 | stdout/stderr、exit category、help/version/protocol/capability在声明为machine-capable的target上通过golden |
| G08 | timeout、cancel、panic/crash、signal、OOM、permission、disk full和partial output产生不同typed terminal reason |
| G09 | WOC四role分别执行role-specific tick/state/authority测试，identity JSON只计probe observation |
| G10 | qualification绑定同代BuildSet/artifact/process/test inventory，omitted/partial/fallback/degraded不计全绿 |
| G11 | package profile不携带未声明tool/probe/fixture，SDK profile的额外target也有receipt与notice closure |
| G12 | 现有App/Hub/Plugins/Tooling owner测试仍通过，跨报告finding不重复计数 |

## 11. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| 18个Rust binary物理清单 | review_complete | 2026-08-16 | 8 product、10 tool/generator/control；12 explicit、6 auto-discovered |
| manifest/entrypoint/consumer审查 | review_complete | 2026-08-16 | Cargo、MVP、Hub、WOC、tool入口与直接测试逐项核对 |
| 参考引擎target/command机制 | review_complete | 2026-08-16 | Unreal receipt、Godot mode routing、Bevy showcase、Fyrox command profile |
| ExecutableTarget/Artifact/Launch/Process/Qualification设计 | design_complete | 2026-08-16 | 本篇第6节；尚未实现schema、validator或迁移 |
| Production与manifest重构 | pending | - | 本篇不修改production、Cargo manifest、workflow或tests |
