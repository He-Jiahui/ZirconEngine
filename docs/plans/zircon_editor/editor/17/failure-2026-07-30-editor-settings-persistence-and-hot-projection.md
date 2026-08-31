---
handoff_kind: failure
status: open
created_at: 2026-07-30
updated_at: 2026-08-26
summary_slug: editor-settings-persistence-and-hot-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/17
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/settings
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
tests:
  - cargo test -p zircon_editor --lib core::settings --locked --jobs 1 -- --test-threads=1
---

# Editor17：设置持久化与高频投影性能交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-07-30 editor core settings current-source 静态审阅。
- 修复责任计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 交接原因：设置 authority、无界 change retention、同步持久化与高频 snapshot 投影跨越 Editor17/Editor05/Runtime11 边界，不能由 Performance01 在局部性能切片中建立第二套缓存或调度器。
- current-source 静态证据：`docs/plans/performance/01/2026-07-30-editor-core-settings-static-review.md`。
- 当前状态：`open / implementation_static_green / primitive_edit_controls_complete / generic_editor_state_complete / schema_enum_pointer_control_complete / string_commit_control_complete / typed_chord_capture_complete / typed_color_editor_complete / context_mutation_coordinator_complete / change_driven_health_authority_complete / operator_retry_ui_complete / file_generation_identity_complete / immutable_projection_pending / dynamic_profile_pending / managed_validation_rebind_required`。本记录不声明 Cargo 通过或产品验收；动态 keymap service、V2 payload identity 与 User locale hot-apply 的审查问题均已完成前向修复。2026-08-26 已删除 viewport feature-owned persistence owner、完成物理文档 lane 合并、接通 bool/numeric/schema-enum/string/typed-chord/typed-color/reset Workbench 基础编辑，并以 Runtime11 terminal observer 建立 User/Project health snapshot、失败通知和 Settings 标题栏 operator retry/status。Request/ticket/health/retry 已使用 target-bound `SettingsFileGeneration`；string 仅在 Enter 时经 schema 提交，chord 硬切为 `EditorKeyChord` 并由独占 capture 在全局 keymap 前处理；Color 使用 schema-owned channel step、结构化 RGBA retained payload 与通用活动 editor 状态。但 immutable file projection/digest、锁外编码、keymap 冲突/解绑、其余结构化 editor、enum 键盘/滚动/无障碍、受管 Cargo 与 F0/F4 动态证据仍未完成。

## 失败现象与复现证据

1. viewport项目吸附步进的单key UI命令clone完整registry，随后同步clone完整Project层、序列化整文档并执行write/fsync/rename/parent-sync；慢盘或大设置集直接进入编辑器调用延迟。
2. EditorManager、retained-host design-token启动入口和viewport分别拥有或加载registry，没有唯一settings authority/generation。
3. `SettingsRegistry::changes`无界增长且未找到生产`drain_changes` consumer；稳定workbench snapshot又反复分配三个静态key并查询多层BTreeMap。
4. current settings text先完整解析为`Value`检查magic，再由generic versioned reader解析；该部分必须复用Editor11/PERF-MVP-570，不建立Editor17私有解析器。

## 最低共享层根因

设置定义、运行时值、change history、持久化调度与 UI snapshot 没有围绕单一 generation authority 收敛。各消费端因此复制 registry 或重新解析/投影，单 key 更新又把整库 clone、完整序列化与同步文件系统耐久化串进 UI 调用路径；无界 `changes` 同时把短期事件日志错误地变成长期存储。

## 架构修复验收

