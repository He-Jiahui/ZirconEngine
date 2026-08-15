---
related_code:
  - zircon_editor/src/ui/retained_host
  - zircon_runtime/src/ui
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface
detail_report:
  - docs/plans/zircon_runtime/runtime/09/2026-08-09-ui-architecture-performance-reassessment.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore
  - dev/slint
  - dev/Fyrox
---

# 01 · Editor Retained UI 架构与性能审查

## 1. 审查边界与证据状态

本计划承接 Editor retained host、Runtime UiSurface、文本与图像缓存、WGPU UI surface presenter以及原生窗口事件循环的纵向性能审查。详细逐路径证据、复杂度表和阶段候选保留在 `docs/plans/zircon_runtime/runtime/09/2026-08-09-ui-architecture-performance-reassessment.md`；本文件只维护 `docs/plans/optimize` 要求的canonical差距、参考契约、实现顺序和工程级验收。

已达到E3的范围：pointer/window事件到damage/present闭环，Editor私有pointer surface的构建与命中，Runtime dirty rebuild/projected hit grid，SVG parse/raster/GPU residency失效链，native resize事务，WGPU transient surface/device loss/multiwindow生命周期，以及Unreal Slate invalidation/hit-test/list virtualization/renderer/RHI lifecycle对应实现。已形成可执行回归、profile场景和样本完整性/预算协议，部分候选达到静态E4；由于共享Cargo lane和current-source Editor构建仍受外部工作区错误阻断，CPU/RSS/p95数据尚未取得，整个计划保持 `in_progress`。

明确未覆盖：Text02活跃owner中的最终glyph/absolute-position cache修复，以及其它会话正在修改的render cache实现。WGPU设备丢失/多窗口surface恢复与IME/无障碍产品闭环已完成E3设计审查，但尚未实现或经过产品故障注入；没有动态数据的项目不得写成性能达标。

## 2. 当前结构性发现

