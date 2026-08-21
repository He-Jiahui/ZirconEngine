---
related_code:
  - zircon_hub/web/src
  - zircon_hub/src/assets
  - zircon_hub/src/plugins
  - zircon_hub/src/learn
  - zircon_hub/src/team
  - zircon_hub/src/settings
  - zircon_hub/src/tauri_app/action_request.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/catalog.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/Cargo.toml
  - zircon_hub/package.json
  - zircon_hub/tauri.conf.json
  - zircon_hub/capabilities/default.json
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/zircon_hub/04-settings-draft-and-source-engine.md
  - docs/plans/zircon_hub/05-frontend-componentization-and-type-safety.md
  - docs/plans/zircon_hub/06-layout-and-visual-standard.md
  - docs/plans/zircon_hub/07-localization-schema-and-coming-soon.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/Accessibility/SlateCoreAccessibleWidgets.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/Accessibility/GenericAccessibleInterfaces.h
  - dev/godot/editor/project_manager/project_list.cpp
  - dev/godot/editor/project_manager/project_manager.cpp
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/project-manager/src/settings.rs
  - dev/bevy/crates/bevy_asset/src
  - dev/Graphics/Packages
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 02 · Web Shell、Catalog、Settings、Team/Cloud、Accessibility 与 Performance 工程化差距

## 1. 结论

Hub前端不是纯静态mock。React页面已经接到Tauri typed action，Rust侧payload普遍使用`deny_unknown_fields`、绝对路径校验和catalog membership检查；Settings有独立draft，Learn只能打开当前catalog内资源；MUI也提供了button、dialog、tabs、table等基础语义。这些边界应保留。

但当前产品壳仍把“演示页面齐全”误当成“工程服务已经存在”。Tauri状态加载或schema验证只要失败，`loadHubState()`就吞掉错误并返回一份写死的中文演示状态，显示“Hub已就绪”“配置健康100%”“Zircon Engine 1.8.2”以及开发机路径；窄窗口还会隐藏唯一的“演示数据”徽标。Assets/Plugins/Learn不是索引服务，而是持有全局session mutex时同步递归读目录；Settings每个按键都发送完整draft并取回包含所有catalog的完整view model；Team只是最近200条Git提交的作者列表；Cloud仍是上一报告中的本地复制交付页。

本轮没有新增P0，记录46个P1和8个P2。不能据此降低上一报告的4个P0：当前Rust source仍因`persist_unchecked(None)`参数不匹配而无法构建Hub。前端`npm run typecheck`实际通过；真实Tauri截图验证因当前二进制不存在且Rust编译被该P0阻断，未使用浏览器fallback冒充产品窗口。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| React web source clean set | 66 / 7,447 | E3：App状态流、全部页面、data/input/overlay/shell/feedback组件、typed action、validator、theme与本地化consumer；0个前端test文件；fingerprint `3e3b4baabc040c8454de4a2796fdacfafaad3aead904ed035b809ffc5fc53c19` |
| Asset/Plugin/Learn/Team Rust clean set | 11 / 2,097 / 26 test attributes | E3：递归发现、scope刷新、Git投影、catalog DTO与coming-soon；fingerprint `dee4b32a71cc75d9977f93e5540c9df96c8c5e1f3a910923dde88fb8242bd9ae` |
| Settings Rust clean set | 6 / 2,528 / 27 test attributes | E3：config字段、draft、browse/save/default、health与source checkout验证；fingerprint `527d56a80e9f972b2906cf0b5307cf62e5ead3cd0366faf418bf59ae3215f9f1` |
| View/action contract Rust clean set | 15 / 5,964 / 59 test attributes | E3：request decode、full snapshot DTO、localized projection与Tauri emit；fingerprint `b32b9b0e51a987b14aa70bb279091a12bafebc35c1c397b39474392c7f5ae0ee` |
| Packaging/security contract clean set | 8 / 2,387 | E2-E3：Cargo/npm/TypeScript/Vite/Tauri config、capabilities与lockfile；fingerprint `1ded19ed06059681eb1537b7d8dd964b1442f77bd005e0b138035be7c747d24f` |

