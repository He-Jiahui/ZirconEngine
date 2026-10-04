---
status: in_progress
review_date: 2026-09-07
plan_sources:
  - docs/plans/astra/optimize/01-review-and-repair.md
  - docs/plans/optimize/zircon_runtime/301-runtime-core-lifecycle-taskgraph-session-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/300-runtime-engineering-gap-synthesis-review.md
  - docs/plans/optimize/zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md
---

# 全域能力缺陷与输入安全修复计划

## 复核范围

本计划保留 Runtime/App 任务与停机、ABI 和资产输入的 canonical finding；各领域实施进入对应编号子计划。完整 Headless host、全进程 shutdown census、高级 provider、VM/WOC 和本地 Hub 服务属于已批准的全域目标，按总计划 W1-W7 推进，不以本页首批修复替代产品能力交付。

每项实施前必须在此补充触发条件、生产路径、根因、旧计划对应项、修改文件和测试。不能仅因存在 unsafe、unwrap 或字符串路径就判定漏洞。

## 优先级与验收

| 优先级 | 问题 | 回归门槛 |
|---|---|---|
| P1 | 错误输入导致路径逃逸、无界分配或进程失败 | 否定用例在副作用前拒绝；有效项目/资产仍可加载 |
| P1 | 错误被后续 cleanup/report 覆盖或完成信号丢失 | primary error 保留；每个已接收任务仅一次终态；取消、panic、超时路径可回收 |
| P1 | 失败发布半状态或破坏旧值 | validate/stage 失败后原状态不变；重试与正常提交有行为测试 |
| P2 | 重复解析、克隆、锁内重工作 | 最低 owner 的有效输出一致；批量 workload 明确分配/工作量预算，测量实际耗时 |

安全修复不引入新的宽泛 facade 或另一个 authority。对既有 open failure 先复核其当前最低 owner，只在相关回归通过后关闭。

## 实施接续状态

以下旧问题描述保留发现时的触发条件；本表记录当前源码与既有候选的接续状态，优先于旧段落的现在时措辞。隔离候选须逐文件整合并封存后才能参加主树验收。

| Finding | 当前状态 | 实施 owner / 验收范围 |
|---|---|---|
| LIFE-A1 | `implemented_pending_validation` | `runtime/07-lifecycle-deadline-and-census.md`；TaskGraph 冻结 scope 集合与终态 census 已在主树，重试/重复关闭保留同一结果；受管执行仍待验证 |
| LIFE-A2/A3 | `implemented_pending_validation` | 旧 WindowId 已过滤；action/wake wait 已限时；task graph、module、process-log 与 owner cleanup 的共享绝对 deadline slice 已落源；受管执行仍待验证 |
| LIFE-A4 | `partially_implemented` | `shutdown_until_does_not_restart_budget_for_later_scope_drains` 覆盖多 scope 阶段不重置预算；非合作式 callback、Session owner command queue、TLS 硬截止时间及全部 ABI 消费者迁移仍未完成 |
| ASSET-A1 | `implemented_pending_validation` | load 已有 metadata 与 max+1 bounded read；有效输入、增长和异常 UTF-8 仍需受管回归 |
| ASSET-A2/A3/A4 | `implemented_pending_validation` | `runtime/08-source-snapshot-integrity.md`；同代快照、显式根目录、打开句柄身份、Windows 引用别名与 save/load 限制对称已有实现；复审补入 glTF retained buffer/pixel 共享预算、直接导入缓存及统一 image view 范围检查，实际产品测试待受管批次 |
| Template pack descriptor admission | `implemented_pending_validation` | Runtime Interface template entry now rejects unknown serialized fields; omitted/empty `target_modes` follows the runtime all-target contract; the 108-test source batch passes, while managed compile/release validation remains blocked by dirty `E:/Git/zr_vm` (latest owner snapshot `87c112d27f25e2a10e2c078d61ef638954d0f8eb`, 112 tracked/submodule + 88 untracked = 200) |
| ABI-A1/A2/A3 | `implemented_pending_validation` | unsafe preconditions、deadline checked_add、profile schema 分配前检查及消费端已在树中；Host/App/Editor、compile-fail doctest 和 release decode 证据待核验 |
| 插件选择与产品资格 | `implemented_pending_validation` | `plugins/01-selection-and-product-eligibility.md`；typed selection report、required fail-closed 与 role admission 已有实现，继续核对产品目录及全部消费者 |
| Hub 事件顺序 | `implemented_pending_validation` | `hub/01-state-publication-order.md`；状态 owner 的 epoch/revision、前端响应与事件准入已有源码及浏览器回归，后端 Rust 和真实 Tauri 状态发布待验证 |
| WOC client product host | `partially_implemented` | `features/app/03-woc-client-product-host.md`；binary 已进入真实 ZrVM 的 bounded state-only lifecycle，离线准备失败可回滚并保留 picker 草稿，并明确报告 presentation/window unavailable；native services、validated presentation、network/persistence 与 managed Windows 产品验收仍待完成 |
| 导出终态 | `implemented_pending_validation` | 复用 `editor/04-export-terminal-outcome.md` 最新所有者修改，合入同一批次，不重复实施 |

