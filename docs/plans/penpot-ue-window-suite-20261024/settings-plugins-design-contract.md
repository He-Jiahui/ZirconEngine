# 设置与插件：设计及接线合同

## 2026-10-25 r96：健康捕获提前返回与旧批次失效合同

生产路径为 `RetainedEditorHost::sync_open_settings_window` → 原 bridge 保存的 `settings_values_context` → 原 `settings_values_context_is_current` → 原 coordinator phase/binding 与 authority generation 检查；不从最新 snapshot 替代显示态。健康读取的 `OperationInProgress` 不能直接跳过该原上下文检查。

`settings_persistence_capture_failed(values_are_current)` 由该实际 caller 消费：始终清 retry；false 同时清值上下文并通知 caller 沿 r94 清理旧设置 native focus/抽屉返回焦点，true 保留完全相同的值上下文/草稿。仅恢复 health 不能复活已撤销的值批次，必须正常 values refresh；未取得 mutation reservation，不代表后续写入、关闭重开排队事件或原生 frame/window lease 已被证明。

新增两条回归定义位于原 `health_materialization_tests::capture_failure`，复用原 `OwnedSourceWriteFixture`、coordinator、authority 订阅与 pending persistence，不创建 String provider/第二状态。当前静态2项，harness unknown、执行未运行。原 r94 native focus 两项为另一个未来 filter，不混入 r78 或 r35/r37。详见[当前源合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-health-capture-retirement-r96/source-contract.json)。Penpot rev355/r95 设计及其失败保持原事实，本片不是设计导出或产品截图。

## 2026-10-25 r95：原生All及叶条目消费的布局合同

真实file=b4a1e3ce-9e6a-81fe-8008-d69d60e4e402/page=49ff1071-f83b-4b72-bb6e-043a3326ec82，saved/fresh rev355。[设计合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/design-contract.json)记录3个native board身份；[布局测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/layout-review.json)是Penpot sidecar，不冒充LayoutReviewCase或产品捕获。

| 设计尺寸 | 导航初始宽 | 详情宽 | All/叶行高 |
| --- | --- | --- | --- |
| 640×520 | 128 | 512 | 40/24/24 |
| 900×620 | 180 | 720 | 40/24/24 |
| 1280×800 | 256 | 1024 | 40/24/24 |

主体实际flex/row、详情fill，旧grid属性仅为inactive元数据。导航/详情模型邻接差均0，不推导UE≤1逻辑像素结论；实时比例/splitter未实现。复用原TreeRow/SearchInput实例及source pluginData/native touched，All实际使用Shared strong 9ac86733-8214-5a04-820a-9074b5c68310，positionData与画面均700/uppercase。叶箭头opacity0保留无子项Hidden的占位，尚不证明产品hit/selection。40/24是当前字体行框加UE padding的设计推导，未有UE实测几何。

All与两个叶角色是明确作者设计例子；实际SettingsNavigationCategory只有domain/keys/labels，不按翻译文案推断生产层级。根/All长文本Ellipsis、叶tooltip及真正分类根继续pending，不拼假省略号。六个setting key只映射既有设计正文，未新造provider/store/clock或称文字value为控件。

Shared published rev5，linked syncedAt=2026-10-24T17:52:24.357241Z、modifiedAt=2026-10-24T20:30:23.583474Z，原生仍提示更新；未同步/ACK/reset或正式导出。r53整页同步失败/r92长根失败保持。三组零重开差仅证明本次Penpot保存稳定；动态语义、normal producer/EXE/实际150%DPI、native origin、托管Rust/plugin及跨族主题均open，`passed=false`、`accepted=false`。

## 2026-10-25 r94：显示批次替换与现有native draft的生命周期

Settings String/Numeric/Chord原生focus按原commit action常量识别。open/目录或值替换/extension失效/category/changed receipt必须先撤销已有focus和原抽屉返回快照，再替换bridge上下文；撤销仅走原native cancel与damage redraw，不调用提交，也不捕获最新authority批次。主窗加原committed model登记的existing presenter，未另造窗口/store/clock。其他输入、原saved unrelated focus、无变化及health-only刷新保持。[两源实际增量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/source-change.patch)。

这是旧草稿生命周期补片，不能代替输入来源合同。原callback_methods是同步调用；没有观测到由该macro创建的异步queue。已捕获/已排队事件仍须在native hit/focus前固定原window/open lifetime+published semantic batch+typed context，在原事件序列内复核；当前full/sparse发布源仍未携带该设置authority envelope。geometry/structure、normal capture元数据、runtime lease和消费时取最新context均不足。[边界及当前公共前像](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/origin-boundary-current.json)。

[独立2定义验证意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/regression-contract.json)只覆盖现有in-process native focus/Enter状态路径，正控确认撤销前可提交。显式fixture不提供String产品provider，不证明真实浮窗生命周期、关闭不重开、IME preedit/取消、GUI或资源就绪。14源74静态定义/原72保留；正常Jenkins源码/member/writer/resource/pool/cache/native证据、类型与执行仍open。

## 2026-10-25 r93：直接值/分类刷新与已有重试显示态同成败

三既有bridge方法现统一遵守原上下文所有权：刷新前撤销值并暂取原retry Option，全部写值/分类/编辑器状态及template刷新完成才安装传入值并原样恢复retry。失败保持两者无上下文；成功不捕获最新authority或pending状态，不重新发出重试。原None必须保持None，尤其真实refresh_after_settings_receipt已先撤销它；不因本桥接补造可点击retry。

原coordinator/authority同锁代际与project binding条件写没有变化。新回归故意让传入batch早于当前authority，要求bridge保留所给generation，后续真实mutation仍需原currentness条件；显示内容不等于当前授权，更不等于事件来源frame/open lifetime。原始category/key、scope、完整operation tuple和persistence pending保持。

[当前14源/72静态定义](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-value-category-materialization-r93/source-pins.current.json)只改原settings_window.rs；新增2定义唯一filter为health_materialization_tests::value_category_state_change::。原health宽prefix现8定义，旧prepared输入不能被静默扩大。用已有OwnedSourceWriteFixture/真实coordinator作为单元fixture，未创建生产provider、原生窗口或假数据预览。实际MissingNode失败/重建恢复属于待执行回归，单独property-write失败尚未注入，GUI/类型/Rust均未证明。[真实调用图与边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-value-category-materialization-r93/source-trace.json)。

精确源码patch83251265…、format/reverse/diff/sourceguard0，不是Jenkins准入或Rust结果。未来仅zircon_editor/defaults off/features[]/Windows lib test/locked/jobs1/thread1/物理E，实际profile与完整源/资源/native门由原authority回执。r92根/All Ellipsis与普通节Clip的源合同/视觉失败保持；本片没有Penpot/公共源/codec修改，passed/accepted=false。

## 2026-10-25 r92：分类文本按实际UE角色消费，单行试验不等于根行还原

[真实文件](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60e4e402&page-id=49ff1071-f83b-4b72-bb6e-043a3326ec82) Project页49ff1071-f83b-4b72-bb6e-043a3326ec82新增三张原生122节点试验板：640=2b6b73e2-4770-8089-8008-d828d6a935a9，900=c149e6b0-945d-80c7-8008-d82aa8f6ff9d，1280=c149e6b0-945d-80c7-8008-d82ad68693b6。来源仍原TreeRow main/component与SearchInput，完整metadata/文字保留，不detach/仿造省略号字符串或共享main；旧失败板c79af4cc…不变。

UE CategoryItem选中的最终样式必须追完整继承：根/All=PrimaryButtonText→ButtonText(Ellipsis)，NormalBold/Uppercase，Root外padding2/14/2/10、All2/11/2/9且保留原内侧2；普通节=NormalText→默认Clip，两者默认不换行并设置真实description tooltip。上述padding不推算总行高，不能用现dense28签UE行框。角色须来自同源typed结构/域映射，不能从翻译标签猜测；两条中文压力字符串不证明生产section分类。

当前auto-width/fill+row clip仅验证同一原生行的单行布局，9个label框宽为76/128/204，保留44左偏移和16高。根/All的Clip试验不符合其Ellipsis；分栏边界相贴、精确内边距裁切、字体/CJK闭包、图标与tooltip仍open。通用源已有text_overflow=ellipsis，应沿既有navigation/navigation-leaf提案267f6013复用同一TreeRow并由Parent/public owner接角色/codec，不全局改Clip、不新建主题/组件/store。[精确源码与更正](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/source-assets.json)及[待接合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/category-source-contract.json)。

保存关闭/全新重开完整data和三尺寸画板像素一致，24图逐张复核；这不是plugin正式export、正常EXE、GUI输入/焦点、150%DPI或双端≤1DIP证明。本片没有源资产/Rust改动，14源70静态定义沿r91原验证意向，不扩原测试请求，passed/accepted=false。

## 2026-10-25 r91：原弹层/滚动 materialization 的双上下文条件保留

toggle_settings_editor、close_settings_editor、update_settings_scroll_offsets 已在同一真实 bridge 中与原 values.take 并列取出原 settings_persistence_retry_context；原 property 写及 refresh_after_state_change 全部成功才恢复同一对象。失败时两者为None，普通修复重建不能恢复权限；实际 health capture 只恢复 pending retry，原 values refresh 再安装实际显示批次。noop 不取走上下文。不改原设置 key/scope、coordinator phase/authority lock、事务/persistence retry tuple，也不从最新 snapshot 换取显示态证明。

实际生产 surface/option.rs 的 Enum/Color 打开/关闭及设置窗口滚动入口沿原 bridge 消费；retry入口仍从 bridge 捕获后交原 coordinator。同族 String/Numeric/布尔/枚举条件写保持。2新静态回归覆盖四transition的成功/失败/修复及noop，既有pending_user_settings/OwnedSourceWriteFixture只用于单元回归，不补产品provider。filter与当前14源见 [r91 contract](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-popup-scroll-materialization-r91/regression-contract.json)，不把2新定义扩入r83/r88或旧终态输入。

仅1 canonical源实际变化，前像3fec0d9c…、后像4e936f57…；原68定义保留，70不算harness发现或执行。失败注入只覆盖已有MissingNode；未独立注入partial property拒绝，也未扩其它direct category/value路径。原native event缺origin frame/open lifetime语义不由本片关闭。正式插件导出、GUI/native捕获、EXE/实际DPI/A-B仍未证明，passed/accepted=false。

## 2026-10-25 r90：SearchInput 原生消费实例约束与正文能力边界

实际 own file b4a1e3ce-9e6a-81fe-8008-d69d60e4e402，Project page 49ff1071-f83b-4b72-bb6e-043a3326ec82，保存/关闭/新浏览器重开均 rev307。复用原 Shared b4a1e3ce-9e6a-81fe-8008-d69d607aac12 rev5 的 SearchInput component 6ddc248b-bd35-80c5-8008-d6c8c3ce892e / main 6ddc248b-bd35-80c5-8008-d6c8c0f9c196；三画板/实例/子节点 ID 及完整 geometry 在 [mutation-saved](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-content-flow-r90/mutation-saved.json)。背景 sourceRef 6ddc248b-bd35-80c5-8008-d6c8c111a1b8、文字 6ddc248b-bd35-80c5-8008-d6c8c13d528a 的消费者宽度和 constraintsH=leftright 为显式原生 override；图标 sourceRef 6ddc248b-bd35-80c5-8008-d6c8c121ce77 与其子树完全不变。外层原 fill/flex 合同不变，不把背景插入文本流，也不修改 Shared main。

实际三尺寸背景左右边界误差为0，文字右内边距8（最大浮点差约3.6e-12），只说明本 Penpot 几何；未测 UE/EXE 字行误差或系统 DPI。34色/3字体样式及引用保持，全部 pluginData/内容哈希精确不变，不重签 guard；正式 exporter 尚未执行。原 v2 search_input.zui 仍是 WorkbenchSearchInputRoot/SearchField，query/placeholder、value_property=query 和 Stretch 合同，不能把本轮6个 native 布局子节点说成已导出的产品控件树。

Property native main 仍缺 description/source/reset 新槽且 required value 空，六条正文是原草稿 frame，不插入假 Text 充当 value。原 String/Numeric typed context、coordinator/authority 条件写与 r88 撤销逻辑继续沿既有源；本轮不改 Rust、不扩测试或宣称 UI 行为通过。当前 [source audit](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-content-flow-r90/native-source-audit.json) 只证明实际源引用有效，library synchronization ACK、共享槽迁移/正式 export/真实语义 producer、normal EXE/150%DPI/主题 A/B 仍 open，passed/accepted=false。

## 2026-10-25 r89：IME 文字落地与属性提交必须保留为两步

真实 Runtime 将输入法已确认文字保留为结构化 `Ime(Commit)`，canvas/retained Session有优先权；当前旧焦点回退仅识别Text。已给 Parent 3路径精确未应用候选，只在最终旧回退提取 Commit 的文字，继续使用原 focus draft insertion。Numeric/String 的 commit_only、原 Enter、saved typed context、coordinator/authority 条件事务均不改；不将平台 IME Commit 全局改成Text、不新增组合store/clock/backend。

两个新 pure translator/fallback 候选覆盖普通Text及IME的中文/组合字符/空文字，并排除Preedit/Cancel/DeleteSurrounding/Enabled；原翻译测试更正为结构化Commit。它们不证明焦点、插入回调、Enter/Escape、组合绘制或GUI。独立filter和实际3前像见 [r89合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-legacy-ime-commit-r89/regression-contract.json)，尚未集成/编译/正式收件。

当前14域源仍 r88/68静态定义。UE composition edit transaction与OnEnter owner commit分别保留为目标；完整composition/undo/UTF8 range、native origin/lifetime、真实provider/4page/typed subtree/export/EXE/实际DPI仍pending，r87真实设计及长标签失败未更改。

## 2026-10-25 r88：公共重建的设置上下文失败合同已落域内源

实际通知走 prepare health→generic prepared rebuild，绕开 r83 health wrapper。真实 bridge 现在先取出原 values/retry，整次准备成功才恢复同一对象；失败撤销，不抓取最新值代替显示态。恢复节点后普通重建仍不能新建上下文；仅原正常 health/value capture 分别恢复其权限。此为原14路径内两源的生命周期最小修改，不改公共主题、PropertyRow、codec、native origin 或 mount/DPI 算法。

新增2个独立未执行回归定义，实际 filter 为 `ui::retained_host::callback_dispatch::template_bridge::workbench::settings_window::health_materialization_tests::prepared_state_change::`。原66定义保留，当前68不是 harness 通过数。原 r83 父模块现在静态包含4定义，不静默扩大历史2定义的执行意向；新范围单独交原执行 authority。现14源/5只读依赖和精确逆增量见 [source audit](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-prepared-materialization-r88/source-audit.json)。

r86 Numeric 解析/条件提交及 r87 native linked 设计仍完整；native input origin/IME、生产 provider、4页、共享 typed subtree/export/normal EXE/实际DPI继续 pending。未给新 Penpot 或 runtime 行为签发通过。

## 2026-10-25 r87：原 linked TreeRow 的域内流式覆盖与测量边界