集合之间有意重叠，用于保留各纵向链的独立指纹。fingerprint算法与Hub 01报告一致：相对路径排序后，将`path + NUL + per-file SHA-256 + LF`串联再计算SHA-256。上述production和config均为clean tracked source；工作树中的既有文档修改没有回退。

### 2.2 动态验证

本轮执行：

```powershell
cd zircon_hub
npm run typecheck
```

`tsc --noEmit -p tsconfig.json && tsc --noEmit -p tsconfig.node.json`以exit 0通过。这只证明当前手写TypeScript内部自洽，不证明Rust JSON与TS类型一致，也不覆盖render、keyboard、screen reader、Tauri IPC、事件乱序或大catalog。

仓库的真实Hub截图流程要求当前`target/debug/zircon_hub.exe`并验证原生窗口标题、bounds和WebView像素。当前没有该binary，Hub 01报告已由Windows管理验证器复现Rust build exit 101，所以本轮没有运行视觉矩阵，也没有接受Vite fallback截图作为当前产品证据。

### 2.3 本轮追踪的产品链

1. `App`先用`fallbackShellState`渲染，再并行请求`hub_state`和订阅`hub-state-changed`。
2. 每个action调用Tauri `hub_action`，Rust在一把`Arc<Mutex<HubRuntimeSession>>`内改变状态并重新构造完整`HubViewModel`；后台progress也广播完整模型。
3. source engine、selected project或settings变化会串行重扫Assets、Learn、Plugins和Team，再把完整数组发回React。
4. Catalog页面把数组再次投影为rows、filtered rows和完整tree；搜索、tab、selection和tree expansion只存在于组件本地。
5. Settings每次输入立即更新本地draft，同时发送包含全部字段的`update-settings-draft`；保存会再次提交完整draft。
6. Team启动Git子进程读取一个repo的identity与最近作者；Cloud从全局action history推导package/install readiness。

## 3. 已有工程基础，重构时必须保留

### 3.1 后端admission不是任意shell入口

- action ID先解析为Rust enum；大部分payload拒绝unknown field，project/resource/output路径有absolute path或catalog/history membership校验。
- Learn的前端path不能绕过catalog直接打开任意位置；resource消失会留下localized failure和recovery。
- Tauri capability目前只授予main window最小window操作，文件、process和project mutation仍由Rust命令控制。

### 3.2 Settings draft与localized projection

- draft和persisted settings已经分离；Discard/Restore Defaults不会立即覆盖磁盘配置。
- save会校验必填字段与source checkout，invalid值以可恢复状态返回，不直接写坏config。
- UI文案大部分由Rust按当前语言投影，React没有到处重新翻译业务消息。

### 3.3 MUI基础控件与显式coming-soon

- dialog、tabs、form controls、icon button和table使用MUI基础组件，不应退回手写div控件。
- 尚未实现的asset import、plugin install/toggle/marketplace、remote sync、account、cloud repository、notifications、team invite/permissions明确列为disabled coming-soon，而不是伪造成功操作。

这些基础只证明局部admission和展示纪律。它们不能替代versioned IPC、degraded bootstrap、authoritative catalog、transactional settings、账号/更新服务或可访问性验收。

## 4. P0状态

本轮没有发现独立于Hub 01报告的新P0。现有4个P0仍全部有效，其中Rust编译阻断直接使当前Hub无法生成可验证产品binary。本报告的P1必须在恢复可编译后按真实Tauri窗口继续动态复核。

## 5. P1：必须进入工程重构主线

