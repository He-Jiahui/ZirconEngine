---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: editor-event-full-reflection-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/build_host_activity_rail_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/collect_tabs.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/build_host_drawer_header_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/build_surface.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/update_measured_frame.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/build_host_page_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/tab_strip_geometry.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/resize_surface.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target/state.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces/ui_writeback.rs
  - zircon_editor/src/ui/retained_host/tab_drag/strip_hitbox.rs
  - zircon_editor/src/ui/retained_host/tab_drag/host_resolution.rs
  - zircon_editor/src/ui/retained_host/tab_drag/route_resolution.rs
  - zircon_editor/src/ui/retained_host/viewport/world_space_ui.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_editor/src/ui/workbench/shell_state.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/model/build/workbench_view_model_build.rs
  - zircon_editor/src/ui/workbench/reflection/model_build.rs
  - zircon_editor/src/ui/workbench/reflection/transient_ui_state.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/responsive_layout.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation/scene_conversion.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry
  - zircon_editor/src/ui/retained_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover/nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics/planned_present
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes
  - zircon_editor/src/ui/retained_host/host_contract/globals
  - zircon_editor/src/ui/retained_host/host_contract/redraw
  - zircon_editor/src/ui/retained_host/host_contract/window
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes
  - zircon_editor/src/ui/retained_host/primitives.rs
  - tools/ui-profile-capture.ps1
reference_sources:
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
  - dev/slint/internal/backends/winit/accesskit.rs
  - dev/slint/internal/core/model/repeater.rs
tests:
  - per-effect reflection projection build-count regression
  - pointer/draft/selection 1000-event coalescing and p95 stress
  - incremental snapshot byte/order/route parity matrix
  - capture-disabled 1000-present zero-artifact-work trace
  - one-shot artifact worker and 1/100/10000-control hit-route scale matrix
  - unchanged asset-content pointer UI-writeback source contract
---

# EditorUI08：每 editor event 同步全量 reflection rebuild

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`ui/host` root 37/37、`editor_event_execution` 13/13、`ui/workbench` 327/327 与 `ui/retained_host/app` 451/451 静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：reflection generation、dirty-domain routing、retained snapshot caching 与 frame coalescing 属于 EditorUI08 workbench projection owner。

## 失败现象与复现证据

`dispatch_normalized_event_with_operation` 在返回/record 前同步 `refresh_workbench_for_effects`；`refresh_view` publish dirty 后立即 `drain_pending_view_refreshes`，只要任意 dirty 就调用完整 `refresh_reflection`。mask 仅被记录，不参与增量选择。全量路径在持有 shell + command locks 时 clone descriptors/layout/view instances/capabilities/extension registries，重复 materialize asset registry、构建 chrome/view model/reflection routes/snapshot，并发布整份 snapshot。

这会让 pointer moved/hover、draft typing、selection、status/progress 等高频事件每次同步重建整个 workbench。即使 event effects 为空，默认 mask 仍回退为 `PRESENTATION_DATA`。Slint 的 AccessKit projection 先检查 global/node `PropertyTracker::is_dirty`，再只对 dirty cached nodes `evaluate_if_dirty`；repeater 也对 row change 定点更新或标 dirty，而不是每事件重建整树。

Workbench 下游进一步确认了放大链：`WorkbenchShellState` 用单 mutex 同时保护 editor state、manager、transient、routes 与 extensions；`EditorState` 生成完整 scene/inspector/assets snapshot，`WorkbenchViewModel` 再复制 active page、drawers、tabs、menu，reflection 又复制 payload/actions/routes。transient hover/focus/drag 虽已消除逐 node String/property descriptor 重分配，仍会扫描整份 reflection node map。本次已修复 hierarchy O(ND)、双资产表面重复投影与若干深拷贝，但这些局部优化不能代替 domain generation。

Retained host 451 文件审查补充了同一链的直接 consumer：每次成功 render submission 都只为 diagnostics 无条件 `mark_presentation_dirty()`，即使诊断 pane 不可见也会把下一帧送入完整 slow recompute；任何非 paint-only dirtiness 又串行重建 layout/chrome/model/geometry、root/workbench template bridges、全部 pane payload/pointer surfaces/native presenters。viewport resize 还会在同次 recompute 内第二次构建 chrome/model。`active_activity_window_template_document_is` 为每次 componentized workbench control/edit/option gate 构建完整 chrome，Inspector 单对象 drag/apply 也读取完整 editor snapshot。

2026-07-30非startup `host_lifecycle` current-source 32/32复读确认上述链仍在，并补齐native/window放大：pure paint-only已经准确短路，这是必须保留的边界；但非paint slow path仍先准备main pane payload与presentation，存在native target时又重复准备Module/Plugin、Build/Export与component showcase。native store无per-window applied generation，每次为target/stale分配集合并对全部target完整apply；floating bounds与host bundle同样每slow path全量同步。成功submit的源码测试仍要求`mark_presentation_dirty()`，故不能直接删除而丢post-submit diagnostics，必须先分离render-stats generation。

同轮还确认pending decision在`tick()`与每次dispatch side-effect结束后两处调用；每次先走会构造完整chrome的active-template gate。它与PERF-MVP-596的stable pending全投影叠加，使交互帧可能重复支付同一decision/template工作。EditorUI08必须以captured generation在每frame最多apply一次，而不是仅优化decision center内部。

Callback dispatch 135 文件审查进一步确认 template bridge 的放大：Workbench recompute 先执行 root/workbench surface layout，应用 drawer/responsive/data sync 后又对 template surface执行 layout；data sync即使多数值未变，也为每个 row/property走 control lookup和mutation。floating focus/drawer/viewport toolbar等单一动作会构建或 clone完整 chrome/layout/model snapshot，只为读取一个 target/settings。此次已让 floating source同尺寸重算 no-op，并消除 responsive tier逐节点 lowercase分配，但 full model snapshot、typed delta和每帧一次 template layout仍必须由本计划统一解决。

