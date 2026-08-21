---
related_code:
  - zircon_runtime_interface/src/resource
  - zircon_runtime_interface/src/project/rel_path
  - zircon_runtime_interface/src/hub_protocol
  - zircon_runtime/src/core/resource/io
  - zircon_runtime/src/core/resource/io/atomic_file
  - zircon_runtime/src/core/resource/io/transaction
  - zircon_runtime/src/core/framework/asset.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/pipeline/manager/driver/asset_io_driver.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager
  - zircon_runtime/src/asset/pipeline/manager/service_contracts
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/ensure_resident.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watch_dispatch.rs
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/watch/map_notify_event.rs
  - zircon_runtime/src/asset/watch/watch_loop.rs
  - zircon_runtime/src/asset/watch/asset_uri_for_path.rs
  - zircon_runtime/src/asset/watch/watched_asset_uri_for_path.rs
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/asset/safe_project_path.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/asset/project/manager/source_uri_for_path.rs
  - zircon_runtime/src/asset/project/manager/source_path_for_uri.rs
  - zircon_runtime/src/asset/project/manager/durable_transaction.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/migration
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_runtime/src/plugin/runtime_plugin
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/graphics/scene/resources/ui_texture.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_editor/src/core/project
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg/cache.rs
  - zircon_hub/src/projects/metadata.rs
  - zircon_hub/src/projects/install_receipt.rs
  - zircon_hub/src/engines/source_engine_paths.rs
  - zircon_hub/src/assets/catalog.rs
tests:
  - zircon_runtime_interface/src/tests
  - zircon_runtime/src/asset/tests/watcher.rs
  - zircon_runtime/src/asset/tests/project
  - zircon_runtime/src/core/resource/io/atomic_file/tests
  - zircon_runtime/src/core/resource/io/transaction/engine/tests.rs
  - zircon_runtime/src/core/resource/io/transaction/recovery/tests.rs
  - zircon_app/src/entry/entry_runner/editor/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformFile.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/PlatformFile.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/Paths.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Misc/PackageName.h
  - dev/bevy/crates/bevy_asset/src/io/source.rs
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/bevy/crates/bevy_asset/src/path.rs
  - dev/Fyrox/fyrox-resource/src/io.rs
  - dev/godot/core/io/file_access.h
  - dev/godot/core/io/dir_access.h
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 25 · Filesystem、Path、URI、VFS、Mount、Watch、Sandbox 与 Atomic I/O 工程化差距

## 1. 结论

Zircon并非完全没有工程级文件系统基础。`ProjectPaths`已经把物理operation path与display path分离，能解析现有junction、SUBST、symlink和未创建尾段，拒绝Windows drive-relative/root-relative输入，并为项目创建和打开保留一次解析后的物理身份。项目asset root和package root会做canonical containment与重复根检查；扫描器拒绝symlink/reparse point并过滤原子事务兄弟文件。`atomic_file`和通用durable transaction具备`create_new` staging、文件与目录sync、journal、owner lock、故障恢复和Windows/Unix替换实现。Asset watcher同时限制ingress和folded queue的entries/bytes，溢出时要求reconciliation；ProjectAssetManager又用project generation、preparation epoch、retry和transactional publication防止旧扫描覆盖新project generation。这些基础应保留并提升为共享层。

真正缺失的是统一文件系统架构。`ResourceIo`只有`read/write/exists`三个同步方法，全仓没有实现或消费者；注册为immediate driver的`AssetIoDriver`只是空unit struct，而模块描述仍宣称“Asynchronous asset I/O and CPU-side decoding”。实际asset residency、artifact、project、plugin、font、Hub、Editor和tooling路径继续直接调用`std::fs`。仓内没有FileSystem provider、Asset Source/Mount Registry、source capability、mount generation、overlay priority、unmount/quiescence、range/stream/cancel/priority I/O或统一watch provider。`res/lib/package/builtin/mem`是固定enum分支，不是可注册的来源实例。

路径语义也在多个局部owner间分裂。`RelPath`使用portable字符串分段，这是正确方向；但`ResourceLocator`仍借用host `std::path::Path::components`，同一`C:/...`文本会因host OS不同得到不同parse结果，精确DTO缺陷由Runtime Interface02拥有。Runtime本篇拥有更底层的logical URI grammar、mount/source映射和OS path codec合同。`source_uri_for_path`与watch URI映射使用`to_string_lossy`，不同非UTF-8物理名称可折叠成同一logical URI；`AssetManager::open_project(&str)`又迫使本来正确的`PathBuf`调用者做lossy转换。Hub另有自己的canonicalize/fallback/path-key算法和手写`file://`拼接，空格、`#`、`%`、UNC与非UTF-8均没有统一编码器。