### 5.1 Bootstrap、IPC 与错误终态

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| ZHUB-UI-P1-01 | Tauri内`hub_state`调用失败、Rust序列化变化或validator失败都会被同一个`catch`吞掉并返回demo fallback。真实故障被伪装成正常Hub。 | `Booting / Ready / BackendUnavailable / ProtocolMismatch / StoreDegraded / Fatal`显式状态机；Tauri production禁止回退demo，fallback只允许Storybook/dev build flag。 |
| ZHUB-UI-P1-02 | fallback写死“就绪”、100%健康、`1.8.2`和`E:\Git\ZirconEngine`等开发机路径；demo徽标在小于1260px时隐藏。 | fallback不得含可混淆的真实产品状态；所有viewport持续显示环境标识，错误页只展示可验证的诊断和恢复动作。 |
| ZHUB-UI-P1-03 | `assertHubShellState`只检查顶层string/array/record和两个UI字段，不验证任何array element、discriminant、tone、range、ID唯一性或嵌套必填字段。 | 由单一schema生成Rust/TS codec，执行完整runtime decode、range和unique-key验证；错误包含字段路径、protocol version与correlation ID。 |
| ZHUB-UI-P1-04 | `HubViewModel`没有protocol/schema version、state revision或capability version；Rust DTO、776行TS类型和庞大fallback是三份手工镜像。TS还把Rust必有的`settingsDraft`声明为nullable，并私有增加`demoMode`。 | versioned envelope + generated bindings + compatibility matrix；demo metadata属于host环境，不混入产品DTO。 |
| ZHUB-UI-P1-05 | action、focus refresh和后台progress都序列化/广播完整view model；`app.emit`结果全部丢弃。catalog越大，状态频率越高，复制和JSON成本越大，consumer失联也没有可见终态。 | snapshot revision + typed delta/event；emit失败进入subscriber health，支持resync、backpressure、coalescing和metrics。 |
| ZHUB-UI-P1-06 | 前端只用“最后action sequence + state generation”抑制旧response，既不取消也不串行backend invoke；相关completion可被无关event丢弃，且没有operation ID确认因果。 | command envelope带client request ID、expected revision和operation ID；Reducer按server revision单调应用，支持cancel/replace/coalesce。 |
| ZHUB-UI-P1-07 | invalid event只写console后忽略；subscription失败把单一task summary改成warning；render crash reset又调用会返回fallback的`loadHubState`。 | connection health独立于task health；invalid payload立刻进入ProtocolMismatch并停止危险操作，保留raw diagnostic digest和重新握手入口。 |
| ZHUB-UI-P1-08 | 全产品只有一个`taskSummary`。Snackbar effect不依赖`taskId/label/detail`，用户关闭一次后，同tone和同recovery的下一项success/error不会重新打开。running状态又在4.2秒后自动隐藏。 | per-operation notification center和task list；以monotonic notification ID驱动announce，running不自动消失，terminal可确认、重试、打开日志。 |
| ZHUB-UI-P1-09 | `activePage`是自由string；未知page静默落到`WorkspacePage`，未知filter/tone等嵌套值也可能越过浅validator。 | 所有discriminant严格decode；unknown route显示协议错误，绝不回退无关页面。 |
| ZHUB-UI-P1-10 | 66个前端源文件没有一个test/spec，package scripts也没有unit、component、E2E或accessibility test。 | 建立codec contract fixtures、reducer乱序测试、React interaction测试、Playwright Tauri smoke和axe/keyboard gate；测试失败阻断发布。 |

