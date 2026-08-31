---
related_code:
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/src
  - zircon_hub/web
  - zircon_hub/assets
  - zircon_hub/icons
  - zircon_hub/capabilities/default.json
  - zircon_hub/tauri.conf.json
  - zircon_hub/package.json
  - zircon_hub/hub.toml
tests:
  - zircon_hub/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
  - docs/plans/optimize/zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md
  - docs/plans/optimize/zircon_hub/04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md
  - docs/plans/optimize/zircon_hub/05-application-host-bootstrap-window-ipc-close-shutdown-crash-recovery-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ProjectEditorRecords.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectEditorRecords.cpp
  - dev/UnrealEngine/Engine/Source/Developer/DesktopPlatform/Public/InstalledPlatformInfo.h
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/godot/editor/project_manager/project_list.cpp
  - dev/godot/editor/project_manager/project_manager.cpp
  - dev/godot/editor/project_manager/project_dialog.cpp
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/project-manager/src/project.rs
  - dev/Fyrox/project-manager/src/upgrade.rs
  - dev/Fyrox/fyrox-build-tools/src/export/mod.rs
  - dev/Fyrox/fyrox-build-tools/src/export/pc.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/package.json
  - dev/Graphics/.yamato/wrench/api-validation-jobs.yml
  - dev/Graphics/.yamato/wrench/package-pack-jobs.yml
  - dev/Graphics/.yamato/wrench/promotion-jobs.yml
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.hdrp/package.json
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.urp/package.json
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
supersedes_currentness_of:
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
  - docs/plans/optimize/zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md
  - docs/plans/optimize/zircon_hub/04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md
  - docs/plans/optimize/zircon_hub/05-application-host-bootstrap-window-ipc-close-shutdown-crash-recovery-review.md
source_recheck_required: true
---

# 06 · Hub Product Control Plane / Project Lifecycle / Process / Delivery / Web / Host / Test Evidence 当前源码全量复核

## 1. 结论

本轮逐文件复核了 `zircon_hub` 当前全部272个tracked文件，而不是抽取若干入口做关键词采样。范围包括98个Rust production文件、66个Web production文件、39个integration test文件、54个视觉资产、4个Tauri生成schema和11个build/config文件；同时沿Project Browser、project record、process owner、export/package、application lifecycle和package validation六条证据链复核本地Unreal、Godot、Fyrox、Bevy与Unity Graphics的22个参考文件。

结论不是“Hub全部是假实现”。当前staged project create、manifest重读拒绝、shared recent protocol与OS writer lease、focus mailbox原子publish、typed action/message ID、按文件hash的install receipt、remote capability保持disabled，以及新近恢复的258项inline unit test harness，都是应保留的真实底座。

但这些底座尚未组成工程级产品控制面。整个应用仍由一个 `Arc<Mutex<HubRuntimeSession>>` 同时拥有配置、草稿、导航、catalog、queue、task和process投影；文件选择对话框、递归扫描、Git、persist与若干refresh可以在这把锁内同步执行。command边界没有版本、principal、capability、immutable target、payload budget或server generation；后台执行仍是无界 `VecDeque`、detached serial worker与单一 `TaskStatus`。Project、Engine、Build、Package、Install和Editor process没有共同的事务、artifact、process owner与terminal receipt。

产品真实性仍是最高风险。真实Tauri内的state load/协议校验失败会被Web层吞掉并显示可操作fallback shell，而后续action仍会调用live Rust backend。Package只是复制项目树，Install再复制目录；没有cook、build set、dependency closure、签名、平台entrypoint、原子激活或rollback。WebView直接拥有window close权限，宿主没有CloseRequested/ExitRequested、stop-admission、worker/process quiesce和terminal checkpoint。这样的实现不能以“能启动、能复制、能拉起Editor”宣称达到或超过Unreal。

Hub01至Hub05合计14项P0、254项P1、55项P2，共323项唯一finding。本轮没有为同一根因重新编号：**322项Open、0项Partial、1项Closed**。唯一Closed是Hub01 `ZHUB-P0-01` 的确定参数不匹配已在当前源码改为 `persist_unchecked()`；这只关闭源码形态问题，本轮没有运行Cargo，不能据此宣称Hub可构建或可发布。其余13项P0全部Open。8个本轮强化事实均映射到既有canonical finding，不增加全局计数。

## 2. 审查边界与可复验证据

### 2.1 当前树冻结