安全检查同样属于“局部措施真实、共享能力不足”。`is_safe_regular_file`先用`symlink_metadata`与canonical containment验证，再由调用者按原path打开，攻击者或并发工具可在check与open之间替换目录项；hard link也不被symlink/reparse规则识别。durable owner lock在`symlink_metadata`后再次按path打开，仍是同类窗口。要把project/plugin/package root当成安全边界，必须由filesystem provider提供root-relative、no-follow、handle-based open/create/rename，而不是继续叠加`canonicalize + starts_with`。

监听器后半段已经有良好回退，但前半段会丢失触发信号：只有`RenameMode::Both`生成rename；`From/To/Any`落入普通Modify。路径映射失败通过`.ok()`静默过滤，`Both`路径数不为2或任一映射失败也直接返回空，因此不会设置已有的`requires_reconciliation`。这不是推翻watcher，而是要让mapping层返回`Mapped | IgnoredByPolicy | ReconcileRequired | Error`，把不确定性送入已经存在的回退链。

本篇拥有跨runtime的physical/logical/display path taxonomy、portable URI codec、FileSystem/Source/Mount Registry、provider capability、secure open、shared filesystem identity、watch event contract、direct-filesystem governance和I/O qualification。Runtime04继续拥有asset artifact/residency与`.zrpack` range mount细节；Interface02拥有公共`ResourceLocator/RelPath` DTO；Plugins01拥有plugin package trust/native admission；Tooling03/08/09拥有cook/DDC/release分发；Hub01拥有Hub本地项目产品流程。本篇不重复这些局部finding计数。本轮登记 **0项P0、40项P1和12项P2**，均未实施。

## 2. 审查边界、方法与 currentness

### 2.1 物理扫描

本轮以`zircon_runtime`、`zircon_runtime_interface`、`zircon_plugins`、`zircon_editor`、`zircon_app`和`zircon_hub`的11,937个production-like Rust文件为路由范围，排除显式`tests/examples`目录与`tests.rs`，并在首个`#[cfg(test)]`前统计production prefix。`PathBuf`信号1,834处/393文件，`Path`信号2,234处/561文件，filesystem/open候选777处/203文件，`canonicalize` 49处/24文件，`read_dir` 45处/34文件，watch/notify信号41处/11文件，symlink/reparse/hard-link/file-id信号113处/28文件，`strip_prefix` 260处/156文件，`starts_with` containment候选406处/198文件。

这些数字不是“203个文件都违规”。platform adapter、durable transaction和最终provider实现本来就应调用OS文件系统；问题是当前没有机器可读的bypass allowlist、owner、root capability或调用层级，无法区分合法backend与绕过VFS、调度、sandbox、事务和观测的业务代码。finding均由接口定义、实现搜索、调用者、打开时序与持久化边界联合阅读确认。

### 2.2 深读调用链

1. `ResourceLocator/RelPath -> ProjectManager source projection -> physical asset roots -> source scan/import -> artifact store -> ensure_resident`。
2. `ProjectPaths resolve -> project/package root admission -> safe regular file check -> importer/reference/meta read`。
3. `notify Event -> path-to-URI mapping -> bounded ingress -> folded batch -> activation queue -> reconciliation/incremental generation -> durable publication`。
4. `PreparedFileWrite -> PathIdentity -> owner lock/journal -> staging -> replace/sync -> recovery`。
5. `runtime_asset_path -> environment/executable/dev/crate candidates -> product asset access`。
6. `Editor/App/Hub PathBuf -> string/path key/file URL -> persistence/process/download consumer`。
7. Unreal `IPlatformFile/FPackageName`、Bevy `AssetSource/AssetReader/Writer/Watcher`、Fyrox `ResourceIo/FsResourceIo`与Godot `FileAccess/DirAccess`的provider、source与mount边界。

### 2.3 currentness

本轮源revision为`ae2be3d865a937b9ed368bf965592045346c64e3`。关键文件`locator.rs`、`resource_io.rs`、`asset_io_driver.rs`、`runtime_asset_path.rs`、`project/paths.rs`、watch mapping/loop、transaction pathing及Hub metadata/install receipt在检查时没有工作区差异；对应blob分别以`49831dc4`、`2ed407c6`、`eb55e530`、`c9c0b5e2`、`1003a851`、`4f171181`、`4621950b`、`5d0778bf`、`1434b273`和`34f76abb`开头。其他Scene、Editor、Hub及产品区域仍有在途工作，因此标记`source_recheck_required: true`。

本篇是source-level review。既有Editor、Hub、WOC与plugin动态验证阻断没有变化；没有重跑不能覆盖TOCTOU、non-UTF8、network share、rename-loss、mount change或power-loss的全量Cargo/product lane，也没有把“当前单机测试通过”当成VFS资格证据。

