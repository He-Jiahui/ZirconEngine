---
title: Plugin Artifact、Marketplace、Third-party Package、Install/Update、Trust、Non-Cargo Surface 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins21
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/plugin/package_manifest
  - zircon_runtime/src/core/framework/project/project_plugin_manifest
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_runtime/src/script/vm/plugin
  - zircon_runtime/src/plugin/export_build_plan
  - zircon_editor/src/ui/host/editor_manager_plugins_export
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/web/src/data/hubData.ts
  - zircon_plugins
tests:
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/tests
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery/payload_cache.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginDescriptor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginReferenceDescriptor.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameFeatures/Source/GameFeatures/Private/GameFeaturePluginStateMachine.cpp
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/editor/asset_library/asset_library_editor_plugin.cpp
  - dev/godot/editor/asset_library/editor_asset_installer.cpp
  - dev/godot/editor/plugins/editor_plugin_settings.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_internal/src/default_plugins.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Fyrox/editor/src/plugin.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/package.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 21 · Plugin Artifact、Marketplace、Third-party Package、Install/Update、Trust、Non-Cargo Surface 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前已经有插件声明、第一方 catalog、项目 enablement、Native/VM discovery、动态库生命周期、hot reload 和 export plan；这些不是空壳，尤其 Native discovery 的路径 containment、symlink/root 检查、深度/文件/字节/时间预算、异步 authority、generation snapshot，以及 VM package 的惰性 bytecode materialization 都应保留。

但这些能力之间缺少一条正式的“发布者制品 -> 仓库索引 -> 可重复解析 -> 下载验证 -> 隔离检疫 -> 事务安装 -> 已安装代际 -> 项目锁定 -> 激活/回滚”控制链。当前39份首方 `plugin.toml` 和39个dist crate仍是源码仓库中的Cargo构建单元；仓库没有任何tracked插件archive或动态二进制，project selection也只保存id、enabled、required、target、packaging、crate与features。换一台机器、切换平台或安装第三方包时，系统无法证明解析到了同一版本、同一来源、同一签名者和同一字节制品。

产品面同时存在两套互相矛盾的真值。真实 `Project Plugins` pane只管理本地manifest、packaging、target、feature、unload和hot reload，外部包必须人工复制到 `<project>/zircon_plugins`；另一套Workbench `Plugin Manager`则写死 `Plugin_Audio`、`Plugin_RenderDoc`、`Plugin_Gameplay`、`v1.8.2`、`18 installed / 3 updates / 1 warning`，所有按钮只路由到静态反馈文本。Hub更明确把plugin install、plugin toggle和Marketplace download标记为v1禁用。当前没有Marketplace产品服务，也没有安装/update能力，只有图标、preview和coming-soon入口。

本轮确认一项新的可执行制品准入P0：export `plugins/native_plugins.toml` 的entry id或package path与实际 `plugin.toml` 不一致时，validator只写diagnostic，仍返回`Ok(true)`并发布candidate；现有测试还明确断言mismatch后 `discovered().len() == 1`。随后 `load_*_from_load_manifest` 会进入 `Library::new`。这使显式staging manifest无法充当制品身份边界，必须改成fail-closed。

Plugins01继续拥有selection前加载、ABI/ownership、签名缺失、通用lockfile和dist shell等共享阻断；Runtime07继续拥有solver、generation、VM/native isolation与lifecycle；Runtime04继续拥有通用artifact store；Editor06继续拥有通用enablement/status UX。本文不重复这些P0，新增owner仅为artifact repository、Marketplace、第三方包、install/update transaction、trust admission、installed generation和产品控制面。本轮登记 **1项新增P0、48项P1、12项P2与48项资格门**。

## 2. 审查边界、规模与currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / test attributes | 冻结事实 |
|---|---:|---|
| Runtime/Editor control | 253 / 50,543 / 1,818,070 / 419 | package/project manifest、native loader全目录、VM plugin全目录、export selection、Editor manager/actions/projection与App caller |
| Product UI/Hub | 8 / 3,720 / 159,387 / 3 | Workbench静态Plugin Manager、route/binding/feedback、Hub coming-soon与web data |
| Manifest/dist/project surface | 121 / 8,797 / 316,314 / 79 | root workspace/README、39份plugin manifest、39份dist Cargo、39份dist lib与两个project manifest |
| Zircon去重合计 | **382 / 63,060 / 2,293,771 / 501** | fingerprint `dc79aca7d60e5295b847f7406ce03a786de6eba64eea99fcaebd4b29519d6ab1` |
| 参考源码 | **19 / 17,691 / 659,576** | fingerprint `5bb518c7c3ca726bd39c25ef4507d97f894e76f1e74d6b08359144f6aca219ca` |