| 集合 | 文件 / 有效文本行 / bytes | 当前观察点SHA-256 | 证据等级 |
|---|---:|---|---|
| Rust production | 98 / 21,605 / 860,439 | `1f027b87691e160cc00fba30ba4c9f9852ae2291c7a1796de9ed3a0ffcf7d803` | E3：入口、domain、state、Tauri、process、settings与projection逐文件审读 |
| Web production | 66 / 7,050 / 257,572 | `1d58fc5110a0523bdb1e2057aaceae6741cf7daf9fe5713c8d08dfe2087d6773` | E3：React shell、全部page/component、Tauri adapter、validator、types/theme/CSS逐文件审读 |
| Integration tests | 39 / 18,341 / 708,770 | `46c17875ddfa084d3712c08ad760fecf919561bcd652eb173cfd564779d0381c` | E3：270项test逐文件分类，区分business behavior与source-shape assertion |
| Visual assets | 54 / 347,396 bytes | `3a06f09a06d09fe7f3586fa2df83ddf6286c75f9548d68645e0e86a53c182190` | E2：48个SVG共286有效行，另有5 PNG与1 ICO；核对production引用边界 |
| Generated Tauri schemas | 4 / 4,586 / 298,295 | `f6f3f69d7184af3a9f5b01b334590824ce227fd58472bea60365db6a4a02162d` | E1/E2：生成物只用于capability/config交叉核对，不当作产品实现 |
| Build/config | 11 / 2,422 / 82,076 | `635bfd7a2b55f2f9a56990d6fd54929509376311c7283fce3279aeb93fc3bdf2` | E3：Cargo、build.rs、npm/TS/Vite、Tauri、capability与tracked dev config |
| Hub union | 272 / 54,290有效文本行 / 2,554,548 bytes | `5690936e9f36db40fad62514d5ca20a75aba4c6ca3e0977dc856a9f60c782abb` | 上述集合无遗漏并去重；二进制资产不把随机换行字节计入有效行 |
| Five-engine references | 22 / 11,686 / 476,569 | `13ed3c742b4ebac0a11f64cbb3d894efe59b9f5259a67fcd11f07fa5e31adbdc` | E2/E3：只引用本地源码可直接证明的project/process/export/app/package原语 |

指纹算法统一为：workspace相对路径转 `/` 并小写，逐文件计算lowercase SHA-256，按path ordinal排序，再对每项 `path + NUL + hash + LF` 的UTF-8字节流计算SHA-256。有效文本行使用仓库既有 `Measure-Object -Line` 口径；PNG/ICO不参与行数。基线提交为 `79f64878f3b9526517644c055ad3bf5cadfccd0f`，观察日期为2026-08-24。

### 2.2 工作树漂移与验证边界

冻结时Hub范围有三处既有未提交改动，本报告把其当前bytes纳入指纹，但不把它们冒充已集成提交：

1. `zircon_hub/Cargo.toml` 删除显式 `test = false`，自动library test harness因此可包含inline unit tests。
2. `action_request.rs` 增加仅测试可见的 `HubActionRequest::parse()`。
3. `project_actions.rs` 把错误的 `persist_unchecked(None)` 改为 `persist_unchecked()`。

本轮是review-only，没有修改production、tests、Cargo、Web、manifest或资产，没有运行Cargo、npm、Tauri真实窗口、Hub+Editor双进程、Playwright、故障注入、网络盘、kill/restart、签名、package/install设备、无障碍或性能基准。静态源码足以确认调用图、owner、锁域、持久化顺序、protocol字段、测试连接方式和明确缺失；它不能证明构建、运行时行为、吞吐、尾延迟或产品资格。

### 2.3 测试证据重新分类

| 事实 | 当前证据 | 判定 |
|---|---:|---|
| inline Rust test attributes | 258 | Cargo manifest已不再关闭library test harness；这是测试接线进展，不是动态通过证据 |
| integration files / test attributes | 39 / 270 | 只有 `project_management_contract.rs` 导入production `zircon_hub` crate |
| source-shape integration files | 38 / 39 | 主要读取Rust/TSX/TOML/文档并断言字符串；可守住声明形状，不能证明业务执行 |
| combined attributes | 528 | 本轮未执行，不能写成528项passing tests |
| frontend test/spec | 0 | 没有DOM、Reducer、keyboard、focus、a11y、Tauri IPC或真实window行为测试 |

## 3. 当前可保留的工程底座

