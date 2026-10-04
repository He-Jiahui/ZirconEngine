---
related_code:
  - zircon_editor/src/tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
tests:
  - zircon_editor/src/tests current physical inventory 490/490 statically read
  - current incremental test owner inventory 16/16 statically read
  - editing test inventory 40/40 statically read
  - ui test inventory 110/110 statically read
  - host/manager test inventory 16/16 statically read
  - host/template_runtime test inventory 18/18 statically read
  - host/retained_event_bridge test inventory 6/6 statically read
  - host/retained_asset_refresh test inventory 4/4 statically read
  - host asset/resource/render boundary inventory 4/4 statically read
  - host/retained_callback_dispatch test inventory 54/54 statically read
  - host/retained_window test inventory 29/29 statically read
  - native template-hover clone preflight source contracts 2/2
  - host retained pointer short-directory inventory 37/37 statically read
  - repeated drawer-resize extent no-op source contract 1/1
  - host/retained_menu_pointer test inventory 40/40 statically read
  - unchanged asset-content pointer UI-writeback source contract 1/1
  - host/retained_host_page_pointer test inventory 6/6 statically read
  - host/retained_tab_drag test inventory 9/9 statically read
  - host/ui_asset_editor_theme_tooling test inventory 5/5 statically read
  - UI asset single-theme-action move source contract 1/1
  - current-source Windows Cargo and managed performance tests pending
doc_type: implementation-evidence
status: incremental_static_complete_dynamic_pending
---

# Editor external tests当前新增owner静态审查（2026-07-22）

当前`zircon_editor/src/tests`物理清单490个文件；相对Git index新增16个owner、删除1个旧toolbar聚合owner。本轮完整阅读新增 **16/16**：editing play/selection/viewport、editor-event retention、message-bus backpressure、project-generation projection、toolbar breakpoint拆分7个、Welcome screenshot与runtime-event bounded pump。

有效性能门：message-bus ignored benchmark覆盖1/5/100 subscribers、1MiB payload、10k latest storm、allocation/RSS/queue-age/publish-p95；retention覆盖10k paused listener、record/byte/age budgets。它们应在current-source受管门运行并保留原始输出。

待修合同：runtime-event test首泵断言gateway 10条全部drain、只apply 3条，10k benchmark也先把全部delivery放入FakeGateway Vec；这证明PERF-MVP-069仅预算callback、不预算ABI ingress/bytes。Runtime10/Editor02改cursor+max events/bytes后必须同步测试为每tick drained也有界，并增加64MiB payload、producer>consumer 60s的pending bytes/age/RSS门。Welcome generation测试只覆盖迟到/取消/typed failure，不覆盖1M draft admission storm，回链PERF-MVP-559。

viewport/toolbar/project其余测试为行为、像素、generation与错误链合同；部分visual tests显式ignore并写artifact，不算自动产品性能证据。current-source Cargo、ignored managed benchmarks、F0/F4产品trace与RenderDoc未完成，整个`src/tests`继续pending。

后续旧目录增量：完整阅读`commands` 5/5、`structure_convention` 1/1及root `jobs.rs/mod.rs` 2/2。命令退役符号守卫原把全部editor Rust源码拼为一个巨型String，本轮改逐文件检查；两个structure tests原各spawn一次相同Python全仓审计，本轮用`OnceLock<Value>`复用一次结果。源码合同2/2与直接audit `classified-and-clear`通过；Cargo test binary仍待受管门。

Gateway旧目录 **4/4** 也已完整阅读。handle用ArcSwap generation/capability Arc且无shared read lock，进程内borrowed World无serialization并有reentry guard，均为合理快路。Session测试只以1×1 RGBA锁定foreign buffer恰好释放且frame在provider unload后仍可读，这个安全合同不能删除；PERF-MVP-023必须用GPU/generation handle替代正常viewport copy，并保留显式fallback的一次有界copy。测试缺1080p/4K copied-bytes、readback/reupload、caller lock/p95门；plugin event ABI仍返回完整owned JSON batch，回链PERF-MVP-069。

`editor_message`外部tests **10/10** 已完整阅读：其中backpressure benchmark具有fanout payload共享、lane count/bytes、allocation/RSS/age/p95硬门；其余9个覆盖typed publish/request/broadcast、dirty merge、reentrant handler锁外调用与target revalidation。未发现新生产根因；`deliveries_for`浅clone只在test配置使用，生产继续drain。current-source managed benchmark仍待执行。

