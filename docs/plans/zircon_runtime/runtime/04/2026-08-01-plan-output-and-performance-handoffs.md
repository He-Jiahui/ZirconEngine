# Runtime 04 产出与性能交接归档

> 来源：[04-asset-pipeline-alignment.md](../04-asset-pipeline-alignment.md) 的直接产出与性能记录。2026-08-01 输出治理将这些记录迁入编号子目录；除 Markdown 相对链接按新目录层级重定位外，历史文字、命令与验证证据保持原样。本归档不代表 milestone 完成。

## 迁入产出记录

- 2026-07-14 migration journal GREEN：受管 job `b337b21337c84d248905915d3ceaf875` 从当前源码通过 `minted_sidecar_commit_crash_is_whitelisted_and_next_apply_converges` 1/1；Plugins08 owner handoff 已 fixed 返回。core-min scene filter 为 595/596，唯一失败位于其他 owner 的 Scene reflection `JsonNumber` 类型漂移；broad `asset::` 仍未关闭，因此本计划继续 `in_progress`。

- Editor03 完整 Runtime 回归门发现 Virtual Geometry debug snapshot integration fixture 仍调用退役的无 resolver TOML API：`待修复（open）`；[failure 交接](../04/failure-2026-07-15-virtual-geometry-debug-snapshot-project-toml-consumer-drift.md)。修复必须迁移到 project resolver-aware 序列化合同，不得恢复 `to_toml_string()` 兼容入口。

- Frameworks05 library gate 转绿后，Runtime04 focused VG test 已真实执行，但根级 Virtual Geometry support fixture 缺少 Plugins13 已规定的 AsyncCompute workload：`待修复（open）`；[failure 交接](../../../zircon_plugins/13/failure-2026-07-15-virtual-geometry-runtime-support-compute-workload-drift.md)。受管 job `1e7cdd7825024a08b236b2edd07c67b9` 为 `0 passed / 3 failed / 4 ignored`；三个失败均发生在 descriptor compile，不能计作 Runtime04 project-TOML 修复失败或通过。

- 2026-07-18 core resource性能交接：typed get/acquire的完整record snapshot、register/reload重复record owner/revision compare及ready sort UUID格式化已止损；acquire payload读取与ref-count增加仍非事务，last release可夹入移除authority，每lease还分配释放闭包且drop同步拿runtime/payload锁，subscriber为无界锁内fanout。Runtime04联动Runtime07以per-resource Arc entry、generation发布、小型last-drop事件、有界回收/event lane与frame batch/cursor收口；见PERF-MVP-327及`docs/plans/performance/01/2026-07-18-runtime-core-manager-resource-static-review.md`。

- 2026-07-18 IBL/import 性能交接：source-cubemap compile options 在稳定帧同步读取完整 `.zribl` 并多次深拷贝 payload，cache miss 同帧还可能二次读；外部 cubemap decode/staging 的 production 入口仍串行，测试 executor 最多只有每 mip 六个 face 任务。Runtime04 联动 Render11/17按 request identity + cache generation 异步加载并发布 resident `Arc` artifact，按 mip×face×tile 调度 asset compute pool 或 GPU bake，稳定帧 frame I/O/decode/clone 与 caller blocking 均为 0；见 PERF-MVP-352/354。

- 2026-07-18 shader asset/prewarm 性能交接：prewarm manifest按variant复制WGSL/include/version并串行validate/compress/write，shader package与IDE/template消费者各自重复解析include。Runtime04联动Render08/17与Editor09发布content-addressed source table、bounded asset worker queue及generation-owned单遍parse artifact；同source正文只存一次、provenance hash=1、changed source scan=1、stable scan=0；见PERF-MVP-357/358。

- 2026-07-18 environment upload artifact交接：IBL resident artifact到GPU之间仍缺预编码边界，source/PMREM/irradiance在render submission线程同步f32→RGBA16F并按face×mip碎片上传。Runtime04联动Render11/13/17让asset/bake worker发布版本化row-aligned upload artifact并由持久staging arena单batch提交；stable转换/上传=0、changed artifact build≤1/generation，复用PERF-MVP-352/354唯一resident owner。见PERF-MVP-380。