## 3. 当前可保留的工程基础

| 基础 | 当前证据 | 保留条件 |
|---|---|---|
| Project path identity | operation/display双路径，解析junction/SUBST/symlink和未创建尾段，拒绝不稳定Windows相对形态 | 下沉为共享`PhysicalPathResolver`，禁止Hub/Editor/transaction继续复制不同算法 |
| Portable project RelPath | 字符串分段统一`/`，拒绝root、drive prefix、`.`和`..` | 补Unicode/非法字符/codec version，并与ResourcePath共享segment grammar |
| Root admission | project/package root canonical containment、重复根拒绝、歧义URI拒绝 | secure open必须绑定已验证root handle，不能只保留path字符串 |
| Safe scanner | 逐目录拒绝symlink/reparse，过滤meta/transaction/auxiliary文件 | 增加file identity、hard-link政策、scan snapshot和check/open一致性 |
| Atomic file | `create_new` staging、sync、platform replace、backup/recovery | 成为统一writer primitive并接受capability/open handle，而非任意raw path |
| Durable transaction | journal、owner lock、multi-file commit、rollback/recovery与metrics | 合并PathIdentity owner，补secure relative create/open和所有writer接入清单 |
| Bounded watcher | ingress/pending按entries+bytes限制，debounce与max latency明确 | mapping不确定必须触发reconciliation，容量政策进入统一source capability |
| Generation publication | watcher/reimport检查project generation与preparation epoch，重试后才publish | mount/source generation也必须进入同一read snapshot和receipt |
| Artifact validation | manifest/chunk数量、bytes、hash与压缩上界在读取前校验 | artifact reader经source/provider调度，Runtime04继续拥有pack/range实现 |
| Fail-closed product root | 显式`ZIRCON_ASSET_ROOT`是authoritative，已选择root不会逐asset回退 | invalid config返回typed startup error；相对asset path必须严格验证 |

## 4. 参考实现给出的边界

### 4.1 Unreal：可包装Platform File与独立Package Mount

Unreal的`IPlatformFile`既有physical backend，又允许通过`GetLowerLevel/SetLowerLevel`叠加sandbox、pak、cache等wrapper；接口覆盖stat、directory iteration、symlink、read/write handle、async read、mapped read、优先级和file journal。`FPackageName`另行维护long package name到local path的mount point，支持register/unregister、override chain和mounted lifetime。可借鉴的是physical I/O、logical package namespace、wrapper与mount生命周期分层，而不是把所有逻辑塞进`ResourceLocator::parse`。

### 4.2 Bevy：Asset Source聚合Reader、Writer与Watcher

Bevy的`AssetSourceId`可命名自定义source，`AssetSourceBuilder`组合unprocessed/processed reader、writer和watcher；`AssetReader/Writer`提供异步stream、meta、directory、remove和rename。它证明来源实例和能力应被注册，而不是由固定scheme enum散落match。Zircon还需要比该参考更严格的mount generation、secure root和durable transaction合同。

### 4.3 Fyrox：ResourceIo是实际消费的异步provider

Fyrox `ResourceIo`声明write/directory capability，覆盖async load/write/move/delete/copy、canonicalize、directory walk和reader，并提供`FsResourceIo`。Zircon同名trait目前既无实现也无consumer，不能把类型存在视为完成。可吸收provider可替换与async surface，但不继承其默认空directory iterator或`exists -> bool`的弱错误语义。

### 4.4 Godot：资源、用户数据与裸文件系统访问域分开

Godot `FileAccess/DirAccess`按Resources、UserData、Filesystem选择backend，并覆盖seek/length/buffer/flush、directory、link、case sensitivity和equivalence；compressed/encrypted/memory/pack又是独立实现。可借鉴的是访问域和provider选择，但Zircon应以typed capability与稳定错误实现，不能依赖全局create function和thread-local last error。

### 4.5 Unity Graphics参考边界

仓内`dev/Graphics`是Unity Graphics package源码，拥有shader、render pipeline和artifact消费端，不包含Unity底层filesystem/VFS实现。本篇只把其中路径消费视为未来graphics provider的consumer，不用无法验证的内部Unity实现补齐证据。graphics具体asset streaming仍由Runtime04、09C和Tooling08拥有。

## 5. Owner裁决与非重复边界