Retained presentation core审查补充最终 replace成本：`apply_presentation` 先 `get_host_presentation()`深 clone完整旧 scene/native/window node DTO，只为保留menu/focus/viewport等少数字段；随后重新构造ShellPresentation，四个 dock与floating pane又clone中间PaneData，再整份 `set_host_presentation`。本轮直接消除了 ModelRc唯一source二次clone、template mapping源行clone、Welcome只读窗口clone，并把workbench父链索引收敛到近O(N)；未变generation仍全量转换/replace的问题必须由本计划的typed patch解决。

Pane conversion审查还确认 Runtime Diagnostics 的特殊放大：普通pane conversion先建dispatch-only body hit surface；diagnostics refresh再从全部host nodes构造synthetic `UiSurface`、rebuild并snapshot成reflector rows；更新后又重建一次body hit surface。一次转换最多三次全树事务，且dispatch-only surface不能作为all-node reflector的语义替代。PERF-MVP-143要求本计划发布generation-owned完整debug surface snapshot，未变generation不再重复build/snapshot。

Native Workbench hit testing同样缺generation-owned surface：每次pointer event仍重新扫描全部node bounds并新建/填充/rebuild完整hit `UiSurface`，之后才做popup/base hit。PERF-MVP-146已局部消除node/row clone并把uniform popup row定位收敛到O(1)，但presentation generation仍必须持有该surface与open-popup z stack；input event不能成为layout/hit-tree build触发器。

Host-contract data审查补充PERF-MVP-147/148：`get_host_presentation()`在pointer move、scroll、keyboard、present和viewport sync等入口深clone完整presentation；其`PaneData`同时嵌入所有pane payload，viewport image还可能复制RGBA Vec。结构snapshot必须以immutable generation handle读取，interaction与viewport image独立分代，active pane只持有对应tagged/shared payload；窄字段查询不得物化整树。

2026-07-30 viewport current-source 34/34复读量化了该放大：新generation在framework clone stored `CapturedFrame`后，Editor import、未读`latest_image` retain、host `to_rgba8`与DTO `to_vec`再做4次整RGBA copy；4K单份31.6MiB，DTO还全量hash。toolbar click为一个frame深clone完整presentation；stable world-space每frame在controller锁内重建commands并复制至少5个String/command。EditorUI08必须让viewport image、toolbar frame和world-space extract各自以immutable generation/shared handle消费，结构presentation不得承载整帧bytes或成为pointer query接口；见`../../../performance/01/2026-07-30-editor-retained-viewport-current-review.md`。

同日pointer-layout current-source 11/11复读确认“下游no rebuild”仍不是stable零工作：activity/browser先clone两份workspace snapshot并构造8个owned list layout；hierarchy复制scene slice并逐row格式化id；menu保留layout后再clone给bridge；Welcome重收集paths。相等bridge返回前这些成本已发生，随后adapter还无条件做11次同值RefCell写与14个空setter调用。EditorUI08必须以domain generation在调用整组pointer projection前短路；EditorUI01只接changed rows/sizes，见`../../../performance/01/2026-07-30-editor-retained-pointer-layout-current-review.md`。

workbench-pointer current-source 7/7进一步定位到单击路径：floating header用已提交shell hit得到window id后，仍重新构造chrome、project context和完整WorkbenchViewModel并持commands锁，仅为解析一个instance；direct路径不使用已有same-window guard。floating tab activate/close成功返回后又构造完整chrome并线性扫描floating windows记录source id，即使route为空。EditorUI08应随presentation generation发布surface→window/active-instance index和focused identity，单击不得重建model/snapshot；见`../../../performance/01/2026-07-30-editor-retained-workbench-pointer-current-review.md`。

native-windows current-source 4/4把逐window放大继续量化：target collection每slow pathclone id/title/tree形成Vec；store再建全id BTreeSet与stale Vec，existing window无applied generation地全部apply。每个apply依次经历完整workbench presentation、toolbar全presentation clone/replace、native ids/bounds第二次clone/replace，合计至少三次完整presentation事务，并无条件调用OS position/size。EditorUI08必须把main/native pane artifact共享、toolbar/native fields合为一次changed patch，并记录per-window applied target/presentation/bounds generation；见`../../../performance/01/2026-07-30-editor-retained-native-windows-current-review.md`。

Presenter 31文件审查补充PERF-MVP-149：Softbuffer为把diagnostics overlay text写进`host_shell`，每次present再次clone完整presentation；overlay/damage fixed-point最多格式化9次，verbose summary在重复判断前分配。Overlay必须成为独立draw/transient generation，不能通过复制结构树注入。

Chrome command stream 40文件审查补充PERF-MVP-151/152：damage stream仍调用完整workbench painter遍历整份presentation，clip只在primitive阶段剔除；现有“patch不重建static layer”测试只是把patch quad标记为Dynamic。EditorUI08必须从dirty generation与section/spatial index直接形成patch，未变node visited=0。正常recording z-index单调，本轮已让软件replay跳过分配排序；产品fallback sort必须保持0并保留显式乱序stream语义。

Paint frame/primitive 44文件审查补充PERF-MVP-154/155：recording-only方形border已从4或4W个quad收敛为一个typed Border；EditorUI08生成patch时必须保留该typed primitive，不能再次展开。Softbuffer圆角geometry应按row/span预计算，且只在fallback renderer owner实现，不得把全尺寸mask塞回presentation snapshot。

图像payload的资源owner由PERF-MVP-150/153补充：EditorUI08发布的draw item只应持稳定resource handle/UV/generation，不得把同一atlas RGBA按node复制进presentation/command。Render13负责resource registry与上传/失效；EditorUI08负责changed resource/node delta，二者不得各建一份像素权威。

Paint theme 24文件审查补充PERF-MVP-161：palette与metrics分别有199/83处全局`RwLock`调用，分布在86/41个文件，且一次appearance更新顺序写三把锁。EditorUI08的presentation/style generation必须同时发布一个immutable theme snapshot；frame/command build只抓一次并沿paint context借用，node/style helper不得继续全局同步查询或观察mixed theme generation。

