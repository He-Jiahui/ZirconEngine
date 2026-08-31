# Runtime editable-text property transaction

日期：2026-08-27

状态：`surface_property_prepare_commit_implemented_unvalidated / widget_and_accessibility_projection_converged_unvalidated / generic_external_projection_converged_unvalidated / derived_property_write_bypass_closed_unvalidated / component_raw_keyboard_edit_authority_removed_unvalidated / canonical_editable_value_property_inference_implemented_unvalidated / semantic_component_projection_only_unvalidated / numeric_external_value_type_preserved_unvalidated / partial_metadata_write_path_closed / composition_clause_clear_path_closed / semantic_text_dirty_invalidation_implemented / focus_loss_property_commit_converged / product_document_transaction_open / managed_validation_pending`

## 问题与参考结论

当前 widget 文本编辑从 TOML metadata 重建 `UiEditableTextState`，再依次写 value、caret、selection 和 composition。动态 `value_property` 若误指向 `visibility` 等结构属性，首项可被 generic mutation 拒绝，而后续 caret/selection 仍成功，形成半提交。每个字段还独立同步 binding/component/dirty，无法表达一次语义编辑。

本地 Unreal `SlateEditableTextLayout.cpp` 使用 `FScopedEditableTextTransaction` 包围语义编辑，transaction 生命周期统一 Begin/Finish edit；cut/paste 也在该边界内完成。Zircon 当前尚无产品 document service，不能伪造完整 Unreal history/document transaction，但 surface 投影至少必须保持同一不变量：整组编辑状态在发布前一次 prepare，失败零写入，成功一次 commit。

同文件的 `SetEditableText` 在外部正文替换时先清 selection、重建正文，并在旧 cursor 对新正文不再合法时把 cursor 修复到末尾；`OnBoundTextChanged` 还区分 focused bound refresh。当前 Zircon surface 先采用其最小保守子集：外部显示文本实际改变时保留仍合法的 caret，否则按 grapheme boundary clamp，清 selection/composition 后走同一事务；同值写不扰动编辑态。focused binding 冲突策略仍留给 document/session authority，不在 widget metadata 层伪造。

## 本次实现

- 新增 `editable_text/property_transaction.rs`，在任何 mutation 前验证 retained node、metadata、动态 value property 域以及 caret/selection/composition grapheme 边界。
- value property 禁止与 surface 结构属性、popup 状态、editable 派生状态、read-only/secure/input policy 和 constraint 配置重名，避免绕过结构 side effect 或同批字段互相覆盖。
- 以固定 10 项栈上属性数组提交 value、caret、selection 和 composition；底层 metadata batch 只写权威属性并返回 change/dirty，不再抢先登记 node dirty。
- surface commit 统一同步 runtime style、component state 和 binding report，只调用一次 `mark_node_dirty`、一次 clipboard revision invalidation。文本值按语义角色强制发布 `layout + text + render` dirty，不再依赖属性名恰好叫 `text/label/value`。
- focus-loss composition commit 复用相同事务；属性事务拒绝时不再继续 best-effort 写剩余字段或发布 commit event。
- accessibility `SetValue`、`ReplaceSelectedText` 与 `SetTextSelection` 不再先写 value、再逐字段同步 selection/composition，而是从 retained state 准备完整候选并以 `AccessibilityAction` source kind 调用同一事务。一次动作从 8–9 份 binding report 收敛为一份；composition 清理包含此前遗漏的 `composition_clauses`。
- `UiSurface::mutate_property` 对可编辑正文属性先路由到同一事务；generic 派生 `caret/selection/composition` 字段单写直接拒绝，不能再制造不一致组合。外部正文变化按上述 Unreal 子集修复编辑态，同值保持 no-op。
- 事务的正文存储值与 `UiEditableTextState.text` 显示文本分离并在 prepare 校验等价投影；普通文本仍提交 `String`，外部 `NumberField` 更新可保留 `Float`，不会为维护 caret 而改变 schema 类型。
- `UiReflectedPropertySource` 到 binding source kind 的映射由 binding report owner 统一提供，整批 retained/component update 保留 RuntimeState、ComponentEvent、WidgetBehavior 或 AccessibilityAction 来源，不复制第二套映射。
- 回归契约覆盖 widget 与 accessibility 的 reserved `value_property` 拒绝后 value/caret/selection/event/binding 全部不变；文本 value 改变推进一次 layout revision，纯 caret/selection 变化不推进 text revision。
- Material editable descriptors不再接收raw `KeyboardText`；component reducer删除独立的edit-state重建、Insert和十项write-back，只保留`ValueChanged/Commit/Focus`的semantic mirror与validation。菜单/command palette的typeahead入口保持独立。
- `UiWidgetBehavior`统一覆盖Material text-input role/component alias；V1编译按`query -> value -> value_text -> text`推断canonical正文属性，显式widget override优先。Surface fallback同步识别`value_text`，使FieldEditor与SourceEditor通过同一transaction写入各自schema主属性。

## 算法与性能边界

每次编辑最多检查并比较 10 个固定属性，复杂度为 `O(1)` 属性项加既有文本状态构造/编辑成本。候选属性容器位于栈上；只有实际变化的字段进入 change/binding `Vec`。没有全树 snapshot、全文 history copy、第二份 document cache或每字段 invalidation。文本 value 改变合并成一次 layout/text revision；caret-only 更新保持 render-only。当前 accessibility 完整替换测试由 9 次 report/18 条 update 收敛为 1 次 report/20 条 update；新增的两条 update 是补齐 `composition_clauses` 的 retained/component 投影，不是额外事务或 dirty 注册。

generic 外部正文更新仍只比较固定十项属性；除既有 `UiValue::display_text` 与 retained state 构造外没有第二次全文扫描、树 snapshot 或 document copy，派生字段拒绝为常数时间。新增回归覆盖外部正文缩短时一次 revision、完整组合态清理、同值 no-op、派生字段拒绝与 `NumberField Float` 类型保持。

component authority收敛减少一套每次输入重建`String`、selection、composition并逐属性写回的路径；canonical property推断发生在descriptor编译/metadata状态解析，不增加glyph/grapheme热循环，也不引入新cache。该改动是消除重复authority的结构性正确性收敛，不是已量化性能优化。

这属于结构性正确性和常态成本收敛，不是已验证性能优化。限定 Windows `cargo test -p zircon_runtime surface_external_text_value_change_commits_complete_edit_state_once --no-run` 在 184 秒内没有产生编译器输出并超时终止；随后`cargo test -p zircon_runtime_interface editable_text_component_roles_share_one_behavior_classification`也在64秒内仍处于依赖编译，已终止该命令的Cargo/Rustc子进程。两次均没有测试结果，不能算测试通过。复用同一E盘target的`cargo check -p zircon_runtime_interface --lib`随后在114.5秒完成，只有9项既有warning，确认interface production lib可编译；它不覆盖runtime crate或测试执行。尚未完成 managed runtime Cargo、allocation/CPU/RSS/power profile、fault injection 或 WGPU 截图，不能声明耗时接近 Unreal 或算法达到产品最优。

## 开放项

该事务仍提交 widget metadata，不是 `TextDocumentId + Revision` authority；没有 edit origin、change range receipt、history grouping/Undo/Redo、external rebase、focused bound-text policy、composition host ack 或 product fault rollback。generic external surface value/derived-field bypass和component raw keyboard edit authority已关闭；`RTE-P1-002`当前源码直写问题静态收敛但动态未验收。数值字段的内部文本编辑仍无locale-aware parse/intermediate-invalid/commit owner。产品M1/M2与`RTE-P1-006/007/010/011/015/032`继续开放。