1. `create_project`使用staging、target rename、backup与rollback，并在发布后重新解析project manifest；invalid output不会直接当成功。
2. shared recent projects具备versioned协议、跨进程writer lease与原子替换；focus mailbox也使用OS lease和atomic publish，而不是裸写共享文件。
3. action ID、message ID、status和部分payload已类型化；remote Marketplace/Auth/Cloud capability当前保持disabled，没有伪造HTTP provider或token实现。
4. Package/Install至少形成分阶段Rust类型，并为installed files生成SHA-256 receipt；这些可作为未来artifact verifier的迁移输入。
5. background worker把长package/build从Tauri command直接调用中移出；当前缺点是owner、预算和durability，不应退回UI线程同步执行。
6. React使用MUI基础组件、统一theme/token、typed action映射和runtime validator；应在其上补齐生成式协议、行为测试与accessibility，而不是重写一套视觉壳。
7. 当前Cargo改动恢复library unit test harness，258个inline测试终于具备被Cargo收集的源码形态；后续应把source guard迁移为真正的domain/host行为测试。

## 4. 当前源码纵向事实

### 4.1 Application host与bootstrap

`main()`只转调 `tauri_app::run()`；`HubCommandState::load()`在window出现前同步加载完整session。启动配置、recent manifest、catalog或store任一错误都可以让用户得不到可见诊断窗口。宿主只监听 `Focused(true)`，没有BootId、instance admission、Booting/Degraded/Recovery状态、CloseRequested/ExitRequested、clean-shutdown marker或restart handoff。

focus refresh与background worker都是丢弃 `JoinHandle` 的 `thread::spawn`。focus线程panic可使 `focus_refresh_pending` 永久保持true；worker和Editor child也没有接入统一application lifetime。所有 `app.emit` 结果被忽略，宿主不知道窗口已销毁、subscriber失联、序列化失败还是terminal state未送达。

### 4.2 Command、admission与锁域

当前31项action和aliases没有protocol/capability/descriptor version。外层request不携RequestId、principal、origin window、expected generation、deadline、idempotency key或confirmation token；payload先完整成为 `serde_json::Value` 再clone/deserialize，没有bytes/depth/items/string/time预算。

path只做absolute检查，没有canonical root capability、symlink/reparse policy、immutable handle或TOCTOU reopen。`target_id`、project ID/path、resource title/path和history ID存在多套alias及precedence。后台请求在排队时不冻结解析后的target/build set/store revision，执行还会改写全局selected project与active engine。

所有command先获取同一 `Mutex<HubRuntimeSession>`。尤其Settings的folder picker是在 `hub_action -> session lock -> apply_action -> browse_settings_folder -> PowerShell FolderBrowserDialog` 链中同步打开；用户只要保持modal不关闭，就能无限期阻塞所有Hub command、state read和background completion publication。这是 `ZHUB-P1-08`、`ZHUB-CTL-P1-11/12` 的直接产品化复现。

### 4.3 Scheduler、task与process owner

build、package、install和open editor共享一个无界 `VecDeque` 与单串行worker，没有items/bytes预算、lane、priority、fairness、dedupe、cancel、deadline、retry、resume、journal或crash recovery。`TaskStatus`只有单槽和0/10/35/100合成进度；Package/Install还被压进 `TaskOperationKind::Project`，无法表达独立artifact和终态。

Build/Git/recycle/folder/open-folder多处仍使用同步process API。Build通过 `Command::output()`完整缓存stdout/stderr，没有stream、deadline、process-tree owner、kill escalation与reap。Editor spawn后 `Child`被丢弃；Ready mailbox不验证first-present或长期PID owner，focus写入也没有ack。系统folder打开只证明spawn成功，不证明用户可见结果。

### 4.4 Settings、config与durability

`HubConfig`没有schema version、migration registry、minimum reader、strict unknown field或future quarantine。load会读取完整TOML并刷新recent manifests；一个坏manifest可阻止boot。save使用固定 `.tmp`，缺single-instance/CAS/unique staging/parent directory fsync/last-good backup/corrupt quarantine，shared recent writer lease还可无限等待。

Settings每次输入把完整draft payload发给backend，没有revision/CAS；迟到请求可以覆盖较新草稿。save先修改内存config、engine与catalog，再durable persist；后续失败会留下memory/disk部分终态。tool probe只检查PATH/existence，jobs只有最小值没有effective CPU/memory cap，repair还会静默选first engine并删失效binding。

### 4.5 Project、Engine与lifecycle transaction

Project create的文件事务值得保留，但Hub级commit不闭合。`remember_lifecycle_project()`在refresh前已经修改selected page/path、recent、metadata、template和active engine；refresh失败后 `record_create_project_kept_folder_failure()`调用 `persist_unchecked()`，可能把已加入recent的项目持久化，同时history/task又宣称“record failed，请Import”。这不是新根因，而是 `ZHUB-P1-26` 与 `ZHUB-CTL-P0-04/P1-26/27/57` 的更精确partial-effect证据。