Host globals/redraw 32文件审查补充PERF-MVP-162/163：每次presentation原有22个最终为空的pane setter和一个mesh-path空sink，14次仍先完整转换asset/welcome/project模型；本轮已删除23个sink、14类转换和空模块，待Cargo。局部redraw仍把全部分离damage无条件外接成一个矩形；EditorUI08与presenter需共享固定小容量region set，按最小面积增量合并并显式记录升Full原因。

Host window 38文件审查补充PERF-MVP-164至166：hover活跃时每次presentation读取会为workbench、四dock、全部floating panes和popup rows clone/collect结构表；hover必须变成stable-index transient generation。Event storm的native request已改为只在pending false→true时调度并新增边沿测试，待Cargo/trace；`about_to_wait`仍不得轮询并覆写未变window properties。

Profiling artifact/hit route 53文件审查补充PERF-MVP-168/169：当前每次成功present都会进入profile export，关闭时仍读环境，开启后同步重建geometry、pretty JSON、software reference frame、PNG并覆盖文件，直接污染WPR/CPU主线程样本；geometry又clone第二套clickable DTO，为每control生成三组字符串样本并重复扫描tabs/panes/routes。EditorUI08必须把capture建模为显式stable-generation one-shot request，把编码/写盘交给有界worker，并让geometry复用本计划建立的generation-owned control/hit/route index。采集脚本只消费明确的最终交互generation，不能依赖每帧覆盖来取得“最新”结果。

Native routing基础类型审查补充PERF-MVP-174：retained `SharedString`只是`String` alias，editor有1,192个引用点、retained host 816个；所有presentation/node/route/interaction clone仍复制字符串字节。EditorUI08的immutable generation DTO必须把id/label/path/action迁到真正COW或`Arc<str>` shared text，editable buffer/format builder显式保留`String`。迁移须有clone-byte、mutation ownership、serde/hash/order与生命周期门，禁止无界全局intern表。

Native button dispatch 104文件审查补充PERF-MVP-176：`button_dispatch_input`在unsupported button判断和active resize/tab-drag capture release之前就调用`get_host_presentation()`，所以本可早退的event仍深copy完整dock/pane/template/RGBA DTO。局部顺序修复后，正常button route仍必须读取本计划的immutable presentation generation handle；不得为每次press/release重新物化完整host snapshot。Snapshot与image generation需分离，capture release在不需要structure时不能触碰二者。

Native pointer damage/redraw 57文件审查补充PERF-MVP-163/173/176：所有focus/status/center/floating/tab-drop frame最终仍外接成单个矩形；floating、tab和template-node damage discovery还会`row_data`深clone候选；void/bool callback迫使consumer预设整个center/status/sibling pane可能改变。固定容量region set、generation-owned borrowed index和typed mutation damage必须一起落地，否则只优化其中一层仍会在paint/upload或DTO复制处放大。

Workbench renderer 102文件审查补充PERF-MVP-177：componentized chrome按top/status/extension多次扫完整nodes；extension每帧两次深clone整表并重建String parent/subtree map，逐node沿parent chain；Welcome约13次已知control全扫，asset frame/extent也被projector/hover/scrollbar重复发现。Presentation generation必须原子提交parent/subtree/control/clip-section paint index与active extension root，damage stream只访问相交segment；stable frame不得把结构projection推迟到paint callback。

Template command pipeline 61文件及runtime-interface转换支撑2文件审查补充PERF-MVP-178：stable paint仍逐帧执行render-command→paint-element→host-command两次转换、owned payload clone和stable sort；template node对可见fallback还线性探测5个primary、dropdown、22个secondary handler。EditorUI08必须按presentation/damage generation保留compiled、already-ordered paint segments，typed role由Runtime09 extract提供；stable generation的element/host-command build、handler probe、sort和payload clone均为0，changed node只替换对应segment。不得另建不受presentation generation失效约束的host-local cache。

Manual icon glyph shapes 35文件审查补充PERF-MVP-179：asset miss会把一个icon展开为2–8个quad；compiled segment可消除重建但不能消除每帧primitive/draw放大。EditorUI08必须上报shipped MVP icon的resource resolve/fallback count；fallback=0时以守卫防回归，fallback>0时只提交Render13单个mask/atlas handle，不得在compiled segment永久固化多quad展开。

Sprite atlas 9文件审查补充PERF-MVP-180：当前paint resolve逐请求扫描目录、canonicalize/stat manifest、clone完整manifest并解码整张atlas。EditorUI08 compiled segment只能消费Editor10/Render13发布的atlas handle+UV generation；paint callback的filesystem/parse/decode/manifest clone必须为0。

Visual assets 41文件审查补充PERF-MVP-181：pixel cache hit前仍构造candidate并逐path exists，hit后深clone完整RGBA；SVG、retained preview、missing icon与MUI dev-source路径仍把stat/hash/copy/parse/raster放在paint。EditorUI08只消费`resource handle + generation + UV/tint`，stable compiled segment的path/filesystem/cache-lock/RGBA-copy工作为0。

Style selector 157文件审查补充PERF-MVP-182/183：部分selector每个surface/border/text/glyph helper都独立读取全局palette `RwLock`，dropdown一次4次、text field 5次、segmented约9次；list/tree/table的border width重复调用border，table又按cell取锁，button递归state与role override可约3至8次。EditorUI08必须让changed-node compile借用同一immutable theme generation，stable node lookup为0。danger/glyph的`format!+lowercase`和button重复command/tab分类必须收敛为projection-owned typed role；compiled segment不得在paint时再从字符串猜角色。

Material primitives 150文件审查补充PERF-MVP-184/186：generic node每帧顺序探测8类primitive，alert/chip/badge等重复扫描variant、取palette、复制label与测量文字；alert tone还在token loop分配`colorX`字符串。EditorUI08 changed-node compile必须一次生成typed material spec和ordered segment，stable probe/parse/theme/text/build为0。复合glyph/effect的compiled segment只携带Render13 handle或typed effect，不得永久固化10个dot quad并让最终draw放大。