- 2026-07-18 mesh deform/resident artifact交接：mesh import/compile须把morph target静态delta、skeleton引用与content/revision keyed GPU-upload payload发布为immutable artifact；render frame不得重新展开target×vertex，也不得对稳定`Dynamic`/CPU-morphed source逐draw调用`GpuMeshResource::from_asset`。Runtime04联动Plugins04/Render03 single-flight发布，stable artifact build/read/copy=0、changed≤1/content generation。见PERF-MVP-385/386/389。

- 2026-07-18 scene resource streamer交接：Runtime04须由asset events发布一次resource/dependency revision generation与批量只读snapshot，CPU I/O、decode、mesh hash/可选wire、material/shader依赖解析及upload command在有界single-flight jobs完成；render线程只按byte/object/time预算应用ready artifact并保留last-good。当前frame unique ensure及material/shader稳定命中已止损，最终stable registry逐资源锁/load/decode=0、changed近dirty resources。见PERF-MVP-404及scene-resources静态证据。

- 2026-07-18 GPUScene delta来源交接：Runtime04/scene extract须发布added/changed/removed dense records和generation，不让Render03每帧从完整draw列表重建live HashSet/entry HashMap；morph/VG静态payload沿用content artifact，pose/weight/transform只携dirty identity。stable extract到GPUScene records=0、changed近delta，见PERF-MVP-405。

- 2026-07-18 shader module artifact补充：当前template/IDE env每次重建builtin includes并重复extract/strip/hash，IDE每stub又全表找依赖、拼接全文和Naga parse。Runtime04按PERF-MVP-358发布content-addressed parsed module与indexed dependency DAG，changed source只更新受影响closure，稳定env/preview/compile scan/hash/parse=0；后台job有in-flight/RSS预算并向Render08/Editor09发布generation ticket。

- 2026-07-18 plugin shading include索引补充：当前每descriptor的forward/GBuffer/deferred token各自全扫ready shader records并同步load正文。Runtime04把normalized include token→resource id/revision/parsed module Arc纳入同一shader generation DAG，构建时报告duplicate，consumer O(1)借用；stable record scan/path normalize/load/source clone=0，见PERF-MVP-358/404。

- 2026-07-22 offline tool补充：`zircon_shader_prewarm`当前对同一asset root重复resource/shader/registry/material遍历并按source重走include DAG，归PERF-MVP-448与Render08 failure；`zircon_export_pack`把全部asset bytes、determinism inputs与delta/target pack多轮复制，归PERF-MVP-449与Editor15 failure。Runtime04提供唯一content-addressed staged asset/source/chunk artifact和revision inventory，consumer不得各自重扫目录或复制完整payload。

- 2026-07-22 Scene LevelManager consumer补充：trait load/save每次按project root重开ProjectManager并全量`scan_and_import`，单scene save还深clone World并在caller线程同步serialize/write。Runtime04把该入口纳入open `project-source-index-targeted-import` failure与PERF-MVP-453：消费prepared project generation/targeted transaction，save走immutable scene artifact ticket+bounded I/O atomic publish；不得保留Scene私有project cache或第二条full-scan truth。

- 2026-07-22 World project I/O补充：legacy save内部第二次World clone、normalize id Vec及builtin locator重parse已止损（PERF-MVP-462）；Level snapshot、宽SceneAsset/NodeRecord投影、完整pretty JSON String和同步fs write仍在caller线程。Runtime04继续按PERF-MVP-453发布generation scene artifact，Runtime11 bounded I/O lane完成single-flight serialize/atomic replace/shutdown flush；见project I/O静态证据与既有open failure。

- 2026-07-22 dynamic scene asset reload交接：每frame无上限drain，逐event完整重建pending Vec形成O(E×P)，superseded DetachOnDrop任务继续耗worker，ready scenes在一个Level world锁内无预算apply。Runtime04联动Runtime11建立per-AssetId latest-only/cancel generation、bounded drain/apply与lifecycle prune；见PERF-MVP-471和`04/failure-2026-07-22-dynamic-scene-asset-reload-bounded-singleflight.md`。