新增编号子计划的细化不改变本页 finding 身份；同目录其他会话的迁移/编译修复分别保持原 owner。

Headless 与 Hub 服务候选已进入主树；Headless 真实 DLL owner 循环与共享关闭 deadline 仍待受管验证。Hub local-service 已通过 Windows Rust 测试 78/78、服务 binary 构建与真实 Keycloak/HTTP 检查 40/40；下载/安装消费者、native broker 和 Tauri 仍待实现或验收。WOC 的 prepare/commit 已强制 VM checkpoint/rollback 合同，`woc_runtime` 也已提供能力约束的生产 `ZrVmProjectVm` adapter，server/headless 已接入本地执行入口；实际 VM 内部保留状态与 Rust envelope 的原子恢复、client host 及网络/持久化闭包仍须共同验收。

## 当前源码确认项：Runtime/App

### LIFE-A1：停机重试与重复调用丢失 scope census（P1）

- `core/runtime/tasks/task_graph/engine_task_graph.rs:217-225` 在 Stopped 返回空 scope 集合，否则只从当前 Weak 注册重建集合；`:236-256` 完成报告仅投影本次临时集合。
- 超时后任务终止并注销 scope，重试成功报告便可能丢失其 submitted/terminal 统计；重复成功调用始终返回空 scopes。旧测试只验证 worker receipt，未覆盖 scope 证据。
- 修复：首次 Closing 固定有序 scope 集合直到完成；成功后保存最终 census，释放强引用时退出 graph state 锁，避免 Drop 注销重入死锁。重复调用复用终态 census。
- 验收：超时 -> 完成任务 -> 丢弃外部 scope/job -> 重试，保留同一 owner、submitted=1、terminal=1、queued/running=0；再次关闭 census 相同；成功后不继续保留 scope Arc，正常创建/销毁不增加注册表成本。

### LIFE-A2：旧窗口事件可销毁当前主窗口（P1）

- `zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs:40-49` 丢弃 `_window_id`，随后 `window_events/dispatch.rs` 将 Destroyed/CloseRequested 路由到当前唯一主窗口。
- 可达场景为 suspend/recreate 后旧 WindowId 的延迟 Destroyed 到达，新主窗口仍会被解绑并退出。
- 修复：在请求帧、输入 dispatch 或 teardown 前比较事件 ID 与当前 Window::id；None 或不匹配直接返回。保留真正属于当前窗口的关闭语义。
- 验收：same/different/None ID 行为与调用顺序；旧 ID 导致 0 unbind/0 runtime event/0 exit，当前 ID 正常派发。窗口事件新增成本为常数比较，无分配。

### LIFE-A3：会话销毁在现有超时前无限等待（P1）

- `dynamic_api/session/registry/session_store.rs:239-241` 在有截止时间的后续 drain 前调用无期限 `wait_for_actions`/`wait_for_wake_callbacks`，分别使用 Condvar wait。
- 修复：销毁入口共用截止时间，action/wake wait 返回是否排空；超时保持 slot 注册、保持新 action/wake admission 关闭、进入可重试终态并返回 TeardownIncomplete，绝不取走或释放仍活跃 session。现有 release-action 与再次 destroy 需保持串行资格。
- 安全依据：App try_destroy 只在成功后卸载库，Drop 在失败时先终止进程，不允许“返回超时但随后正常卸载”。必须复核 RuntimeHost 其他 consumer 同样遵循此合同。
- 验收：短测试 deadline 下分别阻塞 action/wake，及时返回失败、普通新 action 被拒、slot 保留；释放阻塞后重试成功并清空 census。使用通道同步，时间只作宽裕上界，不能靠 sleep 构造竞态。

### LIFE-A4：会话总销毁预算没有覆盖全部阶段（P1）