| 等级 | 发现 | 当前复杂度/影响 | 目标契约 |
|---|---|---|---|
| P1 | Editor列表、资源、菜单和Welcome私有命中树复制逻辑集合 | 原实现有效wheel按逻辑条目全量重建UiTree/dispatcher/route map，典型为 `O(N)`；paint已是 `O(V)`，命中与绘制规模模型不一致 | 固定surface authority + 算术/visible-range投影；稳定数据滚动时authority重建、节点插入和route重建为0 |
| P1 | Runtime popup最终渲染几何与发布hit grid分离 | 旧路径在输入事件期扫描arranged/render命令并重映射，既可能命中placeholder，也把 `O(N+R)`推入热路径 | frame publication时一次构建projected hit authority；frame/instance路径共用同一grid，事件期只查空间索引 |
| P1 | 启动GPU presenter到runtime共享presenter的升级没有状态机 | RenderFramework异步解析返回pending时，旧 `about_to_wait`先销毁可用fallback GPU surface，再重建standalone presenter并强制redraw；输入批次可反复触发GPU surface生命周期 | 保留当前presenter；有界轮询readiness；ready后只做一次destructive handoff；失败稳定回退，不在输入批次重试 |
| P1 | WGPU surface瞬态失败与device loss缺少分层状态权威 | `Outdated/Lost/Timeout/Occluded`已有surface级重试，但公开present合同只有 `Submitted/RetryableNoSubmit`，RHI错误只有通用 `SurfaceUnavailable`；RenderFramework没有device generation/lost observer。共享device的多presenter无法对真正device loss执行一次性协同失效、重建和全量重绘 | RenderFramework唯一拥有device状态与generation；每个窗口独立拥有surface retry/close状态。surface loss只重配本窗口，device loss合并为一次全局恢复，生成新device generation并让全部presenter各重建一次device资源和full redraw |
| P1 | Editor retained window丢弃完整IME事件并维护append-only文本副本 | Winit层已产生Preedit/Commit/Cancel/DeleteSurrounding，Runtime TextInput也支持composition、selection和候选窗geometry；Editor `handle_ime_input`却只提取 `UiInputEvent::Text`，所有 `UiInputEvent::Ime`均被丢弃。`HostTextInputFocusData`只有value/frame，绘制固定无selection/caret，形成第二套不完整文本权威 | 每个focused editable control只有一个text session authority；完整IME事件进入同一编辑状态机并保留UTF-8 range/composition，selection/caret/composition从该状态发布；Enable/Disable/UpdateCursor host request映射到窗口API，焦点切换与关窗显式cancel |
| P1 | Runtime无障碍语义/AccessKit映射没有产品adapter与增量publication | neutral tree/action和AccessKit映射已有完整下层测试，但映射函数只被测试调用，Editor/App没有OS adapter。当前snapshot每次至少三遍扫描全部node并逐祖先判hidden；若直接每帧接线会产生最坏 `O(N*h)` 重建和大量DTO分配 | 每个Surface publication拥有不可变accessibility frame与generation；只对structure/layout/focus/semantic dirty集合及relation dependents更新。每native window一个OS adapter按generation发布delta并把action回送同一Runtime dispatcher，查询期间继续读上一代完整frame |
| P1 | 焦点导航在每个键盘/手柄事件重建并排序全树候选 | modal scope全扫tree，随后递归构建候选、克隆group id、排序并线性空间评分，通常 `O(N + F log F)`；control-anchored popup仍使用未投影tree frame，可能与pointer frame authority分裂 | Surface publication拥有generation-stamped navigation index；复用最终projected geometry、scope/tab order和changed-node集合。event路径只查询索引，render-only变化零更新，禁止tree-local第二几何authority |
| P1 | Editor `virtual_rows` 全量物化逻辑集合，Runtime virtual window只裁剪全量retained children | hierarchy paint/input已为 `O(V)`/`O(1)`，但full shell sync仍克隆每个逻辑行为真实节点并逐行写属性；Runtime measure/arrange仍访问全部children。10,000项常驻节点、字符串和布局工作仍为 `O(N)` | hierarchy删除逐行template镜像；Inspector按visible range + overscan维护稳定bounded slot pool；model count/logical extent与materialized children解耦，scroll只处理entered/exited slots |
| P1 | 真实按钮的bridge mutation、host publication与damage请求没有统一提交 | 物理命中和damage已是局部，但bridge产生的pending node patch没有由PAINT_ONLY/RENDER提交；泛化PresentationChanged则直接选择Full。`sync_viewport_chrome`无consumer，现有paint-only patch计数不对应实际patch，full redraw只有无来源总量 | callback outcome显式携带Workbench projection commit与旧/新damage；允许与render/paint合并走已有changed-row patch，结构/hit同代发布。PAINT_ONLY只能消费已直接发布的interaction/image generation；所有full fallback带typed reason |
| P1 | sparse `ModelRc`只局部化写入，完整迭代退化为`O(N log N)` | changed-row projection、damage和hit-index rebind已与Delta同阶，但overlay iterator对每一行执行binary-trie lookup；`PartialEq`与map/full fallback会把局部写优化摊平成`O(N log N)`，且无patch node/density/old-generation compaction门 | model generation携带exact changed rows，热路禁止value equality；trie增加有序patch cursor与publication-time density compaction，使random get/Delta write保持对数级、顺序遍历摊销`O(N)`；paint/hit/render共享同代storage/delta |
| P1 | auto layout将局部dirty预先扩张到最高连续容器根 | Editor资产951处container中885处为auto layout且boundary声明为0；`mark_layout_dirty`与incremental root选择都用ContentDriven或auto向上传播，ParentDirected auto仍无法隔离。选根后递归measure/arrange整棵subtree | desired-size驱动的自底向上队列：只有cached desired/occupancy变化才通知直接parent，按轴/slot/constraint决定继续传播；arrange最小layout owner并只下推真实geometry变化，未知backend显式fallback |
| P1 | Surface frame与retained render cache在publication、合成和present阶段被重复摊平/编译 | frame任一dirty以及focus/window-only变化都会克隆完整arranged/render/hit数据；Editor toolbar consumer又因顶层generation变化全扫arranged nodes并重建controls。跨层owned extract、multi-Surface flatten和Runtime/Editor command conversion继续clone/hash payload。小状态提交可放大为`O(N+C+H)`，稳定render仍有多段`O(C + payload bytes)` | 顶层`UiSurfaceFrame`原子提交generation，内部layout/hit/navigation/render/a11y为带domain generation的immutable handles；未变域只复用Arc，consumer只订阅真实依赖。render拥有typed element segments，queue传frame handle，multi-surface只组合namespace + segment handles；renderer仅recache changed segments并复用cached batches，owned DTO降为冷路径 |
| P1 | Asset Browser preview generation与materialization导致重复SVG/图片加载 | preview key把全局`catalog_revision`与per-asset revision做XOR：无关catalog变化使全部preview miss，preview-only publish又可能保持旧key；全部filtered assets同步load并物化，128-source LRU在大集合上循环抖动，SVG按intrinsic size同步parse/raster | typed per-asset/project/artifact/target-size preview identity；viewport + overscan stable slots；selection/单asset delta只patch相关项；共享visual tree/raster service按RGBA bytes预算，miss在有界后台worker完成，现有device-shared GPU registry保持唯一GPU authority |
| P1 | preview cache命中后paint仍复制并哈希整幅RGBA | `Image`已用Arc共享像素，但preview paint先`to_rgba8`深拷贝一次、随后再`to_vec`一次并对全部intrinsic RGBA重算content key；稳定GPU命中之前仍有`O(P)` hash和约`2P`瞬时复制，target thumbnail尺寸没有约束CPU product | visual generation拥有带预计算content key/extent/Arc像素的immutable raster product；无tint paint只传handle，尺寸/tint变体按typed key生成一次；owned像素DTO仅限冷路径，稳定paint的clone/hash/product build bytes为0 |
| P1 | 后台事件只合并pending状态，未合并native wake | 同一pending epoch内每次资产/任务发布仍加锁并调用winit proxy；消息风暴可产生 `O(P)` 空wake并与输入事件竞争原生队列 | 只在AtomicBool `false -> true`边沿发送一次native wake；消费后下一epoch恢复通知；10,000次发布验证wake近似1且消息不丢 |
| P2 | `about_to_wait`轮询原生窗口metrics | 每个事件批次调用surface size、scale、maximized、outer position四类平台查询 | 创建时初始化，之后由Resized/ScaleFactorChanged/Moved事件更新；稳态每批次原生查询为0 |
| P2 | damage paint索引每次查询重新去重/排序稳定row | 单cell damage旧路径仍分配HashSet/Vec并执行 `O(K log K)`排序；完整clip执行 `O(N log N)`重排 | generation构建时缓存 `(z_index,row)` 顺序；单cell为 `O(K)` clone、完整clip为 `O(N)` clone，跨cell才去重/排序 |
| P2 | profiling GPU stats逐项锁全局recorder | capture开启时单次present最多49次mutex进入与snapshot字符串构造，污染CPU与damage-to-submit证据；普通非profiling构建不受影响 | 同帧counter一次batch提交、共享timestamp；普通profiling idle零分配，Tracy持续counter语义保留 |
| P2 | 无locator runtime texture扩大为全部visual cache失效 | 不相关纹理事件可清空SVG tree/raster/icon atlas并导致重复GPU upload | 只有显式atlas源执行All；locator资产定向失效；generation lag按fingerprint reconcile |
| P2 | 文本缓存key包含绝对位置且部分失效清空过宽 | position-only变化可能造成layout miss，局部文本变化可能扩大为全cache失效 | 由Text owner按shape/layout与placement分层；必须先以profile证明miss来源 |