分组fingerprint分别为Runtime/Editor control `86a4156892b318561768cdb28085a8b92955ee9018956ef300c512d997dbc377`、Product UI/Hub `dafee734c3d1b0a416c9b6667a0cab7d36db76f493cd94ad2bbeb991b6d39903`、Manifest/dist/project `f7b78e7d076cf9236579e0fffdc7ae2e808fd3a821ab8ea2eb0e560093f54490`。算法为相对路径排序，对每个文件取SHA-256，再对 `path<TAB>hash<LF>` 清单取SHA-256。

冻结revision为`bea1acf91b909525ab1759e2c800858b0eda6528`、coordinator baseline epoch为335。入选范围中App Editor入口、Editor export manager/wizard execution/native registration和Workbench template binding共5个文件存在其他Session或用户修改，因此本文标记 `source_recheck_required: true`。`zircon_plugins/.zircon-cache` 有4个、13,750 bytes的ignored shader cache文件，它不是tracked package store，也不计入382文件。

### 2.2 证据等级与限制

本轮逐文件读取上述382个Zircon文件，并沿 `plugin.toml -> PluginPackageManifest -> ProjectPluginSelection -> Native/VM discovery -> Editor status/action -> App startup -> Workbench/Hub product entry` 追踪纵向调用链。静态证据可证明schema缺项、消费者缺失、mismatch fail-open、产品入口真值和测试固化行为。

本轮没有运行Cargo、真实DLL、Editor/Hub、网络、签名者、恶意archive、跨平台、fault injection、soak或benchmark；没有修改production、tests、Cargo、assets或tooling。用户已明确暂不审查新tooling优化，因此Marketplace/install实现路线只定义runtime/editor/hub产品合同，后续工具迁移由独立Rust tooling计划承接。静态审查不能证明性能已优于Unreal；本文只把性能目标转成可验证gate。

### 2.3 Owner边界

| Canonical owner | 继续拥有 | Plugins21只消费的接口 |
|---|---|---|
| Plugins01 | Native ABI soundness、selection前执行、签名sidecar未准入、SDK/package通用lockfile、dist metadata shell | `VerifiedPluginArtifact`进入loader前必须已经可信 |
| Plugins06/20 | 第一方source/dist/catalog closure、fixture/sample shipping隔离 | Marketplace catalog只能发布eligibility通过的artifact |
| Runtime07 | dependency solver、generation、VM/native isolation、load/unload/hot reload生命周期 | 安装解析输出immutable lock与installed generation |
| Runtime04 | 通用artifact/content-addressed storage、resource cook/cache | 插件store复用底层CAS，不另造通用blob cache |
| Runtime Interface01 | foreign ownership、稳定FFI与host boundary | 第三方native artifact admission不改变ABI owner |
| Editor06 | 现有Plugin Manager status/enablement/native materialization | 新产品面消费repository/install/activation snapshots |
| Plugins21 | repository/index、publisher/source、artifact variant、fetch/verify/quarantine、transaction install/update/uninstall/rollback、installed registry、Marketplace UX/product truth | 本文新增owner |

## 3. 当前真实基础与产品真值

### 3.1 应保留的基础

1. `PluginPackageManifest` 已集中package identity、target/platform、capability、module、dependency、interface、feature、asset、packaging与distribution元数据。
2. Native目录发现会canonicalize root、拒绝越界路径、限制深度/entry/manifest/bytes/wall time，并通过authority发布generation snapshot。
3. Native load manifest读取和TOML解析有候选数与scratch/read budget，不需要为Marketplace重新发明无界扫描器。
4. Native loader在 `Library::new` 后仍会验证descriptor、entry、ABI与registration；应把它保留为“已验证制品后的代码准入层”。
5. VM package discovery拒绝绝对/逃逸path、symlink异常和超预算manifest/bytecode，并支持惰性materialization。
6. Editor project plugin action已经有enable/disable、target、packaging、feature dependency、unload、hot reload与状态投影。
7. Export plan能从project manifest推导first-party crate与NativeDynamic package集合，适合作为locked resolution的下游consumer。
8. Hub明确承认本地v1边界，没有把远程Marketplace悄悄伪装成已完成服务；这份product truth应保留并替换为真实能力门。