| Owner | 本篇拥有 | 邻接报告继续拥有 |
|---|---|---|
| Path/URI Schema | logical/physical/display/file identity分类、segment/escaping/OS encoding和转换receipt | Interface02实现公共ResourceLocator/RelPath DTO与schema migration |
| FileSystem Provider | read/write/stat/list/open/create/rename capability、error与backend layering | Runtime04实现asset artifact/pack reader和residency policy |
| Source/Mount Registry | source实例、priority、generation、collision、teardown、capability与watch binding | Plugins01管理plugin package/trust，Tooling03管理cook/package产物 |
| Secure Root | root capability、no-follow handle-relative open/create、link/file identity政策 | Plugins01/Hub01管理各自principal、admission和产品操作 |
| Watch Contract | raw event映射、不确定性、reconciliation、source generation与observability | Runtime04管理asset generation publication和last-good语义 |
| Durable Write | 统一writer接入、atomicity等级、recovery receipt与filesystem identity | Editor02拥有document save/autosave，Tooling09拥有install/update transaction |
| Qualification | path corpus、mount/fault/rename/network/offline/product lane | Tooling07/10/15管理通用性能、测试和BuildSet evidence |

## 6. P1：Path、URI、Encoding 与 Identity Contract

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FILESYSTEM-P1-001 | 引擎没有明确区分Logical Resource Path、Mounted Path、Physical Operation Path、Display Path、Persistent Path和Filesystem Identity | 建公共path taxonomy；每个API声明接受/返回哪种类型、是否可持久化、是否可用于I/O和其owner |
| FILESYSTEM-P1-002 | `ResourceLocator`的logical path normalization借用host `Path::components`，portable schema结果可随OS变化；精确DTO缺陷由Interface02拥有 | 本篇要求建立纯字符串segment grammar与跨OS固定vector，Interface02据此替换parser且保持迁移owner唯一 |
| FILESYSTEM-P1-003 | URI path、source、package id和subasset label没有统一escaping；`#`无法作为文件名无损roundtrip | 定义versioned percent/length codec、保留字符集、canonical display与parse/format双射，禁止手写`format!("...://")` |
| FILESYSTEM-P1-004 | OS-native文件名到UTF-8 logical path没有政策，Unix bytes和Windows unpaired UTF-16只能被lossy折叠 | 选择“严格portable UTF-8 source set”或可逆platform codec；invalid名称必须typed reject/quarantine，不能替换字符 |
| FILESYSTEM-P1-005 | `RelPath`、`ResourceLocator`、`PathBuf`、URL和process argument之间没有显式转换receipt | 建`PathProjectionReceipt { source, mount, generation, codec, lossless, identity }`供持久化/诊断边界使用 |
| FILESYSTEM-P1-006 | case sensitivity、Unicode normalization、reserved names、trailing dot/space与collision政策分散在OS行为中 | source注册时声明case/Unicode规则，构建deterministic collision index，cook前拒绝跨目标不兼容名称 |
| FILESYSTEM-P1-007 | `ResolvedProjectPath`只覆盖project局部，Hub/Editor/transaction仍各自重建operation/display/key | 抽取共享`ResolvedPhysicalPath`与`FilesystemIdentity` owner，保留project-specific manifest规则在ProjectPaths |
| FILESYSTEM-P1-008 | Hub手写`file://{lossy path}`，没有percent encoding、Windows drive/UNC authority规则 | 提供唯一File URL codec或禁止产品receipt使用file URL；空格、`#`、`%`、Unicode、UNC必须roundtrip |
| FILESYSTEM-P1-009 | recent project、engine、install/download receipt等会持久化absolute/display/path-key字符串，缺少relocation和volume identity | 区分user bookmark、project-relative locator、install root token与ephemeral operation path；迁移时记录old/new identity |
| FILESYSTEM-P1-010 | path/URI schema没有独立version、target portability profile或source provenance | 发布`PathSchemaId`与`SourceIdentity`，纳入project/package/artifact/BuildSet receipt及兼容性检查 |