- Editor17拥有唯一settings authority：注册定义后发布typed key slot与immutable generation snapshot；EditorManager、retained host、viewport和设置页只消费同一authority或delta，不复制完整registry作为第二真相。
- no-op set不得增长revision/event；change delivery使用有entry+bytes+age预算的cursor/delta，不能以无界内部Vec保留整个编辑器生命周期。
- 持久化只接收 typed changed key/scope/generation；Runtime11 共享 bounded lane 按 scope/physical path 合并 latest generation，在 worker 上构建最终文档并复用共享 atomic writer。UI/frame caller 不执行 filesystem；flush/shutdown/error/retry/cancel 有明确 typed 结果。后续必须把 authority trigger 升级为精确 file generation/digest。
- Editor05的snapshot投影只读取预解析typed slot或同generation共享值；稳定settings generation不得再分配`SettingsKey(String)`或遍历definition/session/project/user树。
- startup的user/project source每generation至多读取一次；严格envelope和typed payload遍数遵守PERF-MVP-570。

## 验收

- keys/definitions `1/1k/100k`、events `1/1k/1M`、snapshots `60/120 Hz`、values `0/1KiB/1MiB`、filesystem `0/10ms/2s`、writers/consumers `1/16`。
- 记录registry owners、file reads/decode passes、full clone bytes、key/String alloc、BTree probes、journal/queue entries+bytes+age、writes/fsync、caller wall、RSS与p50/p95。
- 必须满足authority=1、UI caller filesystem wall=0、single-key full-registry clone bytes=0、stable snapshot key alloc/probe=0、journal/queue内存硬有界、no-op event=0；precedence、keymap、design tokens、MRU、snap、restart、crash old/new与shutdown语义保持一致。
- 通过frontmatter focused current-source managed gate，并以F0启动和F4 viewport产品trace证明；静态rustfmt或单元测试不能替代规模与调用线程证据。

## 禁止临时方案

- 不得为viewport、EditorManager或retained host各建私有缓存、线程池或settings副本。
- 不得把整库clone移到后台后宣称单key更新已优化；不得用无界channel替代无界`changes`。
- 不得删除fsync/atomic replace以换取速度，除非先重定义并验证durability/crash合同。
- 不得在Editor17复制Editor11 versioned reader或Runtime11 persistence scheduler。

## 修复结果与回传

Open state：`implementation_static_green / independent_second_review_external_handoff_open / managed_validation_rebind_required`。Editor17 已实现 authority-owned cursor/delta change log、Runtime11 shared bounded persistence lane 与 immutable typed settings snapshot；持久化不在 UI/frame caller 执行。2026-08-04 又把六个内建 key 固化为注册期 slot，snapshot 发布不再解析静态 key，未命中变更复用既有 immutable payload；MRU 已新增 authority 内的 typed mutation，字节预算淘汰会将落后 cursor 明确降级为 snapshot。2026-08-08 current-source second review 发现 project document owner 仍对 production surface 暴露会自行构造 `SettingsAuthority` 的 legacy entry，已按最低 owner 路由 Editor10 `failure-2026-08-08-project-document-settings-authority-legacy-entry.md`；该 forward handoff 未回滚 Editor17 实现，但其未 return 前不得声明全图 authority=1。

2026-08-04 independent second review 的三个 Important 已前向修复并完成再审查，`Critical/Important/Minor = 0/0/0`：`EDITOR_KEYMAP_NAME` 现在解析为动态 authority-backed `EditorKeymapService`，已解析的 manager 与 retained-host keyboard dispatch 会观察后续 override；V2、appearance 与 shell extent 仅以 `Arc<EditorDesignTokens>` payload identity 更新，不再由 authority-local generation 驱动无关 MRU/snap 重投影，也不会跨 authority 错误复用。静态格式和契约检查已通过；尚未进入受管 Cargo、F0 启动或 F4 viewport trace。

2026-08-04 locale hot-apply forward repair 已完成并二次复审 `Critical/Important/Minor = 0/0/0`：`editor.language.locale` 是唯一的 User enum authority（`en`/`zh-CN`）；`EditorContextBuilder` 先以启动快照同步 i18n，再安装锁外 settings subscriber，后续 set/clear 与持久层替换都只传递 immutable snapshot。i18n 以 settings generation 拒绝迟到快照，内部 direct locale setter 已收窄为 crate-local，不能再形成公开的第二 preference 写入口。回归覆盖 builder 的 set/clear 热应用、旧 generation 并发倒灌拒绝，以及 subscriber 在 1 秒内可重入读取 bounded delta。限定 `rustfmt --check`、`git diff --check` 和上述静态契约均通过；Cargo/F0/F4 仍必须由 coordinator 创建 current-source 受管验证，故 handoff 保持 `open`。