- 2026-07-22 dynamic scene session archive artifact交接：save/load/capture在archive、World、DynamicScene、Value与String之间重复深clone/parse/normalize，稳定generation也没有唯一sealed payload。Runtime04发布project/scene/schema generation-owned immutable archive artifact，summary/index/preview/typed serde共享一次capture/validation；Runtime11只消费sealed artifact执行I/O。见PERF-MVP-474和`04/failure-2026-07-22-dynamic-scene-session-archive-artifact.md`。

- 2026-07-22 typed asset event交接：PERF-MVP-492已删除每个`AssetEventReceiver<T>`的专用过滤线程和二级无界队列；底层`ResourceManager::subscribe/broadcast`仍为每subscriber无界排队并持全局subscriber锁逐项clone+send。Runtime04需发布共享有界generation event log/ring与consumer cursor，和PERF-MVP-471场景reload预算共同验收；见`04/failure-2026-07-22-typed-asset-event-shared-bounded-dispatch.md`。

- 2026-07-22 asset readiness交接：generic facade readiness在一次查询内重复clone root、遍历direct/recursive依赖并逐node分别读取registry/runtime/payload；Runtime04按PERF-MVP-493在import/reload generation维护聚合state/dependency revision并提供单generation bulk snapshot，完整report每node最多fetch一次；见`04/failure-2026-07-22-asset-readiness-generation-snapshot.md`。

- 2026-07-22 project/registry静态审查补充：`.zmeta`双TOML parse、dependency O(D²)去重与refresh edge-list深拷贝已按PERF-MVP-494/495止损；`scan_and_import`仍对同root执行约5轮inventory/meta load，stable source仍整文件read/hash，watch单path仍全量meta scan/edge refresh/registry persist。Registry还缺AssetId、reverse dependency与source slots索引，referencer/反解/remove为全表或changes×entries扫描。Runtime04按PERF-MVP-496/497扩充既有`project-source-index-targeted-import` failure，联动Runtime11 bounded I/O与Editor10 generation consumer。

- 2026-07-22 project generation/residency交接：open/watch/import/reimport仍在generation/project锁内构造深候选、执行全scan并同步读取/prepare全部artifact，随后clear/commit整套resource state；lazy `ensure_resident`因此未降低MVP启动和单文件热重载I/O/RSS。Runtime04按PERF-MVP-499在锁外构造metadata/delta generation、短CAS发布、startup working set按需single-flight驻留，联动Runtime11有界jobs；见`04/failure-2026-07-22-project-generation-lazy-residency-publish.md`。

- 2026-07-30 Editor F0消费链补充：current `open_prepared_project`的generation guard覆盖watcher prepare、全量scan/import、resource prepare/commit与broadcast，返回后Editor09又同步全registry读meta/artifact并建catalog，随后载入watcher/document。Runtime04的PERF-MVP-499 candidate generation必须同时供runtime residency与Editor09 catalog消费，不能只缩短manager open却让Editor二次全量I/O；长I/O锁外、短CAS publish、MVP working-set residency与last-good rollback按1/1K/100K assets验收。证据见`../../performance/01/2026-07-30-editor-ui-host-startup-project-current-review.md`。

- 2026-07-22 management projection交接：kind/list/overview/status/issue重复全registry scan+sort并深clone资产，scene records/entities重复加载同一scene，resource list比较器还分配locator String。Runtime04按PERF-MVP-500发布generation-owned compact rows/indices/summary，Editor09仅投影visible page与selected detail，Render17记录stable 60Hz build/clone/sort；见`04/failure-2026-07-22-asset-management-generation-projection.md`。

- 2026-07-23 runtime-interface resource合同补充：`resource/**` 14/14静态审查确认retained editor在任意resource event batch后消费`list_resources() -> Vec<ResourceRecord>`，因此仍会全registry深clone、以locator String排序并在主线程发布宽snapshot；继续并入PERF-MVP-500的唯一generation-owned compact rows/typed status/detail设计。locator normalize与AssetReference/ResourceId stable identity还存在replace、component String Vec、locator formatting和joined hash多层中间分配，按PERF-MVP-564以bit-for-bit golden UUID约束的单遍canonical writer/hash sink收口；owned event fanout继续归PERF-MVP-492，不另建队列truth。