## 3. 参考引擎结论与适用边界

Unreal Slate 的 `FSlateInvalidationRoot`/fast path把widget更新限制到失效集合，`FHittestGrid`在paint时记录paint-space geometry；`SListView`从当前scroll offset开始生成并在viewport填满后停止，复用已有item widget。Slint repeater同样分离model row count、visible offset、cached item height与instances。Zircon应吸收的是单一publication authority、dirty propagation边界、逻辑集合/实例集合分离、可见集合和事件期只读查询，不复制Slate历史兼容层、UObject所有权或另一棵平行row tree。

renderer生命周期同样必须显式。`FSlateApplication::InitializeRenderer`在初始化阶段建立renderer，`DestroyRenderer`在shutdown释放；窗口变化通过既有renderer的viewport resize合同处理。它没有在每个输入/tick中先销毁renderer再探测依赖是否就绪。Zircon的runtime RenderFramework是异步解析，因此需要额外的Pending状态和有界poll，但可用fallback presenter必须持续拥有surface，直到共享renderer已确认ready并进入一次性交接。

Unreal进一步把窗口surface与device故障分层：`FSlateViewportInfo`按窗口保存独立viewport/extent，`CreateViewport`、`RequestResize`和`OnWindowDestroyed`只操作对应窗口，并在native window销毁前等待in-flight present完成；D3D12 RHI在queue/fence层检查device removed reason并集中输出DRED、breadcrumb和page-fault证据。当前参考版本遇到GPU crash会进入明确终止路径，并不能证明“Unreal会自动恢复device”。Zircon应吸收的是分层owner、单次状态转换和可诊断失败；Editor若要进程内恢复，必须由RenderFramework建立device generation与重建协议，不能把真正device loss伪装成可无限重试的surface acquire，也不能让每个窗口自行创建平行device/cache。