### 3.2 定量事实

| 事实 | 当前值 | 结论 |
|---|---:|---|
| tracked `zircon_plugins/**/plugin.toml` | 39 | 都是仓内第一方source package declaration |
| tracked dist `Cargo.toml` / `src/lib.rs` | 39 / 39 | 每包一个Cargo dist carrier，不是安装包仓库 |
| tracked plugin archive/native binary/WASM | 0 | 没有可下载、可验证、可安装的正式artifact格式 |
| 含`[plugins]`的project TOML | 2 | Vampire与WOC只保存selection，不保存resolution lock |
| `ProjectPluginSelection` source/version/digest/signer | 0 / 0 / 0 / 0 | 工程不可重复解析到同一制品 |
| production Marketplace查询/下载consumer | 0 | Hub只有disabled entry，Editor只有preview/static route |
| ignored `.zircon-cache` | 4 files / 13,750 bytes | shader cache，不是package install store |

### 3.3 两套Editor产品面

真实`Project Plugins` pane从builtin runtime/editor catalog和本地native discovery生成行，操作集合是SetEnabled、CyclePackaging、CycleTargetModes、SetFeatureEnabled、EnableFeatureDependencies、Unload和HotReload。`EditorManager::plugin_directory()`固定为project root下的`zircon_plugins`，没有install/download/update/uninstall command；external source只显示`native`，没有publisher、version、license、digest、signer、origin或trust状态。

Workbench `Plugin Manager`不是这套manager的另一个view，而是独立preview：ZUI写死三个插件、四条dependency row、版本warning和统计；navigation spec、template binding和preview action只映射固定字符串；feedback handler返回“Plugin hot reload queued”和“Validation queued 18 installed 1 warning”。这些route没有连接真实catalog、solver、artifact store或native host。必须在产品中删除/隔离demo truth，或改造成真实snapshot consumer。

### 3.4 Hub边界

`coming_soon.rs` 明确发布：`plugin-install` 为“Installing or downloading plugins is disabled in v1”，`plugin-toggle`等待本地manifest稳定，`marketplace-download`为local-only v1范围外。Hub当前project/device install pipeline不是plugin installer，不得因为都使用“install”命名就复用错误状态或宣称已覆盖。

## 4. 参考引擎给出的约束

### 4.1 Unreal Engine

Unreal `FPluginDescriptor` 分开保存creator/support/docs/Marketplace URL、engine version、supported platform/program、module、localization、installed、hidden、sealed、explicit load、pre/post build、plugin dependency和disallowed dependency。`FPluginManager`先发现多个来源/版本，再按project、target、configuration、command line和default policy解析enabled set，最后挂载content并按phase加载module；发现不等于安装，安装也不等于激活。

GameFeature state machine进一步显式区分Uninstalled、CheckingStatus、StatusKnown、Downloading、Installed、Mounting、WaitingForDependencies、Registering、Registered、Loading、Loaded、Activating和Active，并为release/unmount/unregister/unload/deactivate建立反向路径和错误态。Zircon不必复制类名，但必须达到同等级的可恢复状态与receipt，而不是用一个enabled bool替代整条链。

### 4.2 Godot

Godot Asset Library通过HTTP下载到editor cache临时ZIP，校验release提供的SHA-256，展示archive文件树与目标冲突，并在安装后触发filesystem rescan。Editor Plugin Settings单独显示installed plugin的enabled/name/version/author，并在recovery mode停止插件执行。GDExtension manager又独立负责load/reload/unload和initialization level。

Godot并不是Zircon的最终安全标准：其asset installer检测ZIP symlink后会创建link，文件写入也不是内容寻址/签名/事务式package安装。可借鉴的是download/hash/preview/recovery/product separation，不能照搬其archive安全与供应链模型。

### 4.3 Bevy、Fyrox与Unity Graphics

Bevy `Plugin` 只提供build、ready、finish、cleanup、name与uniqueness，是静态Rust app composition下限，不包含第三方package repository。Fyrox区分static/dynamic plugin，提供prepare/reload、register/init/on_loaded/on_deinit和Editor生命周期，但其Rust dylib也不构成长期稳定的公共package ABI。二者只用于校准activation lifecycle，不作为Marketplace或trust参考。