未完成的验收边界保持不变：必须完成 current-source 受管 Cargo、F0 启动和 F4 viewport 产品 trace，并恢复完整 M1 manifest 的 attribution/lease 后才能受管提交和写 `fixed-*` return。当前 Session 的 write_scope 不可变，而若干已归属 M1 UI/viewport source 尚未进入该 immutable manifest；这只延后 accepted closeout，不得回滚已集成源码或宣称 Cargo/产品验收通过。

2026-08-26 current-source 重审后完成 mutation owner 前向硬切：`EditorContext` 持有唯一 `SettingsMutationCoordinator`，User store、active Project binding generation、persistence service 与 pending/retry 生命周期不再位于 viewport。Project open/close 绑定该 owner；viewport 只发 typed Project mutation；command-palette MRU 也不再绕过 coordinator。`set/clear` 在 User source 不可写、Project 未绑定或 invalid 时先拒绝再改 authority；回执区分 no-op、Session applied、persistent queued 和 applied-but-admission-rejected。协调器最多保留 User + active Project 两个文档状态，deferred admission/failed ticket 具有 typed retry，且 shutdown 在 deferred document 未进入 lane 时显式失败。Runtime11 key 同时硬切为 scope+physical path，因此同文件不同 key 共享 one-active/one-latest-pending，而不是复制每 key 的整文档写。

该阶段随后完成 file-generation 身份基础：request/ticket/health/retry receipt 以 `(scope, physical target, SettingsFileGeneration)` 为身份，进程单调代次避免相同路径重绑时 Runtime11 generation 倒退；authority revision 仅保留为诊断。Request 和 deferred admission 自带原 `SettingsStore`，`retry(ticket)` 不接受外部 Store，因此旧 lane key 不能写到新路径。此项仍不是完整 file-generation 修复：worker 仍读取执行时的当前完整层，尚未封存对应 generation 的 immutable file projection/encoded digest，整文档编码也未移出 authority/project owner。file-generation TDD 4/4、Editor17 48/48 与限定 rustfmt 通过；受管 Cargo、1/1k/100k、WPR、写入次数、耗时、RSS 与功耗均未执行，因此 failure 保持 `open`。

2026-08-26 同一 current-source 切片继续接通 Workbench primitive edit：category/toggle/decrement/increment/reset action id 只有一个 UI Settings owner，动态行命中通过当前分类数组索引保持 O(1)。`Int/Float` step、checked integer boundary、finite float quantization/clamp 归 `SettingSchema`，内置 autosave、job quota 与 viewport snap 显式声明步长；UI host 不按 setting key 切换 magic constant，只提交 Context coordinator 并重新捕获当前分类 batch。6 组 Python 静态合同 29/29 与限定 rustfmt/scoped diff 通过；新增 Rust 数值回归未执行，真实输入/绘制和动态性能证据为空，因此只记 `implementation_static_green`，不关闭 failure。

2026-08-26 后续非验收切片补齐 schema-backed enum 指针选择：变体只从 `SettingSchema::Enum` 投影，host 写入前再次验证 variant，open key/open row/value/options 共用 retained projection；hit 以 setting row 和 option row 直接索引，不按 setting key 分支或 click-cycle。changed 选择把 value batch 与关闭弹层合并为一次 retained-tree 刷新，公共下拉几何裁掉的不完整尾行明确阻断而不穿透。按结构规范把 805 行 Settings paint owner 硬切为 `commands.rs` 679 行、`enum_controls.rs` 127 行和 `geometry.rs` 10 行，不留兼容 wrapper。6 组 Python 静态合同更新为 31/31，Editor17 发现集 34/34，限定 rustfmt 与 ZUI TOML 通过；新增 Rust enum 投影回归尚未由受管 Cargo 执行，键盘/滚动/无障碍、真实窗口、动态 trace/功耗均为空，failure 继续 open。