### 5.2 Catalog ownership、freshness 与规模

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| ZHUB-UI-P1-11 | startup、select engine/project、settings save和build完成可在全局session mutex内串行执行Assets、Learn、Plugins与4次Git查询。慢盘/网络盘/Git hang会冻结全部`hub_state`和action。 | catalog/service job在锁外运行，带deadline/cancel；以expected scope revision原子发布immutable snapshot。 |
| ZHUB-UI-P1-12 | 任一`read_dir`、metadata、UTF-8 Markdown、TOML manifest错误都会用`?`中止整个catalog refresh；startup又把refresh作为开窗前全有或全无步骤。 | per-root/per-entry fault isolation；保留Healthy/Partial/Stale/Failed root和diagnostic，不因单个坏文件阻止Hub开窗。 |
| ZHUB-UI-P1-13 | Asset和Learn虽然最后截断256/128项，但会先遍历全部文件，Learn还先读完每个Markdown；Plugin无任何数量/深度/字节预算。 | 扫描阶段就执行entry/depth/time/byte budget，使用增量index、content cache与bounded parser；大目录有进度和取消。 |
| ZHUB-UI-P1-14 | 截断不返回`total/truncated/cursor`，且selected/recent project优先可耗尽全局额度，用户会误以为Engine catalog为空或完整。 | query result包含total、page/cursor、scope totals、truncation reason和继续加载；每scope有公平预算。 |
| ZHUB-UI-P1-15 | catalog只在少数scope动作时刷新，没有watcher、dirty generation、TTL、manual refresh或last indexed time；外部新增/删除长期不见。 | authoritative index消费filesystem/plugin/asset change stream；UI显示freshness和stale reason，并提供可取消rescan/repair。 |
| ZHUB-UI-P1-16 | source catalog无条件加入process current directory和编译时repo root作为“development fallback”，没有debug gate或trust boundary。 | release只扫描已注册engine/project root；development root必须显式opt-in、标明来源并经过canonical/trust校验。 |
| ZHUB-UI-P1-17 | Asset catalog只是除`.git/target`外的任意文件扩展名表，没有asset UUID、importer/version、artifact、dependency、error、thumbnail或registry generation，并与Editor/Runtime三套asset authority重复。 | Hub只查询共享`AssetRegistryService`摘要，不再自建递归文件authority；详细import/reimport归Editor已有计划owner。 |
| ZHUB-UI-P1-18 | Plugin catalog递归寻找`plugin.toml`并直接信任自由ID/category/maturity；一个坏manifest毁掉全表，duplicate plugin ID可产生重复React key；Install/Enable/Update仍不存在。 | 查询共享Plugin Package Registry，ID/version/publisher/target/dependency唯一化；坏包隔离，生命周期动作由签名、resolver和transaction驱动。 |
| ZHUB-UI-P1-19 | Learn只取第一个`# `和第一行摘要，搜索只覆盖当前截断集的title/summary/path；“Open Resource”实际打开父目录，不打开文档。 | versioned documentation index支持全文、语言、engine version、deep link、render/source选择、离线包与内容完整性。 |
| ZHUB-UI-P1-20 | `catalog_scope_key`、category和maturity tone通过English/Chinese display string包含关系反推typed key。新语言或文案修改会改变过滤和状态。 | source/category/maturity在domain层使用enum/stable ID；localization只渲染label。 |
| ZHUB-UI-P1-21 | backend把所有数组塞进每个snapshot；Catalog再构造rows、filtered rows和一份完整tree。没有server query、分页、虚拟化或稳定memo boundary。 | typed catalog query + cursor page；虚拟化list/tree，projection按entity generation增量更新，并设render/frame budget。 |
| ZHUB-UI-P1-22 | Assets/Plugins/Learn三个route复用同一`CatalogPage`组件，React会保留`query/tab/selectedRowId`。Learn的`guide`切到Assets会被当作Engine filter；Assets的`project`切到Learn会得到空表。 | mode成为keyed route state；切换时迁移或重置只对该mode有效的filter，并由schema拒绝非法tab。 |
| ZHUB-UI-P1-23 | `HubTreeView`只在首次mount读取`defaultExpanded`。Catalog mode变化后旧root ID仍在Set，新root不会按prop展开；动态node删除也不清理state。 | controlled tree state按catalog revision和route维护，自动清理不存在node，并保留用户明确操作。 |
| ZHUB-UI-P1-24 | `groupBy`对每个item复制整组数组，单组为O(n²)；Plugin无上限，且每次full state event都会重建rows/tree。 | 线性mutable accumulator或预分组query；用profiler/bundle benchmark验证10k/100k entity，禁止无界DOM。 |
| ZHUB-UI-P1-25 | 未知project一律使用Elysium参考封面；未绑定engine一律显示`1.8.2`；platform只凭路径含`:`或反斜杠猜Windows，否则猜Linux。真实项目被展示成fixture metadata。 | cover来自project manifest/thumbnail service并有中性placeholder；engine/platform来自resolved project requirement与build targets，未知值明确显示Unknown。 |

