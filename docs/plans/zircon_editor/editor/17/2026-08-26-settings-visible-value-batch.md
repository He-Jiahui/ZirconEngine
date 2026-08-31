# Editor17 M3.3 设置可见值批次与 Workbench 基础编辑闭环

## 目标与状态

- 状态：`source_complete_static_green / value_projection_complete / primitive_edit_controls_complete / generic_editor_state_complete / schema_enum_pointer_control_complete / string_commit_control_complete / typed_chord_capture_complete / typed_color_editor_complete / open_window_hot_refresh_complete / context_mutation_coordinator_complete / persistence_health_retry_control_complete / file_generation_identity_complete / managed_validation_pending`。
- 目标：在动态 Settings Workbench 中展示、编辑并热刷新当前分类的有效值与来源，同时保持单一 `SettingsAuthority`、同代际读取、Context-owned mutation 和 frame/paint 零 authority 访问。
- 非声明：本切片完成 bool、numeric step、schema-backed enum 指针选择、commit-only string、typed chord capture、typed RGBA color editor、reset、Project/User persistence status/retry 控件和物理目标/file-generation 票据身份；默认 registry 尚无独立 Chord/Color definition，keymap 冲突/解绑、其余结构化专用 editor、enum 键盘/滚动/通用无障碍、immutable file projection/digest、锁外整文档编码、Cargo、真实窗口截图或性能/功耗验收仍未完成。

## Current-Source 与参考复审

- 旧投影仅包含 definition/schema/scope，若直接用 `resolved_setting` 补值会对当前分类执行 K 次 mutex lock；若把所有通用值塞进每次 mutation 发布的 `SettingsSnapshot`，单 key 更新又会退化为 O(N) payload clone。
- `SettingsRegistry` 的 definition 集在 authority 创建后固定，适合在 `SettingsCatalog` 构建一次 category-path -> keys 索引；有效值仍由 registry precedence 在一次 authority lock 内解析，不能在 UI 建第二份分层状态。
- Unreal `Developer/Settings` 的 `ISettingsContainer`/`FSettingsContainer` 持有 category/section 目录，`ISettingsSection`/`FSettingsSection` 将值对象、可编辑性、保存与状态留在 section owner；Zircon 对齐其“目录与值/保存职责分离”原则，但使用 immutable typed batch 而不是跨边界暴露 UObject。
- 写半程复审确认旧 `SceneViewportController` 持有 active project `SettingsStore`、ticket retention/retry。该 feature-owned 写路径已硬切：Context 现在持有唯一 `SettingsMutationCoordinator`，viewport 只提交 typed mutation，project open/close 负责绑定/清除 Context owner。
- 已打开窗口复审发现两个失效缺口：settings generation 变化不会刷新当前分类值；`SettingsWindowProjection::is_current` 只检查 contribution generation/locale，遗漏 capability 集变化。后者会让 capability-gated plugin setting page 在代际不变时保持陈旧。
- color 接入前复审确认 retained 层仍以 `settings_enum_open_key/row` 为枚举建立私有弹层状态，继续按类型复制会让每个结构化 editor 各自持有一套互斥和失效语义。该状态已硬切为单一 `{kind, key, row}` 活动 editor；投影只在 key 与 schema kind 同时匹配时开放弹层。
- Unreal 本地源码 `Slate/Public/Widgets/Colors/SColorBlock.h` 与 `AppFramework/Public/Widgets/Colors/SColorPicker.h`/`Private/Widgets/Colors/SColorPicker.cpp` 将可点击颜色预览、RGBA channel owner 与 `OnColorCommitted` 分离。Zircon 对齐该职责边界：row 只展示带 alpha checkerboard 的 swatch，弹层持有四个通道控件，值变更仍提交唯一 Settings mutation owner。

## 实现决策