- 当前 `dynamic_api/session/registry/session_store.rs` 对 action/wake 使用截止时间，但 `dynamic_api/session/state.rs` 的后续 scope、module、graph 和 log 仍有独立等待或固定超时，总耗时可超过入口预算。
- 最低 owner 负责在入口构造一次绝对 deadline，并把剩余时间传入已有 shutdown/drain 合同；每个阶段保留已完成 receipt，超时不移除 session slot 或卸载仍活跃 DLL。禁止用仅检查“阶段开始前已超时”掩盖阶段内部无期限等待。
- 验收分别阻塞早/晚阶段，确认总预算、已完成阶段不重复副作用、剩余 owner 保留、释放后可重试及重复关闭结果一致；日志 flush 的进程级 owner 与 session 级调用必须保持现有寿命边界。

### 应修正的旧报告措辞

当前 App 已有 suspended/destroy_surfaces/exiting、逐 viewport teardown；dynamic destroy 已包含 action/wake admission、foreign allocation、watcher/scope/module/task graph/log drain。旧报告中“完全没有这些路径”的描述已过时，LIFE-A1..A3 不应被扩大为重新创建整套生命周期。全 Runtime worker census 和 ProductShutdownCoordinator 产品接线仍未闭环。

## 当前源码确认项：Project/Asset

### ASSET-A1：manifest 大小限制晚于整文件分配（P1）

- `zircon_runtime/src/asset/project/manifest/load.rs:27-30` 先 `fs::read_to_string`，随后 Interface parser 才检查 4 MiB 限制。大文件/稀疏文件可在拒绝前消耗任意 I/O 与内存。
- 修复：复用现有 `MAX_PROJECT_MANIFEST_BYTES`，file metadata 预检与 limit+1 有界读取同时覆盖普通超限和读取期间增长；保留 DocumentTooLarge/Read/UTF-8 错误的现有层级。
- 验收：真实超限文件与 counting reader 确认读取 <=max+1；exact-limit 与 +1、截断/增长/无效 UTF-8；正常 manifest/project-open 回归通过。有效 4KiB/64KiB paired workload p95 回退 <=10%，超限输入不得按文件声明大小预分配。

### ASSET-A2：辅助源路径绕过资产根边界（P1）

- `importer/ingest/import_shader_package.rs:429-475` 将 wgsl_files 原样 join/read；`import_obj.rs:20-28` 对 mtllib join 后磁盘读取；`gltf_decode.rs` 使用外部 URI 文件加载器。缺少对应 owner root 的路径约束时，绝对路径、越界 `..` 或 file URI 可读取主机其他文件。
- 修复：在 importer 现有辅助源入口使用同一规范化 locator/owner-root 验证，在读取前拒绝绝对/盘符/UNC/scheme 和逃出已授权根的路径；处理 percent encoding 与 symlink/reparse。合法根内嵌套引用应保留，不把所有 `..` 一概等同于越界。优先复用已有 snapshot/path API，不能为修复扩大整个 importer public contract。
- 验收：shader/OBJ/glTF 三种真实最小输入的越界 canary、encoded traversal 和 link case（平台支持时）均在外部读取/发布前拒绝；合法嵌套 fixture 正常，错误归因可定位引用。任何范围扩张先补计划。

### ASSET-A3：compound 全量扫描存在重复读与无界聚合（P1，后续依赖切片）

- `project/manager/scan_and_import/sources.rs:307-327` 将 compound 主源与全部成员连接进 Vec，无 per-file/cumulative 预算；shader 随后再次开文件并产生多份 String/payload。
- 建议：复用不可变源快照，分别声明文件数、单文件与总字节预算，哈希以流更新，importer 消费同代 bytes；先验证当前 source digest、reimport identity 和嵌套资产合同，才替换连接语义。
- 目标：接受的 64-file package 每个成员至多一次打开；拒绝超预算前不发布 meta/registry；报告实际保留内存放大率与 p95。此项依赖 ASSET-A2 的源边界，不能靠随意较小的上限拒绝原本合法大资产来声称优化完成。

### ASSET-A4：保存产生无法重新打开的 manifest（P1）

- `zircon_runtime/src/asset/project/manifest/save.rs` 验证结构、序列化后直接原子写入，缺少 load 使用的 `MAX_PROJECT_MANIFEST_BYTES` 4 MiB 限制；合法结构可序列化为超过限制的文档，保存成功后重开失败。
- 在序列化结果进入原子写之前复用同一常量和 DocumentTooLarge 错误，不调整格式、不另设更小上限；失败必须保留已有目标文件及身份。
- 验收实际保存/重开、exact-limit/+1 和预置旧文件的失败原子性；正常保存不额外复制完整文档。

### 已过时的资产缺陷