## 7. P1：FileSystem、Source 与 Mount Registry

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FILESYSTEM-P1-011 | `ResourceIo`全仓只有定义/re-export，无实现、注册或consumer | 删除假完成surface后建立实际`FileSystem/AssetSourceIo` owner；首个local provider必须被asset runtime真实消费 |
| FILESYSTEM-P1-012 | immediate注册的`AssetIoDriver`是空unit struct，模块却宣称异步I/O与CPU decode | driver要拥有队列、provider路由、budget、cancel、shutdown和metrics；未完成前descriptor应fail-close或移除能力宣称 |
| FILESYSTEM-P1-013 | 没有VFS、MountTable或Source Registry，logical scheme直接在业务代码match | 建`MountRegistry`把namespace/prefix映射到provider实例，并把lookup结果绑定mount generation |
| FILESYSTEM-P1-014 | `Res/Library/Package/Builtin/Memory`是封闭enum，plugin/product不能注册HTTP、pack、DLC、overlay或测试source | logical source ID与built-in convenience enum分离；扩展注册需owner、capability、priority和teardown token |
| FILESYSTEM-P1-015 | provider没有readable/writable/enumerable/watchable/seekable/range/atomic/trusted/local/remote能力合同 | 建typed capability query与admission；consumer按required capability选择路径，不用调用后猜`ReadOnly`字符串 |
| FILESYSTEM-P1-016 | `ResourceIo::exists -> bool`吞掉permission、offline、unsupported和transient错误并鼓励check-then-open | 以`open/stat -> Result`为主，exists仅作带error的hint且不得作为安全或原子性依据 |
| FILESYSTEM-P1-017 | `read`同步返回完整`Vec<u8>`，没有stream/range/priority/deadline/cancel或byte budget | 建async read handle与bounded buffer/range API，接入Tasks/I/O scheduler；Runtime04继续拥有asset streaming策略 |
| FILESYSTEM-P1-018 | 接口没有stat/list/open handle/create/rename/remove/fsync/atomic replace，业务只能绕回`std::fs` | 分离read-only source、mutable filesystem和durable writer traits，能力缺失返回stable typed error |
| FILESYSTEM-P1-019 | 没有mount lifecycle、generation、override order、collision、quiescence或unmount receipt | mount注册返回lease/token；lookup/read/watch携generation；unmount先拒新操作、drain handle/watcher再retire |
| FILESYSTEM-P1-020 | package当前强制恰好一个asset root，项目多root也只按first root选择新写入，缺少受控layer语义 | 定义overlay/read priority、write target、duplicate collision和DLC/localization/plugin layer规则；禁止隐式first-wins |

## 8. P1：Containment、Sandbox、Physical Identity 与 Safe Open

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FILESYSTEM-P1-021 | `normalize_runtime_asset_relative_path`静默丢弃prefix/root/`.`/所有`..`，`a/../b`会错误变成`a/b`且absolute输入被重绑到asset root | 改为fallible strict relative parser；仅显式兼容入口可剥一次`assets/`，root/upward输入必须拒绝并记录 |
| FILESYSTEM-P1-022 | invalid `ZIRCON_ASSET_ROOT`通过`panic!`终止，而不是startup configuration error | resolver返回typed error与display path；ProductHost在module activation前fail-close并生成配置receipt |
| FILESYSTEM-P1-023 | source/watch URI投影对每个`OsStr`调用`to_string_lossy`，不同物理文件可映射到同一locator | 使用第4项codec政策并在registry publish前检测collision；mapping失败不得静默忽略 |
| FILESYSTEM-P1-024 | `AssetManager::open_project`和importer capability接口接受`&str`，PathBuf调用者被迫lossy转换 | Rust内部接口接受`&Path/ResolvedProjectPath`；ABI/JSON边界使用明确UTF-8 policy或platform-native transport |
| FILESYSTEM-P1-025 | ProjectPaths与durable transaction各自实现deepest-existing-ancestor、uncreated tail和Windows equality，语义已开始分叉 | 合并为一个physical identity service与conformance suite，transaction只增加final-entry/link政策 |
| FILESYSTEM-P1-026 | `filesystem_identity_key`失败时回退lexical path，Windows又用lossy string、separator replace和Unicode lowercase，与自身精确`CompareStringOrdinal`不一致 | key改为opaque typed identity；失败必须返回Result，Windows比较复用wide ordinal或file identity，不做lossy lowercase |
| FILESYSTEM-P1-027 | `is_safe_regular_file`和owner lock先检查metadata/canonical containment再按path打开，存在check/open目录项替换窗口 | provider提供root-handle-relative open、no-follow/reparse flags与opened-handle identity复核；安全结论绑定handle而非字符串 |
| FILESYSTEM-P1-028 | symlink/reparse拒绝不覆盖hard link，canonical path也不能证明文件对象属于root | 定义hard-link政策并读取volume/file identity/link count；untrusted root可拒绝多link或由broker复制入受控staging |
| FILESYSTEM-P1-029 | 未创建尾段虽从物理祖先解析，但后续create仍按path逐段进行，祖先可在解析后被替换 | secure create逐级在已打开directory handle下创建/验证，返回最终handle和identity receipt |
| FILESYSTEM-P1-030 | project、package、plugin、engine asset、cache、user data和external import没有统一trust/root capability模型 | 建`RootCapability { principal, purpose, read/write policy, link policy, mount, generation }`，跨root复制需显式operation |