Unity Graphics仓内三份`package.json`使用稳定package name、17.6.0版本和精确dependency version，HDRP/URP依赖同版本SRP Core/ShaderGraph等。该snapshot不包含闭源Unity Package Manager、账号、entitlement或registry实现；本文只把它作为manifest/version/dependency可重复性的证据，不外推其供应链行为。

## 5. P0：可执行制品准入必须fail-closed

### MPA-P0-001 · Load manifest的id/path mismatch只诊断却继续接纳并可加载实际候选

`collect_load_manifest()`先按entry的`manifest`读取实际 `plugin.toml`，得到candidate，再调用`validate_load_manifest_entry()`。该函数发现 `entry.id != candidate.package_manifest.id` 时只emit diagnostic；发现实际manifest parent不在声明 `entry.path` 下时也只emit diagnostic；两处分支都不返回false，函数最终执行`Ok(true)`。candidate随后插入snapshot，`load_all/runtime/editor_from_load_manifest()`继续进入`load_candidates_for_module_kinds()`和`Library::new`。

现有 `native_loader_reports_load_manifest_entry_mismatches` 测试构造 `id = declared_weather`、`path = plugins/declared_weather`、`manifest = plugins/actual_weather/plugin.toml`，随后明确断言`report.discovered().len() == 1`，并仅检查两条diagnostic。另一个refresh测试修改manifest内部id后也断言candidate被保留。错误行为已被regression contract固化。

必须把staging manifest定义成不可歧义的artifact receipt：entry id、canonical package root、manifest path、artifact path、target/ABI、content digest与signer必须共同匹配，任一不一致都拒绝candidate并记录结构化admission failure。测试必须反转为零discovered、零library open、零entry invocation，并覆盖sibling package substitution、manifest swap、symlink/junction、case folding、TOCTOU和重复id。

## 6. P1：工程化闭环差距

### 6.1 Package identity、resolution与artifact contract

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| MPA-P1-01 | package manifest没有publisher、author、license、repository、homepage、docs、support或source provenance | `PluginPublisherIdentity + PluginSourceRef + LicenseExpression + SupportLinks`，展示字段与准入identity分离 |
| MPA-P1-02 | dependency只有id/required/capability/interfaces，没有version range、source、artifact、interface version或feature constraint | 可求解的`PluginDependencyRequirement`，包含semver、source domain、target variant和interface/schema范围 |
| MPA-P1-03 | project selection没有resolved version/source/digest/signer/dependency closure | machine-written `zircon-plugin.lock`，锁定完整graph和每个target artifact |
| MPA-P1-04 | selection新建时`enabled`可直接写入且缺省true，只有package id | 用户intent与resolver output分开；未知/未安装id不能被一个bool提升为可激活插件 |
| MPA-P1-05 | package manifest及多数nested DTO不拒绝unknown key | 稳定schema `deny_unknown_fields`，扩展走显式namespaced extension table和schema epoch |
| MPA-P1-06 | distribution只有forms/default packaging/ABI/engine/crate/symbol/entry/assets | 每平台/架构/configuration artifact variant，记录filename、size、digest、ABI、build id、toolchain与minimum host |
| MPA-P1-07 | native library path由dist crate名称和平台命名规则重新推导 | artifact receipt给出唯一canonical relative path，runtime不得猜测发布期决定 |
| MPA-P1-08 | `native_plugins.toml`没有artifact digest、signer、build id、file list、source或install generation | 版本化、签名覆盖的staging manifest，绑定package manifest和每个payload |
| MPA-P1-09 | bounded load manifest parser对未知top-level field使用`IgnoredAny` | admission schema fail-closed；兼容扩展必须版本协商，不能静默丢关键policy |
| MPA-P1-10 | VM manifest只有name/version/entry/capabilities/management | 增加engine/VM ABI、compiler/build id、target、dependency、digest、debug/source-map、signer与state schema |
| MPA-P1-11 | VM payload cache fingerprint只有path、len与mtime | 以verified content digest和artifact identity作为cache key，读取后复核handle identity，防同长同mtime替换 |
| MPA-P1-12 | source/Cargo、NativeDynamic、VM package没有共同的artifact identity | `PluginArtifactId(package, version, source, target, kind, digest)`贯穿resolver、store、loader、UI与telemetry |