### 5.3 Settings draft、validation 与提交事务

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| ZHUB-UI-P1-26 | 每次TextField按键都提交全部9个draft字段，后端再返回包含全部catalog/UI文案的完整snapshot；没有debounce、field delta或本地validation。 | `SettingsDraftSession`支持field patch、debounce/coalesce和局部validation；只有保存或相关scope变化才重建昂贵projection。 |
| ZHUB-UI-P1-27 | 多个invoke可并发，draft没有revision/CAS。较早的完整draft若较晚取得mutex，可覆盖较新输入；save后的迟到update又可把session draft恢复成旧值。 | draft patch带base revision和client sequence；server拒绝/merge stale patch，Save原子关闭revision并取消旧请求。 |
| ZHUB-UI-P1-28 | Settings health把`warn`计入ready，所有不存在的目录都标“使用时创建”并算完成，所以未验证权限、父目录、空间和路径类型也可显示100% success。 | health区分Valid/Creatable/Unavailable/Unverified，completion不把warning算通过；执行真实writability、space、path policy与dry-run probe。 |
| ZHUB-UI-P1-29 | Python/Cargo/Rustup只检查PATH里是否存在同名文件，不执行版本、target、toolchain、architecture或capability探测；显式path甚至只检查exists。 | versioned toolchain probe记录resolved executable、hash/version/target/last checked，支持repair/install和最低兼容范围。 |
| ZHUB-UI-P1-30 | `jobs`只有最小1和`u16`解码，没有CPU/memory policy或上限；UI也不显示effective scheduler budget。 | 根据host资源和任务类型计算允许范围，用户值有hard cap，构建记录resolved concurrency。 |
| ZHUB-UI-P1-31 | draft没有dirty字段、差异预览、离页/关窗决策或autosaved draft；Save/Discard/Defaults始终可点。 | field-level dirty/validation/conflict状态；离页与关闭执行Save/Discard/Cancel，crash后可恢复未提交draft。 |
| ZHUB-UI-P1-32 | save先改`config.settings`并注册engine，再刷新catalog和persist；后两步失败会留下内存config/engine/catalog/draft与磁盘不同步的部分终态。 | prepare/validate/scan outside lock，durable config transaction提交成功后一次swap snapshot；失败保持旧authority并留下repair record。 |
| ZHUB-UI-P1-33 | Settings外层payload拒绝unknown field，内层`HubSettingsPayload`却未`deny_unknown_fields`，拼错或新旧字段可能被静默忽略。 | generated strict patch schema；兼容字段必须显式version migration，unknown字段返回field error。 |

### 5.4 Team、Cloud、Update、Marketplace 与账号能力

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| ZHUB-UI-P1-34 | Team refresh在session锁内同步运行`git rev-parse/config/log`，没有timeout、cancel、process-tree lease；config/log失败又被`.ok().flatten()`吞掉。 | SourceControlProvider异步查询，命令有deadline/output budget/termination；每项结果保留Unavailable/AuthRequired/Failed状态。 |
| ZHUB-UI-P1-35 | Team只取第一个repo，把最近200个commit中最多8位作者称为members和commit count；没有branch/remotes/divergence/locks/roles/permissions。 | 明确分离Local Git Identity、Contributors与Organization Members；真正Team service使用稳定member ID、role、project membership和audit。 |
| ZHUB-UI-P1-36 | 作者email自动投影到页面；topbar把Git identity当登录用户，Account菜单又跳到Team。没有consent、mask、auth session或privacy policy。 | 本地VCS identity默认最小披露；账号由AuthSession authority提供，PII展示有权限、mask和审计。 |
| ZHUB-UI-P1-37 | Cloud install readiness用`defaultDeviceInstallDir !== localized notConfigured`判断“配置完成”，并用任意package history数量判断ready，失败记录也算通过。 | readiness由typed preflight返回project/package/device/status/hash/capability逐项结果，不能比较display string或只数history。 |
| ZHUB-UI-P1-38 | Package/Install/Build/Open Editor等按钮没有按selected target、running operation或queue capacity禁用；重复点击可继续灌入上一报告的无界后台队列。 | server发布command capability与disabled reason；UI防重复并显示queued/running/cancel，server仍做幂等/admission。 |
| ZHUB-UI-P1-39 | Assets import、plugin install/toggle/marketplace只有disabled列表，现有页面却采用完整产品导航和metric外观，容易把file listing误认成authoring/package能力。 | capability registry驱动导航；未达到MVP gate的domain显示明确Unavailable边界，完成后接权威Asset/Plugin service而非页面内补按钮。 |
| ZHUB-UI-P1-40 | Update按钮永久disabled；没有channel manifest、签名、delta/resume、proxy、repair、rollback、engine/Hub兼容或离线介质。 | Signed Update Service独立管理Hub、engine build set和template/plugin channel，支持staging、verify、atomic activation、rollback与enterprise policy。 |
| ZHUB-UI-P1-41 | Remote sync、account、cloud repository、notifications、invite、permissions和collaboration全部不存在。 | Provider边界先定义Auth/RBAC/secret storage、remote repository version/conflict、encryption、offline queue、audit和notification delivery，再开放产品入口。 |

