---
related_code:
  - .gitignore
  - .github/workflows/ci.yml
  - .github/workflows/mvp-editor-windows.yml
  - .github/workflows/profile-feature-contract.yml
  - tools/dev-fast-build.ps1
  - tools/README-fast-build.md
  - tools/check-runtime-domain-features.ps1
  - tools/check-runtime-profile-features.ps1
  - tools/zircon_build_cargo_environment.py
  - tools/zircon_build_shader_prewarm.py
  - tools/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/benchmark_validation_grants.py
  - zircon_runtime/src/asset/artifact/cache_key.rs
  - zircon_runtime/src/asset/artifact/chunk_residency.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/metadata.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/worker.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs
tests:
  - tools/tests/dev-fast-build.Tests.ps1
  - tools/tests/test_zircon_build_cargo_environment.py
  - tools/tests/test_zircon_build_shader_prewarm_cache_contract.py
  - tools/session_coordinator/tests/test_cargo_runner.py
  - tools/session_coordinator/tests/test_workspace_copy.py
  - zircon_runtime/src/asset/tests/artifact.rs
  - zircon_runtime/src/asset/tests/project/manager/artifact_cache_imports.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests/combined_validation_tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataCacheKey.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataCachePolicy.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildDefinition.h
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildWorker.h
  - dev/bevy/crates/bevy_asset/src/processor/mod.rs
  - dev/godot/core/io/resource_importer.cpp
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/Fyrox/fyrox-resource/src/manager.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 08 · 共享派生数据缓存、构建缓存、远程执行与 Artifact 复用工程化差距

## 1. 结论

ZirconEngine 已经有多块值得保留的缓存基础，但还没有工程级 Derived Data Cache。资产 artifact store 采用 schema 4 manifest、BLAKE3 内容寻址 64 KiB chunk、zstd、有界解码、先发布不可变 chunk 再原子切换 manifest，并带内存 residency budget；项目导入又会比较 source digest、importer identity/version 与 config hash，不能把当前本地恢复路径误判为无条件读取陈旧文件。shader prewarm 有 source table、WGSL/WGPU验证、versioned目录和工具侧pair/report合同；UI compiled artifact有完整compile input fingerprint；IBL runtime cache也有BLAKE3 request identity与原子单文件发布。CI的3个workflow共7处使用`Swatinem/rust-cache@v2`，`dev-fast-build.ps1`在找到sccache时也会真正设置`RUSTC_WRAPPER=sccache`。

这些能力目前仍是五套局部协议：asset chunk store、shader variant pair、UI compiled pair、IBL blob、Cargo/rust-cache/sccache。仓库没有统一的Build Function、Action Definition、稳定Action Digest、Record/Value schema、local/remote policy、backend hierarchy、quota/GC、namespace/trust、determinism verifier、request owner、cache health或remote worker协议。现有runtime热缓存和本地文件缓存可以继续服务各自子系统，但不能直接开放为跨工程、跨Build Set或跨机器共享缓存；否则身份不完整、发布非事务、污染与权限边界都会被放大。

shader prewarm当前有两个必须先修复的P0。第一，disk hash只覆盖canonical variant与include hashes，不覆盖WGSL正文、template revision、Naga/WGPU版本；worker又只按这个不足的hash去重。两个不同source可以被报告成两个written variant，磁盘却只保留第一个source。第二，WGSL和metadata分别用固定`.tmp`名字独立rename；并发writer可互相覆盖临时文件，而rename失败时只要目标存在便返回成功且不核对目标字节，最终可能发布mixed generation或把他人的内容当成本次成功。这两点会直接破坏staged shipping cache的真实性。

本轮没有发现产品级remote execution owner。Session Coordinator已提供managed process、workspace materialization、ticket与artifact receipt雏形，但没有把command、input tree、toolchain、environment、platform capability和declared outputs编码成canonical action；也没有input/output CAS、worker registration、sandbox policy、secret boundary或cache hit replay。因此路线必须先建立本地可验证Action Cache，再增加团队共享DDC，最后才允许远程执行。直接把Coordinator的任意argv执行或现有`.zircon/cache`同步到服务器，会把报告06中的认证和可伪造验证风险扩展成远程代码执行与cache poisoning。