### 6.2 Repository、Marketplace与生态metadata

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| MPA-P1-13 | 没有repository client、signed index snapshot或snapshot generation | `PluginRepository`读取immutable signed index，支持官方、private、local和air-gapped source |
| MPA-P1-14 | 没有browse/search/category/tag/platform/engine/price/license filter | server query contract与local indexed search，共用cursor、snapshot和deterministic ordering |
| MPA-P1-15 | 没有release channel、yanked/deprecated/recalled、compatibility或replacement metadata | version publication state machine与resolver policy，已锁版本的撤回处理必须显式 |
| MPA-P1-16 | 没有publisher organization、namespace ownership、verified identity或transfer history | 可审计namespace claim、publisher key rotation、ownership transfer与revocation chain |
| MPA-P1-17 | 没有release notes、support status、certification、download size或quality evidence | Marketplace listing schema与artifact qualification receipt分开，UI不得伪造统计 |
| MPA-P1-18 | 没有private registry/mirror/proxy/auth credential contract | source-scoped credential provider、TLS/pinning policy、mirror priority和secret redaction |
| MPA-P1-19 | 没有offline index snapshot、metadata cache expiry或stale policy | 可验证的离线snapshot、last-known-good、expiry/revocation freshness与manual refresh receipt |
| MPA-P1-20 | 没有commercial entitlement、seat/device/license acceptance状态 | entitlement只进入install policy，不污染runtime ABI；离线许可与撤销行为必须定义 |

### 6.3 Fetch、verify、install、update与uninstall transaction

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| MPA-P1-21 | 没有plugin fetcher的timeout、retry、resume、range、redirect、rate-limit或size policy | bounded downloader，先有declared size/digest，再流式写临时CAS并支持可验证resume |
| MPA-P1-22 | 没有正式archive格式及entry count/path/size/ratio/symlink/junction/hardlink预算 | canonical package archive与streaming extractor，拒绝zip-slip、bomb和host-specific link escape |
| MPA-P1-23 | 没有trust root、publisher signature、timestamp、transparency、revocation或key rotation | 签名覆盖index、manifest和payload digest；offline/expired/revoked策略可审计且默认fail-closed |
| MPA-P1-24 | 没有quarantine、malware/SBOM/vulnerability/license policy gate | `ArtifactAdmissionReport`在安装前汇总来源、签名、SBOM、policy与人工override |
| MPA-P1-25 | 没有content-addressed plugin store与跨项目dedupe | 复用Runtime04 artifact CAS，package层只管理manifest、ref和installed generation |
| MPA-P1-26 | 没有stage -> verify -> expand -> validate -> promote的原子安装事务 | 同volume staging、fsync/journal、atomic generation publication；失败不污染active tree |
| MPA-P1-27 | 没有进程崩溃/断电后的install recovery journal | 启动时replay/rollback orphan transaction，保证旧generation可用且临时空间可回收 |
| MPA-P1-28 | 没有last-known-good generation与一键rollback | update先并行安装新generation，通过probe/migration后切换；保留bounded rollback window |
| MPA-P1-29 | 没有installed file ownership receipt，uninstall无法证明删除边界 | 每generation记录payload、generated/cache/config owner；删除只依据receipt且保护用户数据 |
| MPA-P1-30 | 没有共享dependency refcount、project pin或store GC | lease/ref graph、pin reason、reachability GC、disk quota与dry-run reclaim report |
| MPA-P1-31 | 没有update resolver、compatibility diff、channel policy或downgrade protection | old/new lock diff、breaking capability/ABI/state migration预检、roll-forward与rollback策略 |
| MPA-P1-32 | project manifest、lockfile、installed generation与activation不是一个提交边界 | `PluginDeploymentTransaction`原子发布intent、resolution、install receipt和activation target generation |