MUI X primitives 53文件审查补充PERF-MVP-187/188：chart每paint分配并软件栅格RGBA，shared quad即使无border也取palette锁，使TreeView等单组件主题读取达7至14次。EditorUI08必须让EditorUI06 typed component generation只在changed时编译geometry并借用一次frame theme；stable component不得调用chart raster或shared theme lookup，chart command只消费Render13 handle/typed geometry。

Material state layer 9文件审查补充PERF-MVP-189：当前idle node在确认overlay/ripple均无工作前已经读取palette。局部早退修复后，EditorUI08 compiled style仍须保证stable generation不调用state-layer builder；interaction change只重建对应node segment并借用一次theme。

Template button/glyph/tests 29文件审查补充PERF-MVP-190：kind与glyph各自分配/lowercase同一六字段key，surface/content各跑一次完整style selector，content几何再约十次读取metrics。EditorUI08 changed-node compile必须一次生成typed `ButtonPaintSpec`并由surface/content/indicator共享；stable按钮不得再分类、取theme/metrics或测量文字。

Template field/stepper/tests 21文件审查补充PERF-MVP-191：search身份在identity/geometry/glyph/text/placeholder之间重复执行，每次最多分配5个lowercase字符串；label构建两次，metrics/theme读取按helper放大。EditorUI08 changed-node compile必须一次生成typed `FieldPaintSpec`并由surface/state/glyph/text/stepper共享；stable field不得再字符串分类、生成label、取theme/metrics或测量文字。

Template icon button/tests 18文件审查回链PERF-MVP-178/179/181/182/183：入口已共享一次context/style，但stable frame仍重复component识别、control-id context匹配、theme/metrics读取及resource/fallback glyph command构建。EditorUI08 compiled segment必须让stable icon button以上计数及surface/glyph command build均为0；changed node每项至多一次。

Template axis controls/tests 43文件审查补充PERF-MVP-192：一个Transform value field每paint分别3次读取metrics与palette，axis label又按node重派生5个RGB tone。EditorUI08 changed-node compile必须一次生成typed `AxisControlPaintSpec`并借用统一theme generation；stable control不得读取/派生theme/metrics、复制value或重建command。

Template inspector row/tests 38文件审查回链PERF-MVP-174/178/179/181：resource row在stable paint复制2至3段text并解析2个icon，fallback各展开3个quad。EditorUI08必须提交typed row/shared text/resource handles并复用compiled segment；PERF-MVP-193另行直接消除shadow bool lowercase String分配。

Template property row/tests 21文件审查补充PERF-MVP-194：典型三轴带单位值在command前约14次Vec/String分配，command再复制7段text，helper图约28次metrics与4次palette读取。EditorUI08 projection必须提交typed values/shared text，并以一次theme snapshot编译`PropertyRowPaintSpec`；stable row parse/allocation/theme/command build均为0。

Template selection control/tests 26文件审查补充PERF-MVP-195：checkbox/radio/toggle对同一node完整selector分别调用3、3–4、4次，geometry/label又约3–5次读取metrics。EditorUI08 changed-node compile必须一次生成typed `SelectionControlPaintSpec`并共享style/theme/metrics/label/geometry/resource；stable control以上计数及command build均为0。

Template slider/tests 35文件审查补充PERF-MVP-196/197：editor/runtime tick loop都可被外部值放大为无界quad storm，必须消费Runtime09定义的共享预算；普通/range slider约7–12次读取metrics并重建文字。EditorUI08 changed-node compile必须一次生成`SliderPaintSpec`，stable静态段为0，percent-only只patch动态段。

Template segmented control/tests 28文件审查补充PERF-MVP-198：options每项owned row后又二次复制、selected lowercase，N options使完整selector达N+2次且metrics按leaf反复派生。EditorUI08 changed-node compile必须一次生成`SegmentedControlPaintSpec`并共享options/selected/style/theme/metrics/geometry；stable以上计数及command build均为0。

Template alert/toast/glyph/tests 29文件审查补充PERF-MVP-199：Toast身份先复制+lowercase label后text再次复制，通用Alert把六字段format+lowercase，固定action text也逐paint分配；warning/close fallback各展开8个quad。EditorUI08 changed-node compile必须一次生成typed `AlertPaintSpec`并共享kind/tone/state/theme/text/geometry；stable identity String、selector/theme、label copy与command build均为0，glyph命令预算与PERF-MVP-179共同收敛。

Template chip/chevron/tests 18文件审查补充PERF-MVP-200：带chevron node每paint约4次palette、9次metrics投影，label owned copy且chevron identity重复探测。EditorUI08 changed-node compile必须一次生成typed `ChipPaintSpec`并共享identity/state/theme/metrics/text/geometry；stable以上计数及command build均为0，三段glyph命令预算与PERF-MVP-179共同收敛。

Template status geometry/control/glyph/tests 34文件审查补充PERF-MVP-201：常驻status chip每paint复制完整label、再分配两段text、测量value并约10次投影metrics；signal约6次，manual glyph发出5–6个quad。EditorUI08 changed-node compile必须一次生成typed `StatusControlPaintSpec`并共享kind/state/theme/metrics/text/measurement/geometry，static/dynamic segment分离；stable以上计数及command build均为0。

Template list row/adornment/tests 20文件审查补充PERF-MVP-202：每行约6次完整selector、3次metrics、label copy并进入resource resolve。EditorUI08 changed-visible-row compile必须一次生成`ListRowPaintSpec`，并消费PERF-MVP-177的visible range与PERF-MVP-181的resource generation；stable visible以上及command build为0，offscreen全部为0，shipped adornment fallback=0。

Template table row/cell/action/tests 28文件审查补充PERF-MVP-203：每cell重算完整4列layout并重取metrics，4列行重复4次allocation和16次固定header文字测量；row/cell/action约8次selector，options二次copy再normalize。EditorUI08 changed-visible-row compile必须一次生成`TableRowPaintSpec`并共享typed cells/column layout/theme/metrics/style/measurement/action handle；stable visible以上及build=0，offscreen全部为0。