Own file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` / Project page `49ff1071-f83b-4b72-bb6e-043a3326ec82`，保存并新进程重开 rev295。受控板640=`3d59769b-14c4-8077-8008-d811a4506b74`、900=`d786d7a2-c6f8-80d9-8008-d812a59f5783`、1280=`d786d7a2-c6f8-80d9-8008-d812cd889d31`；长标签设计夹具640=`c79af4cc-85c3-80cc-8008-d81334da9160`。全部由实际 native duplicate 分配ID；没有发明UUID或编辑 Shared。Shared rev5 的 TreeRow component `6ddc248b-bd35-80c5-8008-d6c8c7a79944` / main shape `6ddc248b-bd35-80c5-8008-d6c8c452bf0c` / label source `6ddc248b-bd35-80c5-8008-d6c8c4c69ae5` 保持。

布局：原20%侧栏和单一既有区域所有权保持；原 linked row 改为横向 flow，展开标记、类别图标、标签依次分配。沿现有 `editor.density.gap.small=4`、`gap.medium=8` 对齐间距/内边距；保留原 dense 行高和图标层级；标签 start 对齐、宽度 fill。Penpot 的原生 reverse 层序得到实际从左到右内容流，不改 shapes 列表。隐藏动作的原 visibility override 保持，未添加滚动拥有者、手工坐标补偿或新主题。原 native main 仍是自由 painter 投影；本地流式覆盖不代表正式 source/export codec 已支持。

短标签框在128/180/256侧栏下宽约76/128/204，横向溢出0；长英文/中文仍换行5/4/4并越过固定行高。`positionData` 的行Y跨度记录64/48/48左右，不将其Y约定臆解为 frame 边界。源 `tree_row.zui` 的 `text_overflow=ellipsis` 未由设计端证明；禁止手插省略号、裁剪整集合或重签 metadata 伪装通过。所有4板插件 metadata 与对应原节点相同，source/shapeRef/library revision、覆盖及样式引用见 [metadata-source-audit](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-flow-r87/metadata-source-audit.json)。

仍待原 public/source owner 的导航/叶节点变体、单行/tooltip/ellipsis 和官方导出支持；本片不修改其 Rust/codec/Shared。6属性值是设计夹具，搜索、typed editable tree、实际页scope、provider、input origin/IME 与 normal producer/EXE/DPI 分别保留原合同缺口。14张图和4对重开比较只证设计可编辑及持久化，不构成 UE ≤1逻辑像素或产品验收；passed/accepted=false。

## 2026-10-25 r86：Numeric 输入消费合同与实际 native lane

Parent 已接入的三共享源固定中心 `setting_numeric_value_frame`，Reset/±优先，TextInput 携带原 stable key 与 edit/commit action。两域源现在真实消费 `SETTINGS_COMMIT_NUMERIC_ACTION_ID`：先验证原 active-window document，再从 bridge 读取保存的 typed setting context，调用 `set_numeric_setting`；`CapturedSettingsEdit::NumericText` 沿真实 definition.schema 解析 i64/f64，调用原 `set_from_context(context, definition.scope, key, value)`。schema snapshot 仅供类型/范围，不能冒充显示代际；原 scope、持久化、no-op 和 receipt 刷新保持。

两源 raw/EOL 精确后像见 [源变化](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-consumer-r86/source-change.json)，原 d2b44500 增量的严格逆还原匹配 r85。其余12源、Parent Shared3、r85 conditional capture、原 override getter 均保持。Numeric3定义为独立 `guarded_numeric_text_tests::`，含真实 builtin/coordinator 的 pending/noop/stale/invalid 合同及 parser i64 边界；parser 宽 schema 仅为单元边界，不扩大产品 autosave schema。没有 harness 发现或执行结果，不能作为 GUI 通过。

现有 native 输入：本地 `commit_only` 编辑只保存 draft 并重绘，Enter -> surface commit -> 域消费者，Escape 清除焦点而不派发 mutation。当前 Numeric painter 未读 matching focus draft，提交了 Parent-owned commands/tests 两源最小未应用候选（仅选取焦点 value_text，原几何/±/主题保持）和一个未应用 renderer 定义；此候选尚无 patched canonical guard 或运行结果。

IME 必须按当前生产合同接续。Runtime `translate_ime_event` 与 normalize 保留 Preedit/Commit/Cancel 的 Ime 事件；legacy focused path 的 `platform_text_input` 仅 Text，retained pane 遇 legacy text focus 返回 None。Editor 原翻译测试期待 Commit -> Text 已陈旧；本片未执行它，不把静态矛盾称作已失败 harness。原初稿/修订和30源指纹见 [原生输入合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-consumer-r86/input-lane-contract.json)。Parent/原输入 owner 需保留原组合会话、来源生命周期和 typed 显示上下文；本域不扁平化 Ime、不加第二 composition store。

当前 native focus 仍只保存 key/action/value/frame，没有 originating semantic batch/window lifetime/typed mutation context。消费 bridge 最新有效保存上下文并不能证明关闭重开前事件归属；旧共享 origin 合同继续 open。官方 source/export/normal producer、IME/焦点/Enter/Escape 真正 GUI/EXE/DPI 证据均独立待验。

## 2026-10-25 r85：原 projection generation 与显示值的生产捕获合同

窗口打开及完整目录刷新携带已经捕获的 `SettingsWindowProjection`，经 `capture_settings_values_for_projection_category` 到原 coordinator 的 `capture_values_at_generation`。原 operation-state lock 下先 `require_idle`，再在原 authority state lock 下 `ensure_generation` 并读取 `ResolvedSettingsBatch`，最后仍保存原 owner Weak、project_generation 与 next_project_generation。完整 u64 不经 UiValue/i64 截断，原 MAX 耗尽拒绝继续保留；空分类和 plugin 分类也须匹配原投影代际。不是在旧事件 handler 里取最新 snapshot 代替显示态来源。

原 `capture_values` 保留签名，经共同 private helper 继续原绑定/批次捕获。实际值单独刷新、分类选择、原 coordinator conditional mutation、scope-specific reset、persistence/noop/error cleanup 均保持。新增方法有两个原生产 caller；没有无消费者 scoped wrapper，也没有新 required DTO/public export、独立 registry/store/clock。已捕获 projection 只含目录/本地化定义，本片不把它升级为完整动态语义子树或跨服务原子快照。

条件读取之后发生的变化仍依赖原 conditional commit/reset 拒绝；关闭重开前排队事件、originating frame/window incarnation/lease 与 published semantic batch 仍由公共 owner 接续，不能以最新 bridge context 冒充。四页面显式编辑 scope、真实 String provider、Numeric caller、World/history/restart/normal producer、官方导出/EXE/DPI 均未由本片闭合。

新增独立测试过滤器为 `core::settings::tests::mutation::projection_values_capture_`（静态 3）和 `ui::host::editor_event_runtime_access::settings_projection::tests::projection_values_capture_`（静态 2）。仅真实既有 builtin 定义，项目绑定复用 OwnedSourceWriteFixture；u64 仅用既有 cfg(test) revision 构造。尚未执行，不改原 r35/r37 或 r70/r71/r73/r78/r81/r83 请求与期待。详见 [当前 14 源指纹](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-projection-values-pair-r85/source-pins.current.json)、[测试准备合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-projection-values-pair-r85/regression-contract.json) 与 [结构审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-projection-values-pair-r85/source-audit.json)。

## 2026-10-25 r84：Project分类显隐覆盖及真实三尺寸可编辑板

真实原生 Layers 隐藏三分类行中12个 eye/lock及panel节点，全部由Penpot记录visibility-group覆盖。原12差异仅hidden/touched；文字、几何、原source metadata/shapeRef、34色alpha与字体均保留。UE分类控件只有expander/label/tooltip，此覆盖消除本域不适用操作的设计画面；公共navigation/navigation-leaf尚未发布，未改Shared或内部几何。

覆盖来自真实原生UI，保留原authoring-painter基线；未重签metadata、未补造asset envelope/UUID。两响应式板保持20/80原布局和原六schema字段，12可见性覆盖从原实例复制并持久化；这不等于公共运行时已无眼睛/锁输入路由。

own file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` / Project page `49ff1071-f83b-4b72-bb6e-043a3326ec82`，rev236→258；原1280板 `c5edfcc8-00e6-4419-a463-3324207d5335` 不改尺寸。两新板由实际原生duplicate分配：900×620 `342ce826-b591-800d-8008-d801c759fd53`，640×520 `342ce826-b591-800d-8008-d801e8de8b52`；非新组件master。新增244对象，973总对象/5页，43 Shared heads、594refs(562Shared/32Card)全有效；Shared5整hash保持，另四页/global保持，原对象除root成员和12显隐覆盖均精确保留。

三板保存/关闭/fresh context重开整data hash `1cae08b627751d24b7097167dcd9e7f7a99c32f123a009649455c70e71cc7330` 相同；8原图+2实际裁剪/差异逐张复核，原1280覆盖前后1307设备像素变化，三板各自saved/fresh crop差0。640标签框仍溢出52.00000482472478逻辑像素，仍failed/open；未用短标签可见代替长文本/≤1DIP。库更新banner仍显示，本轮未做整库sync/reset，未证明navigation源guard或正式导出。

属性值仍原schema设计fixture文本，无引擎预览/可写控件证明；正式codec/source export→原ZUI→正常EXE、真实150%DPI、四Toolkit/页scope/主题两族A-B及原11要求仍open。当前14 Rust全同r83；无Cargo/intake/QueryOnly/source capture/freeze/桌面/服务/生成/源码副本。goal active，passed=false/accepted=false；0B释放，证据保留，整体cleanup未关闭。


## 2026-10-25 r83：健康整树刷新与值批次恢复合同

原 `refresh_settings_persistence_health` 在首个健康属性写入前暂取原值上下文，仅成功或无变化后恢复同一批次；准备/整树刷新 Result 错误保持 None。沿原 health tick、receipt 与 retry 回调生效，正常值刷新负责恢复编辑；原 pending persistence、Retry 身份、scope/generation/coordinator 均保留。

失败后健康重建成功只恢复原 Retry 捕获；值上下文仍须原 refresh_settings_values 安装。已有 no-op 不获取最新 snapshot，既有字段/事件/operation tuple 不变。独立准备提示与公共 property Rejected 不因此获得完整错误证明；origin frame/window lifetime 仍需 Parent 集成。

新增独立 health_materialization_tests 两定义，覆盖成功/no-op、真实模板缺节点错误、失败后 pending 与 authority 不变、健康恢复不冒造值批次、正常值刷新恢复；未运行 Rust。当前14源仅此1份变化、13份与r82相同，旧30定义完整保留，32是静态定义数。rustfmt/反向应用/差异空白/change-guard 均 exit0，guard 0 error/warning/suppressed。

r74 Numeric 三公共入口仍未落地，本轮不添加孤立 consumer；公共 Rejected/origin lifetime、四page语义树、provider、正式export→原ZUI→正常EXE/实际150%DPI与族验收继续open。Penpot沿r82实际rev236/Shared5历史观察，本轮无API/设计/画面新证据。0Cargo/intake/source capture/freeze/服务/桌面/生成，goal active，passed=false/accepted=false。


## 2026-10-25 r82：原Card实际共享字体消费与cache边界

实际原20文本绑定Shared5 body/caption/strong（5/10/5），保留五份manifest原文、原Card UUID/main/4copies/32shapeRef、34色引用与alpha、原domain metadata；原69 DIP缩略图和auto-height布局保持，五Card均77 DIP。 原component `81548358-cc2f-47cf-856d-39b395bfecfa` / main `d57fd7f1-bf28-4c97-bc6b-0a52ea4696f1`不变；domain metadata version1/officialCodecMetadata=false及历史flags保留，当前20实际style引用以API读取为准，不伪造新版wrapper/组件UUID或覆盖历史source metadata。原component.modifiedAt是实际主件编辑的全局变化；其余global数据经严格还原核对。

三尺寸原图均保留。640原content比较还覆盖SVG path坐标而失败，不能升级整页源guard；恢复1280后15段文字positionData仍在x309、shape x533，相差224，三张坏图稳定且已保留。原生相同内容编辑为no-op；实际内容临时追加/恢复的首轮因过早hash稳定判断中断，rev207临时字元已沿原编辑路径在208恢复，再处理剩14。现rev236原文全恢复、20字体引用及当前x缓存核对，关闭/全新context重开整data hash一致；修正当前保存态不代表尺寸失效根因已修。

最终current rev236/Shared5，原文/字体/颜色/源引用通过限定核对；当前20 cache x误差只是本次位置缓存诊断，不等于UE文本行框验收。既有35%分类栏、eye/lock/cube、操作按钮/真实fields、选中焦点弹层/首尾滚动及中文长标签仍须相应真实native/EXE验证。原thumb空槽/真实启停、operation/restart、repeat prototype/wrapper、正常producer/正式export→原ZUI→EXE、真实150%DPI、两族A/B及原11要求仍open。只是本域可编辑设计，非引擎预览或验收。 另四页保持r80原完整hash；r81的14源和独立两定义不混入此设计结果。证据r82 final-native-audit/visual-review/source-assets，passed=false/accepted=false。

## 2026-10-25 r81：原bridge显示上下文的物化条件

原 `toggle_settings_editor`、`close_settings_editor`、`update_settings_scroll_offsets` 三入口均采用原字段的take→原属性更新→原UiSurface刷新→恢复同一capture。早退不消费context；成功不改变generation/项目绑定；Result错误不恢复context，后续原sync检查None再做真实values refresh。只增加现有bridge生命周期处理，无新store/clock/DTO/action/控件ID，未修改Shared、主题、codec或正常producer。

新负向Rust定义仅在测试内移除实际 `WorkbenchSceneTitle` 并标记原root layout/input dirty，要求原materialization传播精确 `UiTreeError::MissingNode`；三转换都撤权且authority generation不变，补回测试节点后经原refresh恢复。同原builtin locale和coordinator的成功定义覆盖开/关/滚动关闭及no-op/NaN。两定义未执行，不是产品fixture/provider或harness通过。独立filter为 `ui::retained_host::callback_dispatch::template_bridge::workbench::settings_window::popup_materialization_tests::`；原28定义及旧filter谱系保留。

当前 `control_state::mutate_control_property` 的Rejected报告与Result传播不同，本片未扩大到该共享路径，不能声称覆盖拒绝报告。值context有效也不证明入队事件的originating frame/window/open lifetime或published semantic batch；该公共origin合同仍按Parent原边界。r80真实设计保存/重开证据继续引用，未生成新画面、正式export或EXE证据。当前14源及精确增量见r81 source-pins.current/source-audit/checks；passed=false/accepted=false。

## 2026-10-25 r79/r80：已消费共享字体与可逆源前置

Project own rev199，page `49ff1071-f83b-4b72-bb6e-043a3326ec82` / board `c5edfcc8-00e6-4419-a463-3324207d5335`，Shared5。26个可见自有非组件文本通过原生rich-text属性绑定 body `99eb86f9-12ff-5935-96ad-3d1a985e61f0`、caption `c0224e78-f918-526f-8026-a76cd8f347af`、strong `9ac86733-8214-5a04-820a-9074b5c68310`，同一个source file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`。实际字体族Roboto/id gfont-roboto，正文13.333333/400、说明10.666667/400、标题13.333333/700、lineHeight1.2；原富文本值、fills/alpha/color refs保持。原隐藏旧target及linked/generated字框无本片修改，不用字体覆盖掩盖旧source guard。

原板不是可逆导入的ZUI资产，仅有domain-contract。公共 `captureAuthoringLayoutDraft` 要求原document metadata；不能凭子TreeRow源身份为整个设计板伪造asset/document或guard。已有 `refreshTreeRowContent` / `layoutTreeRowText` 在合法源前置下维护全文/生成几何，当前TS text leftright、icons right不等于已发布旧copy自动迁移。固定left标签x52/w128遇20%窄栏128时越界52；单改leftright也不自动回收无业务动作的lane。导航须在现TreeRow同源声明非叶/叶变体、单一缩进owner、真实展开/选择/命中、相同native/Penpot lane、codec/owner/ref/guard资格及reset/export负例，不复制main或另建store。

保存重开整data `9e67e30b1926c81acbd06e741fb9df7d86adb477a3f97ca28e3b632b6fd0369e`。初次post-update1280绘制证据未稳定保留；恢复→fresh→第二fresh两帧crop0设备像素差仅为Penpot稳定性。900树动作跨入详情、640标签52px越界、CJKglyph/Engine字体、typed controls/provider、四page scope、native origin/frame/window及正式owner export/EXE/DPI均未关闭。r78原14当前pin仍同，r74 Numeric/r73共享variant未应用。详细源前像及8图复核见 r80/visual-review.json、r79/source-entry-decision.json；passed/accepted=false。

## 2026-10-25 r77/r78：真实中文保存重开与 Preferences 弹层 caller 条件检查

r78直接修改原授权14范围内5源（mutation/core tests、host settings_projection、原settings_window bridge、surface/option）：Enum/Color打开消费已保存字段context，经原coordinator state/binding和原authority精确u64代际检查；bridge限制同显示batch且值类型匹配。关闭同key不因context失效而被阻断。没有新store/clock/provider；原写事务、reset scope/raw getter和r73 retry保留。增量patch SHA `a7311ae5b40b14960570ad08eae8611ac71106c623f90752ab4ad2b20f66babb`；5前后像严格内存逆核对、rustfmt/whitespace/reverse/change-guard均通过（0errors/warnings/suppressed）。当前14pin中9沿r76/5为本片，原24+新4共28只指静态定义，不是harness；新core popup_context_check_与bridge popup_context_tests两个filter单列，未混入r70/r71/r73或旧r35/r37请求。r74 Numeric 5前像仍5/5匹配/候选未应用。

r77真实Penpot own file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402`，Project page `49ff1071-f83b-4b72-bb6e-043a3326ec82` / board `c5edfcc8-00e6-4419-a463-3324207d5335`，rev172→**192**；六字段label/description+两原linked分类共14节点通过native DraftEditor改为实际zh-CN字典。1280×800/900×620/640×520并恢复，保存关闭/全新context重开整data hash `bf51cff48f6b6bec1b4bd8af55fd40c4f53de8cdf0cae3aa6feab9aca506c3cc` 一致。Shared5/35 Shared copies/460实际源refs仍有效，其他四页整hash和全部原ID/metadata/source保持；native text overrides保留content-group/text-content-text。原严格content-minus-text比较14/14失败保留，递归审阅证实差异仅文案及native paragraph key，字体/fill/typography引用不变，未重签source guard。

