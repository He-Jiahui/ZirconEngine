# Runtime NumberField edit-buffer and typed commit architecture review (2026-08-27)

## 状态

`numeric_value_kind_corruption_fail_closed_unvalidated /
invariant_ascii_edit_buffer_and_typed_commit_implemented_unvalidated /
enter_blur_escape_policy_implemented_unvalidated / per_key_publish_explicit_opt_in /
locale_type_interface_open / focused_numeric_refresh_open / managed_validation_pending`

## Current-source finding

Material catalog 声明 `NumberField.value: Float`，默认值、min/max/step/large_step 也都是数值。
Surface 的共享文本入口却把 NumberField 分类为 TextInput，而
`commit_editable_text_properties` 对任何内部字符、keyboard、IME、paste 或 accessibility 编辑都使用
`UiValue::String(state.text.clone())` 作为 canonical value。

因此旧路径没有“解析不正确”这么简单，而是 authority 类型已经错位：

- typed numeric model value 与用户正在编辑的字符串共用同一 `value` 属性；
- `-`、`.`、`1e` 等合法编辑过程中的 intermediate-invalid 状态无处保存；
- retained metadata 可先变成 String，component schema/binding 才在更晚层看到不一致；
- Arrow/drag reducer 继续期待 Float，后续行为依赖错误值被哪个投影覆盖；
- 现有测试只覆盖 typed arrow/drag/Commit，不覆盖 Surface text/IME/blur/cancel。

## Unreal reference

主要参考：

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Input/SSpinBox.cpp:937-990`
  的 `TextField_OnTextChanged/Committed`；
- 同文件 `993-1076` 的 `CommitValue`；
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Input/SNumericEntryBox.h:719-741`
  的 `SendChangesFromText`。

Unreal 的结构是：

1. `SEditableText` 保存用户字符串，`ValueAttribute` 保存 typed numeric value；
2. type interface 负责 character admission、`FromString` 与 `ToString`；
3. per-key change 只有在完整字符串可解析且策略允许时才发布 typed value；
4. commit 再 parse，随后按 numeric policy clamp/snap，发布 typed changed/committed；
5. `CachedValueString` 避免稳定值每帧重复格式化；显式 edit/display formatter 可以不同。

## Implemented safety slice

共享 editable property transaction 的 preflight 现在比较 retained canonical value 与 proposed value
的 `UiValue` variant。variant 不同返回 `ValueKindMismatch`，诊断码
`value_kind_mismatch`，发生在 metadata batch 之前。

该不变量保证：

- NumberField 的 `Float` 不会被内部 `String` text edit 覆盖；
- value、caret、selection、composition、component state、binding、dirty 全部零部分写；
- generic external `Float -> Float` 更新继续通过同一 transaction；
- 检查为常数时间，不扫描文本、不构造 tree snapshot、不引入 cache。

回归固定 actual `NumberField`、`widget.value_property = value`、`value = Float(42.0)`，派发字符输入
后要求 Float/caret/event/binding 不变并收到 typed diagnostic。focused Windows runtime 单测约 70 秒
仍无编译器/测试输出，随后核对并终止 exact Cargo/Rustc process tree；该回归尚未取得动态 Cargo
pass。

## Required numeric edit session

完整实现不能把 `value_text` 仅作为另一个随意属性。产品 edit session 至少需要：

- committed value：typed numeric value + model revision；
- edit buffer：String + caret/selection/composition + edit base revision；
- parse receipt：empty/intermediate/valid/out-of-range/non-finite/invalid-character，不能靠字符串日志；
- number format identity：locale/numbering system/decimal/group separator、precision、rounding；
- commit policy：per-key typed change 是否启用，Enter/blur/drag/arrow 的 commit method；
- cancel policy：Escape/host cancel 恢复进入编辑时的 formatted value；
- external refresh policy：复用 focused bound-text 的 pending/revision/conflict owner。

MVP 可以先限定 invariant ASCII/`.` parser，但必须在 DTO 中显式声明 format identity，不能让平台
locale 隐式改变 parse。任何 `NaN/Inf`、overflow、min/max 反转或 step 非有限值 fail closed。

## Algorithm and profile plan

本轮没有做性能优化。实现 numeric session 后才运行同输入规模 profile：