Slate的render cache边界也比当前Zircon完整：invalidation root保存per-widget cached element list、只把`ListsWithNewData`交给batcher重编译，并长期保留cached batches；当帧仍按batch数提交，但不会为全部widget重新hash、克隆和展开draw payload。Slint用item-local cache index + backend generation与property tracker表达同一原则。Zircon现有`UiSurfaceFrame`应扩展为唯一generation publication，multi-surface composite只能引用其segment handles；不能在Runtime set、Editor host和renderer各自再建一份cache。

Unreal Content Browser还把thumbnail集合限制在真实visible items与有界offscreen range，复用`RelevantThumbnails`，并由`FAssetThumbnailPool`按object path + target size缓存、按具体dirty asset刷新且限制每tick生成时间。Zircon当前“全部filtered assets同步load + 128-source LRU”不具备这个规模边界；应迁移per-asset identity、relevancy window、后台预算和稳定GPU handle，而不是增大全局cache常量。

Slint/Fyrox用于交叉核对缓存与Rust所有权边界：缓存按资源generation与内容身份复用，布局/命中状态由发布代际拥有，事件不承担全场景重建。参考只用于确认契约，不以API名称映射代替本地生产链证据。

文本输入与无障碍同样要求“平台桥接器围绕单一retained authority工作”。Unreal `ITextInputMethodContext`要求editable owner同时提供composition状态、selection、range读写、text bounds和screen bounds，并由focus生命周期Register/Activate/Deactivate；不是把IME commit降级成普通字符追加。Slate无障碍则缓存稳定widget/id、只在dirty时重建，默认每tick最多处理100个widget，并在整轮完成前继续向平台暴露上一代children数组。Zircon已有更适合不可变publication的Surface frame，因此应吸收完整上下文与代次提交原则，不复制Unreal的全局singleton或game-thread同步等待。

## 4. 实现顺序

1. 稳定输入与窗口底座：合并后台事件native wake；移除event-loop原生metrics轮询；修复runtime presenter升级状态机；保留可用fallback并建立readiness/backoff/单次handoff计数。
2. 接通真实按钮提交：让bridge pending node patch成为typed callback outcome，允许Workbench projection与render/paint域合并命中已有changed-row patch；消费或删除无效的 `sync_viewport_chrome`，校正paint-only计数，并给full redraw fallback加reason。changed-row model同时发布generation/delta，trie顺序读取使用有序cursor并在publication边界按node density compact，热路不得用`PartialEq`重新发现变化。先证明tool/button click不进入Full，再优化damage范围。
3. 统一命中authority：完成Runtime projected hit grid的下层受管验证，再验证Editor popup产品路径。
4. 统一焦点导航publication：projected hit下层动态通过后，在同一Surface frame generation构建navigation index；先用相关dirty整批重建移除事件期全树扫描，再依据publication profile决定是否做changed-node patch。control-anchored popup必须使用最终投影geometry，不能建立tree-local cache或lazy event rebuild。
5. 消除伪虚拟化：hierarchy先删除逐逻辑行template镜像，复用既有native `O(V)` paint与 `O(1)` arithmetic pointer authority；Inspector再建立有界stable slot pool并接通scroll。随后扩展Runtime model-count/logical-extent/visible-materializer合同，禁止用全量retained child隐藏模拟虚拟列表。
6. 收敛Asset Browser thumbnail与preview raster product：先发布typed per-asset preview identity并删除catalog XOR，再把thumbnail节点改为viewport + overscan stable slots；selection/单asset delta只patch相关项。把同步SVG/image miss移到有界后台worker并复用共享visual tree/raster service，CPU cache按RGBA bytes预算。随后让decode/raster generation直接发布带预计算content key的共享product，删除paint热路`to_rgba8`、二次`to_vec`和全量hash；GPU继续使用既有device registry。
7. 消除其它逻辑集合复制：按shell drag、asset family、menu、Welcome顺序验证已形成的固定authority候选。
8. 收敛布局与paint publication：先把auto-layout传播改为desired-size/axis依赖队列，验证固定layout island内局部变化不会越界、root resize仍只执行一次完整reflow；再验证viewport toolbar点击不重排、tab/header authored-frame局部dirty rebuild，以及damage paint索引复用generation内稳定顺序。
9. 收敛Surface frame与render publication：顶层frame保持单一原子generation，layout/hit/navigation/render/a11y改为分域immutable handles，focus/window-only变化复用未变域；让dirty rebuild直接发布typed element segments。Runtime/Editor/RenderFramework/one-slot queue传共享frame handle，multi-surface用namespace + ordered segment handles组合，删除稳定帧command flatten/global-id rewrite。最后让renderer只recache changed segments并复用cached batches，owned DTO只保留冷路径兼容。
10. 校准文本和GPU：先批量提交GPU present profile counters以降低观测扰动；在上述结构性工作归零后，用profile决定text cache与damage/present剩余长尾；SVG/GPU只验证失效与residency，不重复建立平行cache。
11. 完成RHI故障域：先在 `zr_rhi` 定义可区分surface retry、device unavailable和terminal failure的结果，再由RenderFramework实现唯一device generation/recovery owner，最后让多窗口presenter按新generation各重建一次surface/pipeline/text/image/retained资源并请求一次full redraw。该项是故障隔离与工程完整性门，不作为稳态按钮卡顿的替代解释，也不与前十步的动态profile混成同一收益结论。
12. 收口平台可访问性：先为Editor focused text建立完整IME session回归并删除append-only事件降级，再把Runtime accessibility snapshot变成publication-owned cache，最后为每native window接入OS adapter。两项都先证明产品event/action闭环，再做大树/长composition压力；不得用每帧全树snapshot换取功能表面可用。