`editor_asset_type_registry`外部tests **9/9** 已完整阅读。typed authority、冲突原子性、generation cache与单次contribution内批量排序合同有效；但1,000-entry测试反向逐条`apply_contribution`并要求generation+1000，单条binary insert的Vec搬移累计O(N²)，也把一次plugin/catalog generation拆成千次发布。新增PERF-MVP-562要求transactional batch contribution/reload；旧单条API语义保留但规模验收不能只证明“无full sort”。

`editor_event`外部tests当前 **21/21** 已完整阅读（含先前已读的新增`retention.rs`）。retention分级预算、共享payload、sequence/ack/lag、extension注册原子性、事件归一化/回放与animation no-op合同均有效；但listener查询仅以1–2条delivery验行为，产品路径会先把最多1,024条共享记录展开成owned delivery，再二次展开为JSON，且`notify`仍在全局listener锁下逐owner过滤/入队，继续归PERF-MVP-067。extension/animation测试均为单个或极少descriptor/track/node，不能替代PERF-MVP-538/562/084/215的1k/10k规模、clone bytes、generation build与主线程p95门。测试harness每例重建完整CoreRuntime、改写进程环境并由全局`env_lock`串行，只是验证墙钟成本，复用PERF-MVP-136，不把它误记为产品热点。current-source Cargo、listener polling storm与F0/F4产品trace仍待执行。

`editing`当前 **40/40** 已完整阅读：root 16、`state` 3、`transaction_engine` 9、`ui_asset` 12。事务合同继续证明history按record数而非payload bytes/age约束，selection以JSON authority进入begin/commit/recovery，完整history snapshot复制宽record；100帧gizmo只验最终record=1，却没有stable name/selection clone bytes，继续归PERF-MVP-063/456/549。UI asset测试均为2–3 nodes/rules/imports的小样本；style/tree/inspector/preview/binding/theme细粒度操作反复请求完整`pane_presentation()`、source/document replay、递归schema/表达式依赖图与cascade/compare projection，不能替代PERF-MVP-082/305–311的1k/10k generation、visit/allocation与主线程p95门。

palette drag契约反查生产路径后确认，每次pointer move都重建完整native/component/import palette，只为读取稳定的当前选项；本轮TDD新增`selected_palette_entry`，在选择或document revalidate时刷新，hover resolve直接借用，实际drop只clone单个entry，删除每move和每drop的完整palette build。源码合同3/3、rustfmt与diff check通过；preview projection和slot overlay仍会在target resolution/presentation上按文档规模重建，继续归PERF-MVP-082，不把局部止损误报为闭环。current-source Cargo、1/100/10k nodes/imports的125/500/1000 Hz drag counter、F4产品trace与像素验收仍待完成，因此该目录只进入`pending.md`，不得进入`review.md`。

`src/tests` root当前 **8/8** 已完整阅读：`jobs.rs`、`mod.rs`、两个runtime-event consumer、三个plugin/catalog/authoring owner及`support.rs`。plugin SDK已有同generation extension materialization复用与asset-type registry cache命中合同；runtime consumer旧测试仍以owned `Vec`一次drain全部delivery，进一步确认PERF-MVP-069的ABI ingress/bytes预算缺口。catalog consistency原在多个test中重复扫描全部`zircon_plugins/*/plugin.toml`并重复TOML parse，本轮以`OnceLock<BTreeMap<package_id, manifest>>`把同进程inventory收为一次I/O/parse并提供O(log N) lookup；源码RED→GREEN合同3/3、rustfmt/diff check通过。`support.rs`的flat fixture迁移仍为test-only递归clone，当前未见产品caller；若后续规模fixture证明占比显著，再归PERF-MVP-136按test-process counter处理，不建立产品性能任务。

`ui`当前 **110/110** 已完整阅读：非boundary 51/51，boundary root 18/18、`material_component_lab` 20/20、`zui_asset_governance` 21/21。绝大多数组件、Material和UI asset fixture只有1–7个节点/diagnostic或单次事件，不能替代PERF-MVP-076/082/109/177/219/305–311的1k/10k节点、连续输入与帧预算门。尤其Assets Activity合同明确要求5行样本中视口以下的行仍进入scroll-source geometry，只证明完整offscreen geometry语义，未证明viewport/overscan投影；Editor10与EditorUI06须补1k/10k rows下的visible+overscan materialization、offscreen build=0和scroll p95，同时保留scroll extent与键盘/选择语义。