6原图逐张复核：中文可读，640两个分类label frame实际越实例 **52.00000482472478逻辑像素**；root fill不等于内部文本布局通过。CJK呈衬线回退外观、实际fallback和引擎字体未证明；通用eye/lock/cube、Search/title/All Settings/source/footer英文、schema文本值/真实field/provider/滚动等仍open。57字符原英文旋转步长description前态保留，不把此片当整窗长文本/UE1px/真实DPI通过。最初19001连接拒绝后无认证readiness200、认证读回172再继续，未做服务操作；异常附带headers只在自有诊断脱敏保留事实，认证文件未改。所有初次失败保留。

只读打开检查不预留未来写锁，写时原子条件重校继续；originating frame/window open lifetime/published semantic batch、Enum queued-selection、四page scope、typed subtree、normal producer、正式export→原ZUI→正常EXE/实际150%DPI/两族A-B与原11项仍open。0Cargo/新业务intake/QueryOnly/identity/source capture/freeze/服务/桌面/生成。browser/context退出0，证据/可复现helpers保留、释放0B，goal active，passed=false/accepted=false。

证据：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-popup-context-r78/`（source-change/source-audit/source-pins.current/checks/regression-contract/static-test-expectations）；`.../settings-project-locale-r77/`（native-edit-targets、locale-change-intent、saved/reopened、native-content-delta-audit、visual-review和6PNG）。

## 2026-10-25 r76：Project 比例分栏落地 / Preferences 失败实验精确局部恢复

真实Penpot当前 **rev172**（Shared5），完整保存关闭/新context重开hash `640a3d72c7f7f3701424c5051bc06bab24afa23172eed4f25acdedc98d4c0bb6`。Project原host改为20/80 native grid，1280/900/640两栏宽为256/1024、180/720、128/512；六行和scope footer仍可见。仅自有容器布局，原14 linked fill/ID/style/metadata/35Shared copies及460源refs保持，其他3页整hash保持；没有Shared/公共源写入。

Preferences 同次比例实验产生640px最小内容高度，900/640底部范围footer被挤出；15张原图和2张实际Penpot前后/差异衍生图已逐张看过。只恢复本轮三个Preferences容器约束，Project页整hash保持，原r75 linked fill不撤回；900/640footer恢复，原固定256栏/文字裁切/滚动缺证仍open。精确失败、首次HTTP500枚举编码拒绝/先核rev165整hash后修正的200、保存重开都记录，不做全文件回滚或盲重放。新并排图不是引擎对照，不报UE1px/真实DPI/交互通过。

r74五源Numeric候选仍未应用/3定义未执行，r73原14pins/24静态定义与旧测试谱系分列。无Cargo/业务新请求/服务/桌面/系统DPI/正式export/EXE；origin/window lifetime、typed subtree/四page scope/provider/normal producer/两族A-B及11要求仍open，goal active、passed=false/accepted=false，登记释放0B。证据：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-split-flow-r76/`（summary、完整/局部恢复回执、final-saved/final-reopened、visual-review、15原图+2衍生图、所有失败）。

## 2026-10-25 r74/r75：Numeric 原生产入口候选与两设置页真实 linked flow

本轮 active 独立推进。r74 完整五源候选尚未应用：现 Numeric 已绘制 center frame，但共享 hit-test 只有± Option。沿原 TextInput → saved typed context → 原 coordinator/authority 条件写/receipt 的实际补丁已可审，3共享动作/命中路径交 Parent，2原域消费路径待 fresh 回交后接续；新增3回归仅未应用定义。i64不经f32，不改变 Preferences 原 definition.scope，不用无caller层当完成。候选完整 SHA `cf17abbcbdbded95f19c036a527291c4ac1d713abbed307b299a7876aef4c458`，split共享3和域2均当前 apply-check0，五拟源仅rustfmt语法/格式检查，无Cargo/harness。

真实Penpot r75：own file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` 从rev126至 **rev152**；原Preferences page `b4a1e3ce-9e6a-81fe-8008-d69d60e4e403`/board `92bc794d-9419-4284-bb9c-282b99afb641`，Project page `49ff1071-f83b-4b72-bb6e-043a3326ec82`/board `c5edfcc8-00e6-4419-a463-3324207d5335`。原生Inspector仅为14个原Shared SearchInput/TreeRow设置横向fill；两个原板均测1280×800、900×620、640×520并恢复1280×800。保存关闭/新context重开完整data hash一致 `e4d996fc79cb1c8cda0c5eb37e69f668aec66386fb8c4c7e7009e421f1c567a3`，Shared仍rev5，35 Shared copies/428 Shared+32 Card源refs全有效。未detach/复制main/改源或主题，其他3页整hash/全部ID、metadata、内容和样式保持。

12张原图已逐张复核；组件根右溢出0只证明该层flow。drawer仍固定256px，SearchInput内部仍固定绘制宽度，640 Preferences说明与后续行裁切、900后续行滚动未证明；eye/lock/cube语义、字段控制/provider及真实焦点/弹层仍open。UE `SSettingsEditor.cpp` 原20%初始drawer与分类SHorizontalBox FillWidth1为参照，当前不能报20%自适应完成。71/98/134%是canvas zoom，非实际DPI。

原r73九源增量已落、当前14pins/24静态定义保持；r70两filter、r71 health、r73 retry/bridge、r74候选3继续分列。originating frame/open lifetime/semantic batch、四page scope/typed subtree、normal producer、正式export→原ZUI→正常EXE/150%DPI/双族A-B/逐图和11要求仍未完成。无Rust执行/新业务请求/identity/source capture/freeze/服务/桌面/生成/源码副本/清理，browser/context exit0、释放0B；passed=false/accepted=false，goal active。

证据：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-entry-r74/`（完整/split patch、candidate/current-split-review/summary）；`.../settings-linked-navigation-r75/`（原生修改、8尺寸状态、saved/reopened、12PNG、visual-review）。历史全部保留。

## 2026-10-25 r73 当前 Retry 业务映射与边界

本轮继续 active goal，实际落源是 Parent 原 14 路径内的 9 份 Rust，未改公共主题/组件/producer。原 r64/r66/r70 String/reset/typed 控制、r71 health tick 与全部历史失败保留。

- **真实 caller 已接入**：原 `SettingsMutationCoordinator` 在原 state lock 内读取健康、原 pending document、Project binding 与原 file generation；只生成不可变的原 owner Weak/document/file-generation 上下文。host 正常 open/目录 refresh/health tick 带出并由原 bridge 保存；实际 `surface/option` Retry 消费已物化的上下文，不在事件处理时重新取得最新目标冒充显示目标。
- **事务条件**：原 operation guard 内先核 coordinator 身份及 scope，再核当前项目 document 与待写 file generation，成功后才沿原 pending remove/submit/retry/receipt 路径。拒绝不替换当前待写项，恢复原 phase；原 public `retry_pending(scope)`、persistence/noop/error/receipt 函数体保持。没有新 store/clock/backend/UUID/文件代次分配。重试不改 authority 值，因此不声称增加值写入的 authority fence。
- **展示安装边界**：打开/完整刷新/关闭清退旧目标；read-only 通知 hint 不能授予目标。原 health materialization 全部成功后才保存 captured target，失败保留原 pending-refresh 状态供下一 tick 处理；值上下文、编辑弹层和滚动沿原路径。值提交后的真实 receipt 刷新同样消费配对健康 capture。
- **回归仍待执行**：新增 3 个原 coordinator 行为定义 +1 个原 bridge 行为定义，覆盖 pending replacement、wrong scope、Project clear/rebind、foreign/released owner、phase cleanup、通知失效/正常刷新安装及关闭/无 token 重开。原 19 guarded+r71 health1+本次4是 **24 个静态定义**，不是 harness 发现/通过；旧 scope mutation17/details11 仍各自 open。拟独立测试为 core retry prefix3 与受影响 bridge module4，共7个当前静态定义，禁借旧 r27/r35/r37 身份或已受理 payload。
- **精确增量**：`source-change.patch` SHA256 `addeb544aeb62fd3e1f03eaf92fb3944016e201233d94bd1ee44c5aec3ccd4b3` 已实际应用，不得重复 apply；9 源前后 SHA、内存逆增量恢复原 preimage、原 retry 函数体等价检查均已登记。格式/whitespace/反向应用/精确增量 change-guard 结果以 `checks.final.json` 为准；没有 Rust/Cargo/intake/QueryOnly/source capture/freeze 结果。
- **Shared TreeRow 提案未应用**：`shared-navigation-rows.proposed.patch` 精确列出 codec 与 native commands/label 三个当前 preimage。沿现有 `component_variant` 提议 navigation/navigation-leaf：保留真实分支展开/选择/焦点/图标，取消分类无业务语义的 eye/lock/more 与右侧操作保留宽度；不复用 asset-folder 冒充分类，不改原默认/content/资源域规则。只做 `git apply --check`，canonical 共享写入0，需 Parent/原 public owner 审阅、测试、正式导出。分类计数仍是单独的真实 provider/slot 缺口。
- **真实设计状态**：本次认证 GET 仍为 own rev126/5页729逐页对象、Shared rev5/4masters/34colors/3typographies；本次只读，无 Penpot 重播、UI/desktop/system DPI 操作。r72 四 linked TreeRow fill、三尺寸/保存关闭重开及6原图继续有效；640 eye/lock 裁切和语义失败继续 open，没有宣称新图或视觉通过。
- **边界明确保留**：原事件缺 originating frame/window incarnation/open-lifetime lease，bridge 当前 target 无法识别关闭重开前排队事件。String 生产 provider、Numeric 文本 caller、四 page scope/typed subtree、normal producer/Toolkit/World/Dock/undo/save/restart、正式 source export→原 ZUI→正常 EXE、实际150%DPI、两族主题A/B、双端逐图验收都未完成。缺共享部分仅保留相应 pending，不使整族停工。

本轮证据根：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/guarded-persistence-retry-r73/`。共享源提案与实际 canonical patch 分开。`passed=false`、`accepted=false`、goal active；无服务/桌面/生成/源码副本/授权库/缓存清理，实际释放0 B。

## 当前合同增量：2026-10-25 r71/r72（保留下文 r55 及更早历史）

当前以 [本域实施账本](settings-plugins.md) 最前的 r71/r72 与 [Parent 交接](settings-plugins-parent-handoff.md) 为准。下文 r55/r54 的 rev54/Shared2、无 guarded production caller、SP-C6 无 payload 字段等为对应当时的历史观察，不能继续当成当前能力结论或重放旧补丁。

- Parent 的 SP-C6 optional typed package_metadata 已贯穿原 status→view→payload→TOML，仍不证明 binary provenance、repeat Card/native producer 或正式验收；不重复造 DTO/store。
- r64–r70 已在原 production String/reset/Bool/numeric-step/Enum/color-step/chord 分支消费显示值上下文，并沿原 coordinator 项目 operation guard 与 authority lock 条件 mutation；这只是现存 caller 源码切片。原 scope 语义保持，缺 String provider、Numeric 文本 caller、四 page scope 和事件 originating-frame/window lifetime 不能升级为完成。
- r71 真实 normal host tick 接独立 persistence health refresh，值/目录/通知均 current 也检查；仅增加 UI 待物化位以重试刷新，不替换 typed 显示值上下文。新增 1 行为回归定义未执行；此前 19 guarded、mutation17/details11 等原范围/失败各自保留。3 源格式/guard/reverse/whitespace 仅静态通过，未 Cargo/intake。
- 实际 own **rev126 / 5页 / 729逐页对象**；Shared 独立观察 **rev5 / 4 masters / 34 colors / 3 typographies**。Plugin Browser 原 4 个 TreeRow linked instances 通过 native Inspector 横向 fill；1280/900/640 保留 UE .35f 分栏且 body 溢出为 0，保存关闭重开完整 data 一致。35 Shared copies、460 refs 均有效，ID/metadata/其它四页保持。库更新 banner 仍在，未批量更新或声称全量同步新库。旧 source/staged mismatch 与 r53/r70 失败仍留。
- 640 的通用 eye/lock 仍裁切且不适合分类语义；相关 slot、正式 source owner 导出、真实字段/卡片控件、typed subtree、normal EXE/actual150%DPI/两族主题 A/B 及逐图双端验收继续 open。现有 preferences.zui 仍是单 root，不拿 text fixture、painter role 或 detached 形状当完整 editable tree。

真实源 pin/增量 patch/1 个未来独立聚焦 filter 与 9 份共享输入 preimage 的 frame/window-open/pending-ticket 合同在 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/persistence-health-refresh-r71/`；原生 Penpot 保存/重开/6 图逐图记录在 `.../linked-row-responsive-r72/`。Parent 与原 Jenkins owner 已通过官方 queue 各投递一次实际变化；投递不等于已读/准入。`passed=false`、`accepted=false`，goal active，继续独立实作，不因共享未完善整族停工。


r55 补齐 [最小动态语义树及逐类型 provider](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-semantic-subtree-r55/semantic-subtree-design.md)／[canonical 调用点提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-semantic-subtree-r55/canonical-source-callsite-proposal.md)。所有拟议新 node/control 名称均标为未应用，当前 Preferences 仍只有原 root。实际字段值取同代次 resolved batch，保留 typed schema、原 key/category、目标 scope、raw override、locale/generation；sourceOwner/control/instancePath 与完整业务 item key 分别记录，不从文本 fixture、数组序号或 painter role 伪造。

需要 Parent／原 owner 接线的范围明确为：同一私有 bridge 的 typed capture/call、混合 Row source prototype/slot、真实页面 scope/document 与 commit currentness、scope 目标层显示/default catalog、原 typed numeric setter 的真实 surface callback。schema 没有通用 units／precision／large_step；plugin_pages 仅 metadata；复杂类型与完整 Input Bindings provider 未完成。旧 string/chord 等入口按 definition.scope，coordinator.set/clear 没有 expected_generation，不能称页面 scope 或 CAS 已贯穿。原 r42 wrapper 不重放，原 mutation/receipt 保留。

行／文字几何目前仍来自手工 SettingsWindowLayout 与 root painter；新逐字段证据须接同一真实 UiSurfaceFrame 的 arranged node、render command/text line、hit/focus、window metrics 及源身份，Parent 负责测量/painter/Settings native case，原 public owner 负责 slots/codec/ACK/export。只读再次确认 source PropertyRow 5ab622…／r392 staged bf29… 不一致；未改 F 盘产品、LevelEditor guard、nonce、公共 Rust、窗口、主题、store 或服务。 own rev54／5页720对象、Shared2／4masters／34colors／3typo 仍真实存在；本轮未改 Penpot，r54九图和 r53 whole-page hash 失败均保留。

本阶段只有文档／能力接入合同，无 Rust／正式插件测试或导出、无 managed resource prepare、正常 EXE／actual150%DPI／双端验收，passed=false、accepted=false，释放0 B。

r54 设计身份仍为 own rev54／5 页／720 对象，Shared rev2／4 masters／34 colors／3 typographies，原 PluginMetadataCard 81548358-cc2f-47cf-856d-39b395bfecfa 未替换。新 9 张图覆盖同五页及 Beta／New Plugin／Directories／Properties 板，逐个真实 SVG ID、可见边界、Layers aria-checked 已核对。最终 verified PNG 与本轮逐张检查的原 PNG 字节相同；浏览器图尺寸1600×1100且缩放不一，不当作产品 1280×800／900×620／640×520 或系统150%DPI。[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/visual-review.json) 保留白底、模拟 fields、固定按钮、空 required value、长中文裁切及 modal/focus 缺口。

本轮只做被动浏览器重开、定位和捕获，五页每个节点、components／样式及 Shared 全数据与完整捕获前像精确相同。451 shapeRef（419 Shared／32 本域 Card）由实际组件祖先归属及原 source lookup 保持可核对。原错误 aria-selected 断言的 9 次失败和第一次 stdin 关闭记录都保留，不用 native exit0 掩盖断言失败。此精确不变仅限 r54，被动捕获不能关闭 r53 库同步造成三业务页整页 hash 变化的失败。

正式 Card 尚无 canonical repeat wrapper。[具体源桥接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/source-bridge-proposal.md) 给出原 host 调用点、唯一业务源组件候选、原 typed 数据与六组动作、原有界 cache／UiSurface 池、完整 plugin_id 及现有 assignment binding、源 instancePath、真实测量统一 extent、单 scroll 和原 dispatch 约束。唯一拟议公共字段及重叠路径先交 Parent；没有改源、发明 UUID、detach 或重标容器来伪造来源。20 份起始源输入目前 19 份匹配，外来 catalog 新 e4bba09c… 保留；Field／Number／PropertyRow 描述符未变化，不称已经 source-prepared。

Shared 的 21 个 style ref ID 目前全为 null，34／3 样式真实发布不等于 masters 已绑定或 CJK ready；原 trial 显式颜色引用有效。Field／Number 原源能力存在但未发布为 Shared native master，不能画文字框当可编辑控件。i64 精确文本走原 Field；模板默认及未经 schema 策略的 numeric precision 不作真实值。r54 无服务生命周期动作／桌面／Rust／正式插件 suite／generator；正式 source owner 导出、normal family producer、真实 EXE／150%DPI／双端差异图及未证明业务交互继续 open。passed/accepted=false，释放0 B。