Delete仍不先协调active Editor。startup recent reconciliation仍可把Editor在Hub关闭期间删除的条目复活。Project identity在canonicalization、lossy path和slug之间漂移，`engine_version_req`没有resolver consumer。Engine identity仍由source path hash形成，workspace验证依赖字符串，active/source缺失时可fallback first，sibling editor executable还能绕过project绑定的staged engine。

### 4.6 Catalog、Plugin、Learn与Team

Asset/Learn/Plugin采用递归filesystem扫描，没有depth/time/entry/byte预算；Asset/Learn只是完整扫描后截断，Plugin甚至没有结果上限。任一read_dir、metadata、TOML或UTF-8错误可让整个catalog失败；结果没有total/truncated/cursor/freshness/watcher/root health。

Asset只是扩展名文件表，不是asset registry；Learn只读Markdown标题/首行并打开parent folder；Plugin schema缺strict字段、publisher/license/dependency/digest/signature/entitlement/install state，duplicate ID可进入UI，engine plugin scan还只取第一个existing root。development source roots无条件纳入cwd和compile-time repo root。

Team同步执行Git命令且无deadline/output/process-tree budget，把commit authors误称members，把Git identity误称当前账户并暴露email。Local Git Identity、Contributor、Hub Account、Organization Member、Marketplace Principal必须硬分域。

### 4.7 Package、Install与delivery truth

Package递归复制project，除 `.git` 与 `target` 外没有deny-by-default recipe；没有save barrier、immutable project snapshot、cook、target platform/config/arch、runtime entrypoint、asset dependency closure、shader pipeline、compression、chunking、signature或SBOM。symlink会被静默遗漏，final目录在操作期间直接可见。

Install接受任意目录，没有先验证manifest/hash/signature，直接递归复制到最终路径；不存在inactive slot、atomic activate、update、rollback、repair或uninstall owner。receipt逐文件 `fs::read` 整体载入并手写SHA-256，绝对路径又进入持久化history。生成的 `file://` URL没有percent encoding，并把多个本地文件描述成连续range stream，不能作为真实resumable download contract。

Install worker先完成Package，再完成Install；如果Install失败，成功Package没有独立terminal truth和artifact owner。成功分支也先record package，再record install，持久化失败仍可能形成外部effect与history不一致。这些继续由 `ZHUB-P1-27..36`、Hub03 package gates和 `ZHUB-CTL-P0-04` 拥有。

### 4.8 State、history、message与ViewModel

history固定16条，ID由timestamp/action/target拼接；raw argv、log、PID、绝对路径和output直接持久化/投影，没有sensitivity、redaction、privilege或lazy artifact。snapshot每次clone完整arrays，scope在无选择时fallback latest recent/first engine，高风险quick action仍可enabled。

Message虽有typed ID/params，但不强制param count，顺序replace允许placeholder二次解释，unknown ID roundtrip会退为RawText，raw text仍广泛承载I/O/process/path错误。backend label/target由英文free string再本地化，Rust与TS fallback维护重复文案，没有catalog version、provider namespace、argument type和sensitivity。

ViewModel没有protocol/schema/generation/capabilities。project cover、platform和engine version仍可能由fixture、path样式或硬编码 `Zircon Engine 1.8.2` 推断；unknown/stale template或engine会静默fallback。future timestamp饱和成“just now”，lossy path与display identity混用。

### 4.9 Web shell、IPC与产品真实性

真实Tauri环境中的 `loadHubState()` 捕获所有invoke/validation异常并返回 `fallbackShellState`，但 `dispatchHubAction()`仍调用live backend。这会形成“演示read model + 真实破坏性command”的混合模式；它已由 `ZHUB-UI-P1-01` 和 `ZHUB-HOST-P1-26` 拥有，必须作为M0 fail-closed项处理。

validator只检查顶层数组/record和少数字段。initial load和event subscription分开建立，event没有host/window/session/schema/generation/sequence；subscription失败无retry/resync。前端的action sequence/state generation只抑制某些旧response，不取消backend effect，也不能证明server因果；被忽略的emit与response suppression组合后，UI可长期停在stale state。

全产品只有一个snackbar/task summary。自绘窗口没有drag region，close直接调用native window；Promise被丢弃。custom tree/table/navigation缺完整keyboard/focus/ARIA contract，多个route的local state可能互相污染，大量 `noWrap` 没有tooltip/copy，motion没有reduced policy。CSP为null，bundle只配置NSIS，npm scripts没有unit/component/E2E/a11y/lint/bundle/security gate，部分dependency仍使用 `latest`。