生产反查发现material renderer诊断计数为每条diagnostic clone feature `String`；本轮改为`HashMap<&str, usize>`借用稳定key，源码RED→GREEN合同1/1通过。该止损不关闭Material/UI投影规模任务：MUI/Material测试仍主要验证静态descriptor/theme/token，缺少stable generation下render/native projection build=0、virtualized range、allocation和产品像素/trace证据。

PERF-MVP-136测试基础设施同步止损：Material Lab约74个prototype的目录清单、source、UiV2 document和Material theme selector改为进程级只读`OnceLock`，inventory/feedback/MUI-X/theme测试不再多轮读盘解析；ZUI governance把editor/runtime生产`.zui`清单与typed document集中为单次扫描/解析，根级与21个子owner统一借用；Material meta fixture改进程内单次TOML parse，Material import graph visited从线性`Vec::contains`改`BTreeSet`。组合源码合同29/29、rustfmt和diff check通过。受管Cargo仍被其他Session的CPU lane预约，当前修复只能记为静态完成、动态待验收；F0/F4产品trace、规模counter、像素和RenderDoc仍未完成，因此`ui`与整个`src/tests`继续只留在`pending.md`，`review.md`不变。

`host/manager`当前 **16/16** 已完整阅读：bootstrap/startup、minimal host及native/optional plugin、project generation、runtime lifecycle、UI asset binding/tree/reference/promotion/theme/session/preview/style/inspector/workspace watcher与support。项目投影已有1,100 locator样例并锁定不clone project snapshot、不在Welcome pane投影探测文件系统；native plugin测试仍只有单package，多次status/completion/registration/export调用继续回链PERF-MVP-537/538/541..548。UI asset manager测试全部是2–4节点/极少imports，单动作反复调用完整`pane_presentation`，只能验行为，不能替代PERF-MVP-082/305..311的1/100/10k generation、visit/allocation/p95门。

workspace watcher反查生产热路径后完成一组局部TDD止损：每个notify path不再为“恰好匹配一个asset root”构造临时`Vec`，poll refresh也不再深clone整批changed asset `String`，刷新归一化接口可直接借用batch；源码合同2/2与rustfmt通过。该修改不关闭PERF-MVP-083：channel仍unbounded、单poll仍drain全部事件，主线程仍同步session scan/stat/read/hash/parse/hydrate且没有reverse dependency generation。current-source Cargo、1k/10k watcher storm、F4产品trace与RenderDoc仍pending，故`host/manager`及整个`src/tests`不得进入`review.md`。

`host/template_runtime`当前 **18/18** 已完整阅读：builtin window/pane/surface、host model、dual-host parity、showcase category/selection/state、pane payload与shared surface。测试反复执行document projection→surface→host model→retained adapter并把整树转成字符串集合，规模主要为27节点或单个showcase文档；`node_by_control_id`仍线性扫nodes，stable event也缺build=0/clone-byte门，产品链继续归PERF-MVP-092/093与EditorUI05既有failure。测试墙钟直接止损：74次showcase binding查找改为单次静态binding projection，7次pane spec查找改为一次CoreRuntime/EditorManager descriptor cache；PERF-MVP-136源码合同由8增至10项通过。缓存只持静态测试metadata，不替代产品compiled-document/surface generation。

`host/retained_event_bridge` **6/6** 与`host/retained_asset_refresh` **4/4** 已完整阅读。前者只以单record锁定effect→dirty-domain/notification映射，不覆盖ABI batch、owned JSON或storm；后者只以0–2条change锁定catalog/details/preview/default-scene refresh plan，生产planner为各输入slice单遍且默认scene locator只parse一次。没有新增独立根因，继续回链PERF-MVP-067/069/076/104的bounded ingress、frame budget与generation验收。

`host/{asset_manager_boundary,asset_metadata,render_framework_boundary,resource_access}`当前 **4/4** 已阅读（`asset_metadata` owner为空文件）。其余是ownership/source contract与2条fake ResourceManager行为测试：确认viewport通过RenderFramework/RHI边界、editor asset与canonical resource DTO owner正确，但不证明4K capture copy、resource record clone或高频查询预算；继续回链PERF-MVP-023/104/500，不新增任务。受管Cargo本轮重试仍因`plugins01-plugin-workspace-lockfile-r1-20260722`预约CPU lane而未进入测试体。