## 9. P1：Watcher、Write Transaction、Scheduling 与 Qualification

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FILESYSTEM-P1-031 | notify只有`RenameMode::Both`生成rename，From/To/Any被降为Modified，跨平台rename语义不完整 | 维护rename cookie/pair window；无法配对时转Removed/Added或`ReconcileRequired`，绝不伪装成Modified |
| FILESYSTEM-P1-032 | path-to-URI失败、Both路径数异常或单边失败直接返回空，现有reconciliation机制无法获知 | mapping返回结构化outcome；任何非policy-ignore的不确定性设置reconciliation并发布stable error/metric |
| FILESYSTEM-P1-033 | watcher activation有project generation，但raw watcher事件不携source/mount generation或root identity | `WatchBatch`携MountId/Generation、root identity、sequence与overflow reason；旧mount事件在mapping前拒绝 |
| FILESYSTEM-P1-034 | 777个filesystem/open候选散布203个production文件，没有backend-only allowlist或VFS bypass检查 | 建调用层清单和CI：只有platform/provider/transaction backend可直接OS I/O，业务层需登记受控例外 |
| FILESYSTEM-P1-035 | 没有统一I/O scheduler、per-source concurrency、bytes-in-flight、priority inversion、deadline和shutdown drain | AssetIoDriver接入Tasks但使用独立I/O domain；admission按source/device budget，所有request有cancel/terminal receipt |
| FILESYSTEM-P1-036 | durable transaction很强但只覆盖部分project/settings路径，其他writer各自使用write/rename或私有事务 | inventory所有mutation owner，按single-file atomic、multi-file durable、cache best-effort分级并接入唯一primitive |
| FILESYSTEM-P1-037 | `ResourceIoError`仅NotFound/Io/ReadOnly且承载String，无法稳定判断offline、permission、corrupt、stale mount、cancel、budget或unsupported | 建stable code、operation、source/mount、path-safe diagnostic、retryability与OS cause链，跨ABI映射由Interface owner实现 |
| FILESYSTEM-P1-038 | watcher有局部计数、transaction有局部counter，但没有per-mount requests/bytes/latency/queue/error/retry/reconcile/stall视图 | 统一I/O telemetry schema并限制路径敏感信息；Editor/Hub diagnostics只投影同一source generation |
| FILESYSTEM-P1-039 | 缺少跨Windows/Linux case/Unicode/UNC/SUBST/junction/symlink/hardlink/non-UTF8/long-path/adversarial rename corpus | 建versioned filesystem fixture corpus、property/fuzz/model tests与可重复fault provider，不依赖宿主临时目录偶然语义 |
| FILESYSTEM-P1-040 | 没有local SSD、read-only install、network share、removable/offline、mount replace、watch overflow、power-loss与low-disk产品资格矩阵 | 每个ProductRole发布required source capability和lane receipt；未覆盖的provider/profile必须Unavailable而非best-effort成功 |

## 10. P2：在P1正确性之后建设的高级能力

| ID | 能力 | 前置条件 |
|---|---|---|
| FILESYSTEM-P2-001 | HTTP/object-store/CAS remote source与signed mirror failover | P1 mount identity、range、cancel、hash、trust和offline error完成 |
| FILESYSTEM-P2-002 | USN/inotify journal增量索引与丢事件恢复 | watcher sequence、root identity和reconciliation contract完成 |
| FILESYSTEM-P2-003 | IOCP/io_uring/dispatch-I/O backend与批量submission | provider API稳定并有同语义fallback、budget和profiling |
| FILESYSTEM-P2-004 | mmap/zero-copy reader、shared blob lease与page-fault观测 | handle lifetime、unmount quiescence和memory budget完成 |
| FILESYSTEM-P2-005 | DirectStorage/GDeflate或平台直达GPU I/O | Runtime04/09A resource lifetime、GPU fence和artifact layout完成 |
| FILESYSTEM-P2-006 | compression/encryption/delta/patch filter stack | wrapper顺序、key owner、integrity、random access和recovery语义固定 |
| FILESYSTEM-P2-007 | 独立filesystem broker进程与最小权限content sandbox | RootCapability、principal、IPC ABI和crash recovery完成 |
| FILESYSTEM-P2-008 | Editor preview、PIE、package patch的mount namespace与copy-on-write overlay | mount generation/priority/collision/write target完成 |
| FILESYSTEM-P2-009 | target-aware deterministic casefold/Unicode index与跨平台rename assistant | portability schema和collision admission完成 |
| FILESYSTEM-P2-010 | Source/Mount inspector、URI projection tracer与watch timeline | telemetry/error/path-redaction schema完成 |
| FILESYSTEM-P2-011 | network/removable source offline cache、lease与background resync | source identity、consistency model、CAS和conflict receipt完成 |
| FILESYSTEM-P2-012 | workload-aware prefetch/read coalescing与分布式I/O trace | correctness oracle、priority/cancel和可比较performance baseline完成 |

## 11. 目标架构

### 11.1 类型层