r53 当前真实设计为rev54／5页／720对象，33个Shared外部实例／419个源引用。Shared已实读rev2／34颜色／3文字样式，四主件不变。仅在原受控板（page`1ca0591c-2856-49b0-9d7e-eeb076bafe03`／board`4829a7b5-8807-4e66-b34d-b94fbb8807fe`）给板背景和默认Button leaf链接源颜色`editor.surface.0`与`editor.surface.1`；颜色覆盖、重选、保存关闭及fresh重开都有原生UI和API证据。没有改正式业务板的预期样式，也没有global style A/B、组件Reset或正式产品导出。[保存回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-style-linked-r53/save-reopen.json)及[9图逐项复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-style-linked-r53/visual-review.json)保留原失败。

原生库同步从rev53到rev54后，Preferences/Project/Plugins整页hash改变，完整保全检查继续失败。受控页SVG坐标及文字测量缓存重算，原身份/富文本保留；25标签覆盖、18旧动作隐藏、419源refs/metadata仍有效。其他业务页缺完整节点前像，不能把差异统称为安全缓存，也不以小于1像素的单项差值宣称整族误差达标。公共owner需先审阅[精确差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-style-linked-r53/library-sync-delta-audit.json)，禁止宽泛忽略/重签/整体回滚。

字体source仍使用原custom字体；新Shared`gfont-roboto`的字节/度量未证明，未应用Typography。正式导出对颜色引用的解析/guard责任、canonical源wrapper、真实value/长中文/事件、两族共享样式及EXE全部open。本轮无Rust/ZUI/Cargo/服务/桌面变化，passed=false、accepted=false，释放0B。以下r52及更早事实保留原时点。

r52 当前真实Penpot为rev49／5页／720对象，累计33个Shared linked copies／419个源引用节点。原四配置板中的最终动作接入4个真实Button实例，保存/关闭/两次fresh重开均核对source与覆盖值；84个原节点ID/语义/content/metadata/parent保留，4个旧文字动作隐藏且不占flow。只删除本轮误加的Directories Close copy及其leaf，其他页和Shared指纹不变。身份和实际变更在 [r52回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/dialog-linked-r52/save-reopen.json)；15张原图及差异逐项在 [r52逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/dialog-linked-r52/visual-review.json)。以下旧rev/空库叙述保留原时点。

本次按实际UE补正：Beta沿`SPluginTile.cpp:583`进入YesNo，Editor `Dialogs.cpp:194`的primary Yes在左、No在右，组右对齐；New Plugin沿`SNewPluginWizard.cpp:280–290`为右对齐Create Plugin，缺plan/provider时不可执行；Directories正文沿`SPluginPaths.cpp`无Close，由外层窗口关闭。Properties继续只读manifest字段，Close为当前Zircon能力适配，UE可写元数据和提交/撤销仍是差异。Create opacity0.5只是本实例的设计覆盖，不证明禁用控件/事件；不执行generator、写plugin.toml或共享workspace。

四板仍为中性结构草稿；主/禁用按钮变体、AutoWidth/标准Dialog间距、Field/value、图标/主题/共享样式、实际弹层/焦点/事件、canonical source wrapper及官方导出/正常EXE均未就绪。此次头端1600×1100、deviceScaleFactor1与Penpot画布缩放不等于要求的产品分辨率/150%DPI，也没有双端差异或≤1逻辑像素结论。passed=false、accepted=false；后续继续沿公共owner源与真实producer接入，不以这轮草稿完成冒充产品。

归属：聊天 `01a16d53-33db-7fb2-8f59-11118c3ec933`；Parent `01a16d4e-0491-7353-84fa-038bd7706546`。接续 [窗口盘点](settings-plugins.md) 与 [共享依赖交接](settings-plugins-parent-handoff.md)。这是源码约束下的具体设计；真实 Penpot 已有四窗口的原生流布局结构草稿，公共实例/共享样式、正式设计、产品资产和验收结果尚未完成。

当前 r43 已消费真实 Shared rev1：本族第五页保存8个外部原生 linked copies，4项文字覆盖关闭/重开保留，原四窗口草稿不替换。共享 colors/typographies、完整属性编辑器/语义、正式导出/产品验收仍未完成。下文旧 rev0/无实例说明保留为各自时刻的历史，当前身份见第15节。

Parent 统筹 Penpot 服务、文件/页面分配、公共组件/主题、桌面输入和捕获。本族不启动/恢复服务、不发送共享桌面键鼠事件、不先改写 ZUI；先在分配的真实 Penpot 中建立可编辑设计，再将导出 delta 落回原资产。已分配 file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402`、初始 page `b4a1e3ce-9e6a-81fe-8008-d69d60e4e403`。四窗口、八个结构草稿板经正常 API 校验保存、fresh 无头 context 重开；r26 已有真实 PluginMetadataCard 组件身份与保存/重开证据，公共实例/共享样式及正式设计仍待交付。followup-03 已由 Parent 应用 r34/r39 源扩展，r42 消费 reset 并另交新宿主接口提案；下文 r39 及更早未应用描述保留为历史。公共接口及真实分配见 [组件合同](settings-plugins-component-contracts.md)，不把中性文字草稿算作正式窗口完成。

## 1. 原身份及拟用组件

| 表面 | 已存在的源身份 | 接续边界 |
| --- | --- | --- |
| Editor Preferences / Project Settings | `zircon_editor/assets/ui/editor/components/workbench/floating/preferences.zui`；asset `res://ui/editor/components/workbench/floating/preferences.zui`、根 `preferences`、control `WorkbenchPreferences` | 保留原身份及旧入口消费者。独立页签、宿主和作用域 context 由 Parent 分配；不把原单浮层的尺寸修改当作两窗口实现。 |
| Plugin Browser | `zircon_editor/assets/ui/editor/host/module_plugins_body.zui`；根 `root`、control `ModulePluginsPaneBodyRoot` | 复用真实 `module_plugin_projection`、plugin host actions 及发布的状态 report；改造 body 时保留已有绑定/事件。静态 extension workspace 不承接真实能力。 |
| World Settings | 尚未发现对应原 ZUI/窗口注册；已有活动场景文档 | 新表面身份须从实际 Penpot 导出产生，并经 Parent 注册；不伪造旧 asset ID。场景字段/命令 owner 确定后接入。 |
| 通用弹层 | 已有 Dialog / ConfirmDialog / Notification 与 settings enum/color/string/chord 路由 | 公共 master、遮罩、焦点栈由 Parent 管。本族绑定真实发起控件、目标 key、scope、generation；不得复制平行模态系统。 |

共享 catalog 使用 `WorkbenchPanelHeader`、`WorkbenchTreeRow`、`WorkbenchListRow`、`WorkbenchPropertyEditorRow`、Button、Checkbox、Dropdown、Field、IconButton、NumberField；`value` slot 只接受源声明的一个控件实例。每个重复行保留 native instance/master 身份，实例以真实 setting key 或 plugin ID 定位，不能用列表序号代替稳定键。

当前 `WorkbenchPropertyEditorRow` 是固定高度 HorizontalBox，名称列 `min=60/preferred=max=105`、`clip=true`，只有 `value` slot。它尚不支持本族所需的说明区、来源/局部 reset、长中文标签行高。向 Parent 的 SP-C1 提案是扩充现有组件的 slot/variant 与文本测量合同；本族不复制一套 PropertyRow，不在每个窗口叠加绝对定位 reset 按钮。

布局只使用现有 tokens 和 `unreal/starship.zui`。UE `StarshipStyle.cpp` 的 SettingsEditor.ListBorder 为 Recessed 圆角 4，分类根 padding/字体与普通项不同；PluginStyle 的 thumbnail 显示尺寸 69、外层水平/垂直 padding 8/4、标题与版本 LargeText、说明 NormalText。以上作为待映射的源度量，尚未声称 Zircon 共用 token 已等价；差异交由 Parent 公共主题合同解决。

本域 metadata 组合已在真实 Penpot 保存：`PluginMetadataCard` version 1 / component `81548358-cc2f-47cf-856d-39b395bfecfa`，main 为原 AI tile `d57fd7f1-bf28-4c97-bc6b-0a52ea4696f1`，四 copy identities 见组件合同。全部 40 个原对象保留；没有独立 engine ZUI wrapper，不声明为公共 primitive。现行接线仍需原 `module_plugin_list_slot` 的同代 host snapshot 提供 metadata/status/完整 operation，不从 fixture 推断 enabled 或可用操作。

重开曾暴露 native rich-text override 的细分标记缺失：content-group 没有 text-content-text 时，Penpot 会把 copy 文字同步为主件。已按实际 `libraries.cljc::text-change-value` 恢复 12 字段并失效文本缓存，保留失败；最终两 context 的 API/SVG/UI autosave 后各 20 字段正确、40 个 shape 对应稳定，读回 revn=15。公共 style/font attribute 继承、导出/产品和 ≤1 DIP 验收继续 pending；六张新图只是中性域草稿组件诊断。

## 2. 流式组成和尺寸行为

### Preferences / Project

UE 依据 `Engine/Source/Developer/SettingsEditor/Private/Widgets/SSettingsEditor.cpp`：搜索初始焦点、延迟文本通知、外置 Details filter area、SidebarContainer 的分类 drawer 初始比例 `0.20`、已停靠、`DisablePin(true)`；分类 Border padding 8、顶部 padding 4。分节行为来自同目录 `SSettingsSectionHeader.cpp`。

内容根为 VerticalBox：顶部搜索/筛选/导入导出工具区 → 占剩余高度的分类/详情宿主 → 当前窗口作用域的持久化/重启状态区。分类列表和详情各有一个滚动 owner；状态区不放进长属性列表底部。分类 drawer 和横向分隔关系由宿主布局提供，行/分组通过内在文本尺寸扩展。Dock chrome 的尺寸由 Parent 框架扣除，不能把整个客户端宽度乘 0.20 误当 drawer 的最终像素值。

分类由稳定 key path 构成：All Settings（专用全局项）、分类根、叶节分别保留角色。当前 builtin 的 `domain|key_path` 选择身份、父类别包含后代的语义继续使用；插件贡献页以 bundle + 原 key path 隔离。中文翻译改变显示名时不改变选中 key。切分类重置详情滚动，刷新 catalog 只在选中 ID 失效时回退。

详情按 section 垂直流动：section header（标题、说明、可用操作）→ property rows。行包含标签/说明、类型化 editor、来源、局部 reset；说明 AutoWrap，行高按实际行框扩大，值列保持一致起点。未实现的类型不伪装成可编辑字符串。恢复默认/import/export 只在实际 handler 与能力已就绪后出现在可操作表面；设计状态可以展示禁用原因和依赖注释。

### Plugin Browser

UE 依据 `Engine/Plugins/Editor/PluginBrowser/Source/PluginBrowser/Private/{SPluginBrowser,SPluginCategoryTree,SPluginTile,PluginStyle}.cpp`。顶部 Add / Directories / 搜索 / 筛选；下方 SSplitter 初始左槽 `0.35`，左分类 ListBorder padding 8；右侧 breadcrumb、纵向 tile list、底部操作状态。保留 UE 源中的比例差异，不与设置窗口统一成同一侧栏宽度。

tile 使用水平流：69 DIP thumbnail 槽 → 占剩余宽度的文本/操作列。首行 checkbox + 标题 + 真实 maturity badge + 右侧版本；说明换行、作者/链接按实际 metadata 存在才显示；选择/焦点/禁用的样式复用共享 state roles。详细 feature/target/packaging 继续进入原真实详情投影，不把其原有功能删成纯介绍卡片。

UE 的 All/Built-In/External/Installed/Project/Mods 根按实际来源生成，根下按 descriptor.Category 分组，空分类不显示、重建后恢复合法选择、OnlyEnabled/OnlyDisabled 单选切换。Zircon 暂以真实 `package_source` 和 manifest `category` 为输入提案；未确认 provenance 映射前不能把仓库插件全部标成 Installed/Built-In，也不制造 Mods 类。计数必须来自同次发布的真实 catalog 与过滤结果。源码只明确文字搜索匹配 FriendlyName + Description；不悄悄加入未约定的查询语言。

### World Settings

UE `SLevelEditor.cpp` 创建 HideNameArea、ColumnWidth `0.5` 的 DetailsView，绑定 `GetWorld()->GetWorldSettings()`，切 World 后 force refresh。设计采用顶部搜索/选项 → 可折叠分组 → 类型化属性行；编辑对象固定为当前活动场景上下文，不借用普通 entity selection 冒充 World。场景切换使旧 popup/未提交编辑失效，并恢复合法焦点。

### 四种验证尺寸

| 输入 | 布局约束 | 必查边界 |
| --- | --- | --- |
| 1280×800 / scale 1 | UE 源比例和文本样式的基准；有独立滚动及常驻状态区 | drawer/splitter、列起点、section header、tile thumbnail、文字基线。 |
| 900×620 / scale 1 | 同一流布局缩窄；说明增加行数；工具区按已批准共享规则重排 | 右侧版本/checkbox/reset 不相互覆盖，弹层边界在实际 viewport 内。 |
| 640×520 / scale 1 | 保留导航、搜索、值编辑与状态；分类 drawer 使用 UE 的实际展开/停靠行为 | 不自行发明自动折叠断点；若 UE 非 wrapping 工具区无法完整容纳，逐项记录并交 Parent 决定 overflow，不能裁掉动作后算通过。 |
| 1280×800 / scale 1.5 | 相同逻辑布局，实际窗口 DPI 144，预期客户端 1920×1200 physical，最终以 receipt 核对 | 截图前后 DPI 一致、真实字符行框和 1 DIP 边界比较；不缩放旧 125% 图。 |

这里的尺寸是 LayoutReviewCase 的逻辑 viewport，不是新建固定坐标板的许可。Penpot 中按实际组件布局建立对应 boards/variants 后才记录正式 board ID。

## 3. 已有设置字段与真实控件

权威定义位于 `zircon_editor/src/core/settings/defaults.rs`、`core/script_build/settings.rs`、`core/jobs/quota_settings.rs`。`core/context/builder.rs` 在加载用户配置和构造 job system 前注册四个配额。当前已核对 **17 个内建 runtime keys**（User 10、Project 6、Session 1）；这不是对所有插件贡献 catalog 的封闭枚举。

表中的分类是源 `SettingsPresentation` 的 i18n key path，不是另存的英文分类名。所有已有 label/description 使用 `settings.<完整 key>.label/.description`；例如 locale 是 `settings.editor.language.locale.label`，不在资产中写死显示文案。

| 完整 key | 定义 scope / 分类（省略 `settings.category.`） | schema / 默认 | 对应操作与边界 |
| --- | --- | --- | --- |
| `editor.ui.show_preview_features` | User / editor | Bool / false | Checkbox → 现有 toggle_bool；只控制可见性，不产生未实现 workspace 的真实能力。 |
| `editor.appearance.design_tokens` | User / appearance/workbench | DesignTokens / `workbench_dark()` | 需共享主题专用 editor；当前 painter 没有可操作分支；不能用一个 theme 下拉替代完整 typed 值。 |
| `editor.commands.keymap_overrides` | User / editor/keyboard_shortcuts | KeymapOverrides / 空 delta | 需命令列表、冲突、局部 reset 专用 editor；当前通用 Chord 不是整个 KeymapOverrides 的 handler。 |
| `editor.commands.palette_mru` | Session / editor/command_palette | CommandPaletteMru / 空 | 会话状态，不落用户/项目磁盘；不在持久化表单中伪装普通字符串。 |
| `editor.language.locale` | User / editor/language | Enum `en`,`zh-CN` / en | Dropdown → select_enum；真实动态语言变更，重新投影显示文本并保留稳定选择/焦点。 |
| `editor.autosave.interval_secs` | User / editor/autosave | Int 15..3600，step 15 / 300 | NumberField → 现有 numeric step；真实恢复快照间隔，不声称等同 Save Project。 |
| `editor.script_build.batch_window_ms` | User / editor/script_build | Int 50..1000，step 50 / 300 | NumberField；真实 watch batching；1000 ms 首事件硬上限仍由 script build owner 管。 |
| `editor.jobs.thumbnail_quota` | User / editor/job_scheduling | Int 1..64，step 1 / 2 | NumberField；requires_restart=true；期望配置与启动采用值分别显示。 |
| `editor.jobs.export_quota` | User / editor/job_scheduling | Int 1..64，step 1 / 1 | 同上。 |
| `editor.jobs.interactive_save_quota` | User / editor/job_scheduling | Int 1..64，step 1 / 1 | 同上。 |
| `editor.jobs.play_quota` | User / editor/job_scheduling | Int 1..64，step 1 / 1 | 同上；不能在当前 job system 上假装立即生效。 |
| `editor.viewport.translate_snap_enabled` | Project / viewport/snapping | Bool / true | Checkbox；实际 viewport toolbar 与设置使用同一 typed authority。 |
| `editor.viewport.rotate_snap_enabled` | Project / viewport/snapping | Bool / true | 同上。 |
| `editor.viewport.scale_snap_enabled` | Project / viewport/snapping | Bool / false | 同上。 |
| `editor.viewport.translate_step` | Project / viewport/snapping | Float 0.0001..1000000，step 0.1 / 1.0 | NumberField；不把 toolbar 的格式化显示字符串回存为值。 |
| `editor.viewport.rotate_step_degrees` | Project / viewport/snapping | Float 同范围，step 1 / 15.0 | NumberField；单位 degrees 保持原 schema 意义。 |
| `editor.viewport.scale_step` | Project / viewport/snapping | Float 同范围，step 0.01 / 0.1 | NumberField；step 与 default 不互换。 |