`host/retained_callback_dispatch`当前 **54/54** 已完整阅读。有效快路合同包括：无反馈viewport move不置dirty、相同viewport frame不重建dispatcher、同floating source不重复focus、decorative viewport layer不dispatch、Workbench hover/press/slider/focus只paint-only且不写journal、virtual scene row有created/reused/recycled counter。需要保留这些语义。

规模缺口：layout动作仍普遍layout+presentation dirty；viewport pointer move即使无反馈仍逐event写journal；module/category/popup/scene/inspector sync主要验证单动作最终状态，未记录同generation projection/surface/host-row build、clone bytes或125/500/1000 Hz storm；唯一>1,000节点startup profile被ignore且仅打印墙钟，无预算断言。继续回链PERF-MVP-067/069/076/077/093/099及EditorUI08现有full-reflection failure，不新增重复根因。

PERF-MVP-136再完成一项测试侧止损：`workbench_projection.rs`和`status_bar.rs`的contract-node helper不再通过`ModelRc::row_data`为每个线性probe深clone全部经过的宽`TemplatePaneNodeData`，改用`ModelRc::get`借用；两个临时model调用显式延长owner生命周期。测试查找仍为Q×N，后续应以一次control index解决；本轮只删除clone放大。测试基础设施源码合同11/11、rustfmt与diff check通过，current-source Cargo仍未执行。

`host/retained_window`当前 **29/29** 已完整阅读。该目录覆盖native host input/popup/tab/resize/text/viewport、Workbench reference、窗口/presenter、Material/MUI painter与debug reflector。已有MVP快路合同包括：idle wait不无条件request redraw；相同template/hierarchy hover不重绘，100次同目标hierarchy move不重建presentation；viewport move/button/scroll只进入runtime input并等待新viewport image；text edit只损伤input frame；menu、tab、drawer、floating和close prompt均返回有界damage而不是默认整窗。必须保留这些行为。

缺口仍归PERF-MVP-033/092/093/099/101/164/177..211：像素测试多为1–12节点小样本，全分辨率Workbench测试只扫像素，没有stable-generation command/build/cache-hit/clone-byte预算；平台input没有125/500/1000 Hz allocation/coalescing门；closed popup/dialog/notification只证明零像素，不证明hidden node visit/build=0；viewport image只证明2×2合成，不证明1080p/4K same-generation upload/copy=0；debug reflector仍缺diagnostics-off与10k node materialization门。

生产反查直接确认PERF-MVP-164：`get_host_presentation()`注入hover时，原先四个dock即使无目标也逐pane把全部宽`TemplatePaneNodeData` clone到临时Vec后丢弃，floating layer也会先clone全部window/pane。本轮RED→GREEN加入借用行命中预检：无目标dock只借用扫描，无目标floating layer直接返回；只有命中model才物化替换Vec。源码合同2/2、相关组合合同15/15、rustfmt与diff check通过。命中pane仍需全model替换，最终必须由EditorUI08的stable control index + transient old/new hover generation消除结构ModelRc重建；Cargo、1/100/10k node clone/visit counter、F4 trace和像素/RenderDoc仍pending，因此目录不得进入`review.md`。

Retained pointer短目录本轮共 **37/37** 完整阅读：`retained_activity_rail_pointer` 6/6、`retained_detail_pointer` 5/5、`retained_document_tab_pointer` 4/4、`retained_drawer_header_pointer` 6/6、`retained_drawer_resize` 6/6、`retained_list_pointer` 5/5、`retained_viewport_toolbar_pointer` 5/5。已有正向合同锁定layout/state unchanged不重建、detail scroll不重建两节点surface、document/drawer measured frame借用且重复测量no-op、resize target frame immutable/lock-free、viewport click只upsert被测control。样本只有1–12行/标签，未覆盖1k/10k虚拟列表、125/500/1000 Hz scroll/resize、跨dock规模或stable-generation build/clone预算；继续回链PERF-MVP-092/093/131/163/172与EditorUI08。