Template tree row/geometry/glyph/tests 32文件审查补充PERF-MVP-204：每行约9次selector、15–17次基础theme/metrics投影，guide再约2次/depth，并解析四个resource与copy label。EditorUI08 changed-visible-row compile必须一次生成`TreeRowPaintSpec`并共享typed icon/action/state/theme/metrics/style/label/guide geometry/resource handles；stable visible以上及build=0，offscreen=0。

Template popup row/adornment/tests 45文件审查补充PERF-MVP-205：menu/options对全部row先clone/style/flags，leaf才clip；menu adornment分类两次并重复flag scan、String+lowercase，label/shortcut再copy。EditorUI08先clip-before-clone且单次无分配typed adornment，changed-visible-row compile生成`PopupRowPaintSpec`；stable及offscreen以上与build均为0。

Template section title/glyph/tests 23文件审查补充PERF-MVP-206：icon+strong title每paint约6次metrics/4次palette，label先copy后两层text再各copy，manual icon发出4–6个quad。EditorUI08 changed-title compile一次生成`SectionTitlePaintSpec`并共享typed icon/strong/theme/metrics/label/geometry；stable以上及build为0，列为P1。

Template tooltip/glyph/tests 19文件审查补充PERF-MVP-207：每tooltip约7次metrics与title/body copies，arrow按border+fill逐扫描线发出约2×size quads，hover burst重复build。EditorUI08 changed-visible tooltip一次生成`TooltipPaintSpec`并共享state/theme/metrics/text/geometry；stable target以上及build=0，arrow单mask资源由Render13拥有。

Template notification center 11文件审查补充PERF-MVP-208：header每paint全量clone options统计unread，row loop再全量clone且leaf才clip，visible title/description再copy。EditorLayout09拥有bounded notification generation/unread/overflow；EditorUI08只消费metadata并在row_data前计算visible+overscan，stable/closed/offscreen build均为0。

Template dialog/tests 22文件审查补充PERF-MVP-209：open confirm/alert约14次metrics、8–10次palette并重复variant/severity scan，actions各row_data/String/measurement且title/body copy；closed早退正确。EditorUI08 changed-dialog compile一次生成`DialogPaintSpec`并共享state/severity/theme/metrics/palette/text/actions/layout；stable-open及closed build为0，列为P1。

Template drag overlay 9文件审查补充PERF-MVP-210：inactive早退且active固定≤4 commands/无theme lock，但每move复制同一payload label并重建surface/icon/text。EditorUI08 drag-start/payload generation一次生成`DragOverlayPaintSpec`，same-payload move只patch frame/indicator动态段；需补专用测试，列为P1。

Template command palette 39文件及registry/open-state回查补充PERF-MVP-211：open materialize多份完整catalog；paint全row先clone后leaf clip，每visible row约5–6次metrics并copy label/detail。Editor08拥有immutable catalog/search result；EditorUI08在row_data前取visible+overscan并编译row spec，stable/offscreen build均为0。

本轮只做了 consumer 层确定安全的去重：asset/hierarchy/detail pointer 复用 committed projection，重复尺寸不重建 layout；pane payload 共享一次 visible instance snapshot；重复 drag target 与 palette unchanged hover 不再 publish/置脏。它们不能替代 shell/presentation generation authority。

## 最低共享层根因

dirty mask、message bus 和 reflection snapshot 没有 generation/consumer cursor：invalidations 被立即 drain 成“任何 dirty = full snapshot”，各 projection 也没有按 domain 缓存的 immutable generation。

## 架构修复验收