本轮记录2个P0、52个P1和10个P2。未修改生产Rust、Python、PowerShell、workflow或缓存内容；只新增审查和索引。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 范围 | 本轮深度 |
|---|---:|---|
| asset artifact/CAS/currentness | artifact目录17个文件，加scan/import核心路径 | E3：key、manifest、chunk publication、恢复判断、residency与disk cleanup |
| shader disk/prewarm | 5个核心Rust文件、2个Python owner、合同测试 | E3：source identity、disk key、pair publication、report/currentness与fallback |
| UI/IBL persistent cache | UI store 374行、IBL store 189行 | E3：fingerprint、publication、integrity、eviction与设备身份 |
| Cargo/CI cache | 3 workflows、4类managed environment、fast-build入口 | E3：activation、root、tool pin、reuse、metrics与CI consumer |
| remote execution | 产品与tooling全仓关键字/owner搜索，Coordinator对照 | E2 absence proof；E3 action/worker/security边界设计 |
| reference engines | Unreal DDC/build worker、Bevy processor、Godot import、Fyrox resource manager | E3 owner contract对照，不把普通runtime resource cache冒充DDC |

本轮选定45条Git index record，共14,450行、532,297 bytes，Git-index fingerprint为`a4b2ce9104dfc6c363c0c8aa2062ad42e760a825519d73886c7d2b2dd9fb6459`。这是本报告的复核边界，不代表1006个名称含cache/artifact/derived/build的产品文件全部已达到E3；其余runtime内存热缓存继续由所属系统报告拥有。

### 2.2 动态与定量证据

| 检查 | 结果 | 可支持结论 |
|---|---|---|
| 聚焦Python合同 | 35 passed / 1.684秒 | 当前prewarm artifact validator与managed Cargo path合同内部自洽；不覆盖shader key碰撞和并发pair发布 |
| CI cache扫描 | 3 workflows共7处`Swatinem/rust-cache@v2` | CI已有通用Cargo缓存，不能记录为完全冷构建；仍无Zircon action/record语义和共享DDC |
| sccache activation扫描 | 只有`dev-fast-build.ps1`显式设置`RUSTC_WRAPPER=sccache` | 仅设置`SCCACHE_DIR`的feature checker、build helper与Coordinator不能证明compiler cache已启用 |
| remote execution owner搜索 | 非参考源码未发现REAPI/UBA/FASTBuild/Incredibuild或等价worker owner | 当前没有产品级remote executor；Coordinator只是可演化基础 |
| disk GC扫描 | 只发现内存chunk LRU、按UI asset递归evict及错误项删除 | asset chunk、shader、UI与IBL磁盘缓存没有全局quota、mark/sweep、age/lease或retention owner |
| tracked cache扫描 | contact-shadow plugin下2个`.zircon-cache`shader文件，共8,114 bytes | 历史运行缓存已进入Git；当前`.gitignore`虽忽略该目录，但不会自动移除已跟踪文件 |

本轮没有运行完整Cargo workspace、GPU shader compilation、跨进程并发writer、cache corruption注入、网络cache或remote worker。35项测试通过不改变两个从source control flow直接成立的P0，也不证明跨机器可复用。

### 2.3 正向基线

- asset artifact的immutable chunk先于manifest发布，chunk按BLAKE3校验并有严格raw/manifest大小上限；这是统一CAS最成熟的起点。
- ProjectManager只有在preview ready、source digest、config hash和importer contract全部匹配且每个artifact可读时才恢复；本地currentness逻辑应迁移进Action Definition，而不是删除。
- full project generation已有多文件transaction与故障注入测试；统一DDC应复用其journal/commit经验。
- shader prewarm manifest有source ID、source table、执行内存预算、WGSL validation及可选WGPU module/pipeline validation；工具侧会验证artifact/meta pair和requested dimension覆盖。
- UI compile key覆盖root document、widget/style imports、declared revisions、descriptor registry、component contract及resource dependency revision；字段集合可作为typed input declaration的种子。
- IBL cache用BLAKE3覆盖source key/hash、输入/输出face和mip布局、required contents，并调用runtime通用atomic writer。
- fast-build入口会恢复环境lease、约束Windows物理build root、共享Cargo home/sccache目录，并在sccache存在时真正启用wrapper。
- CI已在Linux、Windows和profile contract使用Cargo cache；重构应补充key/receipt/metrics，而不是删除现有加速。

### 2.4 参考边界

- Unreal DDC把key表示为受约束bucket与`FIoHash`，policy显式区分Query/Store Local/Remote、SkipData/Meta、PartialRecord、KeepAlive与NonDeterministic。Zircon需要同等级的policy语义，但不复制其类型命名。
- Unreal Build Definition把build function、constants、input builds、bulk data、files和raw hashes构造成immutable definition；Build Worker又将host platform、build-system version、function versions、files、executables和environment纳入worker identity。这说明action identity必须同时绑定“做什么、输入什么、由谁执行”。
- Bevy AssetProcessor拥有source/processed reader-writer、processing state、metadata、watcher reprocessing和file transaction locks。它证明processed asset pipeline应有单一owner与事务，但不是共享remote DDC的替代品。
- Godot import链记录source file、importer/version、params、destination files及source/destination MD5，并在EditorFileSystem中处理reimport顺序与threaded importer。它是本地导入currentness参考，不被夸大成remote CAS。
- Fyrox ResourceManager集中管理async load、loader、registry、watcher与lifetime；本轮选定源码没有与Unreal DDC等价的共享派生数据控制面，因此不从其runtime resource cache推导远程缓存能力。
- Unity Graphics仓内是渲染package与测试源码，不拥有完整Unity Editor Asset Database/Accelerator实现；本报告不据此虚构DDC比较。