每一步必须从最低共享层focused回归开始；一项候选未得到current-source动态证据时，不与下一项算法重写混成同一验收结论。

## 5. 验证矩阵与性能门

| 层级 | 必须验证 |
|---|---|
| 单元/合同 | state transition、旧位置拒绝/新位置命中、frame/instance authority等价、pending presenter不drop、不重建surface；tool/button click只提交changed rows且structure/hit generation同代，PAINT_ONLY不得遗留pending projection，projection/full-redraw fallback有typed reason；model stable clone为`O(1)`、单行patch与`log N`同阶、增量consumer只读exact changed rows，sequential iterator/compaction前后值、row identity、paint/hit等价且热路value-equality为0；nested auto layout的desired不变不enqueue parent，fixed outer island截断传播，Auto/StretchContent与root resize仍正确扩张；stable frame的domain/segment/element Arc identity不变且clone/hash/build为0，focus/window-only publication复用layout/hit/render handles，单节点或单Surface变化只替换所属segment，geometry-only patch复用text/image payload，Runtime/Editor pixels等价；typed preview identity无XOR碰撞，无关catalog delta保持其它asset handle，same-path preview epoch只更新目标项，project/size/DPI不串图；preview raster product在stable/geometry/opacity变化下保持product与pixel `Arc::ptr_eq`且paint clone/hash/build bytes为0，单tint/content变化只替换一个typed variant；tab/group/manual/modal导航等价且popup使用最终投影frame，render-only变化不推进navigation generation；hierarchy无逐行template control仍保持selection/rename/drag，Inspector slot回收不改变property key/focus/edit提交目标；单cell/full-clip paint index查询不重新排序；GPU stats batch保持counter集合与条件字段；surface retry不得推进device generation，重复device-loss通知只启动一次恢复；IME UTF-8 composition/selection/delete-surrounding保持同一session，a11y frame/action target generation一致 |
| 规模压力 | 10,000 hierarchy/assets/menu/recent/focusable/Inspector property条目、10,000 render commands与10,000-node nested layout island；N=1/100/10k/100k model执行Delta=1/10/1k及连续10k单行patch，增量paint/hit与Delta同阶、完整iterator总访问线性、old-generation/RSS有界；Asset Browser thumbnail只维持visible + overscan handles，128-source反例和selection不得触发全量同步decode；1/100/1,000 preview与32px目标/4K intrinsic反例的stable paint clone/hash/product build bytes为0；1/1,000/100,000 World nodes下menu/HUD stable capture；10,000次同epoch后台publish；1000次wheel/click/move/Tab/方向导航/局部layout变化；稳定hover不推进结构generation或present，稳定render submit的command/element/hash/clone bytes与World visits为0，稳定thumbnail交互的SVG parse/raster/upload为0，局部click的full target/full redraw/fallback为0；desired不变的layout transaction与dirty count同阶且island外visited为0；hierarchy template row节点为0，Inspector materialized row不超过visible + overscan，Runtime layout visited不随逻辑总数增长；其它节点数与逻辑集合解耦，native wake不随publish增长，navigation event不全扫tree |
| 窗口产品 | 200-step resize、DPI变化、move、drawer drag；resize事务内command snapshot只构建一次；两个native窗口中单个surface lost不得重建健康窗口，device generation变化时两个窗口各重建一次并恢复完整绘制 |
| 资源产品 | 稳定SVG 1000次交互parse/raster/upload为0；单SVG变化只推进对应generation；Asset Browser单asset preview/source变化只更新一个handle/raster/upload，无关catalog/selection变化不触碰其它preview，CPU resident bytes受预算约束；preview paint只传带预计算content key的共享raster product，稳定/geometry/selection变化的RGBA clone/hash bytes为0，GPU shared resolve/upload/write计数与CPU计数分开验收 |
| 可访问性产品 | 中文/日文多阶段preedit、候选range、commit/cancel、delete-surrounding、焦点切换与DPI后candidate geometry；屏幕阅读器读取、focus/activate/value/text-selection/scroll/popup、双窗口与关窗 |
| 性能 | CPU、working set/private bytes、input-to-damage与damage-to-submit p50/p95/p99/max；button callback/recompute target/projection patch nodes/structure generation/damage pixels/full-redraw reason；model generation/delta、patch depth/node allocation/resident bytes、old generations、compaction、random/sequential visits、value-equality fallback与presentation structure copy；layout raw dirty/measure visits/desired changes/parent enqueues/arrange owners/geometry visits/fallback reason；frame domain publications/reuse与clone bytes、consumer domain scans、render extract visits、command/element/hash/payload clone bytes、composite rebuild、changed segments、recached/submitted cached batches与World visits；thumbnail relevant/materialized/entered handles、cache hit/miss/eviction、RGBA resident bytes、sync/async decode、SVG parse/raster/upload、preview product build/content-hash/RGBA-clone bytes、queue age/cancel；logical/materialized row、repeat full scan、property write、layout visited、slot entered/exited/rebound；presenter create/drop/upgrade/poll计数；paint-index full/single reuse与multi-cell sort计数；profiling recorder每present batch/lock次数；device recovery count/duration、per-presenter recreation、full-redraw与shared-image reupload bytes；IME event/action延迟与a11y dirty/full build、visited node、published delta/bytes |
| 故障注入 | RenderFramework pending、resolve failure、runtime surface create failure、fallback create failure；Outdated/Lost/Timeout/Occluded逐项surface retry；device loss、recreate failure/backoff、恢复中关窗、双窗口单surface失败、恢复后shared image generation一致性 |