- `SettingsCatalog` 增加注册期一次构建的 `BTreeMap<Arc<str>, Arc<[SettingsKey]>>`，按 locale-neutral category key path 查直接条目为 O(log C)，不遍历全部 definition。
- `SettingsAuthority::resolved_settings(&[SettingsKey])` 在一次 `SettingsAuthorityState` lock 内返回 `ResolvedSettingsBatch { generation, values }`；每项保留 typed `SettingValue`、`SettingsKey` 和 `SettingValueSource`，未知 key 整批失败，不产生部分结果。
- retained host 打开窗口及每次分类切换时只请求当前 builtin 分类的 K 项批次；plugin 分类得到同代际空批次。批次转换为当前分类的 `value_text/value_source`，paint 只借用 template node data，不读取 authority、registry 或文件系统。
- 字符串 setting 的展示值不经过 metadata `trim()`，避免合法前后空格在只读投影中被悄悄改变；结构化 setting 暂不伪装成可编辑文本。
- production project document 与 viewport 自建 authority 入口复核均已是 `cfg(test)`，本切片没有重复修改或恢复兼容入口。
- 扩展 revision API 从只服务 plugin template 的旧命名硬切为 `extension_projection_revision`。各消费端仍分别保留自己的“已接受 revision”：plugin template 物化失败不能阻塞 Settings 页，Settings 刷新失败也不能伪造 plugin template 已接受。
- Settings 窗口模板保存 settings/contribution generation、locale 与排序后的 enabled-capabilities。窗口关闭先返回；打开且目录 revision 稳定时只替换当前分类 value batch；locale、contribution 或 capability 变化时才重建目录投影，并在分类仍存在时保留选择，否则回退到第一个有直接内容的分类。
- `SettingsMutationCoordinator` 保留唯一 User store、active Project binding generation、`SettingsAuthority` 与 persistence service。`set`、`clear` 和 command-palette MRU 都经该 owner；User source 不可写、Project 未绑定或 source invalid 时在 authority mutation 前失败。
- persistent mutation 回执区分 `Unchanged`、`SessionApplied`、`PersistentQueued` 与 `AppliedPendingAdmission`。每个物理文档只保留一个当前 pending owner，协调器内存上限固定为 User + active Project 两项；延期准入和失败 ticket 通过 typed `retry_pending` 回执重试。
- Runtime11 lane identity 从 `(scope, path, setting key)` 硬切到 `(scope, physical path)`，同文件不同 key 可由共享 lane 合并为至多一个 active + 一个 latest pending。关闭前若仍有未准入文档，协调器返回原始 admission error，不允许空 fence 把未持久化变更报告为成功。
- Workbench 的 category/toggle/decrement/increment/reset action identity 收敛到 `ui/settings/action_ids.rs`。动态 setting row 仍以行号直接索引当前分类批次，bool/numeric/reset 命中均为 O(1)，不会为每次指针事件线性扫描条目。
- 数值步长属于 `SettingSchema::{Int, Float}`，自动保存、任务配额和三项 viewport snap 定义均显式注册 step；整数使用 checked add/sub 后夹紧边界，浮点值量化到 schema step 网格并夹紧有限边界。UI host 不按 key 写 magic step，也不拥有算法，只选择增减方向并把结果提交 Context coordinator。
- changed mutation receipt 返回后立即重新捕获当前分类 `ResolvedSettingsBatch` 并刷新已打开窗口；`Unchanged` 在捕获/刷新/paint 前短路。reset 清除当前有效 override source，default source 为 no-op。`AppliedPendingAdmission` 只显示明确的内存已应用/持久化待准入状态，不伪造 durable 成功。
- enum options 直接来自 `SettingSchema::Enum`，retained 投影只保存 schema variants、当前值与一个 open setting key；投影阶段一次解析 open row，paint/hit 以行索引和 option 索引 O(1) 访问，不按 key 写条件分支或 click-cycle。选择前由 host 再次向 schema 校验 variant，changed 路径把新 value batch 与关闭 open key 合并成一次 retained-tree 刷新；被公共下拉几何裁掉的不完整尾行明确阻断命中，不能穿透到底层 setting。
- string row 复用宿主唯一 `HostTextInputFocusData`：行命中携带 setting key、当前文本与 edit/commit action，输入期间仅更新通用焦点草稿并局部重绘；Enter 才经 `set_string_setting` 复验 `SettingSchema::String { maximum_bytes }` 并提交 Context coordinator。Settings 不持有私有文本缓存、worker 或逐字符 persistence 生命周期。
- chord 不再是任意非空 `String`：`SettingValue::Chord(EditorKeyChord)` 复用 command keymap 的规范化、serde 与事件转换；schema 拒绝空键、modifier-only、dead/unidentified key。`KeySelector` 点击进入唯一宿主焦点的 `chord_capture` 模式，IME 关闭且所有键盘事件在全局 keymap 前消费；Escape 取消，modifier-only 等待，第一个有效 chord 一次提交 Context coordinator 后退出 capture。该分工对齐 Unreal `SInputKeySelector`/`FInputChord`，不保留旧字符串 payload 兼容。
- `SettingSchema::Color { channel_step }` 现在持有正数通道步长；`SettingColorChannel::{Red, Green, Blue, Alpha}` 与共享增减方向只对一个 `u8` channel 做 saturating step，UI 不解析 key、不拥有范围/步长算法。resolved value 以固定四整数 `color_channels` 结构化投影到 retained node，`#RRGGBBAA` 仅用于展示。Workbench row 绘制 alpha checkerboard + swatch，弹层使用共享有界几何提供四行 channel stepper；paint/hit 共用同一 `{kind,key,row}` editor 状态，通道 action 通过 Context coordinator 刷新当前分类 batch，不保留 `SettingSchema::Color` 旧 unit variant 或枚举专用 open state 兼容。
- Settings paint 按 owner 物理硬切：`commands.rs` 只保留窗口/通用行编排，enum control/popup 进入 `enum_controls.rs`，共享内缩几何进入 `geometry.rs`，string/chord 分别进入 `text_control.rs`/`chord_control.rs` 并共用 `field_control.rs`；原 805 行 owner 在继续接入 string/chord 后为 722 行，各专用 owner 127/10/35/33/45 行，均低于 800 行预算且不保留兼容 wrapper。
- persistence health 以独立 UI projection 读取固定 Project/User 槽，标题栏仅对 retryable 状态显示短状态和现有 `refresh-outline` 资产；hit 直接返回 scope，host runtime 只调用 Context coordinator 的 `retry_pending`。mutation/retry/失败通知都复用已有刷新边界，稳定通知帧不轮询 health 或 lane diagnostics。
- persistence request/ticket/health/retry receipt 已使用 target-bound `SettingsFileGeneration`，authority revision 只作诊断。进程单调 file generation 避免 project path 重绑时 Runtime11 同 lane 代次倒退；Request 和 deferred admission 自带原 Store，重试 API 不接受替换路径。该身份基础尚未封存对应整文档字节，不能当作 digest/durable-generation 完成。