现有 `ui/settings/action_ids.rs` 有 category_changed、toggle_bool、number ±、open/select_enum、color channel ±、edit/commit_string、capture/commit_chord、reset_override、retry_persistence。搜索/All Settings/批量恢复/import/export、可直接键入 numeric 值的提交路径尚未接入；设计中这些是明确的新增 action 需求，不能借一个现有无关 action 声称生效。

## 4. scope、重置及持久化状态

实际解析优先级为 **Session > Project > User > definition default**。定义 scope 表示允许写入范围：User 允许 User/Session；Project 允许 User/Project/Session；Session 只允许 Session。初始 UI 的 setter 写 `definition.scope`，reset 则清除当前 resolved source scope。Parent followup-02 后，r28 在原 mutation 文件形成修复候选：原 reset 签名不变，默认以 catalog 定义 scope 为目标，并新增显式 scope 入口；确定本层无覆盖返回 None，被 Session 遮蔽时交原 coordinator 只清除指定层。六项新回归尚未执行，四独立宿主/逐层 presence/禁用 UI 与 typed numeric commit 仍未接线。

独立窗口的第一版提案：Preferences 展示 User 定义，Project 展示 Project 定义；Session MRU 交回命令面板专用管理。Project 定义的 User 覆盖仍合法，保留原数据；要在 Preferences 编辑这些 User 覆盖，须显式提供写入 scope 和对应来源展示，不能在页面拆分时迁移/删除配置。该覆盖入口的最终位置是 Parent 待定设计项。

局部 reset 只清除当前窗口明确选定的编辑 scope。例：某 viewport 值只有 User 覆盖，Project 页面应显示“继承自用户”，Project reset 不可用；不得沿旧 resolved-source 语义删除 User；r28 候选已改默认目标层，但托管 Rust 和正常 EXE 验证仍缺。Project 层有覆盖时，清除后显示 User/default；Session 仍遮蔽时显示实际来源，不能误报已回默认。按钮 tooltip/上下文菜单与点击语义必须一致。

r30 的写入侧对应 `set_setting_in_scope(key, scope, SettingValue)`：六类旧生产 setter 保留原签名与默认 scope，统一执行原 catalog 权限、原 schema 和 coordinator；scope 由宿主页面上下文提供，不能从获胜值来源或可编辑 payload 推导。新增六项 set 回归与原六项 reset 合为十二项，均未执行。后续 body/semantic adapter 仍须保持页面及 project-binding generation 的事件有效性；此函数未增加 expected-generation CAS，也不代替 typed numeric 文本提交或 undo。

分节恢复默认先收集当前 section、scope、binding generation 和确有本层覆盖的 key 集合；展示影响范围、确认取消零写入。现有单 key clear 不等于已有批量原子事务，批量提交/失败部分结果/撤销合同仍 open。UE Set as Default 的目标是默认配置，不能重命名为“保存用户设置”后宣称同功能。当前 `SettingsChangeLog` 只含 key/scope/revision/requires_restart，没有 before-value；它不是现成的 undo 记录。

持久化复用 `SettingsMutationCoordinator` 与 `SettingsPersistenceDocumentHealth`。当前 `persistence_health_projection.rs` 从 Project、User 中只找 retryable/部分 terminal，Queued/Durable 不显示；拆窗口后必须投影当前窗口的 document，不能让 Project 错误占据 Preferences 唯一状态。以下是需接入的展示合同，不是已完成 UI：

| 后端状态 | 目标反馈 / 可用动作 | 禁止的误判 |
| --- | --- | --- |
| Unavailable | 当前 scope 来源未绑定；解释无项目/不可用 context；不提交 | 不写临时假项目路径。 |
| Ready | 来源已就绪；显示当前解析值和来源 | 未发生写入时不显示“本次已保存”。 |
| Queued | 配置变更已应用，正在保存；关联 file/binding generation | receipt 入队不等于 Durable。 |
| Durable | 同一 binding 的最新 file generation 持久化完成 | 旧 generation 完成不能覆盖新变更的 pending 状态。 |
| PendingAdmission | 已应用但尚未获得持久化准入；真实 retry | 不能另开新 store 或悄悄丢弃该变更。 |
| Terminal(Failed) | 保存失败，保留有效内存状态和实际错误；仅可重试的状态给 retry | 重试继续走原 authority，不重新解释为一次新用户修改。 |
| Terminal(BlockedInvalid / MissingWriteDisposition) | 来源无效/缺少写入身份；只读原因及真实定位入口 | 源码明确不是 retryable，不能显示普通重试按钮制造循环。 |

User 位于解析出的用户根 `settings.toml`，Project 位于 `.zircon/settings.toml`；`ZIRCON_EDITOR_APPEARANCE_PREFERENCES` 指向目录。导入/导出采用真实版本化 typed 格式及文件对话框，尚未证实 UE `.ini` 兼容性。任何路径/权限/格式失败均保持原有效配置与来源身份，不创建平行持久化。

需要重启的配额以“当前进程启动采用值”和“已持久化期望值”比较产生 pending 集合；改回采用值后取消 pending，重启成功重新解析后取消。仅 `requires_restart=true` 不足以让未改动行常亮 banner。Restart Now 依赖 Parent 正常 dirty document 关闭、持久化 fence、退出/重启和 EXE 指纹协议。

## 5. Plugin Browser 的数据与事务合同

真实 `PluginPackageManifest` 已有 version/category/description/maturity/package coordinates；`EditorPluginStatus` 仅投影 ID、名称、来源、load_state、enabled/required、target/packaging、capabilities/features/diagnostics，缺少前述卡片字段。SP-C6 请求沿同次 catalog/status 发布补带已有 metadata 与来源 revision，避免 UI 自行重读 TOML 拼另一套 catalog。图标、作者网址、支持/文档链接未见对应字段，未取得真实来源前不补 UE 示例图和假 URI。

`PluginMaturity` 实际为 Core、Stable、Beta、Experimental、Externalized、Stub、Deprecated，默认 Experimental。Beta/Experimental 可依 UE 提示语义；Externalized/Stub/Deprecated 另有真实不可用 policy，不能当作普通 Experimental、仅点确认后启用。Required 项不可禁用。每次 action 先核对 project instance/session/catalog generation 与被确认的同一插件；取消不写配置、确认期间若来源变化则刷新解释并要求重新审阅。

启停沿 `set_project_plugin_enabled_durable`；feature 沿现有 feature/dependency policy；不手改 report.enabled/load_state。Feature 的限定 action ID 通过已有 producer 产生（含 UTF-8 byte length 编码），不能以显示标签或字符串拼接替代。

`EditorPluginStatusReport.enablement` 已发布完整事务。`plugins/host_actions.rs` 已有 `workbench.plugin.enablement.{verb}:{operation_id}`，verb 为 recover/retry/revert/open；原 report 的 admission、committed revision 是继续操作所需身份。设计需展示完整 `(phase,durable,applied,recovery)`，不是新增一份“插件待重启布尔值”：

| 原 report 条件 | 目标反馈及动作约束 |
| --- | --- |
| NotCommitted | 变更未提交；保留原生效/期望状态并展示具体原因；不画成已启用。 |
| CommitRecoveryDeferred / FileRecoveryRequired | 文件结果需恢复；使用 report 的 host-owned admission；未解决前不得再发一笔普通启停掩盖原 operation。 |
| Durable/CleanupDeferred + Applied + recovery=None | 依据完整 report 显示已保存并应用；保留清理延后诊断；读取真实 load_state 复核。 |
| PendingRuntimeApply | 已保存，运行时应用待处理；retry 必须是原 operation ID + committed revision + 合法 Ready session；不用刷新列表冒充应用成功。 |
| RuntimeRepairRequired | 已保存，需关闭项目并修复；沿原恢复协议，不直接覆盖进程运行状态。 |
| RestartRequired | 当前 `status_message()` 的分支文案是“pending change reverted; project may be reopened”；必须区分已回退/可重开和真正有待重启的变更。不能把枚举名直接翻译成“新配置已保存，重启立即生效”。 |
| Complete + NotApplied + recovery=None | 原协议表示无待处理插件变更；不能凭上次 checkbox 点击保留 banner。 |

恢复错误显示真实 cause，选择和滚动保持同一 plugin ID；操作 ID 失配的旧弹层/按钮失效并刷新。当前 feedback 主要为英文 status string，需由本族新增类型化本地化投影；不能把英文拼接字符串解析回状态机。Undo/redo 不覆盖外部安装、原子文件提交或进程重启；仅在实际命令事务支持并验证后列为可撤销。

相关弹窗边界：

- Plugin Properties：部分 `zircon_plugins/*/plugin.toml` 明确 `@generated from Rust PluginDeclaration; do not edit by hand`，例如 Material Editor 的声明在 `editor/src/capability.rs`。不能按 UE descriptor editor 直接覆盖生成清单；只读展示可先设计，写入须明确 editable source 和 Jenkins generation owner。
- Plugin Directories：UE 独立 tab 初始 750×600，`SPluginPaths.cpp` 将 ProjectDescriptor / Other / CommandLine / Environment 分组；项目根可编辑性受配置控制，User 根默认不可编辑，命令行与环境来源只读。修改有 before/after diff，并走 ProjectManager / PluginManager，再刷新。Zircon 当前 `core/plugin/mod.rs::project_native_plugin_directory` 固定为 `<project>/zircon_plugins`；`ui/host/exports/mod.rs::plugin_directory`、live host/development watch 与启停沿此根。可先设计真实来源及只读原因展示，但没有任意搜索根列表 setter，不能用新增 User 设置假装实现 UE 外部目录编辑。
- 插件发现/刷新：`zircon_runtime/src/plugin/loader/discover.rs` 已有 canonical root、bounded refresh ticket、last-good immutable snapshot 与 generation。根身份准备允许 I/O，交互回调只提交已准备 root 的 refresh 并读取发布快照；不得在 UI 每次 keypress 中同步 `discover()`、路径 canonicalize 或等待 manifest refresh。发现候选不等于 DLL 已通过信任校验或加载。多根持久化、去重/优先级、catalog 冲突与 watcher 所有权仍是 SP-C7 的共享合同需求。
- New Plugin：UE wizard 初始 1000×750。Zircon `tools/cargo/src/main.rs` 的真实命令为 `cargo zircon plugin new <id> --kind importer|system|editor [--native] [--root <repo>]`，执行 `plugin/scaffold/mod.rs::scaffold_plugin`；本轮只读源码，没有调用命令或生成包。kind 初始为 None，必须明确选择，没有 importer 默认值；native 初始 false。可用种类、native 开关和 ID 校验直接映射现有后端，不提供无真实对应的 Blueprint/Engine/Content-only 模板选项。ID 必须首字符小写字母、其余小写/数字/下划线，不能尾下划线或连续双下划线，已存在包拒绝覆盖。

New Plugin 的当前作用域是**带 first-party workspace/catalog 的引擎 checkout**，并非任意普通项目目录。脚手架先在内存生成文本并校验插入标记，随后创建包文件，并写 `zircon_plugins/Cargo.toml`、first_party_runtime_catalog 的 Cargo/lib、first_party_editor_catalog 的 Cargo/catalog、`zircon_app/Cargo.toml` 六处共享文件；失败仅尽力回滚，不是跨文件原子事务，report 的 created_paths 也未囊括共享修改。UI 不能直接在当前共享 checkout 启动它，并把“创建成功”视为已编译/已发现/已加载。

SP-C7 的 wizard 接线提案：先显示 kind/native/ID 与实际目标；由原 Jenkins source/generation owner 从同一 Rust scaffold 提取 pure public plan，返回 revision、源/生成器哈希、planDigest、完整 create/modify/touch-unchanged 文件与目录、before/after hash/absence/diff 和结构错误。只写受管元数据，不 spawn/Cargo；当前没有这条已实现/部署的命令。未来明确允许执行时，沿同一业务 request 与 planRef，由服务器记录内部阶段检查点；不修改已受理 operation payload、不重放 pending。生成、manifest 校验、所选插件/依赖的聚焦构建、发现发布、能力注册各留真实 receipt，不用计划成功代替后续阶段。恢复先有逐文件 journal，仅当当前文件仍是本操作 postimage 才恢复，保留他人改动/未知路径，逐项记录 applied/restored/conflicted/unknown 与实际 native/EOF 或未启动，未恢复项保持 open。普通用户项目 SDK scaffold 未定义，不在本族另建模板体系。

r41 按原 Codex 消息 `01a16dbc-2adf-7911-8337-3d49d946952c` 接收只读 authority 回执：`generation-authority-reconciliation.json` SHA-256 `2b247a0384791c87cfb6fea5920f70aaba82ebdeece6f91b7788b82d7f8a48f4`。current cdd145 capsule/5实现/6clients 实际指纹匹配；pinned runner.generate 只含 source-index、render-template。generate 已有 MUTATIONS/writer_hold 是后续接入基础，未提供 plugin plan/完整写集/恢复接口。原 generation authority owner 为 `01a12af6-7511-70d1-a452-61b80678eb56`，Parent 协调 scaffold/catalog 源 owner（尚未确认），本族消费不可用/错误/恢复状态；本消息不派发实施或改变在途请求。

独立核实 `tools/cargo/src/plugin/scaffold/mod.rs:79` 无条件要求 `static-manifest-end`，当前 runtime catalog 源 SHA-256 `05d54294019ea63e51b71baa64b417470d985e520e13b632b20b8cbfe4952cdc` 中对应 literal 数量0。若此前校验/读取成功而进入此预检，会在新包创建前拒绝；这是静态推断，未运行 generator，不能记录成实际失败或盲补旧标记。应由 Parent/真实 catalog 与 scaffold owner 对齐当前注册结构。

已观察实现的参数化物理写集如下，未提供具体 id/kind/native 请求、渲染内容或 after hashes，**不是已生成的具体计划**：

- 新包5文件：`zircon_plugins/<id>/plugin.toml`、`<owner>/Cargo.toml`、`<owner>/src/capability.rs`、`<owner>/src/lib.rs`、`<owner>/src/plugin.rs`，后四路径均在该包根下；importer/system 的 owner=runtime，editor 的 owner=editor。
- 始终重写6共享文件：`zircon_plugins/Cargo.toml`、`zircon_plugins/first_party_runtime_catalog/Cargo.toml`、`zircon_plugins/first_party_runtime_catalog/src/lib.rs`、`zircon_plugins/first_party_editor_catalog/Cargo.toml`、`zircon_plugins/first_party_editor_catalog/src/catalog.rs`、`zircon_app/Cargo.toml`。Editor kind 的 runtime Cargo、非Editor kind 的 editor Cargo/catalog 即使内容相同仍物理写入。
- native 另加包内 `dist/Cargo.toml`、`dist/src/lib.rs`，合计11或13文件；包根、owner、owner/src 和 native 的 dist/dist/src 新目录也必须入计划。原 `created_paths` 只含新包文件，遗漏共享触碰。当前失败路径吞掉逐项恢复错误且递归删包根，不能作为可确认恢复 receipt。

UI 设计仍只表达真实固定来源 `<project>/zircon_plugins`、kind 必选/native=false 和创建服务不可用；没有可执行计划/生成/修复动作，多根 setter 或普通用户项目 SDK 模板。标记/目录结构的静态证据留在诊断与审阅记录，不显示成一次已运行的创建结果。SP-C7、nativeLifecycle/formal 及原 managed 请求状态保持；上述未来接口均未实现/部署。

`enabled_plugin_template_descriptors` 和 `plugin_template_documents.rs` 是插件贡献的 **EditorUiTemplateDescriptor** 同步，不是 New Plugin 包脚手架；`ExportBuildPlan` 的 source_template 是产品导出，两者均不替代上述创建能力。相关名称相近的入口已核对语义，避免误接。

实际 Penpot 已保存上述 New Plugin/Directories/只读 Properties 与 Beta 确认的域内容板；board ID 及真实重开回执见 [公共组件合同](settings-plugins-component-contracts.md)。全部是 source-backed fixture，没有运行时请求、公共 Dialog master、可执行控件或生成操作。UE Wizard 的模板目录/缩略图、内容包、目标目录与创建后跳转仍未等价；不能用三个 CLI kinds 的文字草稿声称 UE Wizard 完成。