- event execution 只合并 dirty domains；每帧/明确 flush 最多发布一次 snapshot，pointer/typing burst 可合并但按钮/transaction 边沿保序。
- layout/presentation/render/asset/command/extension projection 分代缓存；只重建依赖 dirty generation 的 rows/nodes。
- transient update 按 node/path reverse index 定点 patch；未变化 node visited count=0，不能用整树扫描伪装成增量。
- shell/command locks 仅用于获取稳定 snapshots，不跨模型构建、JSON/route materialization 或 publish。
- 1k pointer/draft/selection storm 记录 projection build count、clone bytes、lock wait 和 interaction p95；未变 domain build=0，单帧每 domain build≤1。
- 全量 fallback 仅用于 bootstrap/schema/generation incompatibility，并有计数/原因；route/action/snapshot bytes 与当前语义等价。
- render stats 使用独立 generation/counter；正常 render success 不置 presentation dirty，只有可见 diagnostics/capture consumer 读取，稳定 viewport presentation rebuild=0。
- active template、selection/inspector identity 与 pane visibility 来自 committed generation/index；1k control/edit/option/press 不构建完整 chrome/editor snapshot。
- 同一帧 viewport resize 后 chrome/model build 总数仍为 1；payload、pointer、native-window domain 只消费各自 dirty generation。
- main/native presenter共享同一generation-owned pane artifact；Module/Plugin、Build/Export、showcase每generation总build不超过1，native target只在其applied generation落后时apply，unchanged bounds write=0。
- pending decision/template gate每frame与每decision generation最多执行一次；dispatch只合并dirty generation，不同步重跑stable projection。
- invalidation diagnostics只在counter generation变化时publish；33个stable pointer入口的setter/`RefCell` write=0，paint/slow/render新counter在下一present前各提交一次。
- root/workbench/template surface 每 dirty generation 每帧 layout 总数各≤1；drawer/responsive/data sync提交 typed delta，不以第二次全树 layout收敛。
- floating focus、drawer toggle、viewport toolbar route只读 committed identity/settings/geometry handle，不构建或深 clone完整 chrome/layout/model snapshot。
- presentation owner分别保留结构generation与交互state；更新结构时不先clone完整旧presentation，四 dock/floating只patch changed pane/window，未变 pane conversion=0。
- pointer/keyboard/scroll/present reader只clone immutable presentation handle并在任何reentrant mutation前释放state borrow；1k事件full DTO clone与RGBA copied bytes均为0。
- unsupported button与active resize/tab-drag captured release在presentation acquisition前早退；1k该类event的presentation handle/DTO/RGBA clone均为0，final resize point与drop/cancel语义等价。
- stable presentation generation的paint topology/parent/subtree/control map build与String allocation=0；1/100/10k nodes不按top/status/extension重复全表，changed damage只visit相交paint segment，完整pixel/clip/z parity通过。
- stable presentation generation的paint-element转换、host-command构造、template handler probe、command sort与owned payload clone均为0；单node mutation只重建对应compiled segment，GPU/Softbuffer pixels、clip、z与fallback一致。
- shipped MVP icon asset fallback count=0；确需fallback时每glyph compiled/RHI command=1、raster/upload≤1/key/generation，cache有界且GPU/Softbuffer glyph pixels一致。
- stable atlas generation的paint-thread read_dir/canonicalize/metadata/read/decode/manifest clone=0；changed atlas只让对应compiled segment换handle/generation。
- stable visual-resource generation的candidate/path alloc、exists/stat、global pixel-cache lock、RGBA hash/copy与raster为0；changed resource只patch handle/generation。
- changed node style compile的theme snapshot acquisition≤1，table per-cell theme acquisition=0；stable generation selector/theme lookup=0且frame global theme read≤1。typed visual role随presentation generation提交，changed node classification≤1且不分配String，stable role probes=0。
- changed material node的handler/classification/theme/label/measurement各≤1且临时String=0；stable generation八类probe、variant scan、theme/text/command build=0。复合glyph compiled command=1或显式typed effect，资源与batch由Render13统一拥有。
- changed MUI X component theme acquisition≤1，zero-border quad额外theme read=0；stable chart RGBA/raster/upload与component theme/geometry/command build=0，compiled command只持正确data/theme generation的handle或typed geometry。
- idle state-layer node theme read/command=0；changed overlay/ripple node theme acquisition≤1，stable generation state-layer builder调用=0。
- changed button owned key=0且classification/style/label/measurement/theme snapshot各≤1；stable button以上计数及surface/content command build均=0，现有button pixel/order/text tests等价。
- changed field lowercase allocation=0且classification/style/label/measurement/theme snapshot各≤1；stable field以上计数及field command build均=0，现有field identity/geometry/state/text/icon/stepper pixel tests等价。
- changed Transform axis control theme/metrics acquisition各≤1、palette derivation≤1/theme generation、value owned copy=0；stable control以上计数及command build均=0，现有axis identity/state/geometry/text/order/pixel tests等价。
- changed property row parse=0、owned value copy=0、theme/metrics acquisition各≤1；stable row以上计数及command build均=0，现有axis unit/group/component label/layout/focus/text/pixel tests等价。
- changed selection control selector/style/label各≤1、theme/metrics acquisition各≤1；stable control以上计数及command build均=0，现有checkbox/radio/toggle identity/state/style/geometry/order/pixel tests等价。
- 任意slider tick输入的command count≤Runtime09共享预算且≤track可分辨columns；changed slider theme/metrics各≤1，stable静态segment build/locks/format=0，percent-only只patch fill/thumb/value，现有normal/range/steps tests等价。
- changed segmented control option额外copy=0、selected lowercase=0、selector/style/theme/metrics各≤1；stable以上及command build=0，现有option order/selection/capitalization/style/geometry/tab/pixel tests等价。
- changed alert identity/tone/style/theme/label各≤1且identity临时String=0；stable以上及command build=0。shipped MVP alert glyph fallback=0，或保留fallback的每glyph compiled/RHI command=1；现有tone/state/action/clip/order/text/pixel tests等价。
- changed chip identity/has-chevron/label/palette/metrics各≤1；stable以上及command build=0。shipped MVP chevron fallback=0，或保留fallback的每glyph compiled/RHI command=1；现有state/label/geometry/order/pixel tests等价。
- 30s idle stable status split/format/copy/measure/theme/metrics/build=0；changed control各projection≤1且owned bytes仅最终command，value-only只patch动态段。shipped status glyph fallback=0或每fallback compiled/RHI command=1；现有signal/chip/icon state/text/geometry/order/pixel tests等价。
- changed visible list row identity/adornment/style/theme/metrics/label/resource各≤1，stable visible以上及command build=0，offscreen全部=0；1/100/10k visited=visible+overscan且shipped adornment fallback=0，现有density/state/indicator/asset/order/pixel tests等价。
- changed visible table row column layout=1、稳定theme固定header sample measurement=0且identity/style/theme/metrics/resource各≤1；stable visible以上及build=0，offscreen全部=0。1/100/10k×1/2/4列仅按最终cell commands线性，action fallback=0，现有column drop/minimum/alignment/normalization/state/action/order/pixel tests等价。
- changed visible tree row identity/icon/action/style/theme/metrics/label/resource各≤1且guide metrics acquisition不随depth增长；stable visible以上及build=0，offscreen=0。1/100/10k×depth0/1/8/64仅必要guide commands增长，shipped four-glyph fallback=0，现有depth/indent/guide/disclosure/object/action/state/order/pixel tests等价。
- offscreen popup row_data/style/flag/adornment/build=0；changed visible row adornment classify=1且临时String/lowercase=0、identity/style/theme/metrics/text各≤1；stable全部=0。5/20/1k rows仅访问visible+overscan，现有row/separator/state/danger/shortcut/adornment/clip/z/pixel tests等价。
- changed section title identity/theme/metrics/label各≤1且owned bytes仅最终必要文本；stable以上及build=0。1/100/10k icon/no-icon/strong/disabled/tone保持flat header、strong offset、icon geometry/order/pixel tests等价，glyph预算有产品trace。
- 1k同target pointer move的tooltip generation/build/theme/metrics/text=0；changed tooltip各projection≤1。default/min/max arrow compiled/RHI command=1或fallback产品命中0且有界；target切换、state/layout/shadow/title/body/order/pixel tests等价。
- notification center same-generation unread scan/build=0、closed paint=0；open visited=visible+overscan且offscreen row_data/text clone=0。EditorLayout09的1k/10k storm retention有界、overflow显式且更新amortized O(1)/bounded batch；header/unread/severity/order/scroll/focus/pixel tests等价。
- closed dialog theme/metrics/text/build=0；changed open dialog各projection/classification≤1且每action measurement=1；stable-open 300 frames以上及build=0。现有dialog/confirm/alert、wide/narrow/short action stacking、severity/content/clip/order/pixel tests等价。
- inactive drag overlay全部=0；1k same-payload moves label copy/static build=0、仅patch必要geometry且commands≤4。新增allowed/blocked/indicator测试，保持fallback label priority、cursor offset/target edge/colors/clip/order/pixels等价。
- command palette stable catalog open无全catalog deep clone，1k query无重复materialization；paint只visit/clone visible+overscan，offscreen=0，changed row theme/metrics/text各≤1。保持enabled/when、selection/focus/commit、search/empty/detail/order/pixel tests等价。
- `PaneData`按active kind持有tagged/shared payload，隐藏或不相关pane payload allocation/clone为0；切换、floating、save/restore与native presenter语义等价。
- Softbuffer diagnostics overlay不修改/clone结构presentation；stable fallback帧overlay iteration≤2，verbose summary仅presentation generation变化时构建。
- Runtime Diagnostics消费已提交的all-node debug-surface generation；同generation reflector/build/snapshot=0，changed generation的all-node snapshot与dispatch-only hit surface各最多构建一次。
- Workbench hit surface与open-popup z stack随presentation generation一次提交；1k pointer moves的bounds scan、surface build/rebuild和无popup全node scan均为0。
- Palette、metrics与typography作为一个theme generation原子发布；稳定frame theme全局lock acquisition≤1且per-node=0，1/1k/10k nodes访问数近常数，theme switch布局与pixels等价。
- Presentation不调用空pane setter或构建其model payload；activity/browser/welcome/project最终投影仍只有一个owner且route/pixel parity。
- Damage使用固定容量region set；分离小更新paint/upload面积不随间距增长，满容量合并选择最小面积增量，Full promotion有阈值、计数和原因。
- Hover pointer move只更新transient old/new state与damage；structural presentation/ModelRc node、floating pane和popup row visit/build均为0。
- 同一redraw drain周期native `request_redraw`≤1；idle/1k input-only `about_to_wait`的native window property query/state write=0，resize/move/DPI等事件状态最终一致。