2026-08-26 change-driven persistence health 与 operator retry/status 已前向完成：复用 Runtime11 admission terminal observer，Context coordinator 以固定 User/Project 槽、physical document identity、authority generation 与 submission token 发布 immutable health；Project 切换、coalesce 和 retry 的迟到 callback 均不能覆盖当前 token。subscriber 在 coordinator 锁外收到 snapshot，只有 PendingAdmission/Failed 进入唯一 notification authority；Settings 标题栏以 Project-first/User-second 固定槽显示 retryable 状态，直接 scope hit 只调用 coordinator typed retry。mutation/retry/notification 各自合并为一次 retained 刷新，稳定通知帧不读 health 或 lane diagnostics。health 5/5、既有 Settings 31/31、retry 5/5、Editor17 44/44、英中/ZUI TOML、限定 rustfmt、scoped diff 和 owner budget 通过；新增 Rust Durable/deferred/pane 回归未由受管 Cargo 执行，精确 file generation、产品 trace/功耗仍未完成，因此 failure 保持 open。

2026-08-26 继续按 Unreal `SColorBlock`/`SColorPicker` 职责边界完成 typed color 前向接线。retained 层删除枚举私有 `settings_enum_open_*`，只保留 `{kind,key,row}` 通用活动 editor；`SettingSchema::Color` 硬切为带正数 `channel_step` 的结构 variant，`SettingColorChannel` 只对 R/G/B/A 单通道执行饱和步进。resolved batch 以四整数投影 RGBA，十六进制文本不参与 mutation；Workbench 接通 alpha checkerboard swatch、四行有界 channel popup、typed hit/action 与 Context coordinator refresh。静态契约先 RED 2/2，最终 Settings 窗口 18/18、Editor17 48/48、`py_compile`、限定 rustfmt/scoped diff 与旧状态搜索通过；新增 Rust schema/projection/geometry 回归未由受管 Cargo 执行，默认 registry 无独立 Color definition，真实窗口和动态性能/功耗证据仍为空，因此 failure 保持 `open`。

2026-08-05 按结构约定完成 `core/settings/tests.rs` 硬切为 folder-backed `tests/{mod,registry,persistence}.rs`：30 个测试入口保持一一对应，公共 fixture 和临时目录助手收束在私有父测试模块，未新增兼容包装或生产 API。第一次独立审查发现子模块经 `super::` 使用的 `SettingsPersistenceSubmitError` 未在父模块重导出；现已前向恢复该共享 import，复审最终 `Critical/Important/Minor = 0/0/0`。随后 current-source 静态审计报告 `editor_manager.rs` 与 retained-host `app.rs` 的格式漂移，已以 `rustfmt` 前向收敛；7 路径 `rustfmt --check`、`git diff --check`、30 测试入口和共享 import 守卫均再次通过。`app.rs` 同时承载未提交的 M3 logging boundary work，故该 7 路径检查仅证明当前源码卫生，不替代完整 M1 immutable manifest attribution。最新独立复审还发现 test-tree manifest 未绑定本 failure record；记录现已加入 manifest，最终 re-review `Critical/Important/Minor = 0/0/0`。本条仅记录实现和静态证据，不改变 failure 的 `open / managed_validation_pending` 状态。