## 3. 当前P0

### TOOL-DDC-P0-001 · Shader prewarm用不足的disk hash合并不同source并虚报written coverage

`ShaderVariantCacheDiskKey::from_variant_key()`只散列`ShaderVariantKey::canonical_string()`与include content hashes；WGSL正文、source ID、template revision、Naga version和WGPU version仅写入metadata，不参与路径。`prewarm_shader_variants_to_disk_inner()`又以`written_disk_hashes`按该hash去重：第二个不同source若共享variant/include组合，不会调用`cache.write()`，却仍调用`record_written_cache_entry()`并携带第二个source信息。最终report可以声称两个source均已写入，目录中只有第一个WGSL/meta pair。staging validator只校验report、文件布局和metadata字段自洽，无法证明每个reported source对应磁盘字节。

必须把disk key硬切为`ShaderBuildActionDigest`，至少覆盖canonical variant、完整assembled WGSL digest、include graph digest、template revision、shader compiler/validator executable digest、Naga/WGPU版本、target/backend/capability profile与schema。去重必须比较完整action digest；同一variant映射多个source时要么明确形成多个record，要么验证output digest完全相等后才合并。加入“相同variant/include、不同WGSL/version”的真实worker→report→artifact validator回归，并拒绝旧schema staged cache进入shipping bundle。

### TOOL-DDC-P0-002 · Shader WGSL/meta双文件发布可在并发与rename竞争下返回伪成功

shader私有`atomic_write()`为每个目标生成固定扩展名`.tmp`，没有`create_new`、unique writer ID或lease。两个writer可同时写同一temp；每个record又先独立发布WGSL、再独立发布meta，没有generation marker或journal。如果`rename`失败而目标已存在，函数删除temp并无条件返回`Ok(())`，既不验证现有目标hash，也不确认它属于同一writer/action。并发、crash或第二步失败可留下旧WGSL+新meta、孤儿文件或他人内容，caller仍获得成功entry并写入prewarm report。

必须停止使用该私有writer，改为唯一staging目录中的单record manifest + content-addressed value，或复用runtime durable transaction一次切换generation。publication成功条件必须是已发布record的key、每个value digest和generation全部回读匹配；existing target只能在digest相同后视为幂等成功。加入双writer barrier、第一/第二value crash、rename collision、orphan、mixed generation与restart recovery测试。

## 4. 统一Action与DDC控制面差距

### TOOL-DDC-P1-001 · 没有单一`DerivedDataService` owner

五套cache各自选择root、key、serialization、error和eviction。建立tooling/runtime共享的窄接口，统一request、record、value、policy、backend、observation与cancellation；具体asset/shader/UI/IBL builder仍由所属域拥有。

### TOOL-DDC-P1-002 · 没有稳定Build Function身份

现有版本多为手写整数或自由字符串。为每类派生构建注册stable function ID、semantic version、implementation/tool digest、schema与determinism policy；函数变更必须可审计失效旧key。

### TOOL-DDC-P1-003 · 没有canonical Action Definition

输入散落在meta、request、环境变量与代码常量。定义有序typed definition，覆盖constants、source blobs、dependency records、toolchain、target/platform capability、feature set与declared outputs，并采用canonical binary encoding。

### TOOL-DDC-P1-004 · `LibraryCacheKey`是production-dead且使用`DefaultHasher`

该类型只在自身测试出现，production importer/store没有caller；64位`DefaultHasher`也不是持久、跨版本、跨语言协议。删除虚假的authority，迁移为BLAKE3/SHA-256 canonical Action Digest，并由实际import路径消费。

### TOOL-DDC-P1-005 · Record与Value没有统一schema

asset用manifest+chunks，shader/UI用双文件，IBL用blob。定义record metadata与一个或多个content-addressed value，value需包含raw/compressed digest、size、codec与type；record key不应等于任一output content hash。

### TOOL-DDC-P1-006 · 没有local/remote读写policy

当前fallback root只存在于shader读取。引入QueryLocal/Remote、StoreLocal/Remote、ReadOnly、Bypass、RequireFresh、Partial与KeepAlive等显式policy；shipping cook、interactive editor和CI不能共享隐式默认。

### TOOL-DDC-P1-007 · 没有backend hierarchy与健康状态