`PluginMetadataObject.h` 与真实 `PluginPackageManifest` 的进一步字段对应如下，均未增加写入能力：

| UE Properties 字段 | Zircon 依据与差异 | 待交接或决定 |
| --- | --- | --- |
| FriendlyName / Description / Category | 已有 display_name / description / category；当前 Material Editor 来源为 Rust declaration 生成清单 | SP-C6 同代 report + 生成来源，先只读，不直接写 plugin.toml。 |
| VersionName / 整数 Version | 已有字符串 version；无对应整数序号，不从 semver 凭空生成数字 | 保留字符串版本；整数版本需求由 manifest owner 决定。 |
| CreatedBy | package_company 属包坐标，并非已声明 UI 作者 | 不替代作者字段，schema 扩展由 Parent/manifest owner 决定。 |
| CreatedByURL | 未见专用字段 | 不生成可点击假网址。 |
| DocsURL | 未见专用字段 | 有真实来源后再接导航。 |
| MarketplaceURL | 未见专用字段 | 市场安装协议/来源独立决定。 |
| SupportURL | 未见专用字段 | 不补占位链接。 |
| EditorCustomVirtualPath | 包 asset_roots/content_roots 不是 UE 编辑器虚拟路径 | 不混写资源根，交资源/插件 owner。 |
| CanContainContent | 已有 content_roots/asset_roots，尚无等价可编辑 bool | 不据 bool 设计发明 Content-only scaffold。 |
| IsBetaVersion | maturity 是多值 enum，不是独立可写开关 | 从同代 metadata 只读显示；Beta 提示仍绑定实际启用请求。 |
| IsEnabledByDefault | 项目 enabled 与完整 enablement operation 已有，不等于描述符默认策略 | Properties 不转发 Browser 开关冒充修改默认。 |
| ExplicitlyLoaded / IsSealed / NoCode | 未见逐字段等价编辑合同 | 保留差异，不以 load_state/package_kind 替代。 |
| Plugins / Disallowed / Sort dependencies | 包有 dependencies，host 有真实依赖/feature policy；未建立 UE 属性列表编辑/排序/禁止依赖的写入事务 | 不在只读 Properties 手改生成清单；交 SP-C6/C7 与 host owner。 |

公共确认层需要承接本域的同一 plugin/project/session/catalog identity、发起控件和决策，不创建第二份启停 store。设计中的 Cancel/Enable 是拟用动作标签，尚未证明取消零写入、单次确认、Escape 和焦点返回。

## 6. World 字段逐类对应

| UE 字段/行为 | 已发现的 Zircon 能力 | 设计取舍和缺口 |
| --- | --- | --- |
| World 绑定、切换、无 World | `ProjectSceneDocument`、活动 Scene 与 source revision；`SceneAsset` 当前根为 entities | 窗口必须随真实场景上下文更新。不能往无对应字段的 SceneAsset 根写一份假 WorldSettings 对象；新增文档合同交 SP-C5。 |
| OverrideWorldGravity / WorldGravityZ、WorldPartition、外部 Actor | 有物理服务、刚体 gravity_scale；未发现等价的每 World 重力覆盖或分区文档入口 | 刚体 gravity_scale 不是 WorldGravityZ；缺失字段不展示有默认数值的可操作行。WorldPartition/外部 Actor 的迁移事务单列待定。 |
| World/Physics 配置 | `PhysicsSettings` 有 backend、simulation_mode、fixed_hz、max_substeps、碰撞层/矩阵；physics manager 有 `store_settings` 经 ConfigManager 写入 | 真实能力已存在，但其作用域、durable receipt 与 World undo 未对应，不能说物理未实现，也不能强塞入每场景 World Settings。是否作为 Project/插件设置需 owner 定义。 |
| Lightmass、PrecomputedVisibility、HLOD、Nanite 等 UE 专属分组 | Zircon 有渲染/烘焙/虚拟几何能力线索，本轮未证明逐字段等价与编辑命令 | 按实际子系统逐字段接入；名称相近不证明相同语义。保持 open，不用灰色假字段算覆盖。 |
| 曝光、环境、后处理 | Scene entity/camera/post-process asset 与 component 有真实字段 | 多数属于 entity/volume/camera，不能挪作 UE World 全局字段；与资源/Actor session 协作复用 Inspector，避免双持有。 |
| GameMode、AI、Network | UE WorldSettings.h 有明确分组；Zircon 对应 World 级配置未完成映射 | 引擎 owner 给出 authority、序列化、命令/undo、runtime apply 前保持未实现。 |

无场景、无支持字段、后端能力不可用和查询无结果是四种不同空态。产品只反馈真实上下文与下一步可执行入口；文档记录不适用项，不在产品内堆源码术语。

## 7. 搜索、键盘、弹层及焦点

打开设置/插件后初始焦点按 UE 放搜索。搜索输入只是窗口状态，不写 settings.toml；按稳定 key 保留分类选择，清除后恢复原合法分类。跨分类搜索需要一次同 generation 的全 scope 数据投影，当前只读取选中分类的 resolver 不足以实现；不得过滤屏上这一页后声称全文搜索。普通输入、中文 IME composing 和按键绑定捕获分开；只有已批准命令注册表里的 shortcut 才给 UI 标注，不杜撰 Ctrl+F/F1 路由。

导航使用 TreeView/ListRow 的真实 key selection 与 disclosure。弹层记录发起 source/control/instance、setting/plugin ID、scope 与 context generation；Escape 先关最上层，取消零 mutation，确认只提交一次，关闭后返回仍有效的发起控件。若分类/项目/场景切换使目标失效，落到当前窗口搜索或合法邻近项。上下文菜单的 Copy/Reset/Reveal 仅在有真实 action 和适用能力时显示；尚未找到的 UE 对应项不虚构成标准菜单。

连续 number step/滚轮、enum 选择、chord commit、中文输入、search clear、窗口切回、Dock 移动分别取证。输入焦点和选中样式独立，不能仅靠 selection 背景替代 focus ring；pointer/keyboard 两种返回焦点都需捕获。

## 8. 验证与证据清单（均待执行）

复用已有 `LayoutReviewCase` 和正常 EXE 原生协议，不新增“settings 专用通过标准”。case 的 dpi 是 scale（1 / 1.5），不是 96 / 144；`host` 只用实际支持的 editor/plugin 等枚举，`sourcePath` 必须在资源闭包中解析。case data 由真实 runtime/projection 产生，额外 identity/evidence 放 sidecar，不向严格顶层 schema 塞造字段。未验证的设计态 fixtures 明确 `fixture`，不能顶替实际配置/插件操作证据。

| 用例组 | 操作与必须保留的观测 |
| --- | --- |
| SP-V01 导航/搜索 | 四窗口 default、真实 empty 与 no-results、分类选择、跨分类搜索、清除查询、筛选后 counts/selection 稳定。 |
| SP-V02 真实设置 | locale 动态变更、六个 viewport keys 与 toolbar 同步；typed schema 拒绝非法值；User/Project 来源/覆盖/reset 和跨项目 binding 失效。 |
| SP-V03 磁盘 | Queued→Durable、真实失败/准入等待/retry、无效源只读；关闭重开读取同 file generation；取消 import/reset 零写入。 |
| SP-V04 插件事务 | 实际插件 Required、Beta/Experimental、依赖与启停；取消、durable/runtime 不同步、失败/恢复/旧 operation；磁盘和本进程状态分别复核。 |
| SP-V05 重启 | 配额 desired/active 差异、改回取消、dirty close 取消、正常重启后 adopted 值；插件按真实 operation recovery 走流程。 |
| SP-V06 场景 | 无场景、场景切换使旧编辑失效；实际字段变化、dirty、undo/redo、save/reopen、runtime 效果。字段未落地前本组 pending。 |
| SP-V07 输入与滚动 | selected、focused、popup open、focus-return、scroll-before/after；en、zh-CN、真实长文本，IME/按键捕获不串事件。 |
| SP-V08 样式 A/B | Parent 指定共享 style 做一次真实 Penpot 修改→保存关闭重开→官方导出→原 ZUI/依赖加载→Preferences 与主工作台两族原生变化；局部覆盖保持、reset 恢复继承、进程重开后持久。 |

以上各窗口覆盖 1280×800、900×620、640×520、1280×800 实际 150%。状态是否可产生依赖真实能力；不可产生的记录原因，不能凭生成图片勾选通过。每图保留 Penpot 官方导出、完整原生 capture、并排图、差异图及逐图复核；边界与文字行框 ≤1 逻辑像素是目标，超差有具体测量和原因。

身份 sidecar 须记录 UE revision、Zircon HEAD/dirty scoped hashes、Penpot file/page/component/style identity、原 ZUI及依赖 hash、插件包/发行物、theme source、EXE SHA-256/配置、HWND/PID/creation time、nonce/generation、resource readiness、实际 DPI、readback 与桌面原图关联、release ACK。当前已有实际 Penpot file/page、四个主窗口结构草稿及四个相关弹窗域内容 board 身份；尚无正式组件/style、EXE/capture identity，`passed=false`、`accepted=false`。浏览器截图只验证中性域草稿，不能代替以上官方导出和原生证据。

本阶段为源码/文档核对及本族 Penpot 域结构草稿，不运行 Rust 或插件构建。r14 的三尺寸重排只验证 Project 草稿排序，不替代本节正式用例。正式组件化设计及相应源码变化后，由 Parent 安排已有插件检查与 Jenkins 聚焦验证；每次编译另行声明 package/features/target/filter/profile，物理输出限定 `E:/cargo-targets`，复用既有配置。不得用排队回执、单元测试或历史主窗口截图替代本族原生验收。

### 正常窗口及官方案例的就绪前置条件

`source-asset-map-r25.json` 记录八个实际 board、两个现有原始 ZUI 及显式 style→token imports；这只是静态 import 子闭包。Preferences 内部行由真实 native painter/projection 管理，Plugin 的 `module_plugin_list_slot` 由 host 动态提供，完整 catalog、字体、主题、当前状态与资源必须在正常消费者中同代核验。已核实命令 `editor.settings.open` 无默认快捷键，`view.plugin_manager.open` 打开既有 LeftBottom drawer；独立 Project/World/UE Plugin Browser 宿主尚未注册，不能靠标题替换证明。

当前 `--source-closure` 经 `prepareRealWorkbenchInput` 无条件限制 canonical Level Editor，正常 producer 也固定该 owner；原公共 pipeline owner 需接续支持本族的源闭包/review-host/actual-case 输入，保留所有源和原生证据约束。N57 候选 EXE/DLL/环境/producer/项目共 10 项指纹及 71 项 producer 合同输入匹配，只证明历史候选字节身份；本族资源 ready、正式导出和当前产品闭包仍 pending。详见 `normal-capture-readiness-r25.json`。

`capture-handoff-r25.json` 是提案 sidecar，不是 LayoutReviewCase 或真实 capture request。它列出 1280×800、900×620、640×520@96 DPI 及 1280×800@144 DPI/1920×1200 物理客户区，拟分 25/35/25 分钟完成不同状态；须先满足共享实例/导出/资源/案例 readiness，再由 Parent 明确授予时段及 DPI ownership。PID、HWND、nonce、case hash、七代数不得预填；实际请求按源码保留 50～10000 ms deadline、fresh nonce/同 case、当前 HWND/七代数、GPU 与桌面像素关联、release/resume ACK。没有启动 EXE/producer 或占用桌面；`passed=false`、`accepted=false`。

## 9. Parent 覆盖表接续：本地化与配置入口

依据 [覆盖对账 OVW-27/40](parent-window-coverage.md)，本族还负责 Localization Dashboard、Config Editor、Device Profiles、Input Bindings 的配置入口。它们不是已保存的八个 Penpot 草稿之一；本节只有实际源码盘点和后续设计合同，未新建空板冒充完成。输入指纹记录在本任务 `extended-source-inputs-r23.json`。

这五个 UE 模块目录内的 `.cpp/.h` 搜索未见 `FAssetEditorToolkit` 或 `FWorkflowTabFactory`，实际入口是直接 TabSpawner/内部 TabManager。`EditorStyle/Private/StarshipStyle.cpp:1018,1408,8398,8770` 实际定义 InputBinding ContextFont（Bold 9）/SmallFont（Regular 8）及 KeyboardShortcuts、DeviceProfiles、LocalizationDashboard 的 16×16 图标。ConfigEditor 调用的 `ConfigEditor.TabIcon` 在本次所读 StarshipStyle 未找到定义，不补推断图标；所有字体/图标仍交 Parent 复用现有 tokens/Starship 及公共实例，不在本域另建 style。

### Localization Dashboard

UE `LocalizationDashboard.cpp:142` 为 `LocalizationDashboard_v5`：横向 Targets/Documents 两栈，初始系数 1.0/2.0，Targets 隐藏 tab well、Documents 初始 closed；Targets 内 dashboard settings、Game targets、Engine targets 是真实 Details 对象。`ShowTargetEditor:205` 按 target 对象身份复用 DocumentTab，重复打开 DrawAttention；`CanMakeEdits:237` 取 checkout notice 的实际解锁状态。`LocalizationTargetSetDetailCustomization.cpp` 的 Gather/Import/Export/CountWords/Compile 调用真实 target commandlet tasks，并按 culture/native-culture 条件判定可执行性，不能把这些按钮只绑定消息提示。

Zircon 已核实 `core/i18n/catalog.rs` 从 en/zh-CN TOML 载入并校验翻译，`EditorI18nService` 提供 available locales、fallback、嵌入包错误和事件诊断，`settings_locale_sync.rs` 经唯一 settings snapshot 热应用 `editor.language.locale`。这支持 Preferences 中真实语言设置；不提供 UE target set、文化列表的作者持久化、翻译采集、PO 交换或 locres 编译合同。UI Asset Editor 的 `diagnostics/localization.rs` 能映射真实诊断及目标节点，可与 Widget/资源 owner 交接定位，不能当作 Localization Dashboard 的任务结果。

后续语言配置区绑定原 User key、resolved source 和 mutation health；target 文档区等待可枚举的真实 target ID/revision、文化/报告数据和 job authority。未具备任务能力时不制作可操作的 Gather/Compile/Import 成功态。String Table 资源作者归资源工具 08，本域只提交已核实的资源身份/打开命令；跨窗口焦点、tab 复用仍用 Parent C2，任务取消/失败仍用 C4，不另建 target/job store。

### Config Editor

UE `ConfigEditorModule.cpp:83` 从指定 `FProperty` 打开 `ConfigEditor` Nomad tab。`SConfigEditor.cpp` 顶部为 TargetPlatform selector，下方按平台 `ConfigCacheIni` 的实际文件层级创建同一属性的行；Details 搜索关闭。`ConfigEditorPropertyDetails.cpp:180` 构造 property table，selection mode/unit 都为 None；`ConfigPropertyCellPresenter.cpp:205,211,229` 根据真实可写性启用值编辑、添加该层值或删除指定 static layer 的 key。`ConfigPropertyHelper.cpp` 的源码控制查询当前被注释，实际 fallback 是文件只读/可创建判断，不能照抄成已完成检出集成。

Zircon 对应设计使用指定 SettingsKey 的四来源 Default、User、Project、Session，优先级继续由原 registry 决定。公开 resolved catalog 只证明获胜值/来源，不能推出每层 raw 值与 presence；请求 Parent C1 同一快照补带层 presence/value、source path/file identity、writability/health 和 binding generation。Default 只读，Session 不宣称磁盘持久化；Project 无绑定/无效来源按真实状态显示。值编辑/清除显式携带所选 scope，通过既有 MutationCoordinator，保留 C3 的 inherited-User reset guard。不写任意磁盘配置、不引入平行缓存、不假装支持 UE target-platform INI 层级。

平台专用配置尚无 schema/selector authority，暂列差异；UI 不提供伪造 Windows/Android 选项。Import/Export 格式与批量原子性继续复用 SP-004 待定合同。恢复默认不是删除全部层，撤销需要原事务 before/after 值和 context generation，change log 本身不提供该保证。

### Device Profiles

UE `DeviceProfileEditorModule.cpp:27` 注册 Nomad spawner、返回 MajorTab；`SDeviceProfileEditor.cpp:355` 采用 0.3/0.7 主分栏，左侧选择/pin/view-alone，右侧 source-control/menu 和 profile tabs。独立 profile tab 以 profile name 注册；析构调用真实 `SaveProfiles`。`Runtime/Engine/Classes/DeviceProfiles/DeviceProfile.h` 定义 DeviceType、BaseProfileName/Parent、CVars 和继承查询；LODs/控制变量来自真实 profile 文档。当前主窗口底部创建提示要求修改 DefaultDeviceProfiles.ini，不能根据另一个存在的 create-panel 文件就宣称主窗口有完整 New 按钮流程。