### 6.4 Installed registry、activation与第三方隔离

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| MPA-P1-33 | installed、enabled、loaded、registered、active和healthy被多处bool/状态投影混用 | 正交状态机和单调receipt；UI显示当前阶段、目标阶段、owner和失败恢复动作 |
| MPA-P1-34 | Editor只扫描`<project>/zircon_plugins`，第三方包必须人工复制 | engine/user/project store分层，project只引用lock；dev path package使用显式mutable source mode |
| MPA-P1-35 | native status source只显示`native`，没有origin/version/digest/trust | status row消费`InstalledPluginRecord`和`ArtifactAdmissionReport`，可追溯到repository snapshot |
| MPA-P1-36 | 没有installed package registry或immutable generation catalog | `InstalledPluginRegistrySnapshot`列出available/active/LKG/quarantined generations及refcounts |
| MPA-P1-37 | activation没有验证当前load artifact等于lock与installed receipt | loader只接受generation-qualified verified handle，禁止从任意路径再次discover并推导 |
| MPA-P1-38 | target/platform/architecture/configuration没有安装期variant选择与缺失诊断 | resolver选定精确variant；Editor、Client、Server、Shipping可有不同但可解释的lock projection |
| MPA-P1-39 | 第三方native、VM与first-party source没有trust/isolation tier | first-party static、trusted native、isolated native、VM/WASM分别定义process、capability、filesystem/network policy |
| MPA-P1-40 | update/hot reload没有package data/config/state schema migration合同 | versioned migration plan、dry run、backup、forward/rollback compatibility和failure quarantine |

### 6.5 Product UI、Hub与运营

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| MPA-P1-41 | 真实Project Plugins pane没有install/update/remove/source/trust/dependency plan | 在同一pane接入catalog、details、plan preview、download progress、transaction result与rollback |
| MPA-P1-42 | Workbench Plugin Manager以固定插件名、版本和统计冒充产品状态 | 删除product exposure或改成只消费真实immutable view model；preview data必须标明sample且不进入production route |
| MPA-P1-43 | Workbench hot reload/validate只返回queued文字，没有job id、执行或结果 | 命令返回typed operation handle、progress、cancel、diagnostic和terminal receipt |
| MPA-P1-44 | Hub明确禁用plugin install/toggle/Marketplace，Editor与Hub没有共享authority | 建立单一repository/install service，Hub与Editor作为不同权限client，不复制状态机 |
| MPA-P1-45 | Marketplace只有一个icon与coming-soon entry，没有真实catalog surface | listing/detail/version/dependency/trust/install/update views必须来自同一signed snapshot |
| MPA-P1-46 | 没有第三方开发者onboarding、compatibility matrix、submission/certification或support window | public SDK release、artifact conformance、review pipeline、deprecation/SLA和reproducible sample |
| MPA-P1-47 | 没有artifact operation audit stream、correlation id、bytes/timing/source/result metrics | download/verify/install/activate/rollback结构化事件，secret/path脱敏并支持故障归因 |
| MPA-P1-48 | 没有safe/recovery mode按崩溃归因禁用suspect generation | startup crash ledger、last activation correlation、safe mode、逐包恢复和LKG回退 |

## 7. P2：规模、性能与长期运营差距

| ID | 差距 | 验收目标 |
|---|---|---|
| MPA-P2-01 | catalog未定义10k/100k listing规模 | 100k版本索引的增量同步、查询P95、内存和冷启动预算 |
| MPA-P2-02 | resolver没有大图复杂度与冲突解释预算 | 1k package/10k edge deterministic solve、timeout/cancel和最小冲突core |
| MPA-P2-03 | index/view model没有immutable delta generation | UI更新只应用delta，不重扫目录或重建全catalog |
| MPA-P2-04 | downloader没有并发与带宽公平性 | per-host/global concurrency、priority、pause/resume和foreground budget |
| MPA-P2-05 | archive验证可能形成大内存展开 | streaming hash/extract、bounded buffer和declared/actual size accounting |
| MPA-P2-06 | plugin store没有跨版本chunk dedupe | content/chunk address、compression policy和dedupe收益可观测 |
| MPA-P2-07 | installed registry没有启动关键路径预算 | snapshot/mmap或等价compact index，10k generation下冷启动不递归stat payload |
| MPA-P2-08 | signature/SBOM/vulnerability检查没有缓存currentness | 以digest/policy/revocation generation缓存，避免重复昂贵验证且不跳过撤销 |
| MPA-P2-09 | update没有delta artifact | 支持可选delta但以完整target digest终验，delta失败自动回完整包 |
| MPA-P2-10 | telemetry没有Marketplace漏斗和故障维度 | 查询、plan、download、verify、install、activate各阶段成功率/延迟/错误码 |
| MPA-P2-11 | store GC没有在线增量和I/O预算 | foreground-safe GC、mark snapshot、rate limit、dry run与可恢复sweep |
| MPA-P2-12 | 无与Unreal同场景的插件系统benchmark | 相同package graph/artifact count下比较discover/solve/install/startup/update/rollback；只按数据宣称性能优势 |