建立memory、local filesystem、team/shared、read-only packaged seed层，记录逐层latency、hit/miss/error、bytes与fallback reason。remote故障应降级为local build，除非调用者明确要求remote evidence。

### TOOL-DDC-P1-008 · 没有request owner、priority与cancel传播

asset import、shader prewarm和UI compile各自同步或批处理。统一异步request owner，支持priority、deadline、cancel、batch、coalescing与completion exactly-once，避免Editor关闭后后台写入失效项目。

### TOOL-DDC-P1-009 · 没有negative cache与失败分类

同一不可支持输入可被重复昂贵构建。只对确定性的typed unsupported/invalid input做短期negative record；环境、OOM、设备丢失、网络和取消不得污染共享cache。

### TOOL-DDC-P1-010 · 没有determinism verification

相同action重复构建后的不同output不会被发现。加入抽样双构建、output digest comparison、nondeterministic quarantine与owner告警；进入cook/package的value禁止标记为任意non-deterministic。

### TOOL-DDC-P1-011 · 没有统一namespace和Build Set边界

project path、schema目录与自由cache root承担了隐式namespace。key namespace应包含engine distribution/Build Set compatibility、project tenant和function bucket；允许跨project共享的value必须显式声明无project secret与路径依赖。

### TOOL-DDC-P1-012 · 没有cache receipt可证明命中与currentness

每次get/build/put应输出可选bounded receipt：action digest、backend、record/value digests、bytes、timing、build reason、tool/worker identity与validation result。性能报告、cook和release只消费schema验证后的receipt。

## 5. Asset Artifact与CAS差距

### TOOL-DDC-P1-013 · Asset manifest不绑定构建定义

manifest只有schema、kind、revision、content hash、sizes和chunks。source digest、importer ID/version、config hash、dependency action digests及tool identity只在旁边meta/registry。共享record必须把这些currentness字段绑定到action key和manifest。

### TOOL-DDC-P1-014 · Artifact locator按AssetId覆盖，不是按Action Digest不可变存储

同一`kind/AssetId.zasset`反复替换manifest，适合项目当前视图，不适合作为历史共享record。保留project locator作为可事务更新的pointer，底层record/value改为immutable CAS。

### TOOL-DDC-P1-015 · Restore先信任sidecar currentness再只按locator读取

当前本地流程因先比较meta而可接受，但`ArtifactStore::read()`本身没有expected action参数。API应要求expected record key/build definition，避免未来caller绕过sidecar后把“可解码”误当“当前”。

### TOOL-DDC-P1-016 · Chunk只在单project root内复用

内容寻址chunk是共享CAS种子，但root固定在project artifact目录。增加受控global/team backend和project pointer，不要用目录复制模拟共享；同digest上传/下载必须幂等验证。

### TOOL-DDC-P1-017 · 磁盘chunk没有mark/sweep GC

manifest更新后旧chunk可永久孤立；现有eviction只针对内存residency。实现generation snapshot、reachable-set mark、grace lease、trash/quarantine与bounded sweep，避免与活writer竞态删除。

### TOOL-DDC-P1-018 · 没有磁盘容量和保留策略

2 GiB单artifact上限不等于cache总预算。按backend/project/function配置soft/hard quota、age、LRU/frequency、pinned build与emergency pressure策略，并输出可解释eviction receipt。

### TOOL-DDC-P1-019 · Chunk codec与schema不能独立演进

manifest只有整体schema，chunk descriptor没有codec/version/dictionary identity。统一Value Descriptor需携带codec、level/dictionary digest、raw/compressed digest与size，支持不破坏action identity的存储重编码。

### TOOL-DDC-P1-020 · 没有partial record恢复policy

缺一个chunk就使整个asset读取失败。允许backend间按value/chunk补齐，只有全部digest验证后才promotion到local；调用者可选择partial metadata query但不能得到伪完整asset。

### TOOL-DDC-P1-021 · 没有共享cache poisoning防护

本地BLAKE3证明字节一致，不证明producer可信。remote record必须绑定namespace、worker/build identity和authorization；下载后验证record key、value digest、schema、budget及必要签名，异常backend进入circuit-breaker/quarantine。

### TOOL-DDC-P1-022 · Asset cache观测只覆盖内存chunk residency

当前能看到resident bytes/hit/eviction，却没有disk/remote query latency、compressed bytes、dedup、rebuild reason或import critical path。统一metrics并限制高基数label，性能目标以warm/cold和backend分层衡量。

## 6. Shader Cache与Prewarm其余差距

### TOOL-DDC-P1-023 · Metadata版本字段不参与lookup validation

read path只比较schema、hash和canonical string；template/Naga/WGPU值即使与当前consumer不同仍返回Hit。修复P0 key后，decoder仍应逐字段核对expected definition并报告typed miss reason。