## 5. 五组参考源码给出的约束

| 参考 | 本地源码可确认的工程原语 | 对Zircon Hub的约束 |
|---|---|---|
| Unreal | `ProjectEditorRecords`通过system-wide critical section、latest-load、queued serialized update、timestamp/stale prune和teardown wait管理machine-wide project记录；`InstalledPlatformInfo`把platform/config/architecture/required files/project type表达为typed support；`SProjectBrowser`保存engine identifier/version、target platforms、thumbnail、access time、compatibility和support状态。 | Recent不能是无代际全量覆盖；Project card不能猜engine/platform；安装与open必须消费typed compatibility/build-set，而不是路径和exit code。 |
| Godot | Project Manager读取真实project config，区分missing、unsupported、migration和recovery，提供scan/import/rename/remove、duplicate/copy、tag/favorite/filter/sort及keyboard/accessibility loading状态。 | 一个坏project不能拖垮boot；每项必须保留degraded/recovery状态；扫描、迁移和用户决策是产品workflow，不是按钮外观。 |
| Fyrox | Project Manager长期持有并轮询Child；upgrade/project/settings是独立domain。build-tools export使用typed target platform、destination、used assets、ignored extensions、build target、convert/optimization、cancel flag、streaming process output、Cargo metadata和platform exporter。 | Hub必须拥有process终态；Package要从typed export/cook plan产生可验证image，不能继续把recursive copy称为工程级package。 |
| Bevy | `App` runner返回typed `AppExit`，finish/cleanup与sub-app lifecycle显式；TaskPool/scoped tasks具有owner和完成边界。Bevy没有可比Project Manager/Hub。 | 只借鉴application/task ownership；不能用Bevy缺少Hub源码为Zircon省略安装、恢复、账号或项目控制面背书。 |
| Unity Graphics | SRP packages固定版本与dependency closure，samples声明路径/依赖；Yamato有API validation、package pack、validation/promotion jobs，SRPTests使用真实project/package matrix。该仓不含Unity Hub源码。 | package和promotion必须由manifest、dependency、validation receipt与真实project matrix驱动；不推断闭源Unity Hub的内部实现。 |

这些参考不是要求复制类名。共同约束是：稳定identity、typed compatibility、长期owner、可取消工作、版本化artifact、原子publish、失败可恢复和可执行资格证据。性能目标也必须先固定同功能、同数据、同平台、同构建配置和同质量输出，再比较启动、扫描、build/export、install、内存与尾延迟；当前源码没有资格支持“优于Unreal”的数值结论。

## 6. Hub01-Hub05 finding重判账本

### 6.1 唯一finding汇总

| Canonical报告 | P0 | P1 | P2 | 当前Open / Partial / Closed | 说明 |
|---|---:|---:|---:|---:|---|
| Hub01 | 4 | 36 | 8 | 47 / 0 / 1 | `ZHUB-P0-01`源码形态Closed；`P0-02..04`、`P1-01..36`、`P2-01..08`全Open |
| Hub02 | 0 | 46 | 8 | 54 / 0 / 0 | `ZHUB-UI-P1-01..46`、`ZHUB-UI-P2-01..08`全Open |
| Hub03 | 5 | 72 | 12 | 89 / 0 / 0 | `P0-01..05`、`P1-01..72`、`P2-01..12`全Open；remote disabled只是正确fail-closed基线 |
| Hub04 | 5 | 60 | 15 | 80 / 0 / 0 | `ZHUB-CTL-P0-01..05`、`ZHUB-CTL-P1-01..60`、`ZHUB-CTL-P2-01..15`全Open |
| Hub05 unique | 0 | 40 | 12 | 52 / 0 / 0 | `ZHUB-HOST-P1-01..40`、`ZHUB-HOST-P2-01..12`全Open |
| 合计 | **14** | **254** | **55** | **322 / 0 / 1** | **323项唯一finding**；Hub05三个B0只是既有P0 alias，不重复计数 |

Hub05 `ZHUB-HOST-B0-01..03` 分别继承operation shutdown、effect recovery和multi-writer/config transaction阻断，不是新增finding。Hub03的五项P0是远程能力启用门；当前页面disabled不等于能力完成，所以状态仍为Open，而不是Partial或Closed。

### 6.2 唯一Closed项的证据限制