2026-08-05 settings presentation hard cut 已完成实现与静态门：`SettingDefinition` 删除公共 slash-separated `category_path: String`，改为私有 `SettingsPresentation`，其中 label、description 和非空 category path 都只能是验证过的 locale-neutral `settings.*` key；空段和尾点均被拒绝。七个默认设置与四个 Editor job quota 注册点均已迁移，英中 bundle 补齐 31 个 key，定义/配额测试分别验证每个 embedded locale 都能解析 label、description 与 category。限定 `rustfmt --check`、范围内 `git diff --check`、31 key 双语覆盖和 failure graph 检查（564 artifacts / 0 errors）通过；最终独立复审 `Critical/Important/Minor = 0/0/0`，确认已删除旧私有字段构造、11 个 builtin 均逐 bundle 直接验证、15-path M1 manifest 覆盖生产/配额/资产范围，且主计划不再陈述旧 API。尚未运行本地 Cargo，也不改变 failure 的 `open` 状态。两份 bundle 同时属于当前 M3 candidate，故不得伪造独立 M1 immutable manifest；协调器必须为 M1/M3 构造新的联合 current-source snapshot 后再安排受管 gate。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-26 | M3.3 generic editor state 与 typed RGBA color editor | `implementation_static_green / generic_editor_state_complete / typed_color_editor_complete / product_validation_pending` | 删除 enum-only open state 与旧 unit Color schema，统一为 kind/key/row editor；Color schema 持有正数通道步长，结构化 `[R,G,B,A]` 投影接通 swatch、alpha checkerboard、四通道 stepper、共享 popup 几何、typed hit/action 和 Context mutation refresh。Settings 窗口 18/18、Editor17 48/48、`py_compile`、限定 rustfmt/scoped diff 与旧架构搜索通过；Rust/Cargo、默认 Color definition、真实窗口、F0/F4、规模、耗时和功耗未验证，failure 不关闭。 |
| 2026-08-04 | Settings failure forward repair | `implementation_static_green / independent_second_review_green / managed_validation_pending` | 注册期 `BuiltInSettingsSlots` 消除 snapshot 发布期的 static key parse；MRU typed mutation 收回 authority，viewport mutation保持 design token/keymap/MRU Arc payload 复用；条目/字节/年龄预算都使落后 cursor 要求 snapshot。三个二审 Important 均已前向修复：模块服务改为动态 `EditorKeymapService`，V2/appearance/shell extent 改以 token `Arc` identity 投影。User locale 现经 context startup snapshot + lock-external subscriber 热应用，generation 拒绝迟到快照，direct setter 为 crate-local；回归覆盖 set/clear、倒灌和有界 delta 的可重入读取。两轮独立审查最终 `Critical/Important/Minor = 0/0/0`，局部 rustfmt 与静态契约检查通过。未运行 Cargo，受管 F0/F4 和完整 immutable manifest attribution 仍待协调器安排。 |
| 2026-08-05 | Settings test-tree and presentation hard cut | `implementation_static_green / independent_second_review_green / managed_validation_pending` | 1170 行 `core/settings/tests.rs` 已删除，替换为 `tests/mod.rs`（82 行）、`tests/registry.rs`（648 行）和 `tests/persistence.rs`（445 行）；对照 HEAD 验证 30/30 测试、35/35 函数名无新增或遗漏，逐测试体的归一化比对无行为差异，`rustfmt --check`、范围内 `git diff --check` 通过。结构审计仍确认 `oversized_production_file_count=0`；全局剩余 UI v2 owner 与重复 UI 测试树迁移债务不在本 scope。首次独立审查发现 `SettingsPersistenceSubmitError` 在拆分后未被父测试模块导入，已前向修复并复审 `Critical/Important/Minor = 0/0/0`；后续 static audit 收敛 `editor_manager.rs` 与 retained-host `app.rs` 的格式漂移，后者含有未提交 M3 logging boundary work，故检查不替代完整 M1 attribution。最新独立复审发现 manifest 漏绑本 failure record，现已前向补齐并最终复审 `Critical/Important/Minor = 0/0/0`。本轮再删除 `SettingDefinition.category_path: String` 和默认/配额注册的英文 slash path，改以私有 `SettingsPresentation` 存储 label、description 与 category localization key；七个默认设置和四个 quota 在所有 embedded locale 的解析由新回归覆盖，31 个 key 在英中 bundle 均存在。二审新增的私有字段测试、fallback-only locale coverage、M1 manifest 范围和主计划旧 API 四项问题均已前向修复，最终复审 `Critical/Important/Minor = 0/0/0`；限定静态门和 handoff graph（564/0）通过，未运行本地 Cargo。两份本地化资产与 M3 candidate 共享，后续仅可由协调器创建新的联合 immutable snapshot。 |
| 2026-08-05 | Settings registry/snapshot/authority owner hard cut | `implementation_static_green / independent_second_review_pending / managed_validation_rebind_required` | 865 行 `core/settings/registry.rs` 已前向拆为 265 行 registry（definitions/layers/precedence/change log）、275 行 snapshot（built-in typed slots/immutable projection）和 339 行 authority（publication/subscriber/project-layer lifecycle）；`mod.rs` 仅重导出现有 public surface，不保留旧路径 alias 或兼容 wrapper。TDD 结构守卫先暴露 owner 缺失和超 800 行，拆分后新增 owner/entry-point guards 与既有 M3.1/M2 guards 共 17/17 静态契约、`py_compile`、限定 `rustfmt --check` 和范围 `git diff --check` 通过。此变更未被先前 M1 immutable manifest 包含，故 manifest 已前向扩展为包含两个新 source owner；未运行本地 Cargo，独立二审和 current-source managed validation 均需重新绑定，failure 保持 open。 |
| 2026-08-08 | M1 authority uniqueness second-review routing | `external_handoff_open / managed_validation_rebind_required` | 复审确认 active project-open 路径已经注入 `EditorContext` 的共享 authority，但 `EditorProjectDocument::load_from_project` 仍以公开 production API 自行构造 `SettingsAuthority::with_defaults()`；其当前 call-site 均为 crate tests，说明应 hard-cut 为 test-only helper，不能保留可复活的第二 authority。最低 owner 为 Editor10 project/document loading，已创建 [Editor10 failure](../10/failure-2026-08-08-project-document-settings-authority-legacy-entry.md)；Editor17 不越权修改该 source，且不将这一外部待修复误写为验证通过。 |
| 2026-08-08 | M1 project-layer lifecycle second-review forward repair | `implementation_static_green / independent_second_review_green / external_handoff_open / managed_validation_rebind_required` | 首轮独立审查发现 project cache mutex 在 `replace_persistent_layer` 的 subscriber 回调期间仍被持有，回调内请求 Project 持久化准备会自锁。已以独立 operation gate 和显式 `transition_in_progress` 状态前向修复：load/clear 在回调前清空 cache 并标记 transition，回调的 `prepare_persistent_layer_for_write(Project, ...)` 明确返回 `Ok(None)`，完成 source replacement 后才安装新 cache；回归同时覆盖 load 与 clear 的 1 秒有界回调。修复后独立复审 `Critical/Important/Minor = 0/0/0`，5 组/19 条静态契约、`py_compile`、限定 `rustfmt --check` 与范围 `git diff --check` 已通过。未运行本地 Cargo；Editor10 legacy entry 的 authority=1 外部 handoff 与 coordinator current-source 验证仍未完成。 |
| 2026-08-26 | M3.3 visible value batch/read projection | `implementation_static_green / context_mutation_coordinator_complete / managed_validation_pending` | 注册期 category→keys 索引与一次锁 `ResolvedSettingsBatch` 已接入 Settings Workbench；打开/分类切换只读取当前分类 K 项，值/来源进入 retained node，paint/frame 不触碰 authority 或 filesystem。TDD RED 为 1 error + 2 failures，GREEN 为 3 个模块 9/9，限定 rustfmt/ZUI/diff/file budget 通过。写半程复审发现的 viewport project store、pending/retry ticket 所有权已由后续 Context coordinator 硬切修复；通用字段编辑、1/1k/100k、60/120Hz、Cargo、F0/F4 与功耗仍未运行，failure 保持 open。 |
| 2026-08-26 | M3.3 open-window revision-gated refresh | `implementation_static_green / dynamic_profile_pending / context_mutation_coordinator_complete / managed_validation_pending` | Settings 窗口关闭时不读 extension revision；打开稳态只比较 settings/contribution generation、locale 与 capabilities，纯值变化仅更新当前分类 batch，目录 revision 变化才重建完整 projection。修复 capability 变化未使 projection stale 的缺口，并保留 plugin template/Settings 独立 accepted revision，避免单一消费者失败阻塞另一个。plugin-template 同步迁入命名 owner，composition root 从 805 行回落至 782 行。热刷新 TDD RED 2 failures，最终 3 个 Python 模块 13/13、9-owner rustfmt 与 ZUI TOML 通过；无 Cargo、产品 trace、规模、耗时或功耗数据，failure 保持 open。 |
| 2026-08-26 | M3.3 Context mutation coordinator 与物理文档 lane hard cut | `implementation_static_green / feature_owned_submit_removed / exact_file_generation_pending / managed_validation_pending` | viewport 删除 project store/service/ticket/retry queue，Project lifecycle 改绑 Context coordinator；`set/clear/MRU` 统一前置 source 验证、typed receipt、两文档 pending 与 retry。lane key 改为 scope+physical path，shutdown 拒绝未准入 dirty 的伪成功。mutation 静态合同 5/5，新增 7 个 Rust 行为回归并通过限定 rustfmt 解析；Rust 回归、精确 file generation/digest、锁外 encode、产品 worker health、WPR/功耗未执行，failure 保持 open。 |
| 2026-08-26 | M3.3 Workbench primitive edit wiring | `implementation_static_green / primitive_edit_controls_complete / schema_enum_pointer_control_complete / string_commit_control_complete / typed_chord_capture_complete / product_validation_pending` | bool、numeric decrement/increment、schema enum open/select、commit-only string、typed chord selector 与 reset 通过共享 action owner、索引 hit 和 Context coordinator 接通，mutation 后立即替换当前分类 value batch；enum changed 路径合并关闭弹层为一次 retained 刷新，裁剪尾行不穿透。数值 step/overflow clamp/float quantization、enum variants、string maximum bytes 与 chord validity 归 core schema，UI 无 key-specific magic 分支；string 逐字符只更新焦点草稿，Enter 才提交。Chord 不再持有任意字符串：`EditorKeyChord` 对齐 command keymap identity，`chord_capture` 禁用 IME并在全局 keymap 前消费事件，Escape 取消、modifier-only 等待、有效 chord 一次提交后退出，不保留旧 payload 兼容。Settings paint owner 从 805 行按职责硬切并在接入 string/chord 后为 722/127/10/35/33/45 行。Settings 窗口静态合同 16/16、Editor17 48/48、限定 rustfmt 通过；新增 Rust 行为未执行，默认 registry 无独立 Chord definition，真实窗口、color、冲突/解绑、结构化 editor、enum 键盘/滚动/无障碍、Cargo 与动态性能未验证，failure 保持 open。 |
| 2026-08-26 | M3.3 change-driven persistence health foundation | `implementation_static_green / change_driven_health_authority_complete / failure_notification_projection_complete / operator_retry_ui_complete / file_generation_identity_complete / immutable_projection_pending / managed_validation_pending` | Runtime11 admission terminal observer 主动更新 Context coordinator 的固定 User/Project health 槽；document identity + observation token 拒绝 Project 切换、coalesce 与 retry 的迟到终态。锁外 immutable subscriber 只把 PendingAdmission/Failed 投影到唯一 notification authority；Settings 标题栏用 Project-first/User-second 固定槽显示 retryable 状态，直接 scope hit 只调用 typed retry，稳定通知帧不读取 health 或 diagnostics。request/ticket/health/retry receipt 已使用 target-bound `SettingsFileGeneration`，authority revision 仅诊断，Request/Deferred 固定原 Store。file-generation 4/4、health 5/5、retry 5/5、Settings 窗口 16/16、Editor17 48/48、限定 rustfmt 与 owner budget 通过；Rust 行为、immutable projection/digest、Cargo、产品 trace 与功耗未验证，failure 保持 open。 |