### TOOL-DDC-P1-024 · Runtime依赖WGSL字符串比较兜底失效

live pipeline路径可用cached WGSL与current WGSL比较避免部分陈旧命中，但这发生在读取/解压后，也不覆盖compiler/backend capability。把完整identity前移到query key，字符串比较仅作为debug invariant。

### TOOL-DDC-P1-025 · Fallback root错误会阻断后续fallback

任一fallback返回corrupt/error便立即结束，且只清理primary错误项。定义backend error policy：corrupt entry隔离并继续更低层/重建，permission/auth等系统错误按policy决定fail closed。

### TOOL-DDC-P1-026 · 缺半个pair时静默Miss但遗留孤儿

`read_entry_at()`看到任一文件不存在就返回Miss，不清理另一文件。改成单record manifest后消除pair；迁移期间启动repair扫描应识别orphan并隔离。

### TOOL-DDC-P1-027 · Shader cache没有磁盘GC或配额

schema/version/source变化不断累积目录。按action/value CAS使用统一quota/retention，并允许shipping seed pin住受支持platform集合。

### TOOL-DDC-P1-028 · `ZR_SHADER_CACHE_DIR`绕过中央root分配

环境变量可指向任意绝对路径或project relative目录，没有namespace、lease、trust与capability描述。统一由DerivedDataService解析backend配置，保留变量只作为显式debug override并写入receipt。

### TOOL-DDC-P1-029 · Prewarm report把dedupe视为write而不区分状态

即使key修正，report也应区分built、local hit、remote hit、validated existing、uploaded与deduped same output。coverage验收必须知道产物来源和实际被验证的record。

### TOOL-DDC-P1-030 · Cache artifact validator没有重算action identity

工具检查hash格式、pair、metadata和dimension，但没有从source/tool inputs重建完整key。validator应读取冻结source manifest和tool receipt，重算action与output digest，再发布shipping cache catalog。

### TOOL-DDC-P1-031 · 已跟踪`.zircon-cache`文件混淆fixture与运行缓存

contact-shadow runtime目录已有2个cache产物被Git跟踪，尽管`.gitignore`现在忽略该root。若是测试fixture，应迁到明确`fixtures/`并附source/action manifest；若是运行产物，应从版本控制移除并由prewarm可重复生成。

### TOOL-DDC-P1-032 · Staged shader seed没有目标设备兼容策略

WGSL可能跨设备，但pipeline/binary cache通常不跨driver/backend。catalog必须声明source-level、IR-level或device-binary级别，以及adapter vendor/device、driver、backend、feature/limit和compiler compatibility；不得混用。

## 7. UI与IBL Persistent Cache差距

### TOOL-DDC-P1-033 · UI persistent store直接`fs::write`覆盖

artifact和payload都不是unique staged/atomic publication，crash可留下partial record并退化为永久silent miss。复用统一record/value transaction，不允许cache专用代码重新实现弱writer。

### TOOL-DDC-P1-034 · UI artifact与payload不是同一generation

两个文件独立存储且load API独立，可能得到不同compile generation。把compiled package、runtime payload和diagnostics作为一个record的named values，record manifest一次提交。

### TOOL-DDC-P1-035 · UI record没有独立content digest

反序列化与key匹配能发现部分损坏，却不能验证payload字节身份或跨backend传输。每个value必须有强digest和size，并在decode前执行budget与hash校验。

### TOOL-DDC-P1-036 · UI fingerprint仍是64位碰撞域

输入字段覆盖较完整，但`UiAssetFingerprint.value`最终只有64位。共享DDC使用至少256位action digest；64位可继续作为内存map快速索引，但不能成为持久唯一身份。

### TOOL-DDC-P1-037 · UI key缺Build Set与target capability

schema/compiler整数不足以表达plugin/type registry executable、target architecture、locale/font backend或feature capability。把这些作为typed input或显式证明与输出无关。

### TOOL-DDC-P1-038 · UI按asset递归evict是O(total cache)

evict会遍历并反序列化所有`.zuiart/.zuicache`来比对asset ID，损坏文件被静默跳过。统一index按namespace/function/asset tag维护record引用；GC不依赖扫描所有payload。

### TOOL-DDC-P1-039 · UI cache没有总容量、age或访问治理

只有显式remove/evict asset。接入统一quota、last-access journal、pinned current generation和background compact，且不得在Editor交互主线程递归扫描。

### TOOL-DDC-P1-040 · IBL rejected entry每次启动重复解码

decode/current request拒绝后只返回`Rejected`，不隔离、删除或记录一次性诊断。将坏record移入quarantine并记录reason/digest；后续query直接miss，避免重复I/O和日志噪声。

### TOOL-DDC-P1-041 · IBL GPU output缺少设备与算法实现身份