## 禁止临时方案

- 不得把 refresh 放到后台线程却继续每 event 全量深 clone。
- 不得静默丢弃 edge event 或让 UI snapshot 跳过已提交 transaction revision。
- 不得给每个 pane 各建独立 dirty authority。

## 修复结果与回传

Open state: `待 EditorUI08 实现 domain generations、frame coalescing 与 incremental snapshot，并回传 storm/lock/clone/byte parity`。

2026-07-22增量证据：`zircon_editor/src/tests/host/retained_callback_dispatch`当前54/54逐文件复核。已有合同正确锁定无反馈viewport move不置dirty、same frame/floating source no-op、decorative hit不dispatch，以及Workbench hover/press/slider/focus只paint-only且不写journal；virtual row也有pool created/reused/recycled计数。缺口仍与本failure一致：layout动作普遍layout+presentation dirty，pointer move仍逐event journal，module/popup/scene/inspector同步没有stable-generation build=0、clone bytes或125/500/1000 Hz门；唯一千节点startup profile被ignore且只打印墙钟。本轮仅按PERF-MVP-136让两个test contract-node helper借用`ModelRc::get`行，删除宽DTO probe clone，不改变产品failure open状态。

2026-07-22 `host/retained_window`增量证据：29/29 Rust test owner逐文件复核。native host已用行为合同锁定same-target hover no-redraw、100次hierarchy hover不重建presentation、viewport input等待新image、text field局部damage与menu/tab/drawer/floating有界damage；但像素小样本不证明stable-generation build/clone/cache预算。本轮仅做consumer安全止损：`template_hover/nodes.rs`先借用`ModelRc::get`查目标，非owner dock不再clone全部宽node；`panes.rs`先借用检查floating pane，整层无目标直接返回。源码合同2/2、组合合同15/15、rustfmt/diff通过。命中pane仍替换完整ModelRc，pointer/present读取仍clone结构presentation；本failure保持open，最终验收仍要求transient hover generation、stable control index、1/100/10k node clone/visit计数、Cargo/F4与像素/RenderDoc parity。

2026-07-22 retained pointer短目录增量证据：activity/detail/document/drawer-header/drawer-resize/list/viewport-toolbar共37/37 Rust files逐文件复核，已有unchanged layout/state与重复measured frame no-op。`drawer_resize/movement.rs`原对相同preferred也写transient并mark layout dirty；本轮源码RED→GREEN在capture pointer state更新后比较previous/base，相同值直接返回，合同1/1、rustfmt/diff通过。raw pointer入口宽presentation clone、changed storm逐event layout与release group双事务仍open，分别按PERF-MVP-172/131由本计划实现transient generation/coalescing/typed batch。

2026-07-22 `host/retained_menu_pointer`增量证据：40/40 Rust files逐文件复核。menu/Asset Browser/Assets Drawer合同锁定closed/unchanged不重建、scroll局部damage和single-target hover；其余组件与Blend Space测试多为小规模视觉/状态样本，不能证明stable generation、虚拟化或高频输入预算。生产回查发现asset-content同一hover/scroll state仍调用全量UI writeback并重复设置8项pointer属性；本轮源码RED→GREEN让same state直接返回，组合性能源合同21/21、rustfmt/diff通过。有效scroll仍全量重建pointer surface，Asset Browser/Drawer规模仍小；本failure保持open，最终要求1k same-row move property write=0、10k rows visited=visible+overscan、changed old/new row局部damage，并补Cargo/F4/RenderDoc证据。