已核实 Zircon `zr_rhi::RenderDeviceProfile` 是不可变、绑定实际 device ID/generation 的 adapter facts、features、limits、queue topology 和 budgets；它不提供命名配置、parent graph、CVars、LOD profile store 或编辑器激活事务。在所查 Editor/Runtime/plugin Rust 与 TOML 范围未发现等价 DeviceProfileEditor 入口，不能据此断言全引擎永远不支持。当前保留能力 gap，交 Parent/graphics/platform owner 确定可编辑模型、持久化/平台选择/undo 和实际资源重建。未来只读设备事实须保持自己的语义，不以重命名此对象关闭 UE Device Profiles。

### Input Bindings / Keyboard Shortcuts

UE `InputBindingSettingsViewerModule.cpp:16,38` 注册独立隐藏菜单的 `InputBindingSettings` Nomad tab，容器 `InputBinding`；`InputBindingEditorModule.cpp:524` 注册 `Shortcuts/InputBindings` 并接 export/import/reset/save。每命令 Primary/Secondary 两个 `SChordEditBox`；逐项 reset 恢复两个默认 chord。`SChordEditor.cpp:210,237` 检查同 context 冲突，明确接受后移除冲突命令上的对应 active chord，再设置新 chord；冲突弹层与 focus-loss 提交/取消有独立实现。UE import/reset 的重启要求来自本实现，不直接移植到热应用能力不同的 Zircon。

Zircon 已有 `editor.commands.keymap_overrides` 的 User schema，requires_restart=false；`EditorKeymapOverrides` 是 operation path 到 Option<EditorKeyChord> 的 map。None 是显式保持解绑的 tombstone，删除 map 项才恢复内置继承；不是同一个 Reset。`EditorKeymap` 只有每命令一个有效 chord，提供带 WhenClause overlap 的纯冲突分析；真实 dispatch 对两个同时 enabled 的候选返回 None，不任意选一个。当前未核实编辑器 mutation 路径已强制调用冲突 preflight，也没有专用 capture UI，不能从纯函数存在推断交互已完成。

设计使用同一 command registry 的稳定 operation path、localization keys、effective when/context 和 settings generation；显示一个真实键槽，UE secondary 槽留差异。捕获复用 Parent C1d 的 chord-input/IME/focus 合同：编辑时消费键事件，不触发被编辑的全局命令；Esc/失焦/弹层返回需按实际来源逐项验证。候选 map 先做同代冲突分析，冲突弹层列实际受影响命令；取消不 mutation。确认再绑定须在一次整 map 变更内写新 chord 与相关 tombstones，保留 before/after 和 generation 供撤销；没有该事务时不拆成多个可能部分成功的 UI setter。持久化仍走 MutationCoordinator 的 queued/durable/failed/retry 状态。Import/Export/多键槽只在有真实版本化格式与 schema 后提供，不能用无效果按钮或直接修改嵌入 default.keymap.toml 代替。

## 10. 额外覆盖的跨域消费合同

本节接收 Parent Actor 的 45 项覆盖分工，接续第 9 节与 SP-C1～7，不增加 provider、公共 API、注册项、主题或共享写权限。下表“待交付”均为消费合同要求，尚无对应接线/验收回执；不会把这些描述当成已注册命令或可用控件。核查输入与结果在本任务 `coverage-provider-contract-r32`。

| 覆盖项 | 业务 owner / 本族范围 | 当前能力及待交付 |
| --- | --- | --- |
| OVW-27 Localization Dashboard | 本族配置/生产入口；资源 08 持有 StringTable 作者事务 | 已有 locale 设置和 i18n 同步；target/culture/revision、生产任务执行/取消/产物 receipt 仍 open。 |
| OVW-40 Config Editor | 本族单属性的分层配置表面 | 已有同代 resolved batch、原 coordinator 和 r30 typed scope set/reset 候选；各层 raw/presence、项目绑定/currentness、持久化重开待接。 |
| OVW-40 Device Profiles | 本族 profile 编辑表面；Parent/graphics 提供真实模型与激活 | RHI 设备事实可读；命名 profile/继承/可编辑项/持久化/应用事务 provider 未交付。 |
| OVW-40 Input Bindings | 本族单键槽配置与冲突流程 | 现有 typed User delta、有效 keymap 与纯冲突分析；目标层 map/currentness、捕获、原子确认/undo/焦点待接。 |
| OVW-35 Struct Viewer | 资源 08 负责文档/浏览；Parent 持有 reflection catalog；本族消费类型选择 | 已有已接纳 schema 元数据查询；实时宿主上下文、类型选择回传、字段适用性和焦点路由待接。类型作者/编译不归设置。 |
| OVW-38 String Table | 资源 08 负责资产作者、key/引用事务；本族消费本地化引用与打开 | 当前 UI localized-text 支持 editor/default 表；不等于可编辑 StringTable 资产。真实 asset/key 身份、rename 引用迁移/undo/文档打开 receipt 仍 open。 |
| OVW-28 Device Manager / OVW-39 Source Control | Parent 负责设备/构建/部署及 Jenkins/Git 主体 | 本族只消费 provider 给出的 profile 引用、来源/可写状态与请求 receipt；不把部署、checkout、提交或 changelist 命令塞入设置 mutation。 |

### 配置、语言与快捷键的同源事务

`editor.language.locale` 与 `editor.commands.keymap_overrides` 在 [defaults.rs](../../../zircon_editor/src/core/settings/defaults.rs) 均为 User 声明、requires_restart=false。语言仍走唯一 SettingsAuthority → [settings_locale_sync.rs](../../../zircon_editor/src/core/context/builder/settings_locale_sync.rs) → [EditorI18nService](../../../zircon_editor/src/core/i18n/service.rs)，不另存 dashboard locale。翻译 target/culture 与编辑器语言是不同业务身份；Gather/Compile/PO 交换需要真实生产 provider，不能从 locale 成功推出任务可执行。

Config 消费 [ResolvedSettingsBatch](../../../zircon_editor/src/core/settings/catalog/resolved_settings_batch.rs) 的 key/value/source/generation；它没有每层原值或 presence。Parent C1 所需扩充仍在同一 authority 读域内捕获：各层 presence/raw value、显式可编辑 scope、来源文件/绑定身份、可写原因与原 persistence health；不要在绘制时分别读磁盘凑快照。Default 只读，SessionApplied 不标为 Durable。User-only key 禁止 Project 写；Project reset 只清 Project，未覆盖返回 None，Session 遮蔽时也不追随获胜来源。r30 typed set/reset 没有 expected-generation CAS，公共事件与项目绑定失效拒绝仍 open，不能靠 UI 禁用替代宿主准入。

快捷键候选必须以**目标 User 层原始 map**为基线，另取同代 effective map、command registry/when 和上下文。若 Session 遮蔽 User，把 resolved map 回写 User 会泄漏临时覆盖；现有 resolved batch 不能证明已具备目标层基线。按 [EditorKeymapOverrides](../../../zircon_editor/src/core/settings/keymap_overrides.rs) 的真实接口，使用 `bindings().clone()` 后构造 `EditorKeymapOverrides::new(...)`；没有臆造逐项 setter：

- 绑定：候选 map 中该 operation path 为 `Some(chord)`；显式解绑为 `None` tombstone；逐项恢复继承为移除该 path。重置整个 overrides key 是另一意图，不能用它实现一行 reset。
- 通过 [EditorKeymap::with_overrides/conflicts_with_when](../../../zircon_editor/src/core/commands/keymap.rs) 计算实际冲突；两条 effective when 可交叠才冲突，缺少命令 predicate 时保守报冲突。冲突列表保持 operation path，不用本地化标签作身份。
- 捕获/候选/冲突确认期间不写配置。取消或上下文过期不 mutation，返回原焦点；确认时重新核查代数和命令可用性，在一次整 map mutation 中写新 chord 和被明确接受的冲突 tombstones，保留不相关原始条目。删除冲突命令的 override 可能恢复同一内置 chord，不能代替 tombstone。
- 写入沿原 User coordinator receipt；queued、durable、failed/retry 与 undo 的 before/after/上下文分别处理。当前没有证明 UI 原子确认、失效拒绝、撤销或持久化重开已完成。UE secondary 槽及 INI import/export 继续记录差异。

### 资源类型与本地化引用的回传

与 [资源 08 RT-E06/RT-E09](08-resource-tools.md) 对账后，本族消费选择结果与资源文档打开；不实现 Struct/StringTable 作者事务。所需回传信息是合同要求，不是已增加的 DTO：

| 消费结果 | 身份与准入 | 确认、失效、取消和返回 |
| --- | --- | --- |
| 反射类型选择 | canonical full type path、目录 fingerprint/algorithm、宿主的 World/registry 身份及实时 schema generation、字段允许的类型约束 | 查询结果过滤只控制列举，不授予字段写权限。确认前在同一当前宿主重新解析并校验约束；同名短路径歧义、目录变化/插件卸载或目标字段变化时拒绝旧结果并刷新；取消不修改设置，返回原属性行焦点。 |
| StringTable 引用选择/资源打开 | 资源 owner 提供真实 asset/document identity、stable key/reference identity、document/reference revision 与支持的引用类型 | 显示译文不作 ID；rename/引用迁移、撤销及保存属于资源事务，设置只消费成功回执。资源关闭/重命名/切项目后复核引用；失败保留原值与错误，取消返回原焦点；重复打开交 Parent C2 复用文档。 |

[ReflectSchemaCatalog](../../../zircon_runtime_interface/src/reflect/schema_catalog/mod.rs) 及 [ReflectSchemaResponse](../../../zircon_runtime_interface/src/reflect/schema.rs) 已提供已接纳元数据、完整目录 fingerprint 和短路径歧义判断；[WorldReflection::list_reflect_types](../../../zircon_runtime/src/scene/reflect/world_reflection.rs) 不访问对象实例。**响应 fingerprint 与 snapshot 均不含实时 epoch/generation**；`TypeRegistry::schema_catalog_generation` 由宿主另行维护，需 Parent provider 在同一借用中关联，不能从 fingerprint 推导。字段槽位只对当前字段序列有效，不作跨版本保存身份；类型选择也不证明可创建实例或作者自定义 Struct。

当前 `EditorI18nService::resolve_localized_text_for_locale` 只接受 `editor`/`default` 表；StringTable 资产 provider、键引用 schema 和事务未交付前，不把任意 asset ID 写入现有 table 字段或新增平行翻译 store。

### Parent 设备与源码控制接续

[RenderDeviceProfile](../../../zircon_runtime/crates/zr_rhi/src/device_profile.rs) 的 DeviceId/DeviceGeneration、adapter、features/limits/budgets 是不可变硬件事实，不能持久化为命名 profile 或部署任务 ID。命名 profile 的 stable identity/revision、parent graph、可写项、保存与应用结果由实际 provider 提供后，本族再接表面；无 provider 不列造 Desktop/Mobile 配置。设备连接/部署/取消、构建产物与 Jenkins operation 关联由 Parent 处理。

配置来源只读、保存排队或失败来自现有 persistence health；不从“可写文件”推断 Source Control 已连接、已 checkout 或已提交。若 Parent 提供源码控制操作，本族仅消费真实请求与结果、刷新同一配置来源，失败/取消保留原值；Git 写操作和 changelist 主体仍走 Parent/Jenkins，不能在表单中直接执行。

后续独立工作是继续域内容与现有 projection 的字段映射；消费接线前分别等 C1 原始层投影、C2 文档/焦点、C4 事务/错误及对应资源/设备 provider。当前只完成盘点、设计和业务适配收件；这些新增窗口没有正式 Penpot 板或 EXE 图，原八板/组件身份、全部历史失败和 `passed=false, accepted=false` 保留。

## 11. 第二次接续的当前源码交付边界

Parent followup-02 已再次核对，r33 继续沿原 `settings_mutation.rs` 增加 numeric 文本提交入口，不改 projection 的公开签名、公共 host、原始 ZUI 或已保存 Penpot。`set_numeric_setting_in_scope` 按 catalog 的 Int/Float 解析为 i64/f64；空/中文非数字/逗号、整数小数或溢出被拒绝，NaN/inf/越界交原 schema 拒绝；有效值走原显式 scope/coordinator，错误不写入、不 clamp。新三项回归与原十二项合为十五项，保留 SessionApplied/Unchanged 的真实含义。输入事件、可编辑状态、scope/page/project binding 的当前性、undo 与重开仍待真实 UI 接线，不能把函数存在当作数值控件完成。

r33 当前源 `57dc7a2a36aa883c1f7f1215d3f1ce94d790fb52c4eb9f20221276551cc26602`，最小 patch `8a0cd4b90f5efe9f2550857b347a958e73642b12b383cf828972bbdba9608f8d`。格式/diff/change guard 通过且 0 errors/warnings/suppressions；全部十五项尚未执行，旧 r27 五项 identity 与原失败不重写。当前测试计划的唯一 package 为 zircon_editor、defaults off/features=[]/lib、单 job/thread、`settings_scope_mutation_tests::` filter，Cargo test/adapter 默认 dev recipe，物理 E 根；已准备独立 identity，但没有提交新请求或本地 Cargo。

r34 为 SP-C6 交可审阅的完整十文件提案，未应用：新增 metadata DTO/三处 reexport、原 builtin/native/missing 构造与 fixture 迁移，原公开字段及 enablement tuple 不改。缺包为 None；mixed builtin+native 行保留原 builtin 名称/metadata 优先级，不宣称 loaded binary version。提案增加 native 快照独立性、live 失败保留元数据与完整 enablement/reverted 状态的两个回归，并扩充原缺包断言。提案格式解析和只读 apply-check 通过，没有最终 patched checkout 的 guard/编译证明；消费 UI/共享卡片与 binary provenance 仍待接口协调。精确 hash/文件及缺失字段见 [Parent 交接](settings-plugins-parent-handoff.md)。

r35 实际查询原 Jenkins 请求仍 `ordinary_entry_unproven / Current session authorization is pending`；新 installed capsule 的 11 项指纹匹配，但变更只涉及 material 配方约束，settings 的 external-source capture guard 未变化。原客户端/辅助进程均已退出，不启动服务、不借 material 身份。全部插件/托管 Rust/正式源导出/正常 EXE/四尺寸及实际 DPI/双端逐图验收继续 open；Parent 所阅 r26 图只证明域草稿身份与各自文字，失败图保留，`passed=false, accepted=false`。

## 12. r37：插件详情的恢复态准入

源码追踪确认列表已根据同次 `EditorPluginStatusReport.enablement` 撤下恢复期间的变更动作，Feature Details 却只按 feature flags 生成 Apply intent 和 Tab 控件。本轮在获准的原 `module_plugin_projection/details.rs` 统一三处判断：详情 action/intent 投影、Tab 的 Apply 条件、提交时重新校验。仅对同 plugin 且 `recovery != None` 生效；不改变原启停事务、公共 DTO、control/action ID、operation tuple 或 restart 路由。

恢复期间保留真实 feature 选中、desired/required/blocked 与诊断，追加原 operation `status_message()`；原焦点在 Apply 时移回 features。新报告解除恢复后才生成新的一次性 qualified intent，旧 intent 无效。另一插件的恢复不禁用当前插件。四种恢复、解除后新意图、无关插件及诊断保留共新增三个回归，详情模块共八项，设置 mutation 十五项均未执行。

实际详情源 SHA-256 `7a8263ede2cf3efaafb95ef67d9421aaa966718ea1107f7ca36a85b1de153792`；两份域源码合并 patch SHA-256 `6b57b1f4a12cf67610efb622e72f9a93eb8c990be3b0d7540f54b9888f07ea42`。格式/diff/静态 guard 通过，精确 diff、夹具静态修正及未提交的最小测试声明在 `plugin-detail-recovery-r37`。r34 公共提案原字节未变且 apply-check 仍通过，九个其他基线未变、详情基线前移，仍须 Parent 协调；未应用、不算编译或 UI 验收。

## 13. r39：原值与 presence 的最小 provider 提案

不把 Session winner 写回 User，不从 resolved source 推断被遮蔽的 Project 层 absence。精确未应用提案扩充原 `ResolvedSettingValue` 的私有三层原值和 `override_in_scope(scope) -> Option<&SettingValue>` 只读访问器；原 `SettingsRegistry` 增 pub(super) 层读取，同一 `from_registry` 在原 authority lock 内捕获。`None` 代表该层不存在 override，和默认值/其他层继承分开；原有效值、source、key、batch generation 及已有读方法保持。

此方案复用现有 selected-key batch，没有第二 store、磁盘扫描或分层快照拼接。三个原值仅在存在时 clone；还会增加原 DTO 的私有状态及 PartialEq 内容，需要 Parent/core owner 审核读取成本。现 struct 字段已私有，源码只有一个构造点，无新增 reexport/外部 literal 或持久化格式迁移。三个回归候选覆盖 Session 遮蔽下的 User/Project 原值、默认及继承的目标层 absence、清除隐藏 Project 层后的旧快照稳定；均未编译/执行。