### 5.5 Desktop shell、Accessibility 与 Security

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| ZHUB-UI-P1-42 | Tauri窗口`decorations:false`，web source没有`data-tauri-drag-region`、`startDragging`或对应permission。当前自绘标题栏没有可见移动窗口路径。 | 恢复原生decorations，或实现经过Tauri capability许可的drag region、double-click maximize、system menu和自动化窗口测试。 |
| ZHUB-UI-P1-43 | sidebar在state collapsed或小于980px时把label设为`display:none`，nav item没有`aria-label`/tooltip；media collapse又不更新state，底部按钮仍可能宣称“收起”。 | 响应式状态与可访问状态统一；icon-only nav始终有name、tooltip、current page和expand/collapse真值。 |
| ZHUB-UI-P1-44 | `HubTreeView`没有tree/treeitem/group role、level、expanded、selected或方向键导航；静态`HubList`把所有行标成disabled；`ProjectTable`整行onclick没有tab stop/Enter/Space。 | 实现WAI-ARIA tree/listbox/table interaction pattern，roving focus和完整keyboard selection；只对真实不可用action使用disabled。 |
| ZHUB-UI-P1-45 | 66个文件只有6处显式`aria-*`，没有自定义role/tabIndex/onKeyDown；popover trigger缺expanded/controls关系，路径大量`noWrap`且没有统一查看/复制。 | 建立accessible component contract，覆盖name/role/value、focus return、live region、zoom/high contrast、screen reader和keyboard-only流程。 |
| ZHUB-UI-P1-46 | `tauri.conf.json`把CSP设为`null`。当前React虽会escape text，但未来Learn/Marketplace/Cloud引入远程内容后，任何XSS都直接处于无CSP桌面WebView。 | production使用最小CSP、禁止任意remote script/eval，远程内容sandbox/sanitize；Tauri command按窗口/来源/动作继续最小授权。 |

## 6. P2：维护性与产品质量债务

| ID | 当前差距 | 建议收敛 |
|---|---|---|
| ZHUB-UI-P2-01 | fallback复制了大段中文产品数据、Settings text和coming-soon，Rust `ui_text.rs`、TS types与fallback形成三重维护。 | 只保留最小dev fixture，由versioned schema fixture生成；production bundle不带伪业务状态。 |
| ZHUB-UI-P2-02 | catalog limits、recent limits、popover只显示2个fallback engine等数字散落，UI无来源说明。 | 统一query policy和server-provided page metadata；限制值进入metrics与测试。 |
| ZHUB-UI-P2-03 | `package.json`对Emotion和Node types使用`latest`；lockfile虽固定当前安装，重建lock时仍会无意跨版本。 | 所有direct dependency使用审定范围，Renovate/Dependabot式升级由测试和bundle diff gate。 |
| ZHUB-UI-P2-04 | package scripts只有dev/build/typecheck/tauri，没有lint、format check、bundle budget、license inventory或dependency audit。 | CI加入ESLint、format、dead-code、bundle size、license/SBOM和安全扫描。 |
| ZHUB-UI-P2-05 | 大量路径/名称使用`noWrap`，截断后缺统一tooltip、copy path或reveal action；错误诊断也只进console。 | Long-value component提供可访问tooltip、copy/reveal和support bundle ID。 |
| ZHUB-UI-P2-06 | 两处width/transform transition没有`prefers-reduced-motion`策略。 | theme统一motion preference，reduced模式关闭非必要transition。 |
| ZHUB-UI-P2-07 | sidebar collapse、Catalog search/tab、各页tab都是易失本地state；窗口/页面重建后工作上下文消失。 | 将纯偏好按用户持久化，将query state按route管理；不要把server authority存入浏览器storage。 |
| ZHUB-UI-P2-08 | window minimize/maximize/close promise被`void`丢弃，失败会成为unhandled rejection；engine/user popover也没有操作中或失败反馈。 | window adapter返回typed outcome并显示非阻塞诊断，重要close继续进入上一报告的session shutdown coordinator。 |