1. `LogicalResourcePath`只包含versioned portable segments，不接触host `Path`。
2. `ResourceLocator`组合`SourceId + LogicalResourcePath + SubresourceLabel`，format/parse必须双射。
3. `MountKey { id, generation }`标识一次已发布mount实例；同名重挂不能复用旧generation。
4. `ResolvedMountPath`组合mount snapshot、logical path和provider-local opaque path，不暴露任意拼接。
5. `ResolvedPhysicalPath`保留operation/display view；`FilesystemIdentity`是opaque、fallible、不可用于I/O的比较键。
6. `OpenedFile/OpenedDirectory/RootCapability`绑定已打开对象、owner、policy和identity，安全检查不返回裸bool。
7. 持久化只允许logical locator、bookmark/install token或显式portable path；operation path默认禁止serde。

### 11.2 服务层

| 服务 | 责任 | 禁止拥有 |
|---|---|---|
| PathSchema | segment、escaping、case/Unicode profile、portable validation | OS打开、mount priority |
| PhysicalPathResolver | OS absolute/alias/display/file identity与secure root open | asset scheme和artifact policy |
| MountRegistry | source实例、prefix、priority、generation、lease、collision与teardown | importer、resource residency |
| FileSystemProvider | open/stat/list/create/rename/remove/sync/watch与能力报告 | product fallback和asset decode |
| AssetIoDriver | admission、queue、priority、cancel、bytes-in-flight、shutdown和telemetry | locator schema和project publication |
| DurableWriter | single/multi-file atomicity、journal、recovery与receipt | 任意source discovery和UI状态 |
| WatchRouter | provider event、URI mapping、coalesce、overflow/reconcile与mount generation | import/compile的semantic decision |

### 11.3 请求与发布链

`ResourceLocator -> MountRegistry snapshot -> capability admission -> AssetIoDriver request -> provider opened handle/range -> bounded decode -> artifact/hash validation -> resource generation prepare -> commit/publish`。

write链为`logical destination -> writable mount/root capability -> secure relative create -> stage/sync -> durable commit -> watcher suppression/correlation -> generation publication -> receipt`。任何一步失败都必须产生terminal error并保持旧generation可用；禁止lookup后悄悄回退另一source。

watch链为`provider event + mount generation -> map outcome -> bounded fold -> reconcile/incremental plan -> generation/epoch validate -> durable commit -> publish`。`IgnoredByPolicy`只允许meta、transaction sibling等显式规则；未知rename、编码失败、root mismatch和overflow都进入reconciliation。

## 12. 分层实施计划

### M0 · Truth Freeze与调用清单

- 把`ResourceIo`和`AssetIoDriver`标记为unimplemented capability，停止用descriptor存在证明异步I/O完成；
- 生成direct filesystem callsite清单，按backend/transaction/product bypass分类；
- 固定path/URI/non-UTF8/rename/link corpus和当前source fingerprints；
- 不改变现有ProjectPaths、watcher和transaction行为。

### M1 · Path Schema与共享Physical Identity

- Interface02实现纯字符串ResourceLocator grammar、escaping和codec version；
- 抽取ResolvedPhysicalPath/FilesystemIdentity，迁移ProjectPaths、transaction、Hub/Editor重复key；
- 修复runtime asset relative path的silent component dropping和typed startup error；
- 给所有lossy conversion建立拒绝或显式diagnostic-only边界。

### M2 · FileSystem Provider与Mount Registry

- 先实现LocalFileSystem provider与read-only builtin/memory adapter；
- 建Source/Mount Registry、capability、priority、collision、generation和lease；
- 让asset source/artifact reader真实经过provider，随后删除dead ResourceIo旧surface；
- `.zrpack`/range reader由Runtime04按同一mount合同接入。

### M3 · Secure Root与Durable Mutation

- 实现root-relative no-follow open/create及opened-handle identity复核；
- 定义hard-link/reparse/UNC/network root政策与principal capability；
- 合并PathIdentity算法，把project、settings、scene、Hub/install writers分级接入DurableWriter；
- 用fault provider覆盖replace、sync、low disk、permission、crash与restart。

### M4 · Watch Contract与Source Generation

- map层返回结构化outcome，覆盖所有rename mode和mapping失败；
- WatchBatch携mount/root identity与generation，旧source事件fail-close；
- 保留并复用现有bounded ingress、fold、reconciliation和project generation publication；
- 建event loss/duplicate/reorder/overflow模型测试。

### M5 · I/O Scheduling、Error与Observability

- AssetIoDriver实现I/O domain、per-source queue、bytes-in-flight、priority、cancel/deadline和shutdown drain；
- 收敛stable error code/retryability/OS cause与路径redaction；
- 发布per-mount latency/bytes/errors/reconcile/stall指标和operation receipt；
- 接入memory pressure、DDC和runtime residency，但不跨越其owner政策。

### M6 · 产品资格与高级Provider准入