`ZHUB-P0-01`原问题是 `project_actions.rs` 调用 `persist_unchecked(None)`，而当前签名不接受参数。当前工作树已改为 `persist_unchecked()`，因此该确定源码不匹配按其原始定义Closed。由于改动尚未形成提交且本轮没有运行 `cargo check/build/test`，报告只写“source-shape closed”；任何构建、Tauri启动或发布资格仍为未验证。

### 6.3 本轮强化的8个既有表现，不重复计数

| 当前源码表现 | Canonical owner | 本轮影响 |
|---|---|---|
| Settings folder modal在global session mutex内同步等待用户 | `ZHUB-P1-08`、`ZHUB-CTL-P1-11/12` | 将“慢I/O锁域”强化为可由用户无限维持的全产品阻塞 |
| create已修改recent/selection/engine后refresh失败，failure路径又persist当前内存 | `ZHUB-P1-26`、`ZHUB-CTL-P0-04/P1-26/27/57` | 明确了磁盘项目、recent、history和task truth的partial commit序列 |
| live Tauri state失败退demo，但action继续连live backend | `ZHUB-UI-P1-01`、`ZHUB-HOST-P1-26` | 将fallback风险提升为可操作混合authority，不新增根因 |
| `file://`未编码且本地多文件被伪装为连续range stream | `ZHUB-P1-36`、Hub03 package/download gates | 明确special-character path与resume contract均不成立 |
| inline unit harness已恢复，但39个integration中38个仍不链接业务crate | `ZHUB-CTL-P1-51..53`、`ZHUB-HOST-P1-35..40` | 测试可收集性进步不改变行为资格缺失 |
| catalog source纳入cwd与compile-time repo root | `ZHUB-UI-P1-16` | release scope/trust仍未建立 |
| Install失败会丢失已成功Package的独立terminal truth | `ZHUB-P1-36`、`ZHUB-CTL-P0-04` | Package artifact与Install attempt必须拆分owner/receipt |
| client response suppression叠加server emit结果忽略 | `ZHUB-UI-P1-05/06`、`ZHUB-HOST-P1-19/22..25` | completion可执行成功但没有单调可恢复的UI publication |

## 7. 目标架构与owner边界

Hub应是产品安装、发现、项目工作流和进程编排宿主，不应成为Runtime world、Editor document、asset import或release repository的第二authority。

```text
HubApplicationHost
  -> InstanceCoordinator / BootMachine / WindowSession / LifecycleCoordinator
  -> CommandGateway(versioned envelope, principal, capability, budget)
  -> OperationService(durable registry, lanes, cancel, deadline, effect ledger)
  -> ProjectWorkflow(ProjectRegistry + ProjectPreflight + ProjectTransaction)
  -> EngineInstallationService(BuildSetResolver + InstallationRegistry)
  -> ProcessSupervisor(Editor/build/tool process tree + terminal receipt)
  -> DeliveryClient(cook/build/package/install artifact contracts)
  -> CatalogQuery(Project/Engine/Asset/Plugin/Docs provider snapshots)
  -> Versioned ReadModel(snapshot revision + typed delta + resync)
```

### 7.1 Hub拥有

- process activation、single/multi-instance policy、window session、close/shutdown与crash recovery；
- installed engine/build-set discovery、project registry projection、user operation orchestration；
- command admission、durable operation registry、process supervisor和product-facing receipts；
- provider health、catalog pagination/freshness、secure local settings以及frontend protocol。

### 7.2 Hub不得拥有

- Runtime world/scene/resource truth或另造asset registry；
- Editor document/save/recovery/plugin activation truth；
- 通过硬编码path/version/platform覆盖project manifest和resolved build set；
- 自造release repository、签名或download协议来绕过Tooling09的canonical delivery owner；
- 未经secure credential、RBAC、entitlement、trust和conflict协议就启用remote页面。

### 7.3 必须形成的核心合同

1. `HubCommandEnvelope { protocol, request_id, principal, origin, expected_revision, deadline, idempotency, target_lease, payload }`。
2. `HubOperationRecord { operation_id, kind, phase, resource_lanes, progress, cancellation, effect_disposition, terminal_receipt }`。
3. `ProjectLaunchIntent`与`EngineBuildSetManifest`共同冻结ProjectId、engine requirement、resolved build set、ABI/platform和session generation。
4. `ProjectPackageArtifact`、`DeviceInstallAttempt`与`ActivationReceipt`分别拥有成功/失败，不把三者压成一次复制动作。
5. `HubStateEnvelope { protocol, host_instance, window_session, revision, capabilities, snapshot/delta, resync_token }`。
6. `ProviderSnapshot<T> { provider, generation, freshness, page/cursor, truncated, diagnostics }`，catalog不得再把递归扫描数组当完整authority。