## 7. 参考实现差异

| 参考 | 可确认的工程原语 | 对Zircon的约束 |
|---|---|---|
| Unreal Project Browser / Asset Registry | `SProjectBrowser`有typed project status、filter error、refresh、keyboard handling和engine tooltip；`IAssetRegistry`提供async loading状态、query、scan、added/removed/updated/files-loaded event；registry state有versioned serialize、dependency和filter。Slate/ApplicationCore另有accessible widget、focus、name/help和platform accessible user体系。 | Hub不应每页重扫文件并复制完整tree。Project shell、Asset registry和Accessibility必须是可查询、可增量、可诊断的长期service，而不是CSS和数组投影。 |
| Godot Project Manager | Project List明确把递归scan放到可取消thread，显示scan progress；project item区分missing/version/unsupported，并直接向AccessibilityServer发布listbox/list option role、index、selected和click/focus/scroll action。 | Zircon至少要达到异步可取消扫描、显式degraded item和完整keyboard/screen-reader语义。MUI默认样式不能替代自定义tree/table contract。 |
| Fyrox Project Manager | manager持有`Child`并轮询command queue，project size另开线程，settings有独立load/save窗口和错误反馈。 | Fyrox体量较小，不是最终上界，但仍证明耗时查询和process ownership不应塞进同步UI/session锁。 |
| Bevy Asset | 本地`bevy_asset`提供typed asset ID/path/meta/event/loader/saver等runtime原语，但没有可比的商业Hub、账号、更新器或项目管理器。 | 可参考typed asset change和meta，不从缺失Hub推导Zircon可以省略安装、权限、更新或catalog控制面。 |
| Unity Graphics | 本地`dev/Graphics/Packages`是渲染package源码，不含Unity Hub、Package Manager服务端、账号或Editor accessibility完整owner。 | 后续只对具体Graphics package consumer做比较；本报告不猜测闭源Unity Hub，也不拿缺失源码为当前占位实现背书。 |

## 8. 目标架构

### 8.1 Bootstrap与状态协议

```text
Native Shell
  -> BootstrapState(protocol negotiation, backend/store health)
  -> Versioned Snapshot(revision, capabilities, entity generations)
  -> Typed Delta Stream(sequence, operation id, resync token)
  -> UI Reducer(monotonic apply, degraded/error routes)
```

demo fixture、backend unavailable和protocol mismatch必须是三个不同的host mode。任何decoder失败都应fail closed，危险command disabled，support bundle仍可导出。

### 8.2 Hub只消费权威服务

```text
Project Registry       Engine/Update Service
Asset Registry         Plugin Package Registry
Documentation Index    Source Control Provider
Auth/Team Provider     Cloud Repository Provider
        \                  /
         Query API + paged read models
                    |
              Zircon Hub UI
```

Hub负责安装、选择、查询、诊断和协调，不重新实现Editor asset importer、Plugin Manager或Git成员模型。所有provider都必须有Unavailable/Partial/Stale/Ready状态和离线策略。

### 8.3 Settings transaction

`OpenDraft(base_revision) -> PatchField -> AsyncValidate -> PreviewEffects -> Commit(expected_revision) -> DurableStore -> PublishSnapshot`。Catalog重扫和engine registration属于prepare effect，只有全部成功才能替换active config；失败保持旧配置并保留draft和diagnostic。

## 9. 分阶段重构路线

### M0 · 恢复可执行证据与fail-closed bootstrap

- 先修Hub 01的Rust编译P0并恢复managed Windows build/test。
- 删除production fallback吞错；加入Booting/BackendUnavailable/ProtocolMismatch页面。
- 建立Rust JSON fixture与TS runtime codec契约测试，真实Tauri visual matrix恢复为release gate。

### M1 · Versioned protocol与UI reducer

- 引入protocol/state revision、operation ID、typed delta/resync和subscriber health。
- 将task/notification从全局单槽拆为per-operation projection。
- 对乱序、重复、丢event、断线重连和schema upgrade做确定性测试。

### M2 · Catalog authority与规模

- Hub递归scanner退场，接入共享Asset Registry、Plugin Package Registry和Documentation Index。
- 异步增量index、watcher、fault isolation、cursor query、虚拟化和freshness状态。
- 建立10k plugin、100k asset、百万文件根和坏manifest/permission/network-volume基线。