## 复杂度与验证边界

- 结构复杂度：category lookup O(log C)，一次批读取 O(K) resolve/clone + 1 次 authority lock；frame/paint authority access = 0，filesystem access = 0。窗口关闭时新增 extension revision read = 0；打开稳态为 1 次 shell snapshot lock、O(Cap) capability 比较和常数次 generation/locale 比较，不重建 O(N) definition/plugin page 目录；值失效只做当前 K 项批次，目录失效才做完整投影。
- TDD RED：新增静态合同首次为 1 error + 2 failures，分别命中 batch owner、retained values prop 和 category capture 缺失。
- 热刷新二次 RED：2 failures，分别命中旧 plugin-only revision 命名和缺失的独立刷新契约；失败路径复审又排除了共享“已接受 revision”导致跨消费者阻塞的错误实现。
- GREEN：file-generation identity 4/4、值/产品投影/primitive edit/schema enum/string commit/typed chord/color/mutation owner、health 与 retry 契约保持通过，Settings 窗口 18/18，Editor17 全发现集 48/48。新增 Rust 行为回归覆盖未绑定/不可写/invalid source 前置拒绝、User durable close fence、同文件 clear coalescing、延期准入保留相同 file generation/Store、Session retry 拒绝、integer overflow clamp/float step quantization、enum options/open row、string commit-only 路由、typed chord validity、color zero-step 拒绝/通道饱和边界/结构化 RGBA 投影/弹层几何，以及 health generation/scope/status pane 转换；因受管 Cargo 不可用尚未执行。限定 rustfmt、ZUI/i18n TOML 与 scoped diff 通过。全仓 structure gate 与既存非 Editor17 计划问题不伪写为本切片通过。
- 未运行 1/1k/100k 动态规模、60/120Hz、Cargo、产品输入/绘制、截图、WPR、功耗或 persistence failure/retry；上述 O 表达式是 current-source 算法审查，不是性能验收数据。

## 后续硬前置