## 8. 分层重构路线

### M0 · Truth freeze、静态闭合验证与fail-closed止血

1. 对当前三处dirty Hub源码先运行managed Windows check/test，确认 `ZHUB-P0-01` 真正构建闭合，并保存命令、commit、target与日志receipt。
2. production Tauri禁用demo fallback；state/codec失败进入ProtocolMismatch/BackendUnavailable并关闭破坏性command。
3. folder picker、Git、扫描、persist与process I/O移出global session mutex；先增加能稳定复现阻塞/乱序/partial-effect的RED tests。
4. close action改为backend close intent；在完整host状态机前至少建立stop-admission与running-operation决策。

### M1 · Application host、instance与window lifecycle

1. 引入 `HubApplicationHost`、BootId/InstanceId、phase/deadline/degraded policy和reverse cleanup。
2. 建立single-instance/activation envelope、minimum diagnostic window、clean/unclean marker与safe-mode boot。
3. 统一CloseRequested、ExitRequested、last-window、OS exit和update restart，持有worker/process owners并输出terminal receipt。

### M2 · Versioned command、immutable target与read model

1. 由单一schema生成Rust/TS command、error、snapshot/delta和capability binding。
2. request admission执行size/depth/item/time预算、principal/capability/risk/confirmation与canonical target lease。
3. subscribe-snapshot握手返回server revision和sequence；客户端只按server generation单调apply并可resync。

### M3 · Durable operation registry、process supervisor与effect ledger

1. 用有界、分lane、可取消/截止/公平的OperationService替换无界VecDeque与single TaskStatus。
2. 所有build/Git/editor/shell/tool process进入platform process-tree owner，streaming output有bytes/line/time budget并保证reap。
3. external effect、history、artifact、config projection采用prepare/effect/commit/compensate/reconcile状态与幂等receipt。

### M4 · Project、Engine、Package与Install事务

1. Project preflight表达Exact/Compatible/Upgrade/Future/Missing/Recovery；identity、manifest requirement和resolved build set硬绑定。
2. Engine installation使用publisher/product/version/channel/platform/build digest身份，source location只是可变属性。
3. Package必须消费冻结project snapshot、cook/build result和dependency closure，生成versioned manifest、hash、entrypoint、SBOM与signature chain。
4. Install只接受validated artifact，写inactive staging/slot，verify后atomic activate，并支持update/repair/uninstall/rollback。

### M5 · Bounded catalog、settings、identity与remote activation gates

1. Project/Engine/Asset/Plugin/Docs使用provider registry、immutable generation、fault-isolated root、bounded query/cursor/watch/freshness。
2. Config/draft使用versioned schema、migration、strict decode、revision/CAS、unique staging、backup/quarantine与field patch。
3. Local Profile、Git Identity、Contributor、Online Account、Organization Member和Automation Principal硬分域。
4. Auth credential、organization RBAC/audit、Marketplace signature/entitlement/native admission和Cloud revision/CAS conflict全部通过后，才逐项打开remote capability。

### M6 · 行为、故障、安全、规模与竞争性性能资格

1. 建立真实Rust business tests、Tauri harness、React component/reducer、Playwright desktop、keyboard/a11y和双进程Hub+Editor矩阵。
2. 覆盖disk full、permission、rename/fsync、bad manifest、network share、process hang/crash、kill/restart、PID reuse、event gap/reorder和schema skew。
3. 对10k/100k project/catalog、large logs、large package、slow disk和multi-instance writer建立items/bytes/time/FD/process预算。
4. 与参考引擎做同功能同数据同平台基准，记录p50/p95/p99、峰值内存、I/O、CPU、artifact大小、失败恢复时间和输出正确性。

## 9. 验收门

### 9.1 Host与协议

- [ ] H01：任一store/catalog损坏时minimum diagnostic window仍能出现，并提供stable support code与repair/quarantine动作。
- [ ] H02：第二实例、deep link与project intent通过带identity/ack的activation envelope路由，不产生双writer。
- [ ] H03：close/quit/OS exit先stop admission，再列出并处理queued/running/child work，最终产生terminal receipt。
- [ ] H04：worker、focus refresh与Editor/build process都由宿主持有，shutdown能证明joined/reaped/detached disposition。
- [ ] H05：snapshot/event握手无gap，duplicate/reorder/drop可检测并resync，旧window/session event被拒绝。
- [ ] H06：production backend或protocol失败绝不显示可操作demo state。