两路径为 `core/settings/registry.rs`、`core/settings/catalog/resolved_settings_batch.rs`，均在 `zircon_editor/src/` 下，超出 followup-02 实际写入范围，因此只交 patch，不改源。提案 SHA-256 `38f618eb703ce78b97dd314b2f8ab901257d17805a14f6fb678a9c92fd88ee2a`，source baseline、格式解析、只读 apply-check 和精确 scope 在 `scope-projection-proposal-r39`。未包含 page/edit scope 身份、expected-generation/project-binding mutation fence、scope persistence health/file/writability 或 UI/撤销；这些仍须 Parent 与原语义/宿主 owner 接线。不能以此关闭 C1/C3/Config 或快捷键缺口。

## 14. r42：真实 raw 消费及显式 page-scope 接口候选

Parent followup-03 已应用第13节与 r34 的源提案，12 份实际 after hash 全部匹配。r42 仅在 `ui/host/editor_event_runtime_access/settings_mutation.rs` 替换 reset 的 winner 推断，调用真实 `resolved_settings([key])` 并读取 `override_in_scope(scope)`。空目标层返回 `Ok(None)`，不要求本层持久化源；存在的目标层仍走原 coordinator clear/receipt/error，Project reset 不删除 User。两次读取和 clear 之间未加原子 currentness/CAS，不能由 raw getter 推导 generation fence 或 undo 已有。

新宿主接口按 Parent 约束仅提供精确未应用 patch：`scope-consumer-r42/settings-scoped-values.proposed.patch`，SHA-256 `423e5651517502c8d940b272038f06758ef7f4d9d8783c273407b09b64034508`。只拟扩充原 `settings_projection.rs`，旧签名和调用方保持；新增 `capture_settings_scoped_values_for_category(category_id, edit_scope)`，恰好调用原 category capture 一次。只给原 `ResolvedSettingsBatch` 配对 category ID 和显式 page scope；行迭代借用原 row 及该行 raw 值，不扫描全 registry、不建第二 store、不读磁盘，原 key/effective value/source/batch generation 保留。格式解析与 apply-check exit0，canonical 源未写，不是已接线 UI。

Parent/C1/C2 接线还须携原 catalog/locale/definition.scope 权限、窗口 page scope、project binding 和事件代次；重绘不能按 winner 或默认 scope 改写页面 edit scope。raw presence 只回答目标层有无 override，不提供 writable/durable/resetEnabled，也不允许旧项目事件写入新绑定。当前十七项 mutation 回归和八项详情回归未执行，Jenkins 沿原 owner 的当前源核对，不借旧五项请求证明。

SP-C6 已有真实 `EditorPluginStatus.package_metadata` 四字段，但 `layouts/windows/window/host_data.rs::ModulePluginStatusViewData` 无对应字段，`app/pane_data/view_rows.rs` 不能交给现有 native 卡片。最小共享合同是原 ViewData 的可选 typed metadata、从同一 status row 投影并迁移全部构造点，再由正式 PluginMetadataCard/native 语义子树消费；None 显示不可用。保留原启停/恢复 tuple、选择、焦点、事件和缓存，不把元数据拼入 title/诊断冒充支持，不把 catalog version 当 DLL provenance。共享 DTO、正式资产、公共 theme/masters 均未改。

## 15. r43：四个发布主件的受控消费边界

本族原 file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` 新增验证 page `1ca0591c-2856-49b0-9d7e-eeb076bafe03`/board `4829a7b5-8807-4e66-b34d-b94fbb8807fe`。Shared rev1 与四主件 canonical-record 的 library/component/main UUID、真实 sourcePath、源组件声明/root controlId/catalog document 已逐一核对；当前源码 SHA只作为读到的源指纹，发布记录缺 sourceHash，不能推导 canonical owner 等价已验收。真实主件/引用见 [组件合同](settings-plugins-component-contracts.md)。

只用原生“管理库”连接、素材 drag/drop 的 native instantiate、原 component copy/shapeRef，以及正常 update-file 校验保存本族验证页；没有复制 main、detach、更改 Shared、其它族、现有 ZUI 或主题。原生 linked copies 的 source metadata 完整保留，包括 control-part-list/authoring painter 内部原 ID；正式 exporter 仍须验证通过 shapeRef 解析这些引用，本族不改签元数据冒充合法导出。

验证板保留8个实例，顺序为默认 Button/SearchInput/TreeRow/PropertyEditorRow，再为对应四个文字覆盖。覆盖分别是“恢复本层”“搜索设置 / Search settings”“项目设置 — 长分类名称”“平移步长 (Project)”；它们只是文字设计覆盖，不连接 settings action/query/value，也不赋予可写性。沿原 rich text/touched `text-content-text` 记录覆盖并正常保存，关闭context及两次fresh重开后4/4保留，全部66个复制节点（含8个根）的shapeRef有效。

真实 `value_slot` 存在，原组件声明与 catalog 均要求一个接受列表内的编辑器。当前发布的四主件中没有任何 accepted value editor，因此两个 PropertyRow copy 的 slot保持原样空；不插入不接受的 SearchInput/按钮或假数值。Button/SearchInput/TreeRow可进入后续正式域布局试接，PropertyRow目前只验证原框架与label，不是表单交付。description/source/reset/换行行高、显式 page scope 与状态/action/代次仍沿 SP-C1/C2/C3 合同，由原 owner 合并，不能靠扩充 painter role 解决。

真实 PNG 显示长中文树label和属性名换行超出固定行高，依赖共享组件的原测量/溢出合同；不在本族随意拉高或修改原master。首次native插入的column几何反序，单改stored顺序未重排；已通过本验证板原生 row→column触发布局，最终顺序与flow保留。三张诊断PNG逐图记录，均为1600×1100 headless浏览器/100%canvas、640×520验证板，不是正常EXE、系统150%DPI或窗口族尺寸验收。没有正式ZUI/PNG exporter回执；原插件host/bundle及两族全局样式/override-reset仍待公共owner。

## 16. r46：详情焦点随真实报告更新

UE `SPluginTile.cpp::OnEnablePluginCheckboxChanged` 以当前CanModifyPlugins/descriptor及操作结果控制行为；Zircon optional-feature分页属于本域适配，继续以原EditorPluginStatusReport为唯一事实来源。本轮没有更换公共Button/Table或编辑UI资产，只修原详情状态里失效焦点的归还规则。

报告刷新后，先沿原feature ID重新定位page/移除失效选择，再判断原焦点是否仍可用：Back始终存在；Previous/Next按当前page边界；features需要至少一条；Apply必须由原detail_feature_action为同owner/所选feature给出非空action。有效焦点不改，失效返回features，空列表返回Back。原recovery禁写和一次性intent/报告代次失效保留，Back仍返回原package selection；不从焦点推断持久化、restart、runtime有效状态或metadata来源。

新增3定义覆盖feature移除/必需化/owner不匹配、四焦点空列表和两分页边界失效；检查稳定feature身份、新旧intent、其他plugin/operation不变与缓存复用。当前共11个详情定义，原8个保留，尚未运行Rust或实际键盘/焦点画面。源码/精确增量/格式/guard及原prepared输入过时原因在[域合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-detail-focus-r46/contract.json)。新的native/UI验收仍须同状态、真实能力与正常EXE，不能以这项源码修复关闭整个Plugin Browser。

## 17. r47：消费Parent的同次metadata承载

Parent followup-04已将原 `Option<EditorPluginPackageMetadata>` 经原status行、`ModulePluginStatusViewData`、`ModulePluginStatusPayload` 送至既有 `payload_plugins[].package_metadata` TOML表。version/category/description保持原字符串，包括空值和中文；maturity使用原schema的稳定小写代码，测试oracle改为原Serialize实现。None不生成表，operation-only恢复行仍为None；原operation tuple/action、缺包状态和目录发现路径不改。本族已读实际9份after hash并全部匹配，复用其对应format/guard静态回执，不重做旧r34/r39或r47重复补丁。

自有r47提案在Parent并发接线前只完成内存rustfmt和apply-check，未写canonical源；后续apply-check拒绝已变化路径。原fixture缺字段观察随后由Parent迁移None解决，单fixture提案未生成。旧proposal、失败、未送达队列回执全部保留，标为superseded，不能拿其2个未应用测试计入canonical。Parent实际新增3个metadata定义仍未执行；r46详情11定义、mutation17定义及原managed身份保留，未扩大旧filter覆盖。r42 scoped wrapper仍是精确proposal，等真实页面调用与currentness合同后再接。

数据承载不等于正式卡片或DLL来源证明。现native列表112固定行高没有强塞长description；正式同源MetadataCard、布局、选择/操作/重启与source closure仍需公共owner和域适配共同完成。

## 18. r48：原Plugin Browser业务板中的原生共享实例

文件 `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` 实际rev27，原Plugin Browser page `c6a45a2a-60ca-41fb-a1c4-c5164da3cad3`、board `9d912f75-c86c-4b3a-878d-1007947cbf81` 保留。Header flow中加入Button×2和SearchInput×1，分类flow中加入TreeRow×4；7个根由真实Penpot素材事件调用原native instantiate-component创建，没有脚本复制main或伪造shapeRef。最初直接拖至selection rect不创建实例；只读源码 `viewport/actions.cljs:477` 对viewport-controls目标的要求及后续成功路径均已记录。事件为隔离浏览器中的DOM事件，没有使用桌面或全局剪贴板。

Button分别覆盖为Add、Plugin Directories，Search为Search plugins...；TreeRow依次为All Plugins、authoring、runtime、asset_importer，保持真实分类key。覆盖只修改原label source ref的rich text并记录touched；五个旧文字节点保留ID/content/metadata，隐藏并移出active flow，未删除。所有新节点沿Shared rev1原main引用，91个shapeRef与canonical source metadata一致。原5个metadata tile/本地组件身份和其他4页对象指纹未变。

关闭原context、两次fresh重开后7份覆盖、原父flow和外链均保留，文件总458对象；Shared仍rev1/41对象/4主件/0颜色/0排版。25项身份/保存值结构检查成立；原26项总检查false保留，因为浏览器监听没有记录native mutation请求。直接update-file HTTP200、最终API持久化读回与缺失的逐请求传输记录分别列出，后者继续open。

三张1600×1100/headless/deviceScaleFactor1/约71%canvas的真实图已逐一打开：新增控件和source-copy检查器可见，旧占位不再重复显示；TreeRow仍带不适用的展开、cube、eye/lock，Button/Search仍为240固定宽度，正文为白底中性草稿。这些差异未通过UE复刻审查，不能以Penpot选中边框证明引擎焦点。Add/Directories只记录设计跳转目标，不伪造已接engine action；New Plugin执行、Dialogs与重启仍受原真实authority限制。

原始ZUI、导出codec、公共风格A/B及实际EXE/DPI/双端图均未完成。本轮不授予或申请提前占桌面；等source closure/正常family producer真实就绪再按原协议申请。证据、逐图差异及实例身份在 [r48保存重开记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-browser-linked-r48/save-reopen.json) 和 [逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-browser-linked-r48/visual-review.json)。

## 19. r49：原Preferences / Project导航与搜索消费

同一真实文件现为rev35/5页/712对象。原Preferences page `b4a1e3ce-9e6a-81fe-8008-d69d60e4e403`/board `92bc794d-9419-4284-bb9c-282b99afb641` 中使用Search×1/TreeRow×9；原Project page `49ff1071-f83b-4b72-bb6e-043a3326ec82`/board `c5edfcc8-00e6-4419-a463-3324207d5335` 使用Search×1/TreeRow×3。均沿Shared rev1已发布component/main，由原生素材事件调用原instantiate-component，没有复制main或手工造instance shape。新增14根/254个节点均核对source ref及metadata，原World/Plugin Browser/受控试接页、Shared页及原local metadata组件未变。

Preferences的分类标签沿原All Settings、Appearance/Workbench、Editor/Language/Autosave/Keyboard Shortcuts/Script Build/Job Scheduling，Project沿All Settings、Viewport/Snapping；真实presentation i18n路径、原plain text ID、copy/label/sourceRef和明确User/Project设计scope在`design-changes.json`逐项对应。去掉旧文字的手工空格，不用空格编码新组件层级；实际TreeRow缩进/展开/选择仍未绑定，不能将显示标签或列表序号当category key。All Settings/搜索也没有伪接无关action。

十四份覆盖只改变原实例label richtext和touched，十四个旧文字保留原ID/name/parent/content/metadata，hidden并移出active flow；原schema/default/source scope文字全部保留。第一次重开仍有旧位置空隙，两个原图保留为`*-first-reopen-gap.png`。通过真实原生检查器使原分类容器row-gap临时1再还原原0，触发原生布局重算；没有写子控件x/y/transform，最终两个容器都是原flex/rowGap0/columnGap0。修复后两次fresh contexts分别断言两页native身份、标签、archive及连续排布，六张实际原图均已打开复核。

初次预检使用`pages-index`读JSON而失败，真实字段为pagesIndex；一次只读浏览器诊断把Response.status当函数；一次原文字全节点fingerprint断言在原生flow插入后拒绝继续，随后只读核对content/name/parent/source链接。失败都发生在对应direct mutation之前并保留，完整旧几何差异没有从摘要重建，不声称原全节点guard仍通过。正式source/export guard未调用或放宽，当前pre-archive指纹、原内容及来源分别记录。Parent共享Rust路径本轮未改，r42 wrapper不应用，不启动编译。

浏览器performance实际显示native保存路径为`/api/main/methods/update-file`，与原RPC-only监听不同；校正诊断context listener后记录5份native POST HTTP200，仅存path/status及payload hash/bytes。此前Preferences创建和r48历史逐请求记录仍缺，不重放或补记。25项源身份/保存值/flow结构检查成立；不等于全轮完整传输审计或UE视觉通过。

仍有真实差异：TreeRow显示对象cube/eye/lock及不适用leaf disclosure、分类层级未接；Search根固定240，原1280搜索flow的剩余空间未使用；value_slot接受的编辑器仍未发布，正文白底schema fixture，无真实搜索、value/reset/undo/persistence/restart/currentness或产品焦点。公共同源变体/尺寸/PropertyRow与长标签需求沿SP-C1交原owner，显式scope/代次仍交Parent真实page caller；不新增公共props、UUID、主题、store或独立producer。

本轮只有1280×800逻辑板的1600×1100/headless/deviceScale1/约71%canvas设计图；900×620、640×520、实际150%DPI及完整状态矩阵未在产品新验证，没有正常EXE、双端图、正式ZUI或1逻辑像素测量。passed=false、accepted=false。证据：[保存重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-linked-r49/save-reopen.json)、[逐图差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-linked-r49/visual-review.json)。自有两个浏览器/Node组合均退出且PID不存在，保留有界driver及必要失败证据，释放0 B。

## 20. r50：typed metadata到原卡片的source join

只读API再次确认rev35/五页/712对象、Shared rev1/四主件/零样式，Plugin Browser整页和原业务组件未变。Parent followup-04九源hash均匹配；其当前guard为4ef0bd40…，32ce94d8…是较早消息。没有再次应用patch、增加caller、生成新source或执行Rust。

| 原main中的真实节点 | 已存在的payload字段 | 待正式source owner接线 |
| --- | --- | --- |
| `7fa43274-e6d1-4f55-9b52-18a42d86569c` title | `payload_plugins[].display_name` | 保留plugin_id及当前row/report身份，不以label或索引替代。 |
| `7cea9849-64ee-4195-90e8-1583aa866254` version | `payload_plugins[].package_metadata.version` | 可选表缺失时清除旧值；不当binary provenance。 |
| `fafd3fc1-85e8-4f29-af30-66d444331266` description | `payload_plugins[].package_metadata.description` | 沿同源文字流测量/换行，不能截成旧112行内painter文字。 |
| `7ec7d52b-deaf-4e63-9a65-a0a66697275f` category/maturity | `payload_plugins[].package_metadata.category/maturity` | 使用同一i18n/rich text格式化，七个原schema代码与显示标签区分。 |

这张表是原节点与真实数据的消费提案，不是已安装Penpot/runtime绑定。原component domain metadata明确sourceWrapper=null/officialCodecMetadata=false，component/main缺canonical-record；原`module_plugins_body.zui`是view，`plugin_list_slot`仅声明动态Container/scroll槽，没有对应卡片children/重复原型。原owner需提供可审阅的canonical wrapper或同源重复原型、owner-instance来源、稳定重复身份和事件/slot映射；本域不新造component/master/来源签名。普通canonical master导入要求真实component源，不能把view根冒充卡片。

原Penpot main832×116、description52、category/maturity40，native仍固定112；这些是设计/源码尺寸，不是实际窗口差异测量。正式consumer还需自然换行/行高/scroll、None后清旧值、Some空值、operation-only恢复行及现有action/operation保留。Parent的metadata removal回归只覆盖payload属性，不证明上述渲染/焦点/持久化行为；3个新Rust定义依然未执行。精确source SHA、当前节点和三个阻塞合同在 [r50 eligibility](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/card-source-eligibility-r50/eligibility.json)。12项只读事实核对不升级passed/accepted；本轮Penpot/源码/服务/桌面/生成/编译操作均0，释放0 B。