- 2026-07-22 watcher背压交接：debounce原始事件Vec已按PERF-MVP-501直接改为AssetUri增量折叠，暂存O(E)→O(unique URI)；notify ingress、Pending/Draining changes/errors仍无界，每个event重置debounce可在持续风暴下长期不flush，callback还同步执行project锁内scan/import/resource prepare。Runtime04发布有capacity/max-latency/overflow-reconcile的watch generation，Runtime11只准备affected closure；见`04/failure-2026-07-22-asset-watch-bounded-debounce-generation.md`。

- 2026-07-22 importer registry交接：PERF-MVP-502已删除select逐matcher规范化String分配；registry仍为Vec全扫，capability ranking clone诊断文本，register按new matcher×existing matcher重算key，Default/active registry重复构造/clone。Runtime04联动Plugins12按PERF-MVP-503发布extension/full-suffix/id/plugin immutable generation indices；见`04/failure-2026-07-22-asset-importer-generation-index.md`。

- 2026-07-22 importer source/cook补充：owned root/model/material/glTF material深clone与shader WGSL二次文件读取已按PERF-MVP-502止损；glTF仍source bytes预parse后按path二次read/parse，OBJ重开path，subasset复制mesh/image/VG payload并O(D²)依赖去重/重复hierarchy，IBL cache hit前完整decode，font反复read/decode/metadata。Runtime04/11按PERF-MVP-504发布content/revision keyed source reader与唯一parse/decode/cook/shared artifact；WOC glTF、352/354/358和Text01 font failure承接具体格式，不另建并行truth。

- 2026-07-22 artifact store交接：`artifact/**` 16/16静态审查确认普通cache深拷贝wire DTO并整块serialize/read/decode，UI document还经TOML String中转；IBL三类store整blob读写、candidate再次clone blob且source environment复制texels。PERF-MVP-505已删除zstd独立compressed Vec；Runtime04按PERF-MVP-506发布content-addressed manifest/chunk generation、流式atomic write与按需shared decode，并与352/354唯一IBL resident owner合并。见`04/failure-2026-07-22-asset-artifact-chunked-generation-store.md`。

- 2026-07-22 OBJ format补充：`runtime_asset_path` 2/2与`formats` 9/9静态审查完成；PERF-MVP-507已删除OBJ逐face token Vec并保留少顶点错误优先级。decoder按path整文件read与importer source ticket脱节继续由PERF-MVP-504收口，禁止为formats层建立第二个source cache。

- 2026-07-22 VG cook generation交接：`virtual_geometry_cook` 5/5静态审查完成，PERF-MVP-508已把page offset O(P²)降为O(P)并复用dump排序/借用cluster ids。runtime及glTF/OBJ/model插件仍对每个非蒙皮primitive无条件同步cook且无content+config generation cache；Runtime04按PERF-MVP-509发布唯一immutable VG artifact，feature-off为0 cook，联动Plugins12请求策略与Runtime11有界并行stage。见`04/failure-2026-07-22-virtual-geometry-cook-generation-policy.md`。

- 2026-07-22 asset migration交接：`migration/**` 17/17静态审查完成，PERF-MVP-510已把report改为单String直写。一次命令仍对root执行至少4轮递归并在每reference重复filesystem probe，归PERF-MVP-511的single typed inventory；transaction每document前后完整重写全journal形成O(D²)，并多次整文件read/hash/copy，归PERF-MVP-512的compact durable state log与streaming atomic replace。见`04/failure-2026-07-22-asset-migration-single-inventory-generation.md`和`04/failure-2026-07-22-asset-migration-streaming-transaction-journal.md`。

- 2026-07-22 zrpack底层交接：`pack/**` 17/17静态审查完成；PERF-MVP-513已把manifest/reader/delta lookup切到sorted binary search，unique chunk validation不再按asset复制+重复hash，writer/delta删除全path/全target row clone。base+delta+rebuilt、writer inputs+pack output和promotion validation仍整包多份驻留并同步I/O，继续由PERF-MVP-449及Editor15既有failure统一流式收口；Runtime04提供506 content chunks，不另建pack私有chunk truth。

- 2026-07-22 model/mesh资产补充：`assets/model` 4/4与`assets/mesh` 12/12静态审查完成；PERF-MVP-514已删除model→mesh整份primitive clone和normal/tangent完整index临时Vec，VG ordinal在joint-index属性投影时原位编码。剩余morph/skin/GPU resident静态payload继续由PERF-MVP-385/386/389的content/revision immutable artifact收口，VG请求与cook继续归509；禁止mesh层建立第二套payload/generation cache。