当前 template_pack 已有 descriptor/version/content digest/engine range/provider 和 receipt；App07 对这些结构完全缺失的描述应降为产品接线与验证缺口。glTF 空 texture source 与非法 normal index 已有底层预验证和回归，不重复修复；已有 failure 因缺受管终态而保持 open，不能从源码修复直接关闭。

## 当前源码确认项：ABI Host

### ABI-A1：安全 Rust API 接受可伪造指针并解引用（P0，soundness）

- Interface 的 `ZrOwnedResultV2`/`ZrStatus` 为公开字段 FFI carrier。`zircon_runtime_host/src/foreign_output/state.rs::decode_json` 在仅验证 null/length 后执行 `slice::from_raw_parts`；`error.rs::from_status` 也读取 status 原始 bytes。两者目前可从 safe Rust 调用。
- 修复边界：对接受原始 foreign carrier/releaser 并可能解引用的公共函数显式声明 unsafe，补 Safety precondition，并逐一将实际 App/Editor consumer 的调用放入有所有权证据的 unsafe 区间。若已有安全包装能承载 provider lifetime 则复用；不为本修复新建通用 ABI facade，不改变 V8 wire layout。
- 验收：safe-only compile-fail 文档样例不能采用伪造 carrier；现有合法 provider fixture、错误 diagnostics、重复 release/fused-session 回归保持，跨 App/Editor/Host 调用编译通过。
- 限制：此修复关闭 safe API 的 soundness 漏洞，不能验证恶意原生 DLL 返回的任意地址有效性；真正不可信 provider 需要 host-owned transport 或进程隔离，必须另立跨 ABI 计划。不得把崩溃探针成功当作内存安全验证。

### ABI-A2：解码 deadline 溢出跳过输出释放（P1）

- `foreign_output/decode.rs:26` 使用 `Instant + budget.max_decode_time`；公开 budget 构造器可接受 Duration::MAX，使解析前 panic，跳过正常 reject/release。
- 修复：`checked_add` 失败转为 typed protocol-budget error，走现有一次性 release 和 fuse/accounting 路径。
- 验收：合法小 JSON + Duration::MAX 不 unwind；release 恰好一次、协议失败和计数正确；正常与零预算现有语义保持。此修复为常数检查，无新增常态分配。

### ABI-A3：Profile files 上限在 typed allocation 后检查（P1）

- `decode.rs:27-38` 的 JSON syntax budget 以 encoded bytes+1 为上限；`:51-56` 在反序列化后才验证 max_items。Profile 允许 16MiB/65,536 items，数百万空字符串可先扩张为远大于 wire bytes 的 Vec<String>。
- 修复：在现有 profile 输出 schema 的入口增加分配前条目计数/seed，或引入有输出家族依据的图预算；不能直接将所有 JSON value 上限设为 typed max_items，因为 envelope 和嵌套 payload 不是 typed rows。先覆盖真实 profile consumer，再提出其他家族同类改进。
- 验收：files exact-limit 与 +1、未知嵌套字段、合法大字段、恶意空字符串数组；超过条目预算时业务 Deserialize 未开始、输出释放一次。记录固定 workload decode p95 和分配/RSS，正常同批回退不超过 5%。

IF09 中诊断仍构建 Vec/多 String 的旧 finding 已被当前 single-buffer diagnostic renderer 与 parity test 取代；保留验证状态区别。Runtime API 文档仍写 V7 而当前 source 为 V8，实施跨模块调用安全合同后须一并修正该实质性文档失真。

## 下游完整能力门槛

Headless 必须提供真实无窗口产品入口、运行时服务 tick、取消/信号、终态和有界退出；现有 `run_headless()` 的 compose 后返回不满足该合同。UI 模块必须按配置接入真实 driver，文本、IME、无障碍分别经过宿主输入与可观测输出验证。World/ECS、cook/pack、VM、物理/动画/导航/音频/网络/AI 及高级 provider 分别先复核原编号计划，再沿已有权威 owner 补完整纵向行为；未检查的领域保持待深审。

Hub 服务需要独立 Rust 进程的 axum API、SQLite 持久化和 CAS 文件存储，并接入 Windows 可部署的 Keycloak。沿用 Hub03 身份、权限、签名、安装事务和快照冲突合同；真实登录、越权拒绝、团队变更、目录安装、断电恢复和云冲突恢复为退出条件。静态 DTO、fixture 或仅路由存在不构成完成。

## 实施与批量测试

### Runtime185 场景光源阴影基础接线（2026-09-07）