## 8. 目标架构

```text
RepositorySource[]
  -> SignedRepositorySnapshot
  -> PluginResolver(ProjectPluginIntent, TargetProfile, Policy)
  -> PluginLockfile + ResolutionExplanation
  -> ArtifactFetcher
  -> ArtifactCAS(staged)
  -> ArtifactVerifier(signature, digest, SBOM, policy)
  -> Quarantine / VerifiedPluginArtifact
  -> TransactionalPluginInstaller
  -> InstalledPluginGeneration + InstallReceipt
  -> InstalledPluginRegistrySnapshot
  -> ActivationCoordinator
  -> Native / Isolated Native / VM / Source Host
  -> ActivationReceipt + Health/LKG

Editor Project Plugins ----\
Hub Marketplace ------------> PluginDeploymentService -> immutable view/event snapshots
Export Build Plan ----------/
```

关键边界：

1. Repository snapshot只描述可解析metadata，不直接执行代码。
2. Lockfile是resolver输出，不由UI手写；同一lock在相同target与policy下必须解析到同一digest。
3. CAS只接受流式digest验证后的blob；expanded tree仍要独立manifest与path admission。
4. Installer只发布immutable generation，不在active目录原地覆盖。
5. Loader只接受verified generation handle，不再从任意project目录递归执行代码。
6. Enablement是用户intent，activation是policy、dependency、trust、target和health共同决定的结果。
7. Native第三方默认不与Editor主进程等权；隔离等级由artifact trust与capability policy决定。
8. Hub、Editor和export共享service/snapshot，不共享临时UI状态或重复实现installer。

## 9. 依赖顺序与重构里程碑

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 Fail-closed admission | 修复MPA-P0-001，反转mismatch tests，禁止未匹配receipt进入Library open | G-01至G-06 |
| M1 Identity/schema | package/source/publisher/dependency/artifact variant、strict schema、VM artifact identity | G-07至G-12 |
| M2 Resolver/lock | repository snapshot、deterministic solver、project lock、target projection与解释 | G-13至G-18 |
| M3 Fetch/trust | bounded downloader、CAS、digest/signature/revocation/SBOM/quarantine | G-19至G-25 |
| M4 Transaction install | staging/journal/promotion/installed registry/uninstall/GC/LKG | G-26至G-33 |
| M5 Activation/isolation | verified handle、state machine、third-party isolation、migration、safe mode | G-34至G-39 |
| M6 Product surfaces | 真实Editor/Hub Marketplace与operation jobs，删除静态product truth | G-40至G-44 |
| M7 Ecosystem | private/offline registry、publisher/certification、license/entitlement、audit | G-45至G-47 |
| M8 Scale/performance | 规模、fault、soak、跨平台和对标benchmark | G-48 |

禁止把M6 UI提前接到fake backend；每个UI状态必须来自M0-M5的immutable snapshot/receipt。Plugins01/Runtime07/Runtime04的底层修复与M0-M5并行时，应通过共享合同合流，不另建第二套ABI、solver或artifact store。

## 10. 资格门