2026-07-22 Host Page/Tab Drag增量证据：`retained_host_page_pointer` 6/6与`retained_tab_drag` 9/9 Rust files逐文件复核。已有合同锁定unchanged page layout no-rebuild、overflow paint-only、document edge/drawer/floating route优先级及stale geometry不回退；规模仅2–4 pages、3 tabs、1 floating window。产品drag move虽已same target group不set state，但仍逐move生成group key并读取完整drag state，release再clone全部target tabs到临时strip。本failure保持open，最终以committed route/strip index和transient group identity让1k same-target moves的state DTO read/write、String alloc、surface scan=0，并对1/100/10k pages/tabs/windows记录route probes/clone bytes/p95，保留attach/split/anchor/order语义。

2026-07-30 current-source增量证据：`zircon_editor/src/ui/retained_host/app/host_lifecycle/**`排除startup后32/32、1,548行、7 tests完成逐文件静态复读，组合指纹`4e38ee86b056dd6b7ab76c28f083e4cf748482ba66d9b9a91b6effc0fd0ec4fc`。Godot以pending redraw在draw完成前合并递归请求，Bevy Reactive按事件/deadline驱动并在广播后清redraw request；本failure据此要求Zircon保留连续viewport render能力，但stats/presentation/native-window/decision各按需求generation驱动。managed Cargo、规模counter、independent review、WPR/Tracy、F4与RenderDoc parity未完成，failure保持open；完整证据见`../../../performance/01/2026-07-30-editor-retained-host-lifecycle-current-review.md`。

2026-07-30 invalidation增量证据：`app/invalidation.rs`与`invalidation/**`当前9/9、359行、6 tests静态复读及rustfmt通过，组合指纹`7c7e7787b6d3ffb60a91822b62a19cc66ec07ac3920be523db7467b302fa04eb`。u16 pending合并与paint-only分流不是瓶颈；新增PERF-MVP-601记录33个pointer入口的相同diagnostics `RefCell`重写。managed Cargo、1M pointer/1000Hz setter/write counter、independent review与F4 overlay parity未完成，failure保持open；完整证据见`../../../performance/01/2026-07-30-editor-retained-invalidation-current-review.md`。

2026-07-31 native-window-close增量证据：`app/native_window_close.rs`与子树当前7/7、262行、1 inline test完成静态复读，组合指纹`7e9b2a94469f5f060c14b52485b3491f4c8bc0e4ab4166a7ac8ce62435d8a49d`。floating预检full layout/view clone与`O(V*k)`membership之后，`k`个CloseView仍逐项触发全metadata/window-registry、event/journal/invalidation以及与layout无关的authoring-world scene observe；新增PERF-MVP-602要求typed atomic close batch和纯layout不发布scene-inspection。同步save另交Editor09/14，但最终completion只能一次dirty-domain commit。managed Cargo、1/8/128/1K tabs规模counter、WPR/Tracy、F4 native parity与independent review未完成，failure保持open；完整证据见`../../../performance/01/2026-07-31-editor-retained-native-window-close-current-review.md`。

2026-07-31 workspace-docking增量证据：`app/workspace_docking.rs`与子树当前6/6、327行、1 inline test完成静态复读，组合指纹`cdcad19d0a8aab9b53302c6be0e1644b9dde859c452d216fdf511847c44749b6`。drag move同group前仍String分配/宽state get，release双hit且detach前full model build；collapsed drawer attach+reopen双event。新增PERF-MVP-603要求typed drag generation、single release route与atomic drop；no-move resize双unchanged event补强131，same-preferred止损继续归172。managed Cargo、1M move/10K layout规模counter、WPR/Tracy、F4与independent review未完成，failure保持open；完整证据见`../../../performance/01/2026-07-31-editor-retained-workspace-docking-current-review.md`。

2026-07-31 small input adapters增量证据：`app/{pane_payload_visibility.rs,native_keyboard_actions.rs,workbench_context_menu.rs,menu_pointer.rs}`当前4/4、127行、0 inline tests完成静态复读。visible-kind在一次main/native projection中最多重复全tab扫描7次；context-menu单document gate仍构建完整chrome；menu stable event叠加同值diagnostics写。PERF-MVP-105/106/601验收补充visible-kind scan≤1/model generation、document-id gate full chrome build=0、stable pointer diagnostics write=0。native keyboard无新增热点且外部F2 route保持原样。direct rustfmt 4/4通过；managed Cargo、规模counter、F4与independent review未完成，failure保持open；完整证据见`../../../performance/01/2026-07-31-editor-retained-small-input-adapters-current-review.md`。

2026-07-31 state/visibility helpers增量证据：`app/{asset_surface_pointer_state.rs,reference_drop_payload.rs,runtime_diagnostics_visibility.rs,workbench_snapshot_access.rs}`当前4/4、208行、0 inline tests完成静态复读。前两项为常数正向边界；diagnostics让main/native同model visibility遍历达到6/8次，active document/floating/Welcome仍有full chrome、window scan和path Vec。PERF-MVP-105/106验收补充单generation visibility traversal≤1、active identity query full chrome=0、surface lookup近O(1)；Welcome沿117。direct rustfmt 4/4通过；managed Cargo、规模counter、F4与independent review未完成，failure保持open。完整证据见`../../../performance/01/2026-07-31-editor-retained-state-visibility-helpers-current-review.md`。

2026-07-31 tick projection adapters增量证据：`app/{backend_refresh.rs,job_progress.rs,workbench_notifications.rs}`当前3/3、332行、7 inline tests完成静态复读。无效selected UUID full snapshot、stable progress owned projection和empty pending generation早退分别补强PERF-MVP-104/017/596，pending template gate继续归105。direct rustfmt 2/3通过，唯一失败是外部modified `job_progress.rs`测试断言排版；managed Cargo、规模counter、F4与independent review未完成，failure保持open。完整证据见`../../../performance/01/2026-07-31-editor-retained-tick-projection-adapters-current-review.md`。