### M3 · Settings transaction与toolchain probe

- draft revision/CAS、field patch/debounce、dirty/close决策和恢复。
- versioned toolchain capability probe与jobs资源policy。
- durable commit后原子swap，故障注入覆盖scan/persist/register各阶段。

### M4 · Truthful product services

- Update、Marketplace、Auth/Team、Cloud Repository、Notification以provider和capability matrix落地。
- 未实现能力不再借完整页面/metric暗示可用；PII、secret、RBAC、audit和offline先于入口开放。
- Team改为Source Control + Organization两个明确domain。

### M5 · Desktop、Accessibility、Security 与 release gate

- 修复custom titlebar drag/system menu/window failure handling。
- 完成tree/table/navigation keyboard和screen reader contract。
- 启用CSP、remote content sandbox、dependency/SBOM/bundle budget；Windows scaling、高对比、200% zoom、reduced motion进入发布矩阵。

## 10. 验收门

1. production Tauri后端不可达或payload不兼容时绝不显示demo“Ready”，所有破坏性action不可用。
2. Rust schema变更必须同时生成TS codec并通过旧/新protocol兼容fixture；unknown nested field和值有明确策略。
3. state/event revision单调；乱序、重复、丢包和重连不会回退UI或丢失terminal operation。
4. emit/subscription失败进入可见connection health，能手动resync并导出correlation/support bundle。
5. 连续两个同tone success/error都会产生独立可访问通知；running任务不会静默auto-hide。
6. 单个无权限目录、坏UTF-8文档或坏plugin manifest只降级对应entry/root，不阻止Hub开窗。
7. catalog scan可取消、有deadline和资源预算；全局session锁内不执行filesystem traversal或Git process。
8. Asset/Plugin/Learn query返回total/cursor/freshness/truncation，UI只渲染viewport附近节点。
9. 100k asset、10k plugin和百万文件根下，输入、切页和窗口移动保持既定frame/latency预算且内存有上限。
10. Assets/Plugins/Learn切换不会继承非法tab/query/expanded state；duplicate domain ID在server admission被拒绝。
11. 普通项目不再显示Elysium封面、猜测platform或伪`1.8.2`；Unknown/Unresolved有明确状态。
12. Settings快速输入、并发响应和save race不会lost update；stale patch由revision检测。
13. Settings commit任一阶段失败，active config/engine/catalog和disk仍全部保持旧revision；draft可继续修复。
14. Toolchain health验证resolved binary version/target/capability；不存在或不可写目录不能获得100% success。
15. jobs超出host policy会在UI和server两侧拒绝，不会把65535传给构建系统。
16. Git hang能timeout并终止process tree；Team不会把contributors伪称organization members，也不会默认暴露完整email。
17. Cloud readiness不会把failed history或任意字符串path算通过；所有command由server capability决定enablement。
18. Update/Marketplace/Cloud开放前必须通过signature、rollback、offline、RBAC、secret和audit测试。
19. undecorated Hub可用鼠标和键盘移动、最大化、打开system menu；window API失败有可见反馈。
20. sidebar、tree、list、table、dialog和popover通过keyboard-only、screen reader、focus return、200% zoom和high-contrast测试。
21. production CSP非空，远程文档/marketplace内容不能执行script或调用未授权Tauri command。
22. `npm` unit/component/E2E/a11y、managed Cargo test、真实Tauri截图矩阵、bundle/security gates全部纳入CI。

## 11. 本轮未闭合范围

- 因Hub Rust编译P0和当前binary缺失，未运行真实Tauri窗口、WebView事件、keyboard/screen reader、DPI、高对比或visual reference comparison。
- 未运行真实慢盘/网络盘、Git hang、10k plugin、100k asset、百万文件、IPC洪水或内存/frame benchmark。
- 未连接远程账号、云仓库、marketplace或update server，因为当前代码中没有这些provider；本报告只定义缺失authority和验收门，不虚构外部实现。
- 本报告完成Hub web/UI/catalog/settings/team/service shell首轮纵向审查；后续仍需复核Hub installer/update签名、跨平台bundle、真实网络协议与发布运维，并在相关实现出现后重读本报告。