- 对Editor、Client、Server、Hub、cook/export和installed product分别声明required source capabilities；
- 在Windows/Linux、case-sensitive/insensitive、read-only、network、offline、mount change和power-loss矩阵验证；
- local provider全门通过后，才准入pack/HTTP/CAS/mmap/platform direct I/O；
- 性能比较必须绑定相同BuildSet、artifact、storage、cold/warm cache和workload。

## 13. 验收矩阵

| Gate | 验收条件 |
|---|---|
| FS-G01 | `ResourceLocator`固定vector在Windows/Linux产生相同结构与canonical文本 |
| FS-G02 | path、label、`#/%/space/Unicode`和File URL parse/format无损roundtrip |
| FS-G03 | non-UTF8/unpaired UTF-16按声明政策reject或可逆编码，绝不lossy collision |
| FS-G04 | ResourceIo旧trait不再dead；AssetIoDriver拥有真实consumer、queue和shutdown |
| FS-G05 | 所有logical read由MountRegistry解析并绑定MountKey generation |
| FS-G06 | mount override/collision/unmount/reload后旧handle、watch event和request不能命中新source |
| FS-G07 | provider capability不足在admission失败，不产生partial read/write或隐式fallback |
| FS-G08 | secure root测试在symlink/junction/reparse/hard-link/race下不能越界打开或创建 |
| FS-G09 | runtime asset absolute/upward/invalid env输入返回typed error且不panic、不重绑 |
| FS-G10 | AssetManager与Rust内部project路径不经过`to_string_lossy` |
| FS-G11 | rename From/To/Any、mapping failure和queue overflow均保留变化或要求reconciliation |
| FS-G12 | watcher reconcile后registry/artifact/runtime resource发布为同一project和mount generation |
| FS-G13 | direct OS I/O只存在于approved backend/transaction例外，CI可重建清单 |
| FS-G14 | single/multi-file writer在write/sync/replace/crash/restart故障下保持旧值或可恢复新值 |
| FS-G15 | error包含stable code、operation、source/mount、retryability和安全诊断，不靠字符串分支 |
| FS-G16 | bytes/items/time/depth/queue budget在分配和读取前执行并支持cancel/deadline |
| FS-G17 | Editor/Client/Server/Hub/export各有read-only、low-disk、offline和mount-change产品lane |
| FS-G18 | network/removable provider不可用时能力状态与last-good明确，不显示成功或空结果 |
| FS-G19 | benchmark区分cold/warm cache、local/network storage、read size/queue depth并报告p50/p95/p99 |
| FS-G20 | source fingerprint、mount/path schema、BuildSet和test corpus进入machine-readable evidence |

## 14. 风险、依赖与迁移约束

1. 先修schema再迁移provider。若先把现有`ResourceLocator`直接铺到更多backend，会固化跨OS与escaping缺陷。
2. 先保留ProjectPaths/transaction/watcher正向能力，再抽取共享owner。不能以“统一”为由退回lexical normalize、无journal write或unbounded channel。
3. secure open需要Windows和Unix平台实现，不能用更多`canonicalize + starts_with`测试替代。
4. `res/lib/package/builtin/mem`已有持久数据，hard cutover必须提供codec/mount migration和unknown scheme策略。
5. direct filesystem治理不是禁止所有`std::fs`；provider backend、fault test和durable primitive必须保留底层访问，但业务owner不能无登记旁路。
6. Runtime04、Plugins01、Hub01、Editor02、Tooling03/08/09的实施必须消费本篇合同，不能各自再发明mount、URL、path key或transaction。
7. HTTP/CAS/DirectStorage属于P2，不能用远程source demo替代local provider的正确性、安全与恢复闭环。

## 15. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| 六产品家族path/filesystem信号inventory | review_complete | 2026-08-16 | 11,937 production-like Rust文件；filesystem/open候选777处/203文件 |
| 核心调用链深读 | review_complete | 2026-08-16 | locator/project path/source projection/artifact/residency/watch/transaction/Hub URI链 |
| 参考引擎边界核对 | review_complete | 2026-08-16 | Unreal platform file/package mount、Bevy source、Fyrox IO、Godot access domain |
| Finding与owner裁决 | review_complete | 2026-08-16 | 0 P0 / 40 P1 / 12 P2；与Runtime04、Interface02、Plugins01等不重复计数 |
| Production重构 | pending | - | 本篇只新增review；未修改production、tests、manifest或workflow |

本篇不主张“实现一个万能VFS类”即可完成工程化。完成标准是logical schema、physical identity、source/mount lifecycle、secure opened capability、I/O scheduling、watch uncertainty、durable mutation和产品资格由明确owner串成一条可验证链，并且现有正确基础在迁移后仍能通过相同或更强的故障门。