request key有手写algorithm version和source/layout，但没有shader/action/tool Build Set、adapter/backend/driver或determinism声明。先区分portable source-derived输出与device-specific GPU输出，再决定哪些层允许remote store。

## 8. Cargo、CI与Build Cache差距

### TOOL-DDC-P1-042 · 多个managed环境只设置`SCCACHE_DIR`而未启用wrapper

feature checker、Python build environment和Coordinator创建sccache目录，但不设置`RUSTC_WRAPPER`。它们还继承ambient environment，使同一命令在不同shell中可能启用或不启用sccache。统一environment builder应显式声明compiler cache mode，并在receipt记录wrapper path/version/config。

### TOOL-DDC-P1-043 · sccache工具安装未固定

`-InstallSccache`执行裸`cargo install sccache`，没有版本、checksum或受控tool bundle。固定版本与下载/构建digest，将其纳入Build Set和cache namespace；离线环境使用预置工具包。

### TOOL-DDC-P1-044 · 没有sccache命中与错误准入

只有人工`zr-sccache-status`，build/cook/CI receipt不采集compile requests、hits、misses、timeouts、cache errors或saved time。每次受管build前后采集结构化delta，并为持续零命中/错误设告警而非盲目宣称加速。

### TOOL-DDC-P1-045 · 没有团队级compiler cache backend治理

当前sccache目录只在单机shared root。定义local disk和可选remote backend、TLS/auth、namespace、quota、retention、credential injection与offline fallback；secret不得进入action key、log或cache value。

### TOOL-DDC-P1-046 · CI `rust-cache`没有Zircon action receipt

7处action能减少下载/编译，但workflow没有记录resolved cache key、hit、restored paths、toolchain/feature identity或saved time，也不能被local/Coordinator查询。保留action并增加Zircon side receipt和统一metrics；不要把第三方action内部key当产品DDC协议。

### TOOL-DDC-P1-047 · CI与本地cache namespace不统一

CI matrix、Windows固定target、本地profile目录和Coordinator job目录各自切分输出。定义canonical Cargo Action Identity覆盖workspace/lock/toolchain/target/profile/features/rustflags/build script env与source tree；不同执行面只在证明等价时复用。

### TOOL-DDC-P1-048 · Cargo dependency获取没有镜像、vendor或离线合同

private/shared `CARGO_HOME`会重新维护registry/index/git checkout，当前没有企业mirror、vendor snapshot、checksum inventory或offline build gate。建立依赖source policy与immutable mirror，区分dependency cache和compiler output cache。

## 9. Remote Execution与安全差距

### TOOL-DDC-P1-049 · Coordinator job不是canonical remote action

现有ticket/argv/working directory/target root不能证明完整input tree、toolchain、environment、platform和outputs。新增typed `BuildAction`，拒绝未声明输入/输出及自由继承环境；本地执行也先走同一协议。

### TOOL-DDC-P1-050 · 没有worker registry与capability matching

定义worker immutable identity、OS/arch/toolchain/GPU/SDK capabilities、resource limits和health lease。scheduler只能把action派给满足约束的worker，结果receipt绑定worker incarnation和execution attempt。

### TOOL-DDC-P1-051 · 没有hermetic sandbox、input/output CAS与网络策略

远程执行必须从input CAS materialize只读输入，在隔离workspace运行，只允许声明的输出进入output CAS；默认禁网，显式声明的网络action不得写共享deterministic cache。进程树、deadline、cancel和resource accounting复用Supervisor而非裸Popen。

### TOOL-DDC-P1-052 · 认证缺口会把remote cache变成代码执行与投毒面

报告06已确认当前控制入口token-free、maintainer映射和任意validation argv。remote milestone前必须先完成mutual authentication、RBAC、mTLS/worker attestation、tenant namespace、signed tool bundle、secret broker、audit log和cache poisoning response；否则remote功能必须保持禁用。

## 10. P2治理与体验差距

### TOOL-DDC-P2-001 · Cache术语没有统一词汇表

明确区分runtime hot cache、persistent local cache、DDC record/value、compiler cache、CI cache、shipping seed与artifact evidence，避免所有目录都叫cache。

### TOOL-DDC-P2-002 · 没有`zircon cache status`入口

提供只读命令显示backend、容量、hit rate、health、namespace和最近错误；默认不递归计算昂贵精确值。

### TOOL-DDC-P2-003 · 没有受控warm/prune/verify命令

提供按function/project/platform计划化操作，先dry-run并输出receipt；禁止要求开发者手工删除整个`.zircon`或共享target root。

### TOOL-DDC-P2-004 · Miss reason不可解释

统一输出not found、policy bypass、schema/function/tool/input/platform change、corrupt、unauthorized、quota与backend unavailable，避免只显示“rebuilding”。