- 触发条件：`DirectionalLight`、`PointLight`、`SpotLight` 或 `RectLight` 的 scene/asset 数据此前没有阴影开关，`World::collect_*_lights` 固定发布 `shadow: None`；shadow planner 因而只能消费测试手工构造的 snapshot。
- owner 与参考：最低 owner 为 scene light component、scene asset roundtrip 和 `scene/world/render/lights.rs` extraction。UE 的 `LightComponentBase::CastShadows` 也由光源组件持有，renderer 只消费已发布设置。
- 本轮修复：增加向后兼容的 `casts_shadow` authoring 字段；旧文档缺字段时保持关闭；directional/point/spot 开启时 extraction 发布现有 `LightShadowSettings` 并进入真实 planner，分别生成 cascade、cube face 和 spot pass。Rect 保留作者请求但发布 `shadow: None`，开启时明确报告 shading 与 shadow rendering 尚不支持；不将字段接线表述为 rect shadow rendering 已完成。
- focused regression：scene TOML true/legacy-missing roundtrip、World save/load roundtrip、既有 light frame extract/viewport packet 的 enabled、disabled 与 stable-frame presence、真实 World extraction 到 11 个 shadow slots/passes。Cargo 与图像/GPU 验收由受管 validation batch 执行，本轮静态复核不构成 GPU 通过。
- 独立复审确认同一 scene entity 可以同时拥有多个 light component，单独使用实体 ID 会让 point face 0 与 spot slot 0 的 atlas key 冲突，并让所有 family 读取最后一份 assignment。修复将 atlas/cache key 和 assignment 同时限定为 light type 与实体 ID，不限制已有 ECS 组合能力，也不改变 GPU buffer ABI。新增 `roundtripped_light_components_on_one_entity_keep_distinct_shadow_allocations` 覆盖同实体四类灯的 TOML 保存/重开、World extraction、11 个不重叠 pass、分别为 4/6/1 的有效 slot range 和 Rect 无分配；该回归尚未执行。

M0：当前源码复核、finding 去重及独占范围确定。

M1：按 owner 分别补行为回归、修复和局部格式检查；不得弱化测试期望以掩盖生产缺陷。

M2：将多个任务合并为最小完整的 Runtime/App/Interface 验证批次；涉及持久化或 unsafe 的变更覆盖失败路径、边界值和调用层回归。

M3：独立复审与性能核验；真实进程/DLL/产品验收没有执行时，明确保留其状态，不能从单元测试推导整个能力完成。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Runtime185 RT-LGT-02/05 基础接线 | scene light `casts_shadow` authoring、asset roundtrip、extract 到 planner | `implemented_pending_validation` | 2026-09-07 | `scene_asset_toml_roundtrip_preserves_point_and_spot_lights`、`scene_assets_roundtrip_ambient_and_rect_light_product_fields`、`render_frame_extract_filters_lights_by_camera_layers`、`authored_scene_shadow_switches_reach_shadow_frame_planning`；图像/GPU 待受管证据 |


### 2026-10-03 Navigation r5 正常 owner 与任务消费者续接

本段记录原 Navigation 实施线的37个直接源码变更；publication
`9adb98e26c37fd286f5915c49642299df702ee88c53c078ecc076fa82ed1059b`，严格原始字节重建
`60a90df5bd8de7e229d6e185ebeba5f732cbaa64797bb2f134421913ef632c60`，独立复审
`2814bc1abeacfdf5bd1e07d4eba1863c40d7933cb55af11ca514fcfc1ea407b5`。
Root 没有替换外部变更或重置 index，没有运行测试；Core Session state 原片段仍由
Core 唯一 owner 组合，尚未据此宣称 Session/Editor 全链路接线通过。

`manager/bake.rs` 实际挂载的 generation fixture 已通过 ProjectAssetManagerAccess
获取正常项目资产 owner；正常 CoreRuntime、LevelSystem 和 RuntimeOperationService
使用 tick/poll/harvest 验证候选的加载 handle、新 generation、owner ABA 和失败终态合同。
直接/tiled/dirty/operation 四条 generation-exhausted 入口在修改 task/context 前拒绝，
真实512-byte worker reservation 的 completion loss/retry fixture 保留恰好一次回收断言。
上述是已落源的测试设计与合同，执行数为0；MAX拒绝仍须实测 loaded map、mutation epoch、
保留字节和任务集合保持不变。Recast 输出、普通 DynamicSession/Editor undo/redo、取消与
deadline interleavings、Windows产品、release p95/RSS及Jenkins继续开放，不从源审计关闭
原 Runtime169/其它编号报告的P0/P1或原failure生命周期。
