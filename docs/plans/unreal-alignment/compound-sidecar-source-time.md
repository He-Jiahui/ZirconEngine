# Runtime/Asset：复合资产的生成写回不应变成下一次源修改

**阶段：独立源码调查，可作 Draft；ZE 回归待实跑。** 固定 ZE 基线 [`e89349ca51b4e9d024c194cd3bdf645bd25b6c37`][base]；所引 Runtime/资源事务/Editor 目录路径与本地读到的跟踪内容无差异。一份独立 Windows 文件系统原语旁证不等于运行了 ZE 或 Unreal，也不归因 [#202][observer]。

## 1. 差异：内容可复用，复合描述符的物理时间却能使第二次扫描不再是空操作

单文件资产的源与 `.zmeta` 各有路径。复合资产以合法的 `unit = "compound"` `.zmeta` 作为逻辑目录的持久源：校验 scheme、无 label、路径与描述符匹配，随后 `source.path == source.meta_path`。[普通与复合发现][source-discovery]、[持久映射][source-binding]、[同一物理路径][source-alias]。`materialize_compound_source_snapshot` 捕获描述符字节并以其**物理 last-write time**作初值，再 `max` 已捕获成员及声明辅助源的时间；不是目录本身的 mtime，也不读磁盘文档内旧的 `source_mtime_unix_ms` 来初始化。[成员与辅助采样][source-snapshot]、[主文件打开][source-primary]、[毫秒量化][source-ms]。

同一 `ProjectManager` 可直接同步调用两次 [`scan_and_import`][scan]：`prepare_full_generation(None)`→真实文件提交→安装候选→确认 durable，不需 AssetWatcher、Editor 或重开项目。复合的两种 digest 已有意排除描述符原始字节；源 digest 按成员路径/字节计算，canonical build-input digest 还包含逻辑源身份，作者 `import_settings` 则经 recipe/action key 单独输入。[全扫的 digest/recipe][full-digest]、[canonical digest][canonical]、[recipe 绑定][recipe]。因此这里的「内容不变」指同一作者元数据和成员输入；**不**假设待比较的生成 `.zmeta` 原始字节恒等。

## 2. 影响与反证：能证伪的是某次无变更全扫，不是无限循环或 #202 的七次变化

设 `D0` 为首扫进场时读到的描述符毫秒值，`M` 为两扫间未变的成功捕获成员/辅助源的最大毫秒值（无则 `0`），`P1 = max(D0,M)` 为首扫持久化值，`D1` 为**首扫自然提交后**在目标路径重新读到的物理毫秒值。若两次扫描都正常 durable、同一根记录和其他输入不变，则第二扫重新取 `P2 = max(D1,M)`。**当且仅当 `P2 != P1`，mtime 足以单独让第二扫改变生成元数据与该目录代**；不需要成员/作者变化。`D1 > max(D0,M)` 足够但不必要：时钟回拨或原描述符是未来时间时也可能向下变化。若 `D1 == D0`（平台保留或正好落同一毫秒桶），或 `M` 同时压住两者，则没有此项漂移；其他作者/成员修改仍可以合法发布。`modified()`→Unix epoch→`as_millis()` 向下截整，无法取值、早于 epoch 或无法转成 `u64` 时该快照返回 0；文件系统粒度可能更粗。读取过程中防变化比较使用原 `SystemTime`，不改变目录输入按毫秒量化的事实。[量化与重验][source-ms]

有差异时，不必重导入或换 GUID：完整 [`AssetMetaDocument`][meta-eq] 与 [`ProjectCatalogInputRecord`][catalog-record]（其自身还另存同一个 mtime）都参与相等检查；同 ID、同 locator 则是 `Modified`，不是 Added/Removed/rename。[元数据比较][changed-doc]、[代发布条件][catalog]、[delta 分类][catalog-delta]。有代变化和一次多余 `.zmeta` 替换的风险，不等于资源变成 Error、artifact 损坏或实际 UI 刷新；全扫即使复用目录代仍可单独持久化 registry，不要求全盘零写入。[写回清单][full-write]。多轮之间每次须重新跨过聚合阈值；后两次提交可在同一毫秒桶而自然停住。序号分配也不应当作某个资产的精确次数。

[#202 的冻结记录][observer-evidence] 实测的是同 activation 下输入序号 12→19、外取旧 token 变为 Superseded、watch reconcile/commit 差值 7、注册 Editor 未读到首代；未录得逐次变动的行/字段、实际 native batch/path、全扫与 targeted 选择或内部 refresh 拒绝分支。显式扫描中的此机制与之相关，**并不证明 #202 命中此机制，也不解释其中任一次或全部七次**；正常 watch URI 映射还主动排除侧车与已识别事务临时路径[见过滤][watch-ignore]。不可据此禁用 watcher、修改 token fence 或把 `.zmeta` 本身宣称为已观测到的自激事件。

## 3. 根因与 UE 对照：写回路径与源时间被同一物理对象串联

- **ZE 恢复、重建、失败。** `restore_imported_artifact` 以 Ready、digest、config/action key、importer 契约和可读 artifacts 判断，不以 mtime 拦截 restore；恢复成功仍设置 `meta.source_mtime_unix_ms` 并将其投影到 catalog。首次正常成功和可持续扫描的导入失败也写同一字段。[恢复闸门及更新][full-restore]、[恢复路径投影][full-restored-publish]、[成功][full-ready]、[失败][full-error]。实存但无成员的合法复合目录没有最大值缓冲；内建 Shader 没 `.zshader` 会产生 Error/无制品，故不满足主见证的 Ready 预设，但只要发布 Error 文档也可能自写回漂移。成员捕获中途失败保留已接纳的主文件和其时间，不发布部分成员；在成员已捕获之后若声明辅助捕获失败，留下的则是已完成的主+成员聚合。两者跳过 restore、可走 Error 提交。相反，发现阶段的致命错误（如配对超预算描述符）或主快照完全未接纳会在准备阶段中止；被忽略的非复合/无效描述符没有此 compound 写回。[发现入口][source-discovery]、[失败前缀][full-snapshot-start]、[无 `.zshader` 的处理][shader-empty]。故不能以空目录、无导入器或读取失败冒充主见证成功。
- **真正的 DurableTransaction。** 不是假设 `AssetMetaDocument::save` 直接覆写：全对象不同才序列化成 `PreparedFileWrite`；核对元数据前置条件后通过项目桥提交。`stage_file` 在同目录事务兄弟位置 `create_new`、写/flush/sync 后关句柄，旧目标另备份字节；没有从目标搬 last-write time，也没有在发布后设回旧值。[全扫提交][full-commit]、[项目桥][project-commit]、[暂存][stage]、[备份与同步][stage-copy]、[事务分支与发布原语][tx-commit] / [publish]。真实 Windows 现存目标是 `ReplaceFileW(target, staging, NULL, 0, NULL, NULL)`，再同步目标；[Microsoft 契约][win] 明列保留的是原目标创建时间等属性，不把 **LastWriteTime** 列为保留项，结果文件 ID 属于 replacement。独立本机原语样本只在替换前设旧时间、没有 sleep：旧 target `946684800000`，fresh staging `1794999076829`，成功替换后 target `1794999076829`，独立 member 仍 `946684800000`（Unix ms）。原语记录 SHA-256 `5ab4a6ff047f794b4c91a2c10155f7f58bff58d3b7ad2152f12daf8bd296c251`。这只证本机所用原语/文件系统的 last-write 结果，不是 ZE 回归结果或所有 Windows 卷保证。非 Windows 路径用 `std::fs::rename(staging,target)`；在普通同卷 Unix 上是 staging inode 取代旧目标目录项（目录及 inode 状态时间不同于文件 mtime），不复制旧目标文件的 mtime。[平台分派][platform]、[POSIX rename][posix]。已有描述符是这里正常的 existing-target 路径，不是 create-only `MoveFileExW`。提交前拒绝/未到目标替换不能产生本项写回；替换之后失败再回滚不能据「字节恢复」断言物理 mtime 也恢复，live 回滚是备份字节→新 staging→替换[见恢复][rollback]，不纳入本案成功两扫的结果。
- **UE 同类责任，不同存储。** `UAssetImportData::SetSourceFiles/Update` 把外部源路径的时间和 MD5 记在 asset import data，`Serialize` 将这些信息存入对象归档的 editor-only 段，未把归档自身的时间作为该外部源时间。Editor `GenerateFileCacheConfig` 把目录缓存置于项目中间产物 `ReimportCache`；Auto Reimport 明确禁用 `Timestamp` 判定、启用 `FileHash`，对匹配的单源已导入资产可比较其 import-data hash。底层 `FFileCache::DiffDirtyFiles` 同时支持可配置的 hash 与时间比较，默认配置仍开 timestamp。可借鉴的是源观测和缓存产出职责分离，不是照搬「不观察 touch」、MD5 或 UE 的包格式。

**直接 UE 原测与派生边界（只读注册和断言，未执行；未复制源码或夹具）。**

| 原测 | 实际核对范围 | 对 ZE 的映射 |
|---|---|---|
| `Editor.Auto Reimport.Simple Modify` / `FAutoReimportSimpleModifyTest`，235–315 | 用真实 `DirectoryWatcher::FFileCache`，开启 hash 检测（默认 timestamp 仍开）。用不同文件内容覆盖同一相对路径，等到恰一项 `Modified`、校验路径，提交再读出相同 transaction hash。未断言旧/新 MD5 数值、纯 touch 抑制或写回后的第二次扫描。 | **直接的目录缓存/源修改原测**；用于成员内容改变、精确路径和已提交缓存读回的正控，不复制 latent retry。 |
| `Editor.Auto Reimport.Restart Detection` / `FAutoReimportRestartDetectionTest`，535–632 | 丢弃 cache 对象、停 watch 时改内容；以 `bDetectChangesSinceLastRun` 真/假复开分别期待一条 `Modified`/无待处理改动。原用例标 `Disabled`，仍靠 latent watcher 启动。 | **已有但禁用的磁盘缓存恢复对照**；不是 ZE artifact restore 或可声明运行的回归。 |
| `FDirectoryWatcherSimpleModifyTest`，123–159；`FMatchExtensionStringTest`，8–40 | 前者检查真 watcher 报 `Modified`（延时 latent），后者只断言 File Cache 扩展名匹配。 | **直接但邻近**，不测试源 digest/生成回写；不作为主例的 watcher oracle。 |
| `FInterchangeImportTest`，21–54、127–164；`UInterchangeImportTestStepReimport::StartStep/FinishStep`，51–81、105–154 | 已有 automation 注册，从 TestPlan 枚举步骤；重导入步骤匹配真实对象、可传 override pipeline、调用 Interchange 导入和计划断言。枚举在该树 Mac/Linux 跳过；未核某资产计划对 mtime 的特殊断言。 | **真实显式 reimport 测试框架的对照**；不说它有同一复合副产物或本案原测。 |

本地 UE 参考树声明 `6.0.0 / UE5 / CL 0`，无独立 UE Git SHA（向上解析是 ZE 仓库）。以下为精确 `Engine/` 相对路径、范围和原始 SHA-256，非 Git commit，也不发布 Epic 源码：

```text
3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6  Engine/Build/Build.version:1–10
897aea993ae695cdc4f05574344075bbeab632489e34009eacf2d522727fe005  Engine/Source/Runtime/Engine/Private/EditorFramework/AssetImportData.cpp:179–227,418–444
7883ae2eee8028f8b8d6f0ba2b25b8acfb41f55309ede5211acefed83906a57b  Engine/Source/Editor/UnrealEd/Private/AutoReimport/ContentDirectoryMonitor.cpp:36–90,126–147
c40ffd9fa75dc354b23122e71cc5f1f924014e466fddc6d8a5b900a39f29758f  Engine/Source/Developer/DirectoryWatcher/Private/FileCache.cpp:632–704
7cce13b3aac9b35e82da9819d672263a04a6a5defd7e1598ca194e6f70f535c4  Engine/Source/Developer/DirectoryWatcher/Public/FileCache.h:331–395
c059fd611676ae2ca09efaa6bdf37c5e23be09d0ec5969a1207e4d7ebacfa3f5  Engine/Plugins/Tests/EditorTests/Source/EditorTests/Private/UnrealEd/AutoReimportTests.cpp:42–54,235–315,535–632
158433492b2d8bb8102c69081515cdf889fe8f9c9933f6b972ce79df675328e4  Engine/Source/Developer/DirectoryWatcher/Private/DirectoryWatcherTests.cpp:90,123–159
c9a7e5db685aea573f99f2c5f7d84c8711c17e0282f05a29f8cb6e1e6d4b1f46  Engine/Source/Developer/DirectoryWatcher/Private/Tests/DirectoryWatchterRunTests.cpp:8–40
226ba896bdfda37392ef4c0c1ef9c1d48f6c928628b6f9fc574cc4461057e71a  Engine/Plugins/Tests/InterchangeTests/Source/InterchangeTests/Private/InterchangeImportTest.cpp:21–54,127–164
34183f777cc613979d612cd2bc2fc1c89770800fb390b94d7df70d79854352c3  Engine/Plugins/Tests/InterchangeTests/Source/InterchangeTests/Private/InterchangeImportTestStepReimport.cpp:51–81,105–154
```

## 4. Rust 做法：分离源时间与自己的写入，不删除真实的源观测

在 Runtime 的 source-snapshot/compound 生成边界修正时间的归属，保留已有 `AssetMetaDocument`、digest/action key、文件事务和不可变 catalog 代。先用主回归验证，候选是 compound 采用真实已接纳的成员/辅助时间，描述符的作者含义继续由已解析的元数据/recipe/身份承载。空包或部分失败需要显式无成员策略（如 `0` 表示未观测），不得回退到刚写回的描述符时间，不得把失败前已读的个别成员当成成功快照。普通单文件及其合法辅助源继续观察时间。若确需把**仅 touch 复合描述符**视为源变更，评估仅对引擎生成写回所用 staging 保留已验证的作者描述符时间，并处理回滚/并发契约；不能提交后补碰时间、全局改变 durable IO 的时间行为、屏蔽作者字段或仅从 catalog 相等比较里删 mtime（同一问题仍会重写磁盘）。[Runtime04 旧计划][plan] 明确规定普通 source touch 同 hash 仍能更新 mtime 与目录；当前[已有原生用例源码][ordinary]保护该行为。

## 5. 接口与 ECS：保持物理绑定和已发布代契约，独立于首读/通知实现

无须引入 UE `UObject`、全局 cache、ECS 行规则或改公开 ABI；保持 compound `.zmeta` 为精确持久化路径、原 UUID/标签/子资产与依赖、普通文件的 mtime 可观测性、catalog 元数据与原子提交。共享 [targeted 路径][targeted] 也读同一快照；其 metadata 写与 topology/echo 语义不同，修 shared source 时间须做对照，不把本案直接两扫泛化成 targeted/no-op 的全部修复。邻接计划是 [Runtime04 catalog inputs][plan]（不可变投影涵盖实际输入）、[targeted source/index][targeted-plan]（单源协调与拓扑）、[#134][reader]（Runtime 目录查询）、[#157][scene]（Scene 后缀）、[#159][io-names]/[#175][io-owner]（原子兄弟名与恢复 owner）、[#180][texture]（已安装纹理 identity/ABA）、[#201][status]（日志分类）与 [#202][observer]（注册 Editor 首代可见性）。2026-11-18 18:49 +08:00 的公开 68 个 open PR 完整文件集与本题相关生产/现有回归路径无交叠；all-state 关键词 `compound + mtime`、`source_mtime_unix_ms`、`sidecar + timestamp`、`复合 + 时间戳` 无结果。只表示本次去重范围，不声称未索引或工作区外私有事项也不存在。

## 6. ZE 后续：先落一个固定两扫的独立回归，再验证窄修

1. **主例，源码派生、待运行。** 在已有 [真实 compound Shader 测试模块/最小 include 包形状][compound-fixture] 新建独占工程，只放一个 `res://shaders/<package>`，自己的 Dirty `.zmeta`、一个 `.zshader` 和一个短 WGSL；无跨包 include/警告，显式 import path 不等于项目派生 path。用 `ProjectManager::open` 与默认内建 [.zmeta Shader importer][builtin-shader]，无 mock/插件夹具、GPU/Editor/AssetWatcher。初始文件写完后，仅在**第一次导入前**把描述符和**所有**成员 mtime 固定到 `UNIX_EPOCH + 946_684_800s`，如用现有 `FileTimes`，读回实际毫秒并关闭句柄。输入极少且无外部辅助，不复制模板、夹具或用户配置。首扫确认真正 durable、Dirty→Ready、成员/UUID/locator 正确且根和 WGSL 子资产实际可解码；核磁盘和目录行的 `P1` 都等于已读回的种子聚合毫秒。保存 S1 `Arc`/序号、磁盘 `.zmeta` 字节/解析值、目录行 mtime/content/action/GUID、实际制品，同时**只读**记录首扫自然写回的 `D1`。随后只再调用一次同一 manager 的 `scan_and_import()`；没有写入/touch、sleep、重试、轮询、重开、注入时钟或文件改写钩子。
2. **判据。** 先核两次的成员字节、作者字段、GUID/子资产身份、digest/action key、Ready 与解码后的 artifact 内容相同；比较时只能归一化生成 `source_mtime_unix_ms`，不能隐去其他差异。然后要求两扫之间磁盘与目录行中的 `source_mtime` 相同、元数据 bytes 相同、同一 catalog `Arc`/序号、delta unchanged。基线预期为 `D1 > P1` 时 `P2=D1`、meta 原字节不同、同 ID 恰一条 `Modified`、不复用代；不锁定序号差值或 `D2>D1`。`D1` 是**旧基线触及机制的诊断**，不是所有修法 GREEN 时也必须增加的断言。目标记录未 Ready/未读回/提交不 durable、其他字段也在抖动、或旧基线上 `max(D1,M)==P1` 均不能算本机制 RED；`max(D1,M)!=P1` 且同一合法根与所有其他前置成立而上述三者稳定，则实测**反证**此链。可用现有 [`SourceRevision::observe`][file-revision] 比较真实 `lib://` manifest 的字节/文件身份（本地文件系统支持文件 ID 时；不是对 `.zmeta` 造 mtime），辅证第二次走 cache restore 而不是重建出同字节：成功重导入会准备替换 artifact manifest，[restore][full-restore] 不会。相邻 [已存在的 restart restore][restart-test] / [action key restore][action-test] 是不同用例，不能代替该 Ready compound 见证。
3. **固定正控，分开运行，不污染主例。** 继续要求[普通 `.json` touch、同 digest、exact Modified][ordinary]。在另一复合项目，用真实描述符 writer 作根 `tags` 变化（缓存可恢复，catalog/entry tags 读回必须变化），另外改变正常 `import_settings` 值（同成员 digest，recipe/action key 必须变并保存作者值；未承诺内建 Shader 对任意设置改变 WGSL）。改变同路 WGSL 的有效常量，甚至控制两版成员 mtime 相同，要求 digest/action key 改变、同 ID 的 Shader 根与被修改的 WGSL 子制品实际解码为新数据。可分开的成员纯 touch 控制预设可读回且压住描述符的另一确定时间，证明不冻结整个 compound mtime。已有空/错误/成员失败的回归保留；若采用成员时间方案，再覆盖空目录 Error、重扫和有效成员补齐，保持不发布部分成员。正式验证只在已确认独立 Draft 中加入项目资产的窄测试；同用例先跑原生旧基线，再核生产差异和 paired GREEN，仍不代替 #202 的逐代/逐事件归因或产品验收。本次未运行 ZE、UE、Cargo 或项目 CLI，未改源码、Git 或远端。

[base]: https://github.com/He-Jiahui/ZirconEngine/commit/e89349ca51b4e9d024c194cd3bdf645bd25b6c37
[source-discovery]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs#L224-L324
[source-binding]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/reference_resolver.rs#L46-L74
[source-alias]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs#L325-L335
[source-snapshot]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs#L449-L565
[source-primary]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs#L569-L585
[source-ms]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/ingest/auxiliary_source.rs#L204-L255
[scan]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import.rs#L161-L170
[full-digest]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L166-L222
[canonical]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/build_identity.rs#L227-L249
[recipe]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/metadata.rs#L191-L214
[changed-doc]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/projected_inventory.rs#L118-L121
[meta-eq]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/meta.rs#L77-L115
[catalog-record]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/catalog_input_generation.rs#L18-L28
[catalog]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/catalog_input_generation.rs#L186-L213
[catalog-delta]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/catalog_input_generation.rs#L484-L506
[full-write]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L476-L497
[observer]: https://github.com/He-Jiahui/ZirconEngine/pull/202
[observer-evidence]: https://github.com/He-Jiahui/ZirconEngine/blob/1deb8f48423a8d764c920f0dd6e8b2568fd18100/docs/plans/unreal-alignment/editor-initial-catalog-visibility.md#L72-L88
[watch-ignore]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/watch/watched_asset_uri_for_path.rs#L9-L26
[full-restore]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L552-L632
[full-restored-publish]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L227-L302
[full-ready]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L657-L702
[full-error]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L748-L784
[full-snapshot-start]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L156-L169
[shader-empty]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/ingest/import_shader_package.rs#L435-L463
[full-commit]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs#L53-L67
[project-commit]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/durable_transaction.rs#L50-L76
[stage]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/transaction/stage.rs#L32-L101
[stage-copy]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/transaction/stage.rs#L265-L297
[tx-commit]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/transaction/commit.rs#L14-L39
[publish]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/transaction.rs#L413-L427
[platform]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/atomic_file/platform.rs#L34-L120
[rollback]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/transaction/commit.rs#L264-L306
[win]: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew#remarks
[posix]: https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html
[plan]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_runtime/runtime/04/failure-2026-07-23-project-catalog-input-generation.md#L33-L64
[ordinary]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/tests/project/manager/catalog_input_generation.rs#L22-L74
[targeted]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs#L424-L499
[targeted-plan]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_runtime/runtime/04/failure-2026-07-18-project-source-index-targeted-import.md#L89-L123
[reader]: https://github.com/He-Jiahui/ZirconEngine/pull/134
[scene]: https://github.com/He-Jiahui/ZirconEngine/pull/157
[io-names]: https://github.com/He-Jiahui/ZirconEngine/pull/159
[io-owner]: https://github.com/He-Jiahui/ZirconEngine/pull/175
[texture]: https://github.com/He-Jiahui/ZirconEngine/pull/180
[status]: https://github.com/He-Jiahui/ZirconEngine/pull/201
[compound-fixture]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/tests/project/zmeta/compound_shader.rs#L452-L493
[builtin-shader]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/ingest/asset_importer.rs#L289-L299
[file-revision]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/io/source_revision.rs#L124-L136
[restart-test]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/tests/project/manager/restore_failure_migration.rs#L5-L78
[action-test]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/tests/metadata_build_identity.rs#L89-L154