### TOOL-DDC-P2-005 · Cache指标没有成本单位

除hit ratio外记录bytes avoided、build time avoided、lookup overhead、upload/download和eviction churn；高命中但净变慢不得算优化成功。

### TOOL-DDC-P2-006 · 没有跨版本迁移与sunset清单

每个record schema定义reader support window、writer version、promotion policy和retirement日期；旧root由工具识别并安全清理。

### TOOL-DDC-P2-007 · Cache fixture没有专门目录与manifest

测试输入使用`tests/fixtures/cache/<schema>`并附source/action/output digest；禁止从真实运行目录直接提交二进制pair。

### TOOL-DDC-P2-008 · 没有管理员容量报表

按project/function/backend/age输出top consumers、dedup ratio和unreachable bytes，支持JSON与人类摘要，但避免泄露源路径或tenant内容。

### TOOL-DDC-P2-009 · 没有隐私与内容分类标签

record声明public/project-confidential/secret-derived/device-private等级；secret-derived默认禁止remote store，日志只保留digest和stable logical label。

### TOOL-DDC-P2-010 · 文档没有冷/暖构建SLO

为clean local、warm local、warm team、offline fallback和remote action定义p50/p95、correctness与availability门，才能量化迭代性能是否接近或超过目标引擎。

## 11. 目标架构

```text
Domain Builder
  -> BuildFunctionId + CanonicalActionDefinition
  -> ActionDigest
  -> DerivedDataService(RequestOwner, Policy)
       -> Memory metadata/index
       -> Local filesystem record/value CAS
       -> Shared team record/value CAS
       -> Read-only packaged seed
  -> Hit: verify record key + value digests + budgets
  -> Miss: LocalActionExecutor or RemoteActionExecutor
       -> immutable WorkerIdentity
       -> input CAS + sandbox + declared outputs
       -> determinism/output validation
       -> transactional record publication
  -> CacheReceipt / BuildReceipt / metrics
```

核心边界如下：

1. `ActionDigest`标识构建定义，不等于output digest；同一action的多个named output组成一个record。
2. CAS value只证明字节身份，record还必须证明这些value属于哪个action、由哪个受信worker生成、是否通过验证。
3. project metadata保存当前source到action的事务pointer；DDC保存可丢弃、可重建的immutable派生数据。
4. runtime memory residency不迁入tooling service，只通过相同record/value读取协议消费磁盘或远端数据。
5. remote execution与local executor共享action schema；远程不是另一套自由argv API。
6. cache永远不能成为authoritative source，删除全部cache后应能从source、toolchain与Build Set重建。

## 12. 分层重构路线

### M0 · 先修shader cache真实性

- 硬切`ShaderBuildActionDigest`，把WGSL/tool/template/platform输入纳入key；
- 用单record多value事务替换固定temp双文件；
- 增加碰撞、并发、crash、orphan、mixed generation和旧schema拒绝测试；
- 将已跟踪`.zircon-cache`移动为声明式fixture或移除。

### M1 · 建立Action/Record/Value协议

- 定义canonical encoder、Build Function registry、Action Digest、Record Manifest、Value Descriptor与typed miss/error；
- 把asset、shader、UI、IBL现有identity映射到definition，先只使用local backend；
- 所有publication复用唯一transaction primitive，并输出receipt。

### M2 · 收敛Local DDC与GC

- 建立统一root allocation、namespace、index、quota、mark/sweep、quarantine与repair；
- 迁移asset chunk为共享value CAS，project locator保留为current pointer；
- 加入cache status/warm/verify/prune命令和metrics。

### M3 · 收敛Cargo与CI Build Cache

- managed environment显式启用/禁用sccache，固定tool版本并采集stats；
- 定义Cargo Action Identity与本地/CI cache receipt；
- 建立dependency mirror/offline合同，保留`rust-cache`作为CI实现层而非authority。

### M4 · 团队共享DDC

- 实现authenticated read-through/write-back backend、tenant namespace、TLS、quota和health fallback；
- 上传前执行action/output/determinism验证，下载后执行digest/schema/budget验证；
- 先开放portable asset/UI/shader source-level record，再评估GPU/device-specific数据。

### M5 · 本地Hermetic Action Executor

- Coordinator消费typed action而非任意argv，materialize input CAS、隔离workspace、禁用未声明网络并只收集declared outputs；
- 绑定Supervisor的process tree、deadline、cancel、resource accounting和execution receipt；
- 本地cache hit replay与实际执行必须产生可比较结果。

### M6 · Remote Execution与规模化验收

- 完成认证/RBAC/worker attestation后再启用remote worker registry、scheduler与output CAS；
- 建立worker capability、autoscaling/backpressure、retry/idempotency和poisoning response；
- 以真实工程cold/warm/edit-build迭代、cook和CI wall time对比Unreal基线，不以单次命中率代替产品性能。