证据协议固定为同一source-bound session的`warmup -> measured -> quiescence`。每个interaction必须有单调sequence和typed outcome，recorder必须报告written/overwritten/oldest/newest sequence，measured window丢样直接失败；1000 click、1000 pointer、200 resize各独立运行至少3次。无WPR运行用于预算验收，WPR/Tracy只在失败后归因。现有采集器的“sample存在且分位数有序”不是性能GREEN：暖态input-to-damage p95必须不高于1 ms，CPU damage-to-submit p95不高于8 ms，局部click的Full target/full redraw/无reason fallback为0；稳定pointer move允许typed no-damage但不得present。RSS同时取warmup末端、measured peak/end和quiescence末端，并由cache resident bytes解释保留量。

当前工具候选已经实现schema 5 per-recorder留存证明、zero-overwrite fail-closed、1 ms/8 ms p95门、单核CPU与64/96 MiB RSS增长门。Editor event loop为每个翻译请求保留单调`UiInputSequence`，以一个active input和一个pending present batch发布damaged、intentionally-no-damage、rejected三类typed outcome；damaged sequence允许合法合并到同一次成功present，retry不消费batch，空间保持`O(1)`。latency gate校验每个damaged outcome恰好属于一个有序present range，并分别要求input-to-damage样本数等于damaged outcome数、damage-to-submit样本数等于present batch数，不再使用错误的“全部输入与latency一一对应”假设。