生产回查`drawer_resize/movement.rs`确认PERF-MVP-172局部根因：捕获期间相同坐标/extent仍无条件写transient map并`mark_layout_dirty()`。本轮RED→GREEN以已有transient值或capture base作为previous authority，相同preferred在更新pointer capture后直接返回，不写map、不置layout dirty、不切pointer layout；release仍以latest值提交。源码合同1/1、rustfmt/diff通过。raw event入口的presentation clone、changed storm逐event layout，以及release时left/right双`SetDrawerExtent`事务仍分别由PERF-MVP-172/131保持open。

`host/retained_menu_pointer`当前 **40/40** 已完整阅读，含menu dispatcher/pointer/layout/surface合同、29个组件/状态视觉owner、综合`visual_screenshot.rs`及其Asset Browser、Assets Drawer、Blend Space子owner。已有MVP正向合同锁定closed menu不重建、committed item tree借用、route index按层线性推进、scroll无owned route clone，以及Asset Browser/Drawer scroll只损伤内容区、same hover只改变目标项。多数视觉owner只用约10–30个节点、3–4个popup/list row、4x4/5x5预览图；Blend Space只覆盖固定8 samples、16x10 heatmap和6次transport click。它们不能替代1k/10k行虚拟化、真实1080p/4K image upload/copy、125/500/1000 Hz input、stable-generation build/clone/cache与产品帧时门，继续回链PERF-MVP-109/112/177..211和EditorUI08。

生产回查资产内容事件链确认PERF-MVP-102局部冗余：`write_asset_content_pointer_state`即使hover/scroll state完全相同，也会再次写回状态并调用`apply_asset_pointer_state_to_ui`，后者重复设置tree/content/references/used-by共8个Slint属性。本轮源码RED→GREEN加入same-state early return；真实跨行hover和scroll offset变化仍走原完整同步。编辑器性能源合同组合21/21、rustfmt/diff通过。有效scroll仍按全部资产重建pointer `UiSurface`且virtualization为`None`，继续由PERF-MVP-109/EditorUI01负责；current-source Cargo、1k same-row move property-write counter、10k row scroll build/alloc、F4产品trace与像素/RenderDoc仍pending。

`host/retained_host_page_pointer` **6/6** 与`host/retained_tab_drag` **9/9** 已完整阅读。前者锁定unchanged layout不重建、click不维护measured frames、overflow打开只paint-only；后者锁定document edge优先、drawer/floating attach/split、stale geometry不回退和route-intent单一命中面。样本只有2–4 pages、最多3 tabs和1个floating window。生产回查确认drag move已有same target group不set state，但每move仍生成group key并读取完整drag state；release又从全部目标tabs clone `instance_id/title/host/path`构造临时strip。继续回链PERF-MVP-106/108/131/169与EditorUI08，需1/100/10k pages/tabs/windows的route probes、String/DTO clone bytes、same-target UI writes和125/500/1000 Hz p95。

`host/ui_asset_editor_theme_tooling`当前 **5/5** 已完整阅读。7个行为测试实际每例创建临时project、scan/import、默认world和CoreRuntime，并仅用2 tokens/1 rule验证local/imported compare、batch adopt/prune/refactor语义；测试初始化成本归PERF-MVP-136。产品批量helper本身只clone文档一次并一次性apply/replay，没有per-item compile/save；但helper index查询与pane presentation都会重建完整typed helper/refactor/cascade action Vec，再生成全部String labels，点击只取其中一项。本轮先将两个单项lookup由`get(index).cloned()`改为消费临时Vec的`into_iter().nth(index)`，删除选中action内reference/token/selector字符串的二次深clone；源码合同1/1、组合22/22、rustfmt/diff通过。全列表问题继续纳入PERF-MVP-305与Editor07：按document/import/selection generation缓存typed actions和stable action id，presentation只投影可见labels，mutation后精确失效。需1/100/10k tokens/rules/imports的action builds/scans/label bytes/p95与save/undo/redo parity；current-source Cargo仍未执行。

至此`zircon_editor/src/tests`当前物理清单 **490/490** 已逐Rust文件静态复核：root 8/8、commands 5/5、editing 40/40、editor_asset_type_registry 9/9、editor_event 21/21、editor_message 10/10、gateway 4/4、host 242/242、structure_convention 1/1、ui 110/110、workbench 40/40。该结论只代表源码阅读、计划归因和已列局部止损完成；current-source受管Cargo、ignored benchmark、1/100/1k/10k规模counter、F0/F4产品trace、像素与RenderDoc尚未闭环，因此整个目录继续留在`pending.md`，`review.md`不得更新。