## 13. 验收门

1. 相同canonical action在Windows/Linux支持范围内产生稳定digest；无序map、绝对workspace path和locale不会漂移。
2. 任一source、importer/tool version、config、dependency、target capability或Build Set变化都会产生解释明确的miss。
3. 相同variant/include但不同WGSL/template/compiler版本绝不共享shader record。
4. 双writer并发同key时只有digest相同的幂等成功；不同output触发non-determinism/quarantine。
5. 在每个value publication阶段注入crash，restart后只能看到旧完整record或新完整record。
6. record存在但任一value缺失/损坏时可从更低层补齐或重建，绝不返回partial asset为Hit。
7. 删除项目cache、local DDC与CI cache后能从source clean rebuild，不需要tracked运行cache。
8. asset project meta只在全部artifact record durable后切换generation，失败保持旧generation可读。
9. mark/sweep不会删除被active lease、current project generation、pinned build或in-flight upload引用的value。
10. soft/hard quota、disk pressure与manual prune均产生明确eviction receipt，UI主线程不做全树扫描。
11. remote unauthorized、corrupt、timeout和unavailable均按policy安全降级；RequireRemote模式则明确失败。
12. remote cache返回的record key、value digest、schema、size和namespace全部验证后才进入local promotion。
13. deterministic action抽样双构建output一致；不一致不会上传或进入shipping cook。
14. sccache实际wrapper path/version/config进入Build Receipt；只创建`SCCACHE_DIR`不算启用。
15. managed build输出sccache request/hit/miss/error/bytes/time delta；持续零命中触发诊断。
16. CI `rust-cache` hit/miss和resolved identity可被Zircon receipt消费，但第三方action更新不改变DDC schema。
17. dependency mirror支持checksum验证和离线locked build；credential不进入key、日志或artifact。
18. remote worker只接收typed action，不能通过validation argv执行未声明任意命令。
19. worker失联、cancel或deadline会终止完整进程树，partial outputs永不进入CAS current record。
20. cache status报告backend health、容量、hit/miss、latency和top consumer，且不泄露secret/path内容。
21. cache fixture全部位于声明式fixture目录并能从manifest重算digest；运行目录无tracked产物。
22. cold local、warm local、warm team、offline fallback和remote execution均有p50/p95基线与回归budget。
23. cook/release只接受绑定Build Set、action、worker和output digest的validated receipt。
24. source recheck、schema migration和旧cache sunset都有自动测试，旧reader支持窗口结束后明确拒绝而非误命中。

## 14. Ownership与实施约束

| Owner | 责任 | 不应拥有 |
|---|---|---|
| `DerivedDataService` | action/record/value、policy、backend hierarchy、CAS、GC、receipt | asset/shader/UI具体构建算法 |
| Domain builder | typed inputs、Build Function version、output validation、portable/device policy | backend路径、远端认证、全局quota |
| Build/Coordinator | Build Set、Cargo action、executor、sandbox、worker调度 | 自行发明cache manifest或绕过DDC写共享目录 |
| Security/Operations | identity、RBAC、TLS、secret、tenant、quota、retention、incident response | 修改action语义来提高命中率 |
| CI/Performance | cache receipt消费、cold/warm SLO、regression gate | 把缓存命中直接等价为correctness或性能完成 |

实施时必须遵守三条硬约束。第一，不兼容key/schema采用hard cutover，新旧writer不得长期双写；旧reader只在有明确sunset的迁移窗口存在。第二，先证明local action hermetic与deterministic，再开放remote store/execution。第三，cache failure默认不能破坏authoritative source；shipping/release的RequireValidated策略则必须fail closed并保留诊断artifact。

## 15. 与既有报告的边界

- Runtime 04继续拥有asset importer、registry、serialization、watch与project generation业务语义；本报告拥有跨构建/跨机器action与DDC promotion。
- Runtime 09C继续拥有shader compile/pipeline/PSO运行期正确性；本报告拥有shader派生数据key、持久发布和共享复用。
- Runtime 11A继续拥有UI compile/runtime消费；本报告只拥有persistent compiled record的存储治理。
- Tooling 03继续拥有Cook/Pack/Platform Bundle/Release流水线；本报告提供其应消费的DDC与Build Receipt。
- Tooling 05继续拥有generated source action graph；本报告提供可复用action/cache substrate。
- Tooling 06继续拥有Coordinator认证、lease、validation和process supervision；本报告只定义remote action/cache所需的增量边界，不重复其P0。

本报告不能作为“共享DDC或remote execution已实现”的证据。`implementation_status`保持`pending`，直到M0-M6对应验收门有current-source自动化证据。