交互场景的每个measured Editor进程现在先执行可配置的同进程warmup successful presents；最后一次warmup present只把状态从`Warmup`推进到`RestartPending`，等事件/present callback scope全部析构后的`about_to_wait`边界才reset/restart recorder并清空input outcome tracker，成功后进入`Measuring`，失败则保持不可测状态且不循环重试。source-bound geometry/screenshot仍可在warmup present导出，但其CPU/counter开销随后由recorder reset清除；重启成功后以原子rename发布带当前PID的measurement-ready文件，自动交互在发送前台或鼠标事件前必须匹配该PID，缺失或陈旧marker直接失败。interaction完成后，同一PID继续执行默认2秒quiescence并每100 ms采样working set/private bytes；quiescence末点和窗口内峰值分别受64/96 MiB增长门约束，缺失、跨PID或短于请求时长均fail closed。startup场景继续保持fresh-process语义；多个measured进程之间另保留独立间隔。manifest显式区分`within_process_warm_measure`与`fresh_process_startup`，记录同进程quiescence时长，并把readiness/输出路径源码纳入103项critical-source fingerprint。latency/capture/process/native-resize合同测试分别为7/7、30/30、6/6、4/4，共47/47；Rust格式、PowerShell parser和补丁检查通过。Rust候选仍待managed Cargo，真实Editor三轮CPU/RSS/p95与SVG parse/raster/GPU upload数据仍为空，因此这些GREEN只提升证据可信度，不构成按钮、缩放或SVG性能已经改善的结论。

规模门必须把fixture参数写入manifest：model `N=1/100/10k/100k`与`Delta=1/10/1k`，10,000 hierarchy/assets/focus/layout nodes，以及1/100/1,000 preview与32 px target/4K intrinsic反例。10倍逻辑规模下单Delta visited/alloc/presentation-copy不得同比增长；完整遍历只允许线性增长。任何ring尾段样本、历史二进制、人工命名规模或仅字段完整的报告都不能触发实现裁决。

目标预算不是在静态审查中宣布：暖态input-to-damage p95优先进入1ms量级，CPU damage-to-submit p95不超过8ms；稳定pointer move不请求present，稳定列表wheel不重建authority，pending renderer期间presenter create/drop不随输入事件增长。必须同时给出事件完成数、样本完整性和RSS，不能只报告平均帧率。

所有profile、构建和临时产物必须放在E盘受控目录。动态验证使用官方managed validation，禁止raw Cargo和历史二进制冒充current source。里程碑只有在测试、产品profile和回归均通过后才允许受管commit，并将量化前后数据发送到企微。

## 产出记录与时间