- [ ] G-01 id mismatch时candidate count为0，diagnostic为结构化rejection。
- [ ] G-02 declared package path与manifest parent不一致时candidate count为0。
- [ ] G-03 mismatch、duplicate、escape、symlink/junction测试证明`Library::new`调用次数为0。
- [ ] G-04 manifest swap/TOCTOU测试证明verified handle绑定同一opened file identity与digest。
- [ ] G-05 load manifest unknown security field、unsupported schema epoch与缺失required field均fail-closed。
- [ ] G-06 fuzz corpus覆盖load manifest/path normalization且无panic、越界或fail-open。
- [ ] G-07 package manifest能表达publisher/source/license/support并严格拒绝typo。
- [ ] G-08 dependency能表达version/source/interface/target/feature约束并有negative tests。
- [ ] G-09 artifact variant绑定target triple、configuration、ABI、build id、size和digest。
- [ ] G-10 Native、VM、source package使用共同`PluginArtifactId`且projection不漂移。
- [ ] G-11 VM payload cache以verified digest命中，同长同mtime替换测试不会返回旧/错字节。
- [ ] G-12 39份首方manifest可迁移且generated/file/source projection一致。
- [ ] G-13 repository snapshot签名、generation、expiry和source identity可验证。
- [ ] G-14 resolver在相同输入下byte-for-byte deterministic，输入顺序不改变lock。
- [ ] G-15 lock记录完整dependency closure、artifact digest、signer和target variant。
- [ ] G-16 missing/yanked/revoked/incompatible dependency给出最小冲突解释。
- [ ] G-17 Vampire/WOC在clean machine只依赖lock即可恢复同一artifact graph。
- [ ] G-18 Editor/Client/Server/Shipping lock projection差异明确且可审计。
- [ ] G-19 downloader执行大小、时间、并发、redirect、retry、resume与rate-limit预算。
- [ ] G-20 interrupted download恢复后以完整target digest终验，错误partial不可晋升。
- [ ] G-21 archive path traversal、absolute path、case collision、bomb、symlink/junction/hardlink语料全拒绝。
- [ ] G-22 publisher signature、timestamp、key rotation与revocation端到端通过。
- [ ] G-23 offline stale/expired/revoked snapshot行为符合显式policy且有审计记录。
- [ ] G-24 SBOM、license、vulnerability与manual override形成不可变admission receipt。
- [ ] G-25 verified blob进入共享CAS，跨项目/版本dedupe不混淆package identity。
- [ ] G-26 install只经过stage/verify/expand/probe/promote，不原地覆盖active generation。
- [ ] G-27 每个阶段断电/kill fault injection后可恢复旧generation并清理orphan。
- [ ] G-28 install receipt完整记录owned files、generated files、config/cache/user-data policy。
- [ ] G-29 uninstall不会删除共享dependency、其他generation或用户数据。
- [ ] G-30 update失败保持旧generation active并提供typed rollback原因。
- [ ] G-31 LKG pin、project pin、dependency ref与GC reachability一致。
- [ ] G-32 store quota、dry-run GC、rate-limited sweep和crash recovery通过规模测试。
- [ ] G-33 project intent、lock、installed generation和activation target原子提交或全部回滚。
- [ ] G-34 loader API只接受verified generation handle，任意目录path API退出产品入口。
- [ ] G-35 installed/enabled/loaded/registered/active/healthy状态可分别观测且转换合法。
- [ ] G-36 trusted native、isolated native、VM/WASM、first-party source各有独立policy与tests。
- [ ] G-37 capability/filesystem/network/process policy在第三方恶意fixture下fail-closed。
- [ ] G-38 package data/config/runtime state migration支持dry run、backup、forward和rollback。
- [ ] G-39 startup crash能归因最后activation generation并自动进入safe/LKG模式。
- [ ] G-40 Project Plugins真实显示origin/version/license/digest/signer/trust/dependency/install state。
- [ ] G-41 browse/detail/install/update/remove/rollback全链只消费真实repository/service snapshot。
- [ ] G-42 Workbench固定插件名、版本、统计和queued反馈不再进入production product truth。
- [ ] G-43 Hub与Editor并发操作同一package时由一个deployment transaction authority串行化。
- [ ] G-44 download/install/update operation有job id、progress、cancel、retry和terminal receipt。
- [ ] G-45 official/private/local/offline source、credential redaction、mirror与proxy矩阵通过。
- [ ] G-46 publisher namespace、submission、certification、deprecation、key transfer/revocation可审计。
- [ ] G-47 license/entitlement、audit event、privacy/redaction和support workflow通过产品验收。
- [ ] G-48 100k index、1k package/10k edge、10k installed generation、跨平台/fault/soak/benchmark满足预算；性能比较使用相同输入和公开数据。

## 11. 完成定义

本文完成不等于“有一个Marketplace页面”，而是以下事实同时成立：project可以在clean machine从可信source重建同一lock与同一digest；任意可执行artifact在代码打开前完成来源、schema、path、digest、signature、policy和target准入；安装/update/uninstall是可恢复事务并保留LKG；runtime只激活已安装verified generation；Editor与Hub显示同一真实snapshot；第三方native不会默认获得主进程等权；规模、故障与性能数据达到预算。

在这些条件满足前，产品文案必须继续明确“仅本地manifest enablement/hot reload”，不得把Workbench preview、Marketplace icon、Cargo dist crate或Hub project install描述成第三方插件Marketplace与安装更新能力。