- 2026-07-22 material资产补充：`assets/material` 14/14静态审查完成；PERF-MVP-515已把management summary从9次records遍历降为1次。overview/dependency/readiness/descriptor仍重复物化slot/reference/error集合，dependency dedup与slot反查及shader layout校验可达O(T²/P×schema)，parent chain深clone完整maps。Runtime04按PERF-MVP-516扩充358/360/404的material+parent+shader+texture revision DAG，发布唯一effective material、indexed contract与compact readiness；Render08/Editor09只消费generation artifact。

- 2026-07-22 shader资产补充：`assets/shader` 8/8静态审查完成；PERF-MVP-517已把management summary 14次records遍历降为1次并删除entry stage lowercase分配。property packing首适配最坏O(P²)，variant按entry复制全部defines，readiness/management重复持有宽report。Runtime04按PERF-MVP-518扩充355..358/404的content/schema/include generation artifact，发布确定性近线性layout、共享defines/entry/layout/WGSL与compact counters；full detail只显式请求。

- 2026-07-22 scene资产补充：`assets/scene` 13/13静态审查完成；PERF-MVP-519已把scene/entity aggregate从17/18次records遍历降为各1次，entity list不再先建scene aggregate并clone宽rows。entity overview仍为计数clone完整reference Vec，scene management内嵌全部entity rows且consumer会再次复制。Runtime04按PERF-MVP-520扩充453/474/500的scene generation，发布compact rows/counters/reference indices与selected detail handle；见既有`asset-management-generation-projection` failure。

- 2026-07-22 texture资产补充：`assets/texture` 22/22静态审查完成；PERF-MVP-521已让metadata消费唯一owned descriptor并把Cube LUT默认descriptor构造3→1。readiness仍反复normalize descriptor、parse container/format并拥有format String，Runtime04按PERF-MVP-522发布content+descriptor+device-capability keyed normalized/parsed upload generation；Bevy previous-descriptor GPU复用与texture cache作为对照，consumer不得各自重建plan。

- 2026-07-22 texture payload/chunk补充：array/cubemap/lightmap/IBL/`.zcube`与external cubemap存在source+output+scratch整块峰值、header二次parse和per-face/mip临时Vec复制。PERF-MVP-523并入504/506/352/354/380/404的唯一content chunk truth：worker直接写最终upload-ready chunks，Render13按dirty mip/face/layer预算提交；不照搬Godot 3D texture先consolidate全部image data的全块策略。

- 2026-07-22 root/project asset补充：assets根11/11、`project_document`4/4、`ui`2/2、`sprite_atlas`3/3静态审查完成。PERF-MVP-524..526已删除WAV逐sample checked Result、TTC逐table临时Vec、project document TOML String/重复parse、UI URI String与atlas name成功路径分配。Runtime04按527发布Data/project sealed typed generation，按528发布sound shared source/metadata，按529发布UI direct-reference与sprite name index；全部复用504/506唯一content truth。

- 2026-07-22 Editor asset projection/import补充：EditorAssetIndex pending reconcile已删path clone+二次remove，
  stable rows collect/sort与registry replacement全量validate继续并入PERF-MVP-500/556的唯一ordered asset
  generation；Editor09 import flow的同URI mutex只串行而不single-flight，Runtime04向Editor09暴露稳定
  source/import generation identity与唯一AssetManager ticket，实际import≤1/UUID/generation，不能让Editor复制
  worker或第二import truth。队列budget归Editor14/Runtime11，见PERF-MVP-555。

- 2026-07-23 runtime-interface project合同补充：`project/**` 39/39确认manifest每读走TOML Value→JSON Value→typed，template create还clone全embedded bytes并重复parse/encode，Editor落盘后再load/save。Runtime04按PERF-MVP-568发布content-generation绑定的唯一typed manifest artifact，直接投影summary并给Editor10/Hub03共享；JSON中间层与consumer私有parser/cache硬删。AssetRef/RelPath/project-name分配和asset-root O(R²)按569以borrowed serde、单遍canonical writer、indexed overlap与hard budget收口；迁移walker继续归511/512。