- per-key：character admission、parse attempt、allocation count、changed-event count；
- commit：parse、clamp/snap、format、binding publication 的 p50/p95/p99；
- stable render：数值未变时 `ToString`/allocation 次数必须接近零；
- focused external refresh：pending replace/update/conflict count 与 resident bytes；
- 对比 Unreal 同类型、相同 precision/locale、相同 key sequence，记录 CPU/RSS/power 条件。

在 profile 前不建立额外 formatted-string cache；cache key 必须包含 typed value、format identity 和
format generation，并在外部 value/locale/precision 改变时失效。

## Next implementation order

1. 版本化 numeric edit-session DTO 与 parse/commit receipt；
2. 独立 edit buffer，不再让 `value` 兼任 String；
3. invariant MVP parser、finite/range admission、Enter/Escape/blur；
4. typed event/binding transaction 与 external refresh/revision；
5. locale type interface、format cache和 profile；
6. managed Surface/IME/a11y/clipboard/product WGPU tests。

## 2026-08-29 invariant NumberField MVP implementation update

已按上述顺序完成不依赖验收队列的基础闭环。公共 Runtime Interface 新增 versioned、无正文的
`UiNumberInputReceiptV1`，固定记录 `invariant_ascii` format identity、parse status、commit method 与
commit status；它进入 `UiInputDispatchDiagnostics.number_input`，不会复制或记录用户输入字符串。

产品状态不再让 `value` 同时承担两个类型：

- `value` 始终保持 `Float` canonical model；
- `value_text` 保留活动 edit buffer，`number_edit_active` 决定 input/render 是否使用它；
- 默认字符、IME、clipboard 和 accessibility 文本编辑只更新 buffer、caret、selection、composition；
- `number_publish_per_key = true` 是显式 opt-in，且只有完整、finite、range-valid 的文本才原子更新
  `Float value` 并发布 typed Change；
- Enter 解析、clamp、可选 step snap 后发布 typed Commit；intermediate/invalid Enter 保留 buffer；
- focus loss 对合法文本执行同一 typed commit，对非法文本恢复 canonical display；Escape 恢复并取消；
- NumberField 默认 single-line + number filter，并允许科学计数法的 `e/E`，平台 IME commit 只结束
  preedit，不被误当成字段 Enter commit。

共享 editable property prepare/commit 仍是唯一 metadata/component/binding/dirty 写入口。NumberField
事务把 Float canonical、String buffer、active flag 与固定编辑状态一起 preflight/commit；variant guard
仍在首写前 fail closed。render 与 input 都从同一 retained metadata 读取 active buffer，未引入第二个
document、binding registry、format cache 或 locale 猜测。

源码回归覆盖 buffer 不破坏 Float、out-of-range receipt、Enter clamp + typed Float Commit、非法 Enter
继续编辑、Escape cancel、invalid blur restore、Unreal-style Up/Down canonical step 与坏 step 零写入，
以及 accessibility typed SetValue；descriptor 回归固定 format/policy/state schema，DTO
回归固定 serde/default/version 和 diagnostics legacy default。scoped Rustfmt 与 `git diff --check` 已通过。
本切片尚未取得 managed Cargo、allocator/RSS/latency/power、真实 platform IME/a11y/clipboard 或 WGPU/PNG
通过证据，不能关闭 Text08/Runtime82。locale/precision/rounding type interface、focused external numeric
refresh/revision conflict 与 keyed formatted cache 仍开放；在取得基线前不实现 cache 优化。

热路径追加 8 个固定 profile counter：parse 次数/输入字节、edit/commit 决策、typed publication、clamp、
snap 与 keyboard step。property transaction receipt 直接携带已计算的 numeric decision，普通编辑只
parse 一次。V1 edit buffer 的命名硬上限为 128 bytes；节点上限只能降低，不能提高，直接 parser 对
超限输入返回 `TooLong`。

`SSpinBox::OnKeyDown` 的外层路由复审确认 Up/Down 不能先落入单行文本光标移动。Surface 现在于通用
text-edit action 前处理无修饰 Up/Down，基于 canonical Float 和正有限 `step` 原子步进；活动的临时
buffer 不参与算术，成功后规范化并退出 edit mode。坏 policy/step 与算术溢出 fail closed，receipt 标记
`KeyboardStep/Rejected`，保留原 buffer 和 edit-active 状态且不发布 typed event。
Windows managed build + `number_field` lib-test 批次约两分钟零输出且无终态后只结束本地等待；未查询、
轮询或重试 coordinator，因此仍记为 validation pending，不能形成 commit 或性能结论。