### 9.2 Command与operation

- [ ] C01：所有action从versioned descriptor生成并声明scope、risk、capability、queue class、idempotency与receipt。
- [ ] C02：payload bytes/depth/items/string/time预算在完整分配前执行，超限返回typed error。
- [ ] C03：absolute path不能越过root capability、symlink/reparse与generation lease；queued target不可随selection漂移。
- [ ] C04：queue有items/bytes/lane/fairness预算，支持cancel/deadline/retry并在restart后reconcile。
- [ ] C05：operation progress来自真实work units；Package、Install、Build、Launch都有独立phase和terminal outcome。
- [ ] C06：external effect与history/config/artifact commit之间的每个failure point都有fault-injection恢复证明。

### 9.3 Project、Engine与delivery

- [ ] P01：active Editor存在时Delete/Engine update/remove必须经过coordination decision，不能直接破坏authority。
- [ ] P02：project requirement、resolved build set、ABI/platform与launch session共同冻结并写入receipt。
- [ ] P03：future/missing/upgrade/recovery project以typed preflight呈现，不被静默打开或删binding。
- [ ] P04：Package不包含recovery/cache/credential/source-only泄漏，且输入snapshot变化会abort/retry。
- [ ] P05：manifest、每文件hash、entrypoint、dependency closure、SBOM、signature与validation receipt完整可复验。
- [ ] P06：Install通过inactive slot原子激活；中断、校验失败或新版本启动失败均保留旧版本并可rollback。

### 9.4 Catalog、settings与remote

- [ ] D01：坏目录/manifest只降级对应entry/root；扫描有depth/time/entry/byte预算、cursor、truncation与freshness。
- [ ] D02：settings field patch有revision/CAS，durable save具备migration、backup/quarantine、unique staging与multi-writer测试。
- [ ] D03：Git author/identity、Hub account、organization member和entitlement principal在schema、UI与audit中完全分离。
- [ ] D04：Auth/Marketplace/Cloud每项capability只有在credential/RBAC/trust/conflict/rollback资格通过后才能启用。

### 9.5 UI、测试与性能

- [ ] Q01：39个integration合同不再把source string presence标作业务通过；关键domain和Tauri链有真实执行测试。
- [ ] Q02：React reducer覆盖response/event乱序，所有页面具备keyboard、focus、screen reader、zoom与reduced-motion资格。
- [ ] Q03：CSP、dependency audit、license/SBOM、bundle budget、compromised WebView和native capability矩阵进入CI。
- [ ] Q04：10k/100k catalog、slow/network disk、large package/log和queue pressure下没有无界内存、UI冻结或silent truncation。
- [ ] Q05：Hub+Editor双进程close/crash/restart、focus ack、PID reuse、stale mailbox和project delete矩阵有machine-readable receipt。
- [ ] Q06：只有同功能、同数据、同平台、同构建、同正确性门通过后，才允许发布“优于Unreal”的性能比较。

## 10. 实施禁令

1. 禁止继续通过新页面、hardcoded fixture、fallback值或自由字符串把未实现能力投影成Ready。
2. 禁止把更多I/O、process、dialog、scan或provider callback堆进 `HubRuntimeSession` 全局mutex。
3. 禁止为维持旧接口留下compat action alias、双schema、旧queue wrapper或第二套project/asset/plugin authority；迁移采用硬切换。
4. 禁止把recursive copy、exit code 0、mailbox Ready或spawn成功分别称为Package、Build、Editor Ready或Open成功。
5. 禁止用source-shape测试、未执行的test attribute、typecheck或静态截图替代真实行为、故障和产品资格。
6. 禁止在secure provider、credential、RBAC、signature/entitlement、conflict和rollback关闭前启用远程按钮。

## 11. 状态与产出记录

本轮完成Hub全部272个tracked文件与22个参考文件的当前源码静态复核，重判Hub01-Hub05全部323项唯一finding，并建立M0-M6与28项验收门。产出仅为review、metadata和索引；没有实施任何production修正，也没有运行动态验证。

当前账本是14项P0、254项P1、55项P2：322 Open、0 Partial、1 Closed。`ZHUB-P0-01`仅source-shape Closed，其余13项P0全部Open。实现前必须重新检查当前三处dirty Hub源码、重取指纹并先完成M0动态基线；在Host、Command、Operation、Project/Engine/Delivery和产品真实性闭合前，Hub仍是可演示的本地launcher/control shell，而不是与Unreal同级的工程化产品控制面。