1. 在已完成的 `(scope, physical target, SettingsFileGeneration)` 票据身份上增加 immutable file projection、dirty/durable/failed generation 与 digest；完整编码必须移出 authority/project lock，并保留原子写与失败 dirty 状态。
2. change-driven health authority、PendingAdmission/Failed 通知与 Settings operator retry/status 已完成 source；继续以受管 Cargo 和真实产品 failure/retry/project-switch/shutdown 验证，UI 不得每帧读取 lane diagnostics。
3. string commit-only、typed chord selector 与 typed RGBA color editor 已接入；继续接 keymap 冲突/解绑、enum 键盘/滚动与通用无障碍焦点。bool/numeric/schema enum/string/chord/color/reset 已完成 source，但默认 registry 仍无独立 Chord/Color definition，仍需真实窗口输入/绘制验收，其余结构化值继续进入专用 editor。
4. 按 failure 的 1/1k/100k、突发多 key、60/120Hz 与真实产品 trace 采样后，才能判断文件写放大、authority lock、投影、caller wall 和功耗是否达标。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-26 | M3.3 通用活动 editor 状态与 typed RGBA color editor | `source_complete_static_green / generic_editor_state_complete / typed_color_editor_complete / product_validation_pending` | 删除 `settings_enum_open_*` 专用状态并硬切为唯一 `{kind,key,row}` editor；`SettingSchema::Color { channel_step }` 与 typed channel owner执行单通道饱和步进，resolved batch 结构化携带 `[R,G,B,A]`。Workbench 接通 alpha checkerboard swatch、四通道 popup、共享有界几何、索引 hit、typed action 和 Context mutation refresh，不解析展示字符串且不保留旧 unit Color/enum-open 兼容。静态契约先 RED 2/2，最终 Settings 窗口 18/18、Editor17 48/48、`py_compile`、限定 rustfmt/scoped diff 和旧状态搜索通过；新增 Rust 回归未执行，默认 registry 无独立 Color definition，Cargo、真实窗口、性能与功耗仍未验收。 |
| 2026-08-26 | M3.3 当前分类值批读取与 retained 只读投影 | `source_complete_static_green / context_mutation_coordinator_complete / managed_validation_pending` | 新增 category index、一次锁同代际 `ResolvedSettingsBatch`，打开/分类切换投影 value/source，paint/frame authority 与 filesystem 访问为 0。3 个 Python 模块 9/9、限定 rustfmt、ZUI TOML、file budget 与 diff 通过；后续已完成 Context mutation owner，通用产品字段编辑、Cargo 与动态性能仍未执行。 |
| 2026-08-26 | M3.3 已打开 Settings 窗口代际门控热刷新 | `source_complete_static_green / open_window_hot_refresh_complete / managed_validation_pending` | contribution/capability/locale 变化才重建完整目录，纯 settings generation 变化只刷新当前分类批次；窗口关闭不读取 extension revision。扩展消费者各自提交 accepted revision，避免 plugin template failure 阻塞 Settings；plugin-template 同步迁入命名 owner，`app.rs` 为 782 行。3 个 Python 模块 13/13、9 个 Rust owner 限定 rustfmt、ZUI TOML 通过；未运行 Cargo、真实窗口、动态性能或功耗。 |
| 2026-08-26 | M3.3 Context-owned typed mutation 与物理文档合并基础 | `source_complete_static_green / feature_owned_submit_removed / file_generation_identity_complete / immutable_projection_pending / managed_validation_pending` | viewport 删除 project store/service/ticket/retry queue，project lifecycle 改绑 Context coordinator；`set/clear/MRU` 共用前置验证、typed receipt、两文档有界 pending 与显式 retry。lane key 为 scope+physical path；request/ticket/health/retry receipt 进一步硬切为进程单调 `SettingsFileGeneration`，authority revision 仅诊断，Request/Deferred 固定原 Store，重试不能替换目标。file-generation TDD 4/4、Editor17 48/48、限定 rustfmt 通过；Rust 回归未执行，immutable file projection/digest、锁外编码、产品状态与动态性能仍待完成。 |
| 2026-08-26 | M3.3 Workbench bool/numeric/schema-enum/string/chord/reset 与 persistence retry 基础编辑链 | `source_complete_static_green / primitive_edit_controls_complete / schema_enum_pointer_control_complete / string_commit_control_complete / typed_chord_capture_complete / persistence_health_retry_control_complete / file_generation_identity_complete / product_validation_pending` | action identity 单一所有者；动态 row 以索引 O(1) 命中 bool、数值减/增、schema enum open/select、string text focus、chord key selector 和 reset。数值 step/边界/量化归 Settings core schema；enum options 与二次 variant 校验归 schema；string 逐字符只更新 commit-only 草稿，Enter 才提交；chord 硬切为 `EditorKeyChord` typed payload，`chord_capture` 关闭 IME并在全局 keymap 前独占事件，Escape 取消、modifier-only 等待、有效 chord 一次提交，不建立第二输入或 persistence 生命周期。标题栏对 retryable Project/User health 显示短状态与直接 scope hit，operator action 只调用 typed retry。file-generation 4/4、Settings 窗口 16/16、Editor17 48/48、限定 rustfmt 通过；新增 Rust 输入/持久化/pane 投影回归未执行，默认 registry 无独立 Chord definition，immutable projection/digest、color、冲突/解绑、结构化 editor、真实窗口、Cargo 与动态性能仍待完成。 |