| 里程碑 | 状态 | 证据 |
|---|---|---|
| 全链静态审查与参考引擎对照 | in_progress | detailed report 15.1-15.69；仍缺current-source动态profile |
| pointer/list/popup/layout候选 | static_candidate | scoped rustfmt/diff与PowerShell合同；Rust managed validation待lane |
| visual cache失效域 | static_candidate | 无locator texture不再触发All；动态SVG/GPU residency待验证 |
| event-loop/presenter生命周期 | static_candidate | native wake false-to-true edge、readiness-before-drop、50ms pending poll、单次handoff与stable fallback已实现；动态事件风暴/故障注入待lane |
| WGPU surface/device/multiwindow生命周期 | design_ready_e3 | Zircon present/RHI/RenderFramework/shared image生产链与Unreal Slate viewport、D3D12 device-removed诊断已纵向核对；确认surface transient已有局部重试，device generation/recovery authority缺失；实现与故障注入均未开始 |
| IME/accessibility产品闭环 | design_ready_e3 | 确认Runtime translator/TextInput/accessibility/AccessKit下层能力存在；Editor retained IME consumer丢弃 `UiInputEvent::Ime`，OS accessibility adapter无生产调用，naive snapshot接线会全树扫描；目标authority与压力门已定义，未改生产源码 |
| focus/navigation publication | design_ready_e3 | 确认每次event执行modal全扫、候选递归/排序与方向线性评分，并使用未投影tree frame；目标Surface-owned index、依赖扩张、popup geometry parity与10,000节点压力门已定义。因与冻结projected-hit路径重叠，未改生产源码 |
| virtual-row materialization | design_ready_e3 | 确认hierarchy native paint/input已为 `O(V)`/`O(1)`，但template repeat仍按全部逻辑行创建真实节点，Runtime virtual window仍全量measure/arrange child；hierarchy删除镜像、Inspector bounded pool、scroll/focus/edit回归及10,000项复杂度门已定义。相关源码存在外部改动，本轮保持只读 |
| button mutation/publication/damage | design_ready_e3 | 确认真实物理命中与damage已局部，但bridge pending projection未由PAINT_ONLY/RENDER提交，泛化presentation会退化Full；`sync_viewport_chrome`无consumer，paint-only patch计数不对应实际patch，full redraw缺reason。目标typed outcome、同代structure/hit commit和1000次交互门已定义，未改外部owner源码 |
| sparse model generation/delta | design_ready_e3 | 确认changed-row projection、damage与hit rebind已局部，但外部`ModelRc` overlay iterator逐行trie lookup，使full iter/equality/map为`O(N log N)`且无node-density/old-generation compaction门。已定义exact delta、有序cursor、publication-time compaction及1/100/10k/100k模型门；相关候选源码本轮只读 |
| auto-layout dependency propagation | design_ready_e3 | Editor资产951处container中885处为auto且无boundary声明；两层预传播与递归measure/arrange会把局部dirty扩到最高auto root。已定义desired-size/axis queue、containment、unknown fallback及10,000-node island门；相关Runtime/Editor文件有外部改动，本轮只读 |
| generation-owned Surface/render publication | design_ready_e3 | 确认任一dirty及focus/window-only frame publication都会深拷贝未变域，multi-surface合成、owned RenderFramework/queue合同和Runtime/Editor command conversion还会重复clone/hash/展开payload；已定义单一顶层generation + 分域handles、typed element/batch reuse、menu/HUD generation投影及1/100/10,000 command与1/1k/100k World-node门。相关Runtime UI候选为外部未跟踪源码，本轮只读 |
| Asset Browser preview identity/relevancy | design_ready_e3 | 确认catalog XOR造成全局过失效/碰撞，preview-only publish可保持旧key；全部filtered assets同步load与128-source LRU会在大集合循环抖动，SVG按intrinsic size独立parse/raster。已定义typed identity、visible + overscan、后台预算、共享visual service、字节预算与1k/10k资产门；相关Editor源码本轮只读 |
| preview raster product reuse | design_ready_e3 | 确认`Image`虽然共享Arc像素，preview paint仍先`to_rgba8`深拷贝、再`to_vec`并全量hash intrinsic RGBA；GPU命中前每次command rebuild仍为`O(P)`且约`2P`瞬时复制。已定义generation-owned raster product、预计算content key、target/tint typed variant与clone/hash/build bytes零门；相关Editor paint源码本轮只读 |
| profile sample integrity and budget gate | static_candidate | `ProfileSnapshot`与recorder已发布per-recorder capacity/written/overwritten/retained/sequence留存证据，Editor merge保留独立authority；schema 5以`UiInputSequence`发布三类typed outcome和`O(1)`成功present batch，验证damaged membership、retry保留、zero-overwrite及1 ms/8 ms p95预算。runner执行同一measured进程内warmup present，在`about_to_wait`边界reset/restart recorder，以PID-bound readiness marker阻止自动输入抢跑，并在interaction后对同一PID按100 ms采样quiescence RSS；103项critical-source fingerprint覆盖该合同。Pester为7/7 + 31/31 + 6/6 + 4/4 + 4/4 = 52/52，Rust rustfmt与PowerShell parser静态通过。managed Rust验证和真实Editor CPU/RSS/p95、SVG/GPU产品profile仍待完成，不宣称性能达标 |
| source-bound hierarchy scale input | static_candidate | `hierarchy_scroll`现在可在measured Editor启动前从canonical `renderable-empty`模板生成仓库外、非C盘的真实scene，精确写入`N=1..100,000`个可加载实体，再通过正常`--project`路径打开。source manifest schema 2记录project manifest与scene的SHA-256/bytes、N和实际请求的wheel operation count，并在启动前重新验证root、精确文件所属、N与实体数、hash/length；普通与device-namespace C盘路径均在写入前拒绝，任一输入篡改后fail closed。100,000实体生成实测4.741 s、21,888,895 bytes，终态managed heap约+1.38 MiB；临时E盘目录已删除。该切片只建立真实hierarchy N输入，尚未发布Editor观察到的logical/materialized/visible/overscan，也没有执行逻辑节点增删Delta；asset/focus/layout/preview规模入口与产品profile仍待完成 |
| 2026-08-15 retained UI dynamic profile precondition | blocked | UI12 managed editor test job 在 AA tests 前被未追踪的 `zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus/` 拆分目录阻断。`support.rs` 与 `toolbar_anchor.rs` 的 attribution hash 已过期但 lease 仍归 active `editor-ui12-zui-design-convergence-v4-20260811`；不跨写该目录。待 UI12 续租、修复 import/visibility 后，才运行 managed current-source 测试、10,000-item 压力和 CPU/RSS/p95 profile；本轮没有性能数据或算法达标结论。 |
