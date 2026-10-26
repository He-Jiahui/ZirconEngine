# 设置与插件窗口族

## 2026-10-25 r96：保存健康读取失败的真实调用链清理

`sync_open_settings_window` 在保存健康捕获失败的提前返回处，现消费原 bridge 已显示 typed context，经现有 runtime/coordinator 的 phase、项目 binding 与 authority generation 只读检查决定是否撤销。检查不能证明仍有效时，同时清除值上下文和 r94 原生/抽屉旧草稿焦点；仍有效时保留同一值上下文与草稿。无论哪条分支，旧 retry context 均撤销，待正常健康捕获重新安装。没有新增 set/reset/retry 操作、store、clock 或后续 mutation 的预留权。

实际修改授权14路径中的2份 Rust，内存严格逆增量14/14恢复本轮原始 SHA；保留原74个静态测试定义，新增2个（当前76不是 harness 发现或通过数）。回归定义通过原 coordinator `clear_project` 的真实通知阶段观察 `OperationInProgress` 与旧值失效，并验证 pending User 写入/retry ticket 不被消费、健康刷新不能恢复旧值；另一条保留仍有效批次。Rustfmt/源补丁 guard 通过；Cargo、正式 source/resource/native 准入及 GUI 未运行。

[本轮源合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-health-capture-retirement-r96/source-contract.json)与[最小验证意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-health-capture-retirement-r96/validation-intent.json)记录当前14源、精确补丁和两个独立未来 filter。r78 静态回执仅按其原字节保留，不重放旧业务身份/输入。原 frame/window/open lifetime 与 semantic origin、其他控件/四页 scope、provider、正式导出、normal EXE、实际150%DPI及逐图验收仍 open，passed/accepted=false。本轮未改 Penpot/共享公共源/资产或服务，释放0 B。

## 2026-10-25 r95：真实三尺寸导航设计及独立重开

专属文件实际rev330→355/5页2193对象。原生Ctrl+D在Project Settings页新分配3×122对象；原1826非目标根节点、目标根其他字段/原子项顺序及另外四页保持。All聚合行采用UE原“All Settings”文案、Shared strong及大写覆盖；导航padding8，All/两个叶行高40/24/24。隐藏generic cube；叶箭头opacity0保留UE Hidden占位。主体实际flex/row、详情fill，各尺寸导航128/180/256。普通分类根/provider、实时20%约束和可拖动分隔栏未完成。

保存及新浏览器重开均rev355/data=02fc7deebcf7d5d294fae5a3bfb92ff451e6068f08286d0f86afddaa6ad6e59a。最终[引用审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/native-audit-r02.json)：87copy=83Shared+4Card，另1原Card main；1264shapeRef、2318非空styleRef均有效，161空引用单列；新366对象source pluginData逐项等原板。没有detach、新建公共main/主题或修改原失败板。

三尺寸板内saved/fresh差异像素均0，23张[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/image-review.json)已实际复核。[640并排图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/penpot-before-after-640.png)仅为Penpot对比：13780变化像素中356在导航外，未归因或称整页安全/UE≤1px通过。首轮旧缓存、嵌套编辑器、迟positionData/hash guard及null引用检查失败均保留，只修工具。

Shared实际rev5/4main/34色/3typography；linked syncedAt早于modifiedAt且仍提示更新，未全库Update/ACK/reset。25项[源码及资产指纹](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/source-assets.json)记录实际UE继承链。本片没有Rust、正式plugin/export、源资产写回、服务或桌面动作；正文仍是设计文字。根Ellipsis/tooltip、字体闭包、真实字段/层级/选择/焦点、producer/EXE/150%DPI与两族主题A/B继续open，r94进展及全部旧失败保持。goal active，`passed=false`、`accepted=false`。

## 2026-10-25 r94：替换显示批次时撤销原生旧草稿及抽屉返回焦点

实际落源原授权14中的settings_window_actions.rs与surface/option.rs。原native文本focus仅保存控件/action/草稿/编辑框；sync/open/category/changed receipt替换显示批次时，旧草稿仍可能进入后续Enter，抽屉保存的previous_focus也能恢复旧草稿。这是当前源码调用链推断，未作GUI竞态复现。[精确两源增量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/source-change.patch) SHAc9c07f6f3dc6905809c1ce6556e66c181cad740f5da98777473db70cede5f98e；两前像严格内存逆增量均匹配，原14中其余12源及72定义保留。

原设置回调现在在open、目录/值失效刷新、extension currentness错误、分类切换与changed receipt刷新前，撤销主窗及committed model已登记浮窗中三种Settings commit action的native focus，并清除原content_drawer_native_previous_focus中的设置草稿。复用原cancel与frame damage/full redraw入口，不发绑定/commit、不改coordinator/authority锁、scope、generation、persistence/retry tuple。无变化/纯health刷新保留原焦点，其他输入和其他抽屉返回焦点保留。[真实调用图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/source-trace.json)。

新增native_focus_retirement_tests下2个待执行定义，含撤销前Enter正控、String/Numeric/Chord撤销后late text+Enter不发旧编辑、原保存焦点清理与其他输入仍能Enter；显式单元focus fixture不冒充生产String provider。现74只是14源静态定义，不能据此宣称harness或GUI通过。[独立最小filter意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/regression-contract.json)交原执行authority，不改旧r35/r37/r51/r91/r93请求或payload。

初次rustfmt新代码排版失败和两次报告逆增量/路径规范化诊断错误保留；修正后format/reverse/scoped diff及精确两源guard均exit0、0errors/0warnings/0suppressed。callback_methods实际直接同步调用，无新建的deferred queue；本片不实现原frame/window/open-lifetime+published semantic batch来源封装，关闭不重开原调用点、IME preedit/cancel与排队事件仍open。[当前公共9路径及发布边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/origin-boundary-current.json)仅只读，5/9同r71、4后继均原位保留。

本轮0Cargo/intake/QueryOnly/capture/freeze/桌面/DPI/Penpot/Shared/.zui/服务/新业务身份/源码副本，r92 rev330仅历史设计证据。正式export→产品加载、normal EXE/150%DPI/两族A-B、生产provider/typed subtree及原11要求继续open，goal active，passed/accepted=false，释放0B。

## 2026-10-25 r93：值刷新与分类重建失败撤销原重试上下文

本轮fresh14源仍r91，沿真实调用点继续修复同一生命周期：sync_open_settings_window可先安装health重试上下文再刷新值；分类回调原仅撤销值。这两条生产路径的直接template刷新失败时，旧retry仍可能被getter返回。refresh_after_settings_receipt本身已先清两上下文，该事实保留，不把它误报为新的提交后重试缺陷。

在原settings_window.rs的refresh_settings_values、refresh_settings_values_and_close_editor、select_settings_category各加take/restore原Option，只有全部property写和直接模板刷新成功后恢复；错误保持None。原invalid/unhandled/closed返回分支、传入typed batch、原pending tuple/authority/CAS/clock/persistence和14源边界均保留，未新增字段/store/provider。[实际源码增量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-value-category-materialization-r93/source-change.patch) before4e936f57…→afterfc74f80a2257cc781d59d94282784da4822e28961c52efc67a63733c5e3f3f79 /93453B，7生产新增行含1注释；严格内存逆增量精确还原前像。

新增health_materialization_tests::value_category_state_change下2定义，成功矩阵覆盖三transition×原Some/None，传入generation故意早于最新authority，仍只保存传入批次；分类无效/未处理和关闭noop保持。失败矩阵用原WorkbenchSceneTitle MissingNode制造真实模板错误，检查两上下文撤销、pending/authority代次不变，普通重建不复活、显式health/value捕获各自恢复。原70定义全部保留，现72只是静态定义，health宽prefix从6变8，不能扩旧prepared范围。[独立最小验证意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-value-category-materialization-r93/regression-contract.json)只选新2定义，不重投旧r35/r37/r51/r91身份。

初次rustfmt4处新模块排版失败保留；修正后format/reverse/scoped diff与精确1源guard均0、0errors/0warnings/0suppressed。Git LF→CRLF索引提示保留，源实际LF未变。类型/Rust/GUI未执行、无正式intake/source/resource/native回执；无Penpot/Shared/.zui/codec/服务/桌面/DPI动作，r92设计按历史证据保留。正常EXE/事件原window/frame来源/typed subtree/正式export/实际150%DPI/双族A-B仍open，goal active，passed/accepted=false，释放0B。

## 2026-10-25 r92：长标签单行原生试验与根/节样式继承更正

真实 Penpot 原长标签失败板保持，另用原生复制得到三张122节点的 linked consumer 试验板，只有新板使用原 text auto-width/fill、行 Clip content 和既有 Search 左右约束。640×520、900×620、1280×800 的9行均为单行、原28高，分类栏仍20%；这些是长文本设计压力输入，不是生产 provider。当前rev330/5页1827对象，完整data SHA 031ce1d0b1ba2da70a84acf7993e74396b6508fb05634d1d4b2de4301b79ecfa。

进一步读取 UE StarshipCoreStyle.cpp 的完整继承后，更正初始“所有分类默认Clip”的过宽判断：NormalText/普通节为Clip，但ButtonText显式Ellipsis，PrimaryButtonText继承它；根分类/All再复制PrimaryButtonText并大写，另有额外上下间距。当前三试验板仅证明单行/行裁切，根行省略号/大写/粗体/间距未还原；不能把默认FTextBlockStyle当最终选中样式。[角色合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/category-source-contract.json)保留该失败及更正。

关闭保存浏览器后由全新浏览器重开三尺寸，完整data一致，三张完整向外取整画板裁剪均0设备像素变化；[24张逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/image-review.json)和[像素比较](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/pixel-comparisons.json)是Penpot内部证据。仍可见分栏边界文字相贴，前后比较有六处不变值的小栅格差异；内侧裁切/字体/tooltip/图标语义和UE≤1逻辑像素没有通过。正文仍是结构草稿，非真实PropertyRow编辑。

API独立核对71 Shared copies、1063 refs（1031 Shared/32 Card）及1880样式引用有效。新366节点完整metadata与文字内容保留；57 SVG绝对路径逐坐标符合原生float32平移，并非内容整hash不变。初次过宽内容hash审计exit1及r02把跨页复用zero-root计为唯一ID的计数错误都保留，r03报告总对象1827/唯一ID1823。1460原节点精确不变（仅Project zero-root全前像未记录，不声称其其它字段精确）；Shared rev5完整data未变。[正式回读](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/reopen-verification.json)及[引用审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/native-source-audit.json)。

本片0 Rust/.zui/Shared/codec修改、0Cargo/intake/QueryOnly/服务/桌面/DPI；14域源仍同r91、70仅静态定义。原失败/请求/事件来源生命周期/typed provider/正式导出→加载/正常EXE/实际150%DPI/两族A-B继续open；goal active、passed/accepted=false、释放0B。

## 2026-10-25 r91：弹层与滚动重建失败时同时撤销旧重试上下文

本轮 fresh14源仍r88，实际布尔/枚举/步进/reset已用显示态上下文，未重复补旧 caller。进一步追到真实 Preferences 弹层开关/显式关闭/滚动：这三 bridge 方法绕过 r88 generic rebuild，失败前只取走 values、却保留 persistence retry 上下文。在已授权 settings_window.rs 内增加6行生命周期逻辑：重建前同时取出两捕获，全部成功才原样恢复；原 noop 分支不变，没有新 store/clock/generation 或持久化动作。

实际1源 after SHA 4e936f57ebe4ea284e94f79101e31d05d8423891cf8d66ce666e9d9e9ab0d8e4 /82092B。新增 health_materialization_tests::popup_scroll_state_change 的2定义，四实际 transition 各覆盖成功/noop、原 MissingNode 重建失败及原节点恢复后的显式 health/value 再捕获；原68定义全部保留，当前70只是静态定义，未执行 Rust。严格内存逆增量精确还原3fec0d9c…，rustfmt/reverse/scoped diff及精确1源guard均0，patch fe7819d3a940f4066e24126cfb20754f5c84375955c9d1098a9cd9e0ae9bb989，0errors/0warnings/0suppressed。[实际源码及14指纹](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-popup-scroll-materialization-r91/source-pins.current.json)和[独立最小验证意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-popup-scroll-materialization-r91/regression-contract.json)。

未修改 public native input/Shared/export/其它 bridge 行为；本轮未运行Cargo、创建新request/attempt、重投旧r35/r37、QueryOnly、source capture/freeze、服务/桌面/Penpot。r90真实rev307设计保留。部分property写失败/其他direct refresh、原生frame/window事件来源、GUI输入/正式导出/normal EXE/实际150%DPI/双族A-B仍open，goal active、passed/accepted=false，释放0B。

## 2026-10-25 r90：真实 linked SearchInput 的内部宽度与保存重开

在原 Project Settings 三个受控画板上继续原生 Penpot 设计：外层 SearchInput 原已 fill，但背景仍240、文字框仍204。使用原生宽度输入框和“左右固定”约束，仅修改原6个子节点；背景为640/900/1280，文字框604/864/1244，保持28左/8右内边距、16图标、原文字与样式。依据实际只读 UE SSettingsEditor.cpp:228–243 的 FillWidth 搜索框；未新增组件、ID或画板，未 detach、编辑 Shared、重签 source guard 或正式导出。

真实文件 rev295→307，仍5页1461对象；其余1455对象和全部 pluginData 精确不变。关闭原浏览器后新浏览器重开，完整 data SHA 6feb4a69acadc8aad29397fa2acec4e27b9f9e3000052c51924e1cd32152187a 一致，三尺寸保存/重开画板裁剪均0设备像素变化。独立核对 Shared rev5、59 Shared copies、862 refs（830 Shared/32 Card）和1448样式引用有效；这是源引用核对，不是跨文件同步 ACK。15张图已逐张查看，原 r84/r87 长标签失败画板逐节点保持；[r90 证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-content-flow-r90/reopen-verification.json)及[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-content-flow-r90/image-review.json)。

六条属性正文仍为 domain 结构草稿，不是已接通的 PropertyRow/Field；真实 Property main 的 value_slot 仍空且尚缺新槽，未用文字填充冒充编辑能力。Search provider/筛选/焦点/IME/弹层、live resize、正式插件导出→产品加载、正常 EXE/真实150%DPI及双族主题 A/B 未由本片证明。原14 Rust 均同 r88/68静态定义，r89 IME三源仍仅候选；0Cargo/新验证请求/服务/桌面/源码或资产修改，goal active，passed/accepted=false，实际释放0B。

## 2026-10-25 r89：旧焦点 IME 已提交文字的三路径精确候选

继续检查 r88 恢复入口时，实际 `sync_open_settings_window` 已通过 context currentness 区分缺失与代际0，因此没有为已处理的情况重复改源。转向 r86 输入缺口，fresh 核对30原输入源，29仍原SHA，唯一后继是已落 r88 的 settings_window；原14源当前全部仍 r88。

已形成 Parent 公共 owner 的3路径最小未应用 patch `348792fba80cd4cf6d727ff69f40f064961421e3bbe503d1081b83259d530f4a` /4457B：只在 canvas/retained Session 先行处理后的旧文本回退中接收 `Ime(Commit).text`，原 Runtime/Interface 继续保留结构化事件；文字进入原 draft insert，不把输入法 Commit 当作设置 Enter 提交。另修正旧翻译测试的 Text 断言，新增2个 pure translator→fallback 回归候选，覆盖中文/组合字符/空字串与预编辑/取消/delete-surrounding的排除。

三个原前像和提议后像、所有旧测试名字保留，见 [candidate](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-legacy-ime-commit-r89/candidate.json)。原/候选均仅 stdin rustfmt exit0，严格内存正向重建3后像一致，`git apply --check --whitespace=error` exit0；没有写3份公共 canonical Rust，未声称 final patched-checkout guard/类型/Rust/GUI通过。原14内68定义保持，额外2仅候选，旧1为更正候选；不以70或3签发执行结果。

UE 对照仍为实际只读 SlateEditableTextLayout.cpp：Begin/EndComposition处理编辑事务，OnTextCommitted(...OnEnter)另行触发。此片只补已提交文字传输，完整预编辑/取消/组合期间Enter/撤销、native事件顺序与原window/frame/open lifetime仍open。r86 painter提案、String生产provider、4页、typed subtree/export、normal EXE/实际150%DPI/两族A-B继续open。仅向 Parent交精确公共候选，集成后再由原authority按独立两个最小filter与当前revision受理；本轮0Cargo/intake/QueryOnly/source capture/freeze/Penpot/服务/桌面，goal active，passed/accepted=false，0B。

## 2026-10-25 r88：通知重建失败撤销设置编辑与重试上下文

已沿实际通知生产入口补齐一处 r83 未覆盖的边界：`notifications.rs` 先准备 persistence health，再直接调用真实 bridge 的 `refresh_prepared_state_change`。本片仅在已授权 `componentized_window.rs` 的设置上下文生命周期中，进入两个可失败步骤前取出原 values/retry；全部成功才原样放回，任一 Result 失败保持 None。后续公共重建成功不会制造新的编辑权限；原 health capture 可以恢复实际 retry，原 values refresh 再恢复原显示批次。没有修改通知源、DPI/mount/input/publication 实现或原 coordinator/authority 条件写。

两份实际 Rust 增量：bridge `02421b14…→4d4c0b35…`，settings_window 回归源 `5cb7988a…→3fec0d9c…`。严格内存反向删除本片增量可还原完整前像，原14源的另外12份与 r86 完全一致，原66静态测试名字全部保留。新增 `health_materialization_tests::prepared_state_change` 的2个回归定义，覆盖成功、通知准备后成功、两种重建失败、节点恢复、health-only 与显式 values 恢复；68只是定义计数，未执行 Rust。

源 patch/guard 摘要 `e1e8809b7a9bc36d95c51a51eebb66b65ef066e6a909c7ec45b9ab240c9f2a8a`。rustfmt、精确 diff whitespace、反向 --check、change-guard 均 exit0；guard 0 errors/warnings/suppressed。最初格式检查只报3处新断言换行，已保留失败并修正；Git 原 LF→CRLF 提示原文保留，两源实际仍 LF。独立最小 filter、当前14pins及未来执行合同见 [r88](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-prepared-materialization-r88/regression-contract.json)，不扩旧 r35/r37 或 r83 已有请求。

本片没有 Cargo/intake/QueryOnly/source capture/freeze、服务/桌面/Penpot/正式导出。Penpot 最后实际观察仍为 r87 rev295/Shared5，长标签失败保留；第二个 publication 错误分支虽经过同一撤销代码，未新增独立故障注入证明。native originating frame/window/open lifetime、Numeric painter/IME、真实 String provider、4 page scope/typed subtree、Toolkit/normal producer/EXE/实际150%DPI/两族 A-B 继续 open。goal active，passed/accepted=false，0B释放。

## 2026-10-25 r87：Project 导航原生流式验证板，长标签失败保留

真实 Penpot 从 rev258 保存到 rev295，5页/1461对象；原973对象除 Project 零根的子项成员表外逐节点哈希不变，其余4页和文件全局数据完全不变。通过原生复制新增4块可编辑板/488对象，没有新建 main、detach 或重签源 metadata。Shared 保持 rev5，59个外部实例、862条 shapeRef（830 Shared/32 Card）均有效，4板完整插件 metadata 与原板对应节点一致；576处样式引用保留。实际身份与完整读回见 [fresh-reopen-audit](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-flow-r87/fresh-reopen-audit.json)。

仅在受控板的原 TreeRow linked instance 上用原生 flex 和原有 token 数值设置横向内容流、标签填充剩余空间；640/900/1280板的侧栏宽128/180/256，标签宽约76/128/204，短标签框横向溢出均0。关闭浏览器再以新进程重开，完整文件数据哈希 `bde8d7853e5dc3b5f32fa1bf8f8a785e2b57b6f660e6dfc15ba510dead549a26` 一致，4个画板区域截图均0设备像素变化。既有 r84 的52px失败原板及所有历史保留。

长英文/中文设计压力板实际产生5/4/4行，固定行高下文字覆盖邻行；最大英文行宽81.546875，超过约76宽文本框5.546875。故仅关闭受控板短标签框的横向溢出问题，长文本/ellipsis/tooltip、UE类别图标及叶节点语义继续 open。14图已逐张复核，见 [visual-review](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-flow-r87/visual-review.json)；并排/差异图是 Penpot 两状态比较，不是 Editor 双端验收。6个属性值仍明确为源 schema 设计夹具，没有假编辑或搜索能力。

本片0 Rust/资产修改、0 Cargo/新请求/QueryOnly/source capture/freeze/服务/桌面/DPI/正式导出。9份 UE/组件/catalog/token/Starship/桥接源哈希与 r84 相同。沿既有 Settings Form/App Sidebar 映射及 UE 本地源码；公共 navigation/leaf/单行文本合同仍交原 owner。r85/r86实际实现与独立回归意向不变，正式 plugin/Rust/EXE/150%DPI/两族A-B 未通过。goal active、passed/accepted=false，0B释放，保留必要证据和2个受控 helper。

## 2026-10-25 r86：Numeric 原提交消费者已落源；原生输入缺口单列

已 fresh 核对 Parent 应用的 Shared3 原始后像，三个文件保持 `0eec18ff…/5784eb92…/aea53a37…`，本片未重 apply 或改写。随后只在本域两源应用原 r74 `domain2.proposed.patch`（`d2b44500…`）：host `settings_mutation.rs` raw `c19f8023…` / 55087B，surface `edit.rs` raw `28fa2140…` / 6007B。两源原 CRLF 保留；其 LF 正规化 SHA 与原候选精确一致，不能把 LF 候选 hash 当实际 raw hash。全14源内存反向还原匹配 r85，另外12源不变；原 r85 六源捕获条件及 r83 bridge 均保留。

真实中心 hit 的 commit action 现在经原 `ControlEdited` 分支、bridge 保存的 typed display context、原 schema 的 i64/f64 解析，进入原 coordinator `set_from_context`，保留同锁条件 mutation、definition scope、receipt/noop/persistence/error 行为。原3个候选回归已成为定义；限定14源的原63个静态定义保留。两源 rustfmt/diff/reverse/change-guard exit0，guard 0/0/0。未执行 Rust、GUI 或 native 输入，单独最小 filter 见 [r86 测试意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-consumer-r86/regression-contract.json)，不并入旧 r35/r37/r70/r71/r73/r78/r81/r83/r85 请求。

输入链已从实际生产源追踪：本地 draft 的 commit_only edit 只重绘；Enter 派发原 commit，Escape 清除焦点。Numeric painter 当前仍读快照 value_text，已给 Parent 两源未应用小补丁 `ebcef833…` 和1个未应用 renderer 回归，保持原中心矩形/颜色/±路径。IME 有实际接口不一致：Runtime 当前保留 `Ime(Commit)`，Editor legacy fallback 仅接受 Text，retained pane 在 legacy text focus 激活时也拒绝此路由；旧 Editor 翻译测试仍期待 Text。此为生产源码静态差异，无实际失败运行回执。曾据旧测试形成的报告前稿已保留并更正，见 [输入修订](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-consumer-r86/input-lane-correction.json)，不把 IME 称为接通。

native originating frame/window/open lifetime、4 page scope、String provider、正式 editable subtree/export、normal EXE/实际150%DPI/双族 A-B 和全族验收仍 open。r75 rev152 是历史，r84 rev258 为最后一次本域观察，本片无 Penpot/Shared/桌面/服务动作；52px 长标签盒失败保留。goal active，passed/accepted=false，实际释放0B。详见 [机器交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-consumer-r86/handoff.json)。

## 2026-10-25 r85：Preferences 原生产入口的目录投影与值批次配对

已在 Parent 授权的 14 路径中修改 6 份 Rust，接通真实 `open_workbench_settings_window` 与目录完整刷新入口。两处使用原 `SettingsWindowProjection.settings_generation()` 捕获值批次；原 coordinator operation-state guard 保持项目绑定，原 authority state lock 内先比较完整 u64、再读取同一 registry 的 resolved values。发现代际变化时返回原 `StaleGeneration`，不把新值批次拼到旧投影。值单独刷新、分类刷新和原 String/reset 条件提交继续使用原流程；没有新增 store、clock、provider、必填 DTO 或 action/control ID。

新增 5 个回归定义（core 3、host 2），覆盖陈旧捕获后的 phase 可用性、相同值代际下项目重绑、u64/耗尽边界，以及真实 builtin 分类/空分类与原投影配对。均未执行；限定 14 源中原 58 个静态定义及原测试内容保留。六源格式、diff、反向检查和 change-guard 通过，增量摘要 `2008c2c4…`，0 errors/warnings/suppressed；严格内存反向还原均匹配 r84，另外 8 源字节不变。首次 Python wb 返回 Err22、14 源未变，之后 apply_patch 落盘成功；失败与恢复分别保留。

此片只增加生产捕获条件，不提供 native originating frame/window/open lifetime，也不让 locale/contribution 的跨服务读取成为原子。原 catalog 投影含本地化定义，不能称为完整 editable tree；未发现生产动态 catalog 注册来证明所谓目录成员竞争。搜索仍缺真实输入/query/filter/geometry 入口，已给出 8 个共享源的只读合同，未添死 handler。r84 Penpot 最后观察仍为 rev258，未在此片重开或截图；52px 长标签盒溢出、正式导出、4 page scope、String provider、normal EXE、150% DPI、全族通过继续 open。

证据：[r85 增量与静态检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-projection-values-pair-r85/handoff.json)、[独立两 filter 准备](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-projection-values-pair-r85/regression-contract.json)、[搜索实际缺口](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-projection-values-pair-r85/search-gap.json)。没有 Cargo/intake/新身份/source capture/freeze/服务/桌面/Penpot 操作；`passed=false`、`accepted=false`，goal active，实际释放 0 B。

## 2026-10-25 r84：Project分类显隐覆盖及真实三尺寸可编辑板

真实原生 Layers 隐藏三分类行中12个 eye/lock及panel节点，全部由Penpot记录visibility-group覆盖。原12差异仅hidden/touched；文字、几何、原source metadata/shapeRef、34色alpha与字体均保留。UE分类控件只有expander/label/tooltip，此覆盖消除本域不适用操作的设计画面；公共navigation/navigation-leaf尚未发布，未改Shared或内部几何。

own file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` / Project page `49ff1071-f83b-4b72-bb6e-043a3326ec82`，rev236→258；原1280板 `c5edfcc8-00e6-4419-a463-3324207d5335` 不改尺寸。两新板由实际原生duplicate分配：900×620 `342ce826-b591-800d-8008-d801c759fd53`，640×520 `342ce826-b591-800d-8008-d801e8de8b52`；非新组件master。新增244对象，973总对象/5页，43 Shared heads、594refs(562Shared/32Card)全有效；Shared5整hash保持，另四页/global保持，原对象除root成员和12显隐覆盖均精确保留。

三板保存/关闭/fresh context重开整data hash `1cae08b627751d24b7097167dcd9e7f7a99c32f123a009649455c70e71cc7330` 相同；8原图+2实际裁剪/差异逐张复核，原1280覆盖前后1307设备像素变化，三板各自saved/fresh crop差0。640标签框仍溢出52.00000482472478逻辑像素，仍failed/open；未用短标签可见代替长文本/≤1DIP。库更新banner仍显示，本轮未做整库sync/reset，未证明navigation源guard或正式导出。

属性值仍原schema设计fixture文本，无引擎预览/可写控件证明；正式codec/source export→原ZUI→正常EXE、真实150%DPI、四Toolkit/页scope/主题两族A-B及原11要求仍open。当前14 Rust全同r83；无Cargo/intake/QueryOnly/source capture/freeze/桌面/服务/生成/源码副本。goal active，passed=false/accepted=false；0B释放，证据保留，整体cleanup未关闭。


## 2026-10-25 r83：持久化健康刷新失败撤销编辑上下文

原 `refresh_settings_persistence_health` 在首个健康属性写入前暂取原值上下文，仅成功或无变化后恢复同一批次；准备/整树刷新 Result 错误保持 None。沿原 health tick、receipt 与 retry 回调生效，正常值刷新负责恢复编辑；原 pending persistence、Retry 身份、scope/generation/coordinator 均保留。

新增独立 health_materialization_tests 两定义，覆盖成功/no-op、真实模板缺节点错误、失败后 pending 与 authority 不变、健康恢复不冒造值批次、正常值刷新恢复；未运行 Rust。当前14源仅此1份变化、13份与r82相同，旧30定义完整保留，32是静态定义数。rustfmt/反向应用/差异空白/change-guard 均 exit0，guard 0 error/warning/suppressed。

源 `5cb7988ab371aba87453e7c0b73f58990cc27ec397d255d9c149bdd32e89c158`；patch/guard `79dae0d5e5abe6df8b8b59129767922bfa0efd0cbc9395393e75776595efaba0`。证据 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-health-materialization-r83/`。

r74 Numeric 三公共入口仍未落地，本轮不添加孤立 consumer；公共 Rejected/origin lifetime、四page语义树、provider、正式export→原ZUI→正常EXE/实际150%DPI与族验收继续open。Penpot沿r82实际rev236/Shared5历史观察，本轮无API/设计/画面新证据。0Cargo/intake/source capture/freeze/服务/桌面/生成，goal active，passed=false/accepted=false。


## 2026-10-25 r82：PluginMetadataCard共享字体与真实缓存失败修正

实际原20文本绑定Shared5 body/caption/strong（5/10/5），保留五份manifest原文、原Card UUID/main/4copies/32shapeRef、34色引用与alpha、原domain metadata；原69 DIP缩略图和auto-height布局保持，五Card均77 DIP。

三尺寸原图均保留。640原content比较还覆盖SVG path坐标而失败，不能升级整页源guard；恢复1280后15段文字positionData仍在x309、shape x533，相差224，三张坏图稳定且已保留。原生相同内容编辑为no-op；实际内容临时追加/恢复的首轮因过早hash稳定判断中断，rev207临时字元已沿原编辑路径在208恢复，再处理剩14。现rev236原文全恢复、20字体引用及当前x缓存核对，关闭/全新context重开整data hash一致；修正当前保存态不代表尺寸失效根因已修。

own file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402`，Plugin Browser page `c6a45a2a-60ca-41fb-a1c4-c5164da3cad3` / board `9d912f75-c86c-4b3a-878d-1007947cbf81`，rev199→236，Shared5；最终整hash `39bd214f2d9b04535323e92a982ed182c839c67c52b9dc45f15914c38521d4f9`。另外四页整hash保持，460refs（428Shared/32Card）当前全部有效。10原图逐张复核、两张实际Penpot裁剪对照/原差异图；不是EXE对照或≤1px通过。

原thumb空槽/真实启停、operation/restart、repeat prototype/wrapper、正常producer/正式export→原ZUI→EXE、真实150%DPI、两族A/B及原11要求仍open。只是本域可编辑设计，非引擎预览或验收。 当前14 Rust仍与r81同源，旧30静态定义/managed请求均未执行或重投；无新Cargo/intake/QueryOnly/source capture/freeze/服务/桌面/DPI/生成。6实际browser均exit0；必要失败与helpers保留、释放0B、cleanup未关闭。证据 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-plugin-card-flow-r82/`；goal active，passed=false/accepted=false。

## 2026-10-25 r81：弹层物化失败后的值上下文处理已落源

实际修改原14授权路径中的 `settings_window.rs`：Enum/Color打开、关闭及滚动关闭弹层，在首个属性写入前撤下原显示值context；原template重建成功后才恢复同一份typed capture。任何传播的属性/重建Result错误保留context为空，交现有正常values refresh恢复，不在事件里补取最新snapshot。无变化/无效滚动仍不改context，原coordinator、精确u64代际和项目绑定条件写保持。

新增独立 `popup_materialization_tests` 两个静态回归定义，覆盖成功路径，以及移除真实模板节点触发原UiSurface缺节点错误、撤权和正常刷新恢复；尚未运行Rust。r78原28定义保留，新增2单列，不能用旧filter或旧r35/r37回执覆盖。14当前pin中13沿r80，仅本源变化；严格内存逆增量、rustfmt/whitespace/reverse/change-guard通过，guard `d98b25c85eb7d76550d632e80c2ab3afce9d87a4b754d695f190d0a540d59485`（0errors/warnings/suppressed）。

本片仅处理传播的Result错误；通用control_state吞掉Rejected报告、原事件origin/window/open lifetime/semantic batch、四page scope和其它控制动作不在本次完成范围。证据根 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-popup-materialization-r81/`。Penpot沿r80 rev199/Shared5，本轮未访问或修改；原900/640溢出、正式导出、正常EXE/实际150%DPI及两族A/B继续open。0Cargo/正式intake/QueryOnly/source capture/freeze/服务/桌面/生成；goal active，passed=false/accepted=false，必要证据保留、释放0B。

## 2026-10-25 r79/r80：真实 Shared typography 消费与导航源重排边界

本族 Project 原板 `c5edfcc8-00e6-4419-a463-3324207d5335` 的26个可见自有文本已接真实 Shared5 的 body/caption/strong；原隐藏旧动作和所有 linked/generated 文本、guard、ID、source、富文本全文及颜色引用保持。一次原生 `update-file` HTTP200，own rev192→193；三尺寸原板 resize 后恢复1280，最终 **rev199**，保存关闭/新context重开整data SHA `9e67e30b1926c81acbd06e741fb9df7d86adb477a3f97ca28e3b632b6fd0369e` 一致，其他四页完整hash不变。body/strong 13.333333、caption 10.666667，Roboto；实际token源 `dc23135b…` 与发布metadata匹配。没有给 inherited linked guard 强套字体样式。

8原图逐张复核。首张更新后1280绘制未与最终帧一致，保留为失败证据；恢复/重开/第二次重开两帧的同一画布crop均0设备像素差，正确前后改变17595设备像素。并排/差异只比较实际Penpot，不能充当EXE或1逻辑像素验收。26自有文字框在1280×800、900×620、640×520父框内；900树动作挤入详情边缘、640分类label框仍超出52.00000482472478，整窗未通过。Roboto400/700已在浏览器加载，CJK实际glyph/fallback及引擎字体未证明。

r79实读源：公共TS已有保留全文的guarded reflow及leftright/right约束，而现原linked copy仍保留left固定几何。当前Project主板仅 `devZirconSettingsPlugins/domain-contract`，没有原可逆ZUI asset/document；未实际调用worker/export、不重签未知generated形状，不加宽UE20%drawer掩盖问题。最小同源导航variant、展开/选择/命中、codec和合法refresh合同按当前9源pin交Parent，旧r73 proposal保持未应用。r78当前14 Rust仍14/14一致；本轮无Rust源码及执行，原28静态定义不升级。

证据根：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-project-typography-r80/`（summary、handoff、prepared、mutation-intent、saved/reopened、stable-capture-receipt、visual-review、8原图及并排/差异）；`.../settings-tree-label-flow-r79/source-entry-decision.json`。0Cargo/intake/QueryOnly/source capture/freeze/服务/桌面/DPI/生成；两个自有browser/context退出0。保留必要证据/helpers、释放0B、cleanup未关闭。正式ZUI→资源加载→正常EXE/实际150%DPI/两族A-B、provider/typed subtree/origin与原11要求保持open，goal active，passed=false/accepted=false。

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

## 2026-10-25 r73：原持久化重试的真实 caller 与待写文档条件核对

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

## 2026-10-25 r71/r72：真实持久化健康刷新与本族 linked TreeRow 自适应

本轮直接接续 active goal，没有等待公共 Field/Toolkit/正式导出完成。实际变更仅限 Parent 原 14 路径中的 3 份 Rust；原 r64/r66/r70 caller、原 coordinator/authority 及全部历史候选和失败保留。

- **r71 实作已落 canonical**：现有正常 host tick → `sync_settings_projections` → 打开的设置窗口，独立读取原 `SettingsPersistenceHealthProjection` 并刷新健康提示。原通知链会在服务 revision 不变时提前返回，订阅器只对失败发布 toast，因此不能拿通知变化代替每次持久化终态。新路径在目录/值均 current 的提前返回之前执行。
- Bridge 仅新增一个“待物化”布尔位：在首个健康属性写入前设置，原 refresh 全部成功后清除，错误保留给后续 tick 重试；打开/目录完整刷新/关闭成功后清除。没有第二 settings store、clock、backend、项目/文件 generation 或事件 tuple。健康更新不会替换原显示值/项目绑定上下文，不关闭值编辑弹层、不重置滚动。
- 新增 **1 个尚未执行**的行为回归：真实原 authority/coordinator/persistence service 在受限 lane 中产生 PendingAdmission，重试保留原 file generation、仅改变 health generation；原 bridge 消费已准备状态，重复无变化不刷新，值上下文/滚动/Enum 弹层保留，关闭后忽略健康更新。测试使用已有 owned fixture 作为测试资源，未把 fixture 当产品 provider。本轮 Rust 执行 0；r64–r70 的 19 个 guarded 定义及原 mutation17/details11、旧 Jenkins 请求各自仍未通过。
- 3 源 rustfmt、diff whitespace、增量 patch 反向检查通过；change-guard 静态 digest `330316e69bc68b574b130c8f5fe28e30693641a3f1734fa22ab1fa2b8b1c641a`，errors/warnings/suppressed 均 0。它是相对原 HEAD 的 3 源 guard；本轮增量 patch SHA `dd26cc0444adf0651c841c28e6729a63ac464d60f91a81c3858449d4b2fa24f1` 已落源，不能重复 apply。未执行 Cargo/intake/QueryOnly/source capture/freeze。
- **r72 真实 Penpot**：文件 `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` 从 rev116 到 **rev126**；Plugin Browser page `c6a45a2a-60ca-41fb-a1c4-c5164da3cad3` / board `9d912f75-c86c-4b3a-878d-1007947cbf81`。仅通过原生 Inspector 把 4 个原 TreeRow linked instance 设置为横向 fill，实际保存 `touched=[layout-item-h-sizing]`；不 detach、不复制 main、不写 Shared。UE `SPluginBrowser.cpp` 原 `.35f` 分栏保留。1280/900/640 三板侧栏为 448/315/224，列表 832/585/416，grid 右溢出均 0，修复旧 r70 的约 45.714px 窄板溢出。
- 保存、关闭、全新 context 重开，完整 data hash 一致。5 页/729 逐页对象、全部旧 ID/metadata/层级/4 其它页保持；35 Shared copies、428 Shared+32 Card shapeRef 全部对上实际源。当前独立观察 Shared **rev5 / 4 masters / 34 colors / 3 typographies**；库更新 banner 仍可见，没有全文件更新或声称全部实例已同步新库。源主件/颜色/排版未改。
- **逐图复核仍未通过**：6 张实际 Penpot 原图已逐张看过。640 下固定的通用 TreeRow 锁图标仍有裁切，插件分类不适用的 eye/lock 行为/slot需与公共 owner 对齐；卡片仍是 metadata 内容草稿，真实 checkbox/启停与 restart/dialog/focus、正规 repeated prototype/单滚动/业务 producer 尚未证明。Canvas 71/98/134% 是缩放，不是系统 DPI。旧 r53/r70 等失败图与诊断均保留。
- 原生事件仍只有 control/action/value 等 tuple，缺 originating frame 与 Preferences open/window lifetime；最新 bridge 上下文不能证明关闭重开前的排队事件。`shared-origin-contract.json` 记录 9 份真实共享输入路径 preimage 与最小 Parent 合同：在原 published frame 的 focus/hit 产生时携带原 source window、business cursor、typed 显示上下文及 owner 管理的 open-lifetime 身份；原 tuple 不变，消费时校验，关闭/源替换撤销。没有新建 lease/clock，也没有抢写这些共享路径。
- 原 retry scope 字符串还没有“显示中的 document/file-generation ticket”，本片健康提示不是 retry authority。String 生产 provider、Numeric 文本 caller、四 page scope、typed subtree、World/Dock/undo/save、正式导出→资产→EXE、真实 150% DPI、双族主题 A/B 和逐图原生证据继续 open；不以这些共享缺件整族停工。

回执：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/persistence-health-refresh-r71/`（source-change/audit/checks/validation-handoff/shared-origin-contract），`.../linked-row-responsive-r72/`（baseline、原生覆盖/尺寸/保存重开、6 原图及 visual-review）。本轮所有隔离 browser/context 正常关闭、退出 0；临时浏览器目录 0 文件。无服务/桌面/系统 DPI/EXE/生成/源码副本/授权库/缓存清理操作，实际释放 **0 B**。`passed=false`、`accepted=false`、goal 保持 **active**。


## r70 当前：监督恢复后的真实设计与操作接线

goal 已实际 active，按用户监督恢复继续并行，不因等待公共完整交付再 blocked。自有Penpot实际rev116/5页729对象，原Plugin Browser板与Card身份、35 Shared copies/460 shapeRef、metadata和其他四页保持；复用Shared5既有颜色，完成保存关闭与新上下文重开。21张原图逐图复核；900比例布局和窄窗工具栏换行有实证，640仍有原生网格最小内容宽度/右边裁切，百分比实验仅本项恢复，所有失败保留，不报视觉通过。

三个原域Rust源已把Bool/数值步进/枚举/颜色步进/Chord提交接入保存的显示上下文，沿原coordinator来源/项目绑定与authority同锁u64代际条件写入；原String/reset及scope/persistence/事件保持。新增2定义，guarded静态19，Rust未执行；三源format/diff/增量reverse最终exit0，guard0错误/警告/抑制，其他11授权源pin保持。LF/CRLF初败保留。四页scope、排队旧窗口事件、typed subtree/Numeric文本caller/真实providers、正式导出/normal EXE/实际DPI/A-B及原11要求仍open；passed/accepted=false。未Cargo/intake/identity/source capture/freeze/服务/桌面/scaffold，释放0B。

[本轮实际产物、差异及精确源指纹](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/resumed-design-actions-r70/summary.md)。下方全部原历史保持。


## r66 当前：String/reset 显示上下文核对原 coordinator 来源

原已授两源内补齐条件上下文的生产者身份：原唯一状态 Mutex 用 Arc 持有，显示上下文 Weak 指向同一原 state；在原 operation guard 内先核对 owner，再比项目计数/值代际。不同 coordinator 的相同局部计数不再互相接受；不新增 authority/store/UUID/代际或事件 lease。原持久化/no-op/retry/phase 清理函数体保留，host/bridge/authority/共享资产未改。

新增共享 authority 和相同项目计数/已释放 owner 两项定义，当前 guarded 静态17，原15名称与 core旧测试全文保留；host原17 scope模块仍匹配。未执行Rust。两源格式/guard/diff/增量reverse均exit0，0 errors/warnings/suppressed。旧 r64/r65 源 pin 与报告保留历史，当前指纹在新报告。

事件缺 originating frame/window incarnation/lease，关闭重开前排队事件仍未解决；String产品 provider/其他caller/四页scope/typed tree/producer/export/EXE/DPI/A-B/全族继续open。goal实际active，未自行改状态；无Cargo/intake/新身份/source capture/freeze/服务/桌面/Penpot/生成/清理，释放0B，passed/accepted=false。

[本轮实际源增量和边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/guarded-preferences-caller-r64/followup-r66-summary.md)


## r65 当前：修复 String 回归夹具与补齐待重试/并发写入覆盖

只改原授权范围的两个测试所在文件，r64 生产代码未改。静态审阅发现上轮新 String 夹具从空 registry 构造 authority，真实 snapshot 要求内置 slots，会在断言前 panic；现继承原默认 registry。新通知 probe 改 Weak，避免测试自身持有环。新增 pending admission/retry identity 与条件 set/clear 并发直接写入两项定义，当前 guarded 静态定义 15，原 17 scope 模块保留；均未执行 Rust。

首轮 formatter 1224 文件映射占用及一次有界成功重试均保留；最终两源格式/guard/diff/增量 reverse 检查通过，0 errors/warnings/suppressed。其余12授权源 pin 匹配 r64，原13源整体验证保持历史。goal 服务本轮真实返回 active，原 blocked 记录不删。

无 Cargo/新 intake/identity/source capture/freeze/服务/桌面/Penpot/清理；释放0B，passed/accepted=false。来源帧/窗口 incarnation、String 产品 provider、四页 scope、其它 caller/typed subtree/正式export/normal EXE/实际DPI与全族验收仍 open。

[本轮具体修复和当前指纹](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/guarded-preferences-caller-r64/followup-r65-summary.md)


## r64 当前：Preferences String/reset 已接原 authority/coordinator 的条件生产回调

Parent guarded-caller 14 路径移交已实施，实际 13 Rust 源变更。原打开/分类/刷新/receipt 捕获完整 u64 和原项目绑定上下文，真实 bridge 保存；String commit/reset 消费此上下文。值代际比较与 set/clear 同原 authority 锁，项目绑定比较在原 coordinator 操作阶段，沿原持久化/receipt/错误释放。raw getter、原 17 scope 测试模块和原行为保留。

新增 13 个回归定义未执行；格式、13 源 guard、diff/增量反向检查通过，0 errors/warnings/suppressed。未 Cargo/新 intake/identity/source capture/freeze/服务/桌面/Penpot；0 B 释放，passed/accepted=false。旧 r35/r37 pins 已不能用于这次新增变更，原稿不改。

缺 originating frame/window incarnation/lease，不能证明关闭重开前排队事件拒绝；当前内置 catalog 没有 String 生产字段。其它控件/四页 scope/typed subtree/normal producer/export/EXE/DPI/全族仍 open；r63 17/19、r53、r51 与旧失败不关闭。goal 服务实际仍 blocked，未声称服务已恢复。完整源指纹、增量和静态证据：

[本片交付记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/guarded-preferences-caller-r64/summary.md)


## 当前接续 r63：真实 Button 颜色引用覆盖与原生重置

2026-10-24T20:51Z 独立 API：本族 rev69 / 5页 / 729对象 / 35个 Shared native copies / 460个有效源引用（428 Shared、32本地 Card），Shared rev5。新增 copy `4d7f131f-5bed-803b-8008-d72e9bbaae38` 与子文字 `4d7f131f-5bed-803b-8008-d72e9bbaae39`，原受控页/板不变。实际 Button 来源为2节点：component `6ddc248b-bd35-80c5-8008-d6c8c01096f3`、main `6ddc248b-bd35-80c5-8008-d6c8bd9b057f`。原生 Assets factory 直接继承公共 owner rev5 发布的 `editor.surface.1` 颜色引用 `599586e3-022c-500b-ac80-5d9c490139af` / `#242424`。

已实际保存默认 rev67 → 原生局部 `#ffb800` 覆盖 rev68 → 关闭、新上下文重开 → 原生菜单“重置覆盖项” rev69 → 再关闭、新上下文重开。覆盖阶段去除 ref 并产生 fill-group，重置恢复原 Shared ref pair 并清除 fill-group；两次 fresh reopen 完整 data hash 各与保存阶段相同，ID/shapeRef 不变，不 detach、不手动补 refs、不点击整库更新。仅这一个新 Penpot 实例的颜色继承、覆盖、原生组件重置与持久化有实证；Engine scope reset 与跨两族全局主题 A/B 仍 open。

严格只读 audit 原件17/19，保留两项不全等：原生插入首位导致验证板67个旧节点重排/缓存变化，14份 SVG path 的绝对 M/L/C 坐标为 x+0/y+44，8份旧文字 content 保留；新文字 reset 仅 positionData 不等。旧 IDs/metadata/refs/fills/strokes 保留，四业务页及非页面数据完整哈希不变；仅内存恢复原 trial objects 后回到 rev66 完整 data hash，不代表整板几何安全。默认与即时 reset PNG 字节相同仍不证明行框≤1逻辑像素。七原图逐图复核，中文长标签裁切、required value 空槽与 r53 原失败继续保留。首次双 SVG 同 ID 定位失败为0 RPC/未构造，attempt02 只修实际外层板框选择。

明细：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/button-override-reset-r63/summary.md` 及同目录 audit/geometry-content-audit/visual-review。当前 data SHA `3bde8805d34541611944675c26564bea18874c7e3565f1c2d2c089a59d8e513f`；所选 source 合同 SHA `c9ed9455acbcdfe2c230cfe3d42961b2afb02c9c4f436c7113b2ab0b50f08790`。官方 referenced-paint exporter、Property 新槽、动态语义与正常 Settings producer 沿原 owner/Parent 边界，不伪造 source export 或 EXE。SP-C6 current-review-r02 未提交准备链沿原 pins；无新 request/operation/attempt、Cargo/QueryOnly/source freeze、服务/桌面/DPI改动。family passed/accepted=false，实际释放0B。


r62 实际接入：[World Settings 原生搜索实例、保存重开及差异证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/world-search-linked-r62/summary.md)。沿原World页a705d2f3…／board cc3d2d68…／flex搜索容器7fe1efc9…，用真实Assets工厂新增Shared SearchInput原生linked copy a4938591-07f1-8024-8008-d727e651b296，7个source refs全部对应原component6ddc248b…c3ce892e／main…c0f9c196；没有detach、复制main或手填refs。原生DraftJS覆盖Search World Settings，原纯文字Search的ID/content/metadata保留并隐藏退出flow；正常Inspector横向column-gap 0→1→0刷新后，真实文件rev61→66，关闭及新上下文重开完整data hash一致。现5页727对象、34外部copies、458refs（426 Shared／32 Card），原720个页内对象身份、其余四页和全部非页面data保持。

公共来源分时记录：构造时Shared2，恢复时实际Shared4，最后独立只读Shared5（41objects／4masters／34colors／3typo）；Shared5 Button main实际已有1处非空颜色引用，不能再把旧master有效引用0写作当前总体现状。未点击整库更新，官方get-file-libraries实际current rev5与syncedAt17:52:24并存，不当成消费者已同步5。Search canonical metadata保持；[source桥接提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/world-search-linked-r62/public-search-stretch-proposal.json)精确标出：实例根fill1280，内部surface仍240.0000095、label204.0000032且left约束，原.zui SearchField是Stretch、UE Details搜索是FillWidth(1.0f)。需原公共owner修正既有投影/master伸展与export合同；域未手动拉宽内部形状、改公共源或造新Field。

失败与差异保留：原disabled row-gap超时、Shared4≠2先停、column-gap无values属性的count guard先停，随后只读Inspector/source核对定位正确enabled原生输入；get-file-libraries错用id的400及改file-id后200均留回执。World原六节点明确改变：四个原本无positionData的文本生成缓存，隐藏旧Search几何变化，body按原flow补40高度、region加入child；四原文字content与metadata全保留。其他四页和非页面data精确保持，不扩大成全页geometry不变或≤1px通过。六张原图均直接逐图查看，r53原整页同步失败保持open。

当前仅结构设计接入：World provider/filter/clear/focus/undo/save、正式normal Settings producer与源导出/资源闭包、公共slot/真实field、Rust/EXE/实际150%DPI/双端及两族A/B仍open。SP-C6沿已消费current-review-r02及原未提交prepared，不新造候选/request/operation/attempt、不重放reset。本轮0源码/.zui/服务/桌面/Cargo/generator，passed=false、accepted=false，释放0 B；下方全部历史原文保留。


SP-C6 当前准备链已消费 Jenkins 原 owner 的 [current-review-r02](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/settings-plugins-sp-c6-20261024/contract.current-review-r02.json)，本域[只读核对记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/sp-c6-receiver-binding-r58/summary.current-review-r02.md)。九份Rust SHA与Parent followup04 after及原19:09准备链全同，测试f7c6eb14…和4ef0bd40… guard原已纳入prepared，没有新源码候选／request／operation／attempt；四个名称仍只是静态定义，不是harness结果。原r27/r35/r37三输入字节保留。

原owner 19:53实际公开入口ready，capsule为0b594015…，同fc553228…／start-1792836412672596900，11代码／197 driver核对及纯内存配方证据沿原链复用；本域实际selector／entry／runner／capsule四磁盘pin全匹配，没有重复HTTP或QueryOnly。旧API/Home mismatch仅作历史失败，不再报当前阻塞。Parent note当前已不同旧十路径guard时的SHA，故复用相同九Rust的格式／静态事实，不称新十路径guard或完整capture-ready。

剩余真实source/member／writer／资源预约／物理E target／pool/cache锁／native guard与EOF／正式收件和Rust回执仍缺，captureReady=false、runnableNow=false。既有最小package_metadata Editor lib/test/defaults-off/features[]/locked/jobs1/thread1 scope未变；本轮只消费记录和改自有说明，未执行wrapper／Cargo／source capture／freeze／新提交／服务／Penpot／桌面／清cache，不把入口ready当产品通过。正式插件／UI／normal EXE／实际DPI及族passed/accepted继续false/open，释放0 B。下方r58与旧receiver描述保留为各自时点的历史。


r60/r61 实改：[World与四配置弹窗共享色](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/world-dialog-shared-color-r60/summary.md)及[New Plugin固定目录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/new-plugin-fixed-root-r61/summary.md)。真实自有file仍b4a1e3ce-9e6a-81fe-8008-d69d60e4e402，已rev57→58→61／5页720对象；Shared rev2、33外部copies、451 refs保持，未改公共文件。r60五个原board与47个自有正文同步Shared既有填充及缓存；15张before/after/fresh重开原图逐张看过，逆转52处填充可精确重建rev57全data，字体／几何／metadata与其他三页不变。

r61用真实Penpot原生Layers＋DraftJS正文编辑，三次保存HTTP200：原Target改Plugin directory，Engine checkout改固定<project>/zircon_plugins，并说明当前没有可执行项目级计划与恢复API。三原text IDs、字体／颜色引用和metadata保留，实际浏览器重新测量行框；仅三个文字节点的content／positionData变化即可精确重建rev58全data，关闭重开data保持。原生新增三个paragraph key明确记录，非业务field key；初次严格样式check因此exit1和四次未输入的UI守卫停止均保留。11原图（含两个同字节重复图）逐项复核，无源码／guard重签或人造宽度。

这是设计整改：World仍空provider／文字搜索；对话框缺真实modal／focus／引擎事务，ID/kind等仍文字。New Plugin维持kind必选／native=false及Create不可用；六共享路径不是完整plan／write-set，未generator。正式源owner导出／真实Field与PropertyRow／normal Settings producer、Jenkins源码与资源准入／Rust、正常EXE／actual150%DPI／两族A/B／逐图双端比较仍open。r53三业务页保全失败不由本次审计关闭。没有公共Rust／ZUI／产品包／服务／桌面变更，passed=false、accepted=false，释放0 B。


r59 实改：[两个设置业务板共享色修正与原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/domain-shared-color-r59/summary.md)。真实自有Penpot rev54→57，5页720对象不变；Preferences/Project两个原board与82个自有正文节点使用Shared2既有背景/文字色引用。首次正文仍黑的原图保留；源码定位SVG读取positionData填充缓存，单标题探针可见生效后同步相同颜色缓存，已保存/关闭/fresh重开并逐张看11原图。仅逆转本次预期填充可在内存精确重建rev54完整data：字体/文本/几何/metadata不变，其他三页、33 copies与451 refs保留；不关闭r53历史保全失败。

仍是设计整改，值为源schema样例；真实Field/PropertyRow/value provider、正式原owner导出、正常EXE/实际150%DPI/双端比较均open。未改公共source/theme/store/产品包，未新发Jenkins或执行Cargo/服务/桌面；passed=false、accepted=false，释放0 B。

r56 新结论：[r46 共享 consumer 来源对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-consumer-reconciliation-r56/summary.md) 已将两处旧依赖逐项对应 Parent SP-C6 before/after。host_data 当前 585c706d…、payload module_plugins 当前 9b9455ae… 精确匹配 Parent 第四次接续已落源结果；九份 metadata after 均匹配。内存各移除一行 package_metadata 可精确重建 r46 旧 SHA，非换行差异；未应用、回滚或修改共享源。details ea903d46… 与另外12份 Rust 仍匹配；17 mutation / 11 details 未执行。

r35/r37 原 prepared 与 r46 review-only 四份字节保留。旧 consumer pin 继续作为历史输入，不自动代填新闭包或提交草稿；实际源准备、外部控制计划、资源/writer、server 收件、Rust、插件、正常 EXE 和实际 DPI 回执仍 open。本次不查询 pending、不换身份、不运行 Cargo、不触 Penpot/服务/桌面；passed=false、accepted=false，释放0 B。

r55 新交付：[最小动态 Settings 语义子树与 provider 映射](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-semantic-subtree-r55/semantic-subtree-design.md) 已完成实际源码梳理，[精确合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-semantic-subtree-r55/semantic-subtree-contract.json) 记录 54 源指纹、16 符号定位、11 现有组件与 10 schema。原 preferences.zui 仍只有 preferences／WorkbenchPreferences 一个 root；未将其称作完整可编辑树。真实 typed projection／batch 有 schema 与同代次三层 raw override，现有 root UI 表转换会丢失这些信息，故接入点要在 open/refresh/select/receipt 路径 typed 转表之前。

需要 Parent／原 owner 接线的范围明确为：同一私有 bridge 的 typed capture/call、混合 Row source prototype/slot、真实页面 scope/document 与 commit currentness、scope 目标层显示/default catalog、原 typed numeric setter 的真实 surface callback。schema 没有通用 units／precision／large_step；plugin_pages 仅 metadata；复杂类型与完整 Input Bindings provider 未完成。旧 string/chord 等入口按 definition.scope，coordinator.set/clear 没有 expected_generation，不能称页面 scope 或 CAS 已贯穿。原 r42 wrapper 不重放，原 mutation/receipt 保留。

行／文字几何目前仍来自手工 SettingsWindowLayout 与 root painter；新逐字段证据须接同一真实 UiSurfaceFrame 的 arranged node、render command/text line、hit/focus、window metrics 及源身份，Parent 负责测量/painter/Settings native case，原 public owner 负责 slots/codec/ACK/export。只读再次确认 source PropertyRow 5ab622…／r392 staged bf29… 不一致；未改 F 盘产品、LevelEditor guard、nonce、公共 Rust、窗口、主题、store 或服务。 own rev54／5页720对象、Shared2／4masters／34colors／3typo 仍真实存在；本轮未改 Penpot，r54九图和 r53 whole-page hash 失败均保留。

本阶段只有文档／能力接入合同，无 Rust／正式插件测试或导出、无 managed resource prepare、正常 EXE／actual150%DPI／双端验收，passed=false、accepted=false，释放0 B。

r54 当前核实：真实文件仍为 **rev54／5 页／720 对象**，Shared 为 **rev2／4 masters／34 colors／3 typographies**。在同五页及四个弹窗板完成 9 张新原图；修正原错误 aria-selected 断言后，9 个实际 SVG board ID／可见边界及原生 Layers 的 aria-checked 均核对通过。新图与逐张看过的原图 SHA 相同。捕获前后五页、components／styles 与 Shared 全数据完全不变；451 个 shapeRef（419 Shared、32 本域 Card）仍可追溯。这是隔离浏览器设计证据，不是 EXE／DPI／交互验收。白底、模拟 fields、固定按钮、空 required value 和长中文裁切继续未通过，见 [逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/visual-review.json)。

已提交可审阅的 [PluginMetadataCard 最小源桥接提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/source-bridge-proposal.md) 和 [精确源合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/source-bridge-contract.json)：原 host 的真实调用点／virtual_rows prototype，消费同份 typed payload 的 metadata 与原六组动作，复用原 UiSurface 池，在现有有界 pane cache 保留 surface，以完整 plugin_id＋实际 assignment binding 区分重复项，唯一 list scroll，真实文字测量后配置 extent。拟议共享可选承载先由 Parent 核对；未改 host／catalog／运行时／DTO／操作 tuple，也未把容器根伪装成 Card。

公共 catalog 已由其他 owner 从起始 4c2f849c… 继续变为 **e4bba09c3ba16d55b90eabb5a821ab3e76eb0606f9afc17e9f656e2d16fe0a99**。68 组件及 Field／NumberField／PropertyEditorRow 三个描述符逐字段不变，20 项提案输入当前 19 项匹配，不能称完整 source-ready。NumberField 10 props／PropertyRow 4 slots 原源能力存在；Shared 仍未发布 Field／Number 原生 master，不另画假输入。i64 用原 Field 精确文本提交；没有 schema 策略时不以模板 42／0..100／precision0 当真实值。另核实 Shared master 内 21 个 style ref ID **实际全为 null**：字段存在不证明绑定，发布的 34／3 样式仍真实存在。

r53 同步造成三业务页整页 hash 变化的失败保持 open，本轮精确不变不关闭旧失败。SP-r51 原 managed 请求、17 mutation／11 details 的未执行事实及资源／source 准入边界保留。本轮未启停／延长服务，未占桌面，未跑 Cargo／正式插件 suite／generator，释放 0 B；family passed=false、accepted=false。

r53 当前状态：真实文件已到 **rev54／5页／720对象**，33个外部原生实例和419个源引用保持可核对。Shared实际为 **rev2／4主件／34颜色／3文字样式**，不再沿用旧空样式结论。仅在原受控验证板接入`editor.surface.0`和默认Button的`editor.surface.1`，完成原生颜色选择、局部填充覆盖、重选共享样式及关闭/重开；这不是全局样式修改、组件Reset或产品生效证明。

原生更新本文件的库同步后，Editor Preferences、Project Settings、Plugin Browser整页哈希发生变化，**完整保全检查失败，继续open**。受控页差异包含SVG坐标重算和文字测量缓存变化；25个既有业务标签、18个隐藏旧动作及419个源引用/源metadata仍逐项有效，但其他业务页没有完整节点前像，不能断言只变了派生数据。原失败及所有9张原图保留：[差异审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-style-linked-r53/library-sync-delta-audit.json)、[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-style-linked-r53/visual-review.json)。

34颜色已对上现行`editor_tokens.zui`及alpha；3文字样式的`gfont-roboto`与现有源`custom-495ff285-7c71-49bc-bc4e-df20017e2d66`字节/度量未证明，未套用文字样式。颜色引用的正式插件处理、库同步变化审阅、canonical源导出和正常EXE继续交公共owner。没有新增Rust/ZUI、Cargo、桌面或服务操作；17 mutation／11 details仅静态定义、未执行，family passed=false、accepted=false。以下r52及更早内容保留各自历史事实。

r52 当前设计已实际保存至本族 Penpot rev49：5页、720对象，累计33个真实 Shared linked copies／419个带源引用节点。此次仅在原 Plugin Browser 页的四个配置弹窗草稿中接入4个 Button 实例（8节点）；其 source file/component/library rev1/shapeRef 和受支持的文字覆盖均由实际 API 与两次全新浏览器 context 重开核对。84个原弹窗节点的ID、语义、内容、metadata和父级保持；4个旧纯文字动作保留并隐藏。另一个本轮误加的 Directories Close 实例及其文字子节点已用原生删除移除，未删除原节点。其他四页、原 PluginMetadataCard 和 Shared 源指纹不变。见 [保存与重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/dialog-linked-r52/save-reopen.json) 和 [逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/dialog-linked-r52/visual-review.json)。

此次重新核对实际 UE `SPluginTile.cpp`、`SNewPluginWizard.cpp`、`SPluginPaths.cpp` 与 `Dialogs.cpp`，将原草稿动作修正为 Beta **Yes（左，目标 primary）／No（右）**、按钮组右对齐；New Plugin 为右对齐 **Create Plugin**；Directories 正文不含 Close，关闭由 Parent 窗口/Dock提供。Create 的0.5 opacity仅表达不可执行设计状态，不是公共 disabled 变体或真实禁用事件；完整只读plan/恢复authority仍缺，不执行生成。Properties 的只读metadata及 Close 是当前能力适配，UE可写元数据事务仍open。

15张真实 Penpot 浏览器原图逐张保留，含旧动作/错误顺序和 inspector 尚显示上一板尺寸的历史图；最终目录图等到原生 inspector 为750×600后重捕。当前只完成源引用、设计状态和保存持久化核对；中性色正文、固定240×32按钮、primary/disabled/AutoWidth、Dialog宿主、共享样式/Field/value、实际事件/焦点及正式源导出/正常EXE均未完成。未运行Rust/Cargo、服务操作、桌面输入或系统DPI；三个自有浏览器均已退出，临时目录0文件，释放0 B。**family passed=false、accepted=false**，不把此轮结构草稿当成UE视觉或产品验收。

以下r51/r50/r49及更早记录保留各自时点和失败；其中rev35/712、29 copies/411节点是旧观察，不是当前设计状态。

r51已核对原Jenkins owner的r45只读对账及两份prepared原字节；未再次QueryOnly或重投。本域完成当前r46详情候选审阅，给出同业务链的精确字段提案：r35只更新details依赖哈希、r37只更新details源哈希为 `ea903d46…`；原prepared `a2f79cba…` / `5f59d811…` 保留且各16/17匹配。原17 mutation / 11 details均未执行。此 [源owner审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-source-review-r51/candidate-revision-review.json) 不是新payload、正式收件或完整编译源闭包。原external_control_forbidden、完整external Gitlink/控制/编译源准备及实际资源writer仍open；按owner 15:49:42Z观察，E资源至少缺19.02GiB才到35GiB保留线，尚未计recipe。无需用户手工grant，不改policy/原身份/在途请求或清理他人产物；immediatelyExecutableScope=null。本轮只做候选审阅和自有记录，0 Cargo/0服务/0桌面/0清理，释放0 B；passed=false、accepted=false。

当前接续为 **r49 / Preferences与Project的原生导航和搜索**：真实文件现为rev35、5页/712对象。在原Preferences板接入SearchInput×1/TreeRow×9，原Project板接入SearchInput×1/TreeRow×3；14个根/254个节点均沿Shared rev1的真实原组件。标签覆盖和14个旧文字ID/content保留隐藏，原User/Project scope与全部schema文字未改。原生间距检查器将两个分类容器临时1再还原原0，刷新真实flow，修复首次重开留下的空隙；修复后两次fresh重开检查两页引用、标签及连续排布。World/Plugin Browser/受控试接页和Shared指纹不变。当前全文件共29个外链根/411个复制节点，原PluginMetadataCard身份保留。

r49实际观测到页面保存走`/api/main/methods/update-file`，修正旧RPC-only诊断监听后捕获5份原生HTTP200；先前Preferences创建及r48缺失的逐请求记录仍缺，不重放或补记成功。六张真实浏览器原图已逐一打开（含两张首次空隙失败图），25项源身份/保存值/flow检查成立，仅为设计证据。原body仍是schema fixture，搜索/分类动作、PropertyRow/value、scope/currentness、全局样式、正式ZUI/EXE/实际DPI/双端逐图均未通过；passed=false、accepted=false。本轮0 Rust源码修改/0 Cargo/0生成/0服务操作/0桌面输入，所有自有浏览器退出，释放0 B。证据在 [r49保存重开记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-linked-r49/save-reopen.json) 及 [逐图差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-linked-r49/visual-review.json)。

r50只读卡片接入审计再次实际API读到rev35/712对象，Plugin Browser整页及原业务组件未变。Parent followup-04仍为SHA `1a8c91fd…`，9份源after hash全部匹配，当前guard为`4ef0bd40…`，较早消息中的`32ce94d8…`不覆盖现回执。数据链已具备；具体UI阻塞是原PluginMetadataCard的sourceWrapper=null/officialCodecMetadata=false且无canonical-record，原view的`plugin_list_slot`只有动态Container，没有相应源卡片原型。原节点到既有payload字段的精确消费映射及最小source/测高/producer合同在 [r50记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/card-source-eligibility-r50/eligibility.json)，未导出、写共享源、伪造绑定或运行Rust。

以下r48及更早状态按各自时点保留，不能覆盖上述当前状态。

r48历史接续为 **Plugin Browser 业务板共享实例**：真实文件已保存到 rev27，5页/458对象；原 Plugin Browser page `c6a45a2a-60ca-41fb-a1c4-c5164da3cad3`、board `9d912f75-c86c-4b3a-878d-1007947cbf81` 内新增7个原生外链实例（Button×2、SearchInput×1、TreeRow×4）。7份标签覆盖在关闭原context、两次fresh重开后保留，91个新节点的shapeRef及源metadata核对有效；原5个文字占位ID保留并隐藏，其他4页与Shared对象指纹不变。仍是设计草稿，未导出正式ZUI或连接产品动作。25项源身份/保存值结构核对成立；原26项总检查为false，native mutation监听0条的证据缺口保留，不改记为传输成功。逐图差异和真实预览在 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-browser-linked-r48`。

r47已消费 Parent followup-04 的9份共享Rust接线，当前after SHA全部匹配；原optional metadata已由status→ViewData→payload→TOML贯通。自有8路径未应用提案被并发Parent实现取代，apply-check失败和Parent队列100条失败保留，禁止再应用。Parent已补原host pane测试夹具的None，3个新metadata回归尚未执行；当前Parent guard digest为 `4ef0bd4010bf47a26037490aa347b8109f4cea86b1daf36cb82107a139517919`，只复用对应源静态证据。域mutation17/details11及r45 pending事实不变，未重投或运行Cargo。下文r43/rev22、r46和旧guard均保留历史，不覆盖本段现状。

**全族passed=false、accepted=false。** 公共TreeRow的分类适用控件、Button/Search流式尺寸、正式MetadataCard/共享样式/导出、scope代次准入、World/undo/restart、正常EXE与全部尺寸/实际150%DPI/双端逐图仍open；本轮没有桌面输入、服务操作或源码副本。

本聊天：`01a16d53-33db-7fb2-8f59-11118c3ec933`。Parent：`01a16d4e-0491-7353-84fa-038bd7706546`。

当前为**源码/具体设计已记录，真实 Penpot 四窗口保留原生流布局草稿，并新增共享组件受控试接页**。Parent 统一协调服务、公共组件/主题和桌面；本族不启动/恢复服务、不占用共享桌面。r43 独立 API 已确认公共库 rev1/isShared=true/四个主件，本族通过原生管理库连接和素材拖放生成8个 linked copies，4项中文/长文本覆盖在关闭及两次 fresh context 重开后保留；当前 own rev22/五页/367对象，原域主件身份不变。colors/typographies仍0，四窗口正式设计/可用表单仍未完成。草稿的16个设置键及五份实际 manifest metadata 是设计 fixture，非运行预览。Parent followup-03 的 r34/r39 已应用；r42 的 raw-scope reset 消费保留，r46又在原详情状态修复报告刷新后的失效焦点。当前17项 mutation/11项详情回归均未编译执行。新 page-scope 宿主接口、metadata UI、公共语义与正式导出/EXE仍待交接；没有产品 ZUI 改写、桌面/DPI捕获或新的 Jenkins 身份。`passed=false`、`accepted=false`。

r46 是实际域内 Rust 增量：报告移除所选feature、使Apply不可操作或改变分页后，焦点按同份报告返回现有features；空列表返回Back。只改原 `module_plugin_projection/details.rs`，保留原选中、缓存、启停/恢复tuple及Parent metadata构造，无公共DTO/注册/ZUI改动。格式/diff/静态guard通过，新增3个测试定义使详情共11个，均未执行；旧prepared inputs引用的details源已旧，完整身份及最小验证合同见本文r46节。

r45 已接续 Parent 核实的 Jenkins 原恢复r2，并对本族原r27身份执行一次正常模块入口 `-QueryOnly`：exit1 / `ordinary_entry_unproven` / `Current session authorization is pending`。这是本族的新实测，未调用Cargo或新收件，已将原输出和当前候选引用交回既有Jenkins owner自动收件对账；不要求用户手工grant、不换ID。preflight/查询时实际installed为 `9c684aef…`，其manifest及5server/6client均匹配，同原driver/runtime；Parent的 `20b3…` 是较早正确观测。当前Rust、正式UI/EXE及全族验收仍open，详见本文r45节。

r44 已消费原 Jenkins owner 的 followup-03 准入审阅：审阅文件 SHA、Parent 十二份 after hash、本域当前 mutation/details、两份 prepared inputs 的来源与依赖均已独立核对。当前仅 review-ready，`formalAdmissionIssued=false`、`formalTerminalIssued=false`、`capture-ready=false`。raw/native/live/mutation/details 分别为3/3/1/17/8个静态 Rust 测试定义，均未执行；原 r27/r40 失败及 r35/r37 候选不改写，不重投或新增身份。详见本文 r44 节及[交接记录](settings-plugins-parent-handoff.md)。

相关 New Plugin、Directories、只读 Properties 和 Beta 确认也已保存域内容草稿，明确真实来源及能力限制。当前共八个域草稿 board；r21 两个独立 context 读取 revn=11，相关文字分组及缓存缺陷经重开图复核修复，历史失败保留。公共 Dialog/输入 master 未到位前，不把文字动作标签视为可执行按钮。源字段、实际 board 身份及失败/修复证据接续在 [设计合同](settings-plugins-design-contract.md) 和 [组件合同](settings-plugins-component-contracts.md)。

## 参考身份与盘点方法

只读参考根：`E:/Git/ZirconEngine/dev/UnrealEngine`，实际 HEAD `75a76ee963792d5a316f738519e966937f770036`。`Engine/Build/Build.version` 实际声明 `6.0.0`、`BranchName=UE5`，不擅自按常见 UE5 截图替换当前源码目标。

本族的顶层入口是 TabManager 注册与 Slate 组合。对 SettingsEditor、EditorSettingsViewer、ProjectSettingsViewer 和 PluginBrowser 的 `.cpp/.h` 搜索未发现 Toolkit / TabFactory 定义，因此不虚构独立 `FAssetEditorToolkit` 或 `FWorkflowTabFactory`。World Settings 由现有 Level Editor TabManager 创建。

以下路径以只读 UE 根为基准。初次 61 个输入的 SHA-256 见本任务 `source-inputs.json`；后续设计新增依据单独记录，不覆盖历史指纹。字段/组件/接线细节见 [具体设计合同](settings-plugins-design-contract.md)。

| 窗口/关联表面 | UE 实际入口及行为依据 | Zircon 适用性与原因 |
| --- | --- | --- |
| Editor Preferences | `Engine/Source/Editor/EditorSettingsViewer/Private/EditorSettingsViewerModule.cpp:48,96`，`EditorSettings` Nomad tab，设置容器 `Editor`；`HandleSpawnSettingsTab` | 适用。已有 User 分层、设置 catalog、真实表单 mutation 和持久化服务；当前通用弹层尚不等于独立 UE 页签。 |
| Project Settings | `Engine/Source/Editor/ProjectSettingsViewer/Private/ProjectSettingsViewerModule.cpp:43,97`，`ProjectSettings` Nomad tab，设置容器 `Project` | 适用。已有 Project 分层及项目绑定/错误来源；需与 User 导航和写入目标明确分开。 |
| All Settings / 分类树 | `Engine/Source/Developer/SettingsEditor/Private/Widgets/SSettingsEditorCategoryTree.cpp`、`SSettingsEditorCategoryItem.cpp:29` | 适用。应保留 All Settings、父组/叶项区别、展开及选中；当前原生设置投影不含 All Settings 专用项。 |
| 搜索、筛选、快捷键检测 | `.../Widgets/SSettingsEditor.cpp:233,348,366,1101`，真实搜索建议、选项菜单、可选按键搜索模式 | 适用。当前 `settings/action_ids.rs` 没有搜索 action；不能用一个未接线输入框充数。 |
| 设置分节头与单属性重置 | `.../Widgets/SSettingsSectionHeader.cpp:129,364,390`；`Engine/Source/Editor/PropertyEditor/Private/UserInterface/PropertyEditor/SResetToDefaultPropertyEditor.cpp:85` | 适用。局部清除 override 已有后端；分节恢复默认、Set as Default、事务/撤销语义仍须分别核对，不可把清除顶层 override 等同清空所有分层。 |
| Import / Export、Import All / Export All | `.../Widgets/SSettingsSectionHeader.cpp:228,284`、`.../Widgets/SSettingsEditor.cpp`；实际 DesktopPlatform 配置文件对话框 | 适用。已有类型化 `SettingsStore`，尚未发现对应用户 action；采用 Zircon 版本化配置格式，不能假装支持 UE `.ini` 反序列化。 |
| 默认配置可写性/检出提示 | `.../Widgets/SSettingsSectionHeader.cpp:344,390` 及 SettingsEditor checkout notice；`Engine/Source/Developer/Settings/Private/SettingsSection.cpp` | 本地不可写/无项目/无效文件反馈适用。UE 专属源码控制 provider 的检出交互待定，当前不得伪造成功。 |
| World Settings | `Engine/Source/Editor/LevelEditor/Private/LevelEditor.cpp:163`；`SLevelEditor.cpp:1279,1695,2183`；`Engine/Source/Editor/DetailCustomizations/Private/WorldSettingsDetails.cpp` | 窗口适用。实际目标是绑定当前 World 的可停靠 Details tab、选择改变后刷新。Zircon 有活动场景与持久化 authority，尚未找到对应 World Settings 文档/注册入口，字段能力逐项待映射。 |
| World Settings 中 UE 特有 GameMode、Lightmass、HLOD 等字段 | `WorldSettingsDetails.cpp`、`Engine/Source/Runtime/Engine/Classes/GameFramework/WorldSettings.h` | 待定；必须逐字段对应真实引擎能力。未实现的 UE 系统不以装饰表单或默认假值完成，窗口本身仍在范围内。 |
| Plugin Browser | `Engine/Plugins/Editor/PluginBrowser/Source/PluginBrowser/Private/PluginBrowserModule.cpp:38,56,384`，`PluginsEditor` tab；`SPluginBrowser.cpp`、`SPluginCategoryTree.cpp`、`SPluginTileList.cpp`、`SPluginTile.cpp` | 适用。已有动态 Module Plugins 投影、durable project selection、native live host；应复用这条真实链路。另一个静态 extension workspace 不算交付。 |
| 插件启停确认、依赖处理、实验/测试标记 | `.../SPluginTile.cpp:558`；禁用依赖询问、启用 beta/experimental 警告、写入失败、pending enable state | 适用。需映射真实 dependency/feature policy；取消必须不写配置。依赖闭包后端存在，不表示已经有 UE 弹窗。 |
| 重启提示 / Restart Now | `.../SPluginBrowser.cpp:323,354`、`PluginBrowserModule.cpp:371,392`；设置的 `ConfigRestartRequired` 元数据由 `SSettingsEditor.cpp:470` 通知 | 适用。区分“期望配置已保存”和“本进程实际加载状态”；当前设置行的 requires_restart 标记不等于完整 pending-restart 生命周期。 |
| Plugin Directories | `PluginBrowserModule.cpp:40,72,429`，`PluginDirectories` Nomad tab，`SPluginPaths.cpp`；四种外部来源、权限/只读与 diff 后更新 | 表面适用。当前真实 native 根为 `<project>/zircon_plugins`，已有 canonical discovery/refresh/generation；任意多搜索根配置及 UI setter 尚未对应，不能制造可编辑来源。 |
| Plugin Properties | `PluginDescriptorEditor.cpp:28,44,85,88`；真实模态、MetadataObject、自定义属性、提交描述符 | 待定。Zircon 部分 `plugin.toml` 明确由 Rust PluginDeclaration 生成，不能照搬 UE 直接写描述符；需核实可编辑源、原子保存与图标导入能力。 |
| New Plugin / Plugin Creator | `PluginBrowserModule.cpp:39,61,412`、`SNewPluginWizard.cpp`，默认 1000×750 | 适用表面、接线待定。已找到真实 `cargo zircon plugin new`，有 importer/system/editor 与 native 选项；当前修改引擎 first-party workspace/catalog/app 六个共享文件，非任意项目级生成，须 Jenkins/Parent 统筹。 |
| 颜色、枚举、按键、路径选择弹层 | PropertyEditor；`Engine/Source/Runtime/AppFramework/Private/Widgets/Colors/SColorPicker.cpp`；设置表单内实际 schema | 适用的通用弹层交给共享组件 owner。当前颜色仅 RGBA 分量步进，不等同 UE 完整颜色选择器；返回焦点必须验证。 |
| Localization Dashboard（OVW-27） | `Engine/Source/Editor/LocalizationDashboard/Private/LocalizationDashboard.cpp:142,205,289`；MajorTab 内 Targets/Documents 两栈、按 target identity 复用文档、配置锁定及实际 commandlet tasks | 配置/本地化入口适用，完整目标作者流程待定。现有 `EditorI18nService`、en/zh-CN 和 UI localized-text 诊断不等于 target/culture/gather/compile/PO 导入导出。详见设计合同第 9 节。 |
| Config Editor（OVW-40） | `Engine/Source/Editor/ConfigEditor/Private/ConfigEditorModule.cpp:38,83`、`SConfigEditor.cpp:36,79`；指定属性的 target-platform/file hierarchy | 分层配置表面适用。复用 SettingsAuthority/MutationCoordinator；当前 resolved projection 不提供所有层原值/缺省状态，平台 INI 层级也无等价合同，不能直接编辑磁盘文件代替。 |
| Device Profiles（OVW-40） | `Engine/Source/Editor/DeviceProfileEditor/Private/DeviceProfileEditorModule.cpp:27`、`SDeviceProfileEditor.cpp:355,386,400`；0.3/0.7 分栏、pin/view-alone、继承/CVars/TextureLOD | 窗口归属本域，业务能力待定。现有 RHI `RenderDeviceProfile` 是设备 generation/limits/budgets 的不可变事实，不是可编辑命名配置/继承文档，不能改名冒充。 |
| Input Bindings / Keyboard Shortcuts（OVW-40） | `Engine/Source/Editor/InputBindingSettingsViewer/Private/InputBindingSettingsViewerModule.cpp:16,38`；`InputBindingEditorModule.cpp:431,524`、`Widgets/SChordEditor.cpp:210,237` | 适用。已有 User keymap delta、实时命令 dispatch 和纯冲突分析；缺专用捕获/提交 UI。Zircon 单 chord/command 与 UE Primary/Secondary 不同，解绑 tombstone 和恢复继承必须分开。 |
| Plugin Reference Viewer / Plugin Audit 独立工具 | `Engine/Plugins/Experimental/PluginReferenceViewer`、`Engine/Plugins/Experimental/PluginAudit` | 不作为本轮设置配置窗口交付；是另行启用的实验依赖图/审计工具，不是 PluginBrowser 注册的配置页。若 Parent 扩展范围再立项。 |
| UE Marketplace / Fab、UE 专属在线帐号及 SDK 下载 | PluginBrowser 的外部跳转与对应 UE 插件 | 不适用为 Zircon 配置后端；不制造可点击但无真实服务的入口。Zircon 插件来源跳转须验证真实 URI 与可用权限。 |

## 当前可复用能力及真实缺口

Parent Actor 的额外 45 项覆盖分工已接收；权威归属为 [parent-window-coverage](parent-window-coverage.md)。本族牵头 OVW-27 的配置/生产入口和 OVW-40 的 Config Editor、Device Profiles、Input Bindings；SP-014～017 继续 open。OVW-35 Struct Viewer 与 OVW-38 String Table 的资源文档/作者事务由 [资源工具 08](08-resource-tools.md) 持有，本族只消费类型选择、本地化引用及文档跳转。OVW-28 Device Manager/部署、OVW-39 Source Control 主体归 Parent，设置只消费已核实的 profile/只读来源状态。新增收件不授予这些 provider 或共享文件的修改权。

逐项 provider/意图/取消/失效/焦点及持久化边界见 [设计合同第 10 节](settings-plugins-design-contract.md#10-额外覆盖的跨域消费合同)。r32 只核查当前源码并更新本域合同，复用原 Penpot 八板身份和 r26 保存/关闭/重开证据；没有新增板、虚构资源/类型、后端、正式导出或验收。特别是反射目录内容 fingerprint 不包含实时 generation，快捷键 resolved map 不代表 User 原始 override，不能据此直接提交。

| Zircon 权威入口 | 已从源码确认 | 尚未证明 |
| --- | --- | --- |
| `zircon_editor/src/core/settings/{authority,registry,mutation,persistence,io}.rs` | User / Project / Session，schema 校验、设置来源、异步持久化、durable health、无效 Project 来源保护、retry；User 根由 `ZIRCON_EDITOR_APPEARANCE_PREFERENCES` 解析，文件 `settings.toml`；Project 为 `.zircon/settings.toml` | UI 导入导出、分节批量重置/默认保存、重启后操作验收。排队或内存变更不能显示“已保存”。 |
| `core/settings/mutation/health.rs`、`ui/settings/persistence_health_projection.rs` | 后端有 Unavailable/Ready/Queued/Durable/PendingAdmission/Terminal，含 scope/binding/file generation；当前 UI 只优先显示 Project/User 的可重试或部分 terminal | 拆窗口后需按 scope 投影 Queued/Durable，避免其他 scope 的错误遮蔽当前窗口状态；这是展示缺口，不是后端没有 durable 状态。 |
| `zircon_editor/src/ui/settings/settings_window_projection/capture.rs`、`ui/host/editor_event_runtime_access/settings_projection.rs` | 从真实 catalog 与有能力的插件贡献生成本地化分类与设置；选中分类读取 subtree 的 resolved values | User/Project 分离、跨分类全文搜索、插件页实际内容。插件页 descriptor 的存在不等于其 body 已可操作。 |
| `zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs` | 初始 setter 写 definition.scope、reset 清除 resolved source；r28/r30 候选改为显式 scope set/reset，六类旧 setter 共用原 catalog/schema/coordinator 校验 | 旧调用签名与默认写层保留，Project reset 不再追随 inherited User；十二项回归未执行，页面 context/generation、禁用状态、undo 与正常 EXE 仍待接线验证。 |
| `zircon_editor/assets/ui/editor/components/workbench/floating/preferences.zui` | 稳定资产 ID、根 `preferences`、control ID `WorkbenchPreferences`；绑定的 catalog/values/generation/health 与局部滚动状态 | 当前根没有 authored 子节点。分类与属性实际由 `ui/retained/contract/paint/settings_window/**` 原生绘制；只编辑根尺寸不能还原内部 UE 表单。 |
| `tools/penpot/apps/zircon-zui-plugin/src/bridge/zui-native-painter-role.ts` | 有明确的 native painter 类型集合与源 prefab 身份解析 | 未注册 settings-window / WorkbenchPreferences 的动态设计映射；本轮必须保留真实来源与可编辑组件身份，不得用展平图片掩盖。 |
| `zircon_editor/src/ui/retained/app/settings_window_actions.rs` | 真实 mutation 后刷新；安装 design tokens，更新 host theme | `DesignTokens` schema 在当前设置 painter 中未见可操作 editor。全局主题设置界面、local override、重置及两族传播待 Parent 协作。 |
| `zircon_editor/assets/ui/editor/host/module_plugins_body.zui`、`ui/retained/app/module_plugin_projection/**`、`ui/retained/app/plugins/**` | 实际发现插件、选择/功能/目标配置、durable 插件启停、live host、开发监听和错误回报 | UE 分类/搜索/卡片、确认弹层、待重启生命周期、完整键盘/焦点与持久化验收。 |
| `zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs`、`ui/host/exports/reports/editor_plugin_status.rs` | 原 manifest 已有 version/category/description/maturity，当前 UI status report 尚未带出；成熟度包含 Beta/Experimental 与 Core/Stable/Externalized/Stub/Deprecated | 可沿现有 report 补接 metadata，不另建 catalog；图标/作者/文档链接无真实字段时不伪造。共享 report 变更先交 Parent SP-C6。 |
| `ui/host/exports/reports/editor_plugin_enable_report.rs`、`ui/retained/app/plugins/host_actions.rs` | 已有完整 phase/durable/applied/recovery、operation ID、revision、admission 及 retry/revert/recover/open 路由 | 需 UI 类型化本地化展示和正常产品验证。`RestartRequired` 还用于“变更已回退、可重开”，不能一律画成已保存待重启。 |
| `zircon_editor/src/core/plugin/mod.rs`、`zircon_runtime/src/plugin/loader/discover.rs` | native 根 `<project>/zircon_plugins`；已准备 root 的非阻塞 refresh、last-good snapshot、generation | 多根列表/优先级/配置持久化及 editor action 未对应；交互回调不可同步扫盘。 |
| `tools/cargo/src/plugin/scaffold/{mod,templates}.rs`、`tools/cargo/src/main.rs` | 现存代码含包生成、ID 校验/拒绝覆盖、manifest 同步及六共享文件接线；r41 核实 static-manifest 标记缺失，当前结构预检不满足 | selected generation 只有 source-index/render-template；缺 plugin planning API、完整写集/恢复 receipt。11/13 文件含同内容重写与新目录，created_paths 不完整；未调用生成器，不能把静态推断当运行失败。 |
| `zircon_editor/assets/ui/editor/production/extension_plugin_manager_workspace.zui` | 固定 `Plugin_Audio` / `Plugin_RenderDoc` / `Plugin_Gameplay` 条目；`ui/retained/dispatch/workbench/feedback/data_production.rs:122` 明确返回 `Not implemented (preview)` | 不计为真实 Plugin Browser；不能将其静态截图送验收。需要复用真实 Module Plugins 链路，保留原身份，避免破坏其他预览消费者。 |
| `zircon_editor/src/core/project/scene_document.rs`、`ui/workbench/project/editor_project_document.rs` | 项目持有完整 Scene、场景来源 revision、项目加载/持久化材料 | 不能从“有 Scene”推断已有 World Settings 字段、撤销命令或运行时更新。 |
| `zircon_runtime/src/core/framework/physics/settings.rs`、`zircon_plugins/physics/runtime/src/manager/settings.rs` | 有真实 PhysicsSettings 与经 ConfigManager 的 store_settings；SceneAsset 有真实 entity/camera/post-process 字段 | 尚未对应每 World 字段/作用域、持久化回执和撤销；刚体 gravity_scale 不等于 WorldGravityZ，不能误称整个物理能力缺失。 |

## 布局和交互合同

本地参考是上述 UE 源码。注意当前 `SSettingsEditor` 已采用 SidebarContainer，分类 drawer 初始占比 `0.20`、初始停靠、禁止 pin；Plugin Browser 使用 `SSplitter`，左槽初始 `0.35`。不能照旧截图把两个窗口都做成固定宽度同版面。

- **Preferences / Project**：外层页签及 Dock 归 Parent；内容用真实分类树、顶部搜索/筛选、分节头和属性行。分类树与详情各一个滚动 owner。属性 label start 对齐；值按控件语义排列；长文本不靠空格、坐标补偿或逐图偏移。窄尺寸保留所有功能，分类栏的 drawer 入口遵循 UE 当前 Sidebar 行为，具体断点须实际测量后记录。
- **World**：在当前活动场景上工作的 Details pane；不复制第二份 World。顶部搜索/选项、可折叠属性组、真实类型化属性、局部 reset。切场景、关闭场景、无活动项目及修改 dirty 状态必须显式反馈。
- **Plugins**：顶部 Add / Directories / 搜索 / 筛选；左侧分类和真实计数；右侧 breadcrumb 与可滚动真实插件 tile；底部 restart notice 随真实待生效状态出现。空列表与搜索无结果分别本地化。缩窄时不裁掉启停、状态和重启提示。
- **焦点**：打开时按 UE 设置搜索/Plugin Browser 搜索行为建立初始焦点；分类、属性、上下文菜单可键盘到达；Escape 先关闭顶层弹层；弹层取消/确认后回到发起控件。按键捕获不能把同次 key event 再写入搜索/文本。页面返回、Dock 移动、复用窗口应保持合法选中项，失效 ID 回退到有内容分类。
- **写入**：UI 只通过现有 authority / mutation coordinator / plugin durable policy。区分 inherited、override、pending、durable、failed、retry 和 requires_restart。恢复默认不触碰未选中 scope 或他人的配置。撤销只报告经过命令栈/持久化实际验证的范围，未支持的重启及磁盘变更不能伪装可撤销。
- **公共组件**：复用 `WorkbenchPanelHeader`、`WorkbenchTreeRow`、`WorkbenchListRow`、`WorkbenchPropertyEditorRow` 的 `value` slot（Checkbox/Dropdown/Field/NumberField）、Button、IconButton。Penpot 重复项必须是 native instance。composite masters 的现有未完成限制不能用 detached copy 绕过。
- **主题**：沿用 `components/catalog.toml`、`theme/editor_tokens.zui`、`unreal/starship.zui`。共享 token / catalog / native painter role 的变更由 Parent 明确归属；本族只提供内容和局部 variant。详细提案见 [Parent 交接](settings-plugins-parent-handoff.md)。

## Penpot 身份、往返及验收

实际本族 file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402`、初始 page `b4a1e3ce-9e6a-81fe-8008-d69d60e4e403`，来自 [Parent 当前统筹记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md) 和实际 `penpot-assignments.json`。[真实设计文件](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60e4e402&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60e4e403)。历史 revn=0 空文件、revn=1 四空页面及 r8 重开证据保留。随后在本文件经正常校验保存四个 native flex 草稿板，初次读回 revn=3/vern=0；原 context 关闭后，fresh context 对四页面逐一重开、SVG 板身份和八张 UI 截图均有回执。实际 page/board ID 与证据边界见 [组件合同](settings-plugins-component-contracts.md)。

草稿只包含可编辑区域、文字、类型/来源映射及 fixture，未创建公共 master、颜色或字体样式；公共组件数与共享样式数仍为 0。普通 Penpot 字体用于中性结构草稿，不是新增 Zircon 主题。两张设置页的 initial/fresh 图 hash 不同，文字栅格观感有差异，尚无字体就绪证据；不据此宣称视觉往返一致。正式组件/样式、官方导出、产品加载及双端逐图验收仍未完成，r10/r11 只证明本文件草稿保存与源码映射。

草稿首次缩窄出现倒序，失败图及 r12 未生效的排序请求保留。核对 Penpot 普通 flex 的 stored/visual order 约定后，r13 正常校验修复 71 个容器排序，读回 revn=6，ID/域 metadata 保留。r14 fresh context 对 Project 草稿实际编辑三种逻辑尺寸、七项排序检查均通过并恢复 1280×800；只属草稿诊断，不填入下表正式窗口验收，也不证明全部文字、公共组件或 DPI 达标。

每次往返记录：file/page/board/component ID、实例与 master 对应、流式约束、共享 style/token ID、原 ZUI node/control ID、binding/event/slot、资源依赖闭包 SHA-256。保存、关闭原 context、fresh 认证重开同文件；官方 PNG 和 ZUI 导出后核对原身份，再将 delta 落到原资产。不从截图重写一套窗口。

修改生效试验必须单独保留 A/B 链：**真实 Penpot 共享样式编辑 → 官方 ZUI delta → 原始源资产及依赖 hash → 正常产品资源加载 hash → 新原生截图**。同时对设置与主工作台两个族取证；记录 local override 的保留、reset 后继承与进程重开持久化。仅根文件 hash 相同不能否定依赖变化，也不能证明变化已加载。

复用 `tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-review-contract.ts` 的 `LayoutReviewCase`，以及 `tools/analysis/visual/{record-editor-normal-window-evidence.py,zircon_editor_normal_frame_evidence.py}` 的正常窗口证据协议。case 中 `sourcePath/host/viewport/dpi/locale/state/data/themeSourcePath` 绑定实际输入；原生抓取保留 nonce、generation、HWND、PID/creation time、EXE SHA-256、资源 readiness、实际 DPI、GPU/readback 与桌面原图关联及 release ACK。过期 hold 必须重新捕获。

| 维度 | 必须逐窗口执行的实际输入 | 当前结果 |
| --- | --- | --- |
| 尺寸与 DPI | 1280×800、900×620、640×520；另 1280×800 实际 150%（scale 1.5，预期客户端 1920×1200 physical，仍核对真实 receipt） | 全部未运行；不能把当前系统值或历史主窗口 125% 图当作通过。 |
| 通用状态 | default、empty、selected、focused、open、focus-return、scroll-before、scroll-after、long-en、long-zh | 全部未运行。空数据来自真实空源/过滤结果；长文本 fixture 必须标明用途，不冒充产品数据。 |
| Preferences | 分类/全文搜索/无结果；bool/enum/string/chord/color；override、单项 reset、分节 reset；持久化 pending/durable/failure/retry；关闭重开 | 未运行。 |
| Project | 无项目/有效项目/配置损坏/只读；项目值与用户值继承；切项目后 binding generation；reset 不写其他 scope | 未运行。 |
| World | 无场景/活动场景/切场景；分类/高级属性/修改/单项 reset；undo/redo、save/reopen、实际场景变化 | 未运行。 |
| Plugins | 分类/搜索/启用与禁用筛选；真实依赖/不可用原因；启停确认与取消；磁盘保存失败；pending restart 与实际加载状态；重开 | 未运行。 |
| 双端对比 | 同内容/状态/逻辑尺寸/实际 DPI 的官方 Penpot 图、完整原生图、并排图、差异图、每图人工复核和关键边界/文本行框误差 ≤1 DIP | 无本族图，不生成装饰对照或相似度通过结论。 |

资源已就绪且有效捕获才可 `passed`；双端比较与逐图复核都通过才可 `accepted`。设计、功能和原生验收分别记录，任何一项 pending 都不得升级整个族。

原生捕获前置条件已在 `source-asset-map-r25.json`、`normal-capture-readiness-r25.json` 和 `capture-handoff-r25.json` 细化。实际 Preferences 源为 1 node / 1 control / 0 authored events（交互由 native painter/host 管理），Module Plugins body 为 18 / 18 / 5；保留五条原事件及 `module_plugin_list_slot`，不能只按静态文件推断运行时子节点闭包。当前官方 `--source-closure` 与正常 producer 仍固定 Level Editor owner；本族的 review host、源闭包与同案例 producer 入口须由原公共 pipeline owner 接续。没有生成伪造的 LayoutReviewCase。

只读核实 Parent 的 N57 历史候选：正常 `zircon_editor.exe` 为 238736896 B，SHA-256 `c728fdc3a891491c8f47198282b4dccb31771ff91e6c7b4383e461d89ca1fba6`；runtime、启动环境、producer 配置/模块、项目/场景等共 10 项指纹，以及所记录的 71 项 producer 合同输入均匹配。它们不证明当前本族资源已部署或完整产品源闭包已重验；未启动 EXE/producer、重放创建项目、发送输入、修改 DPI 或捕获。拟申请 25 / 35 / 25 分钟三个有前置条件的时段，覆盖基线、窄窗/语言/业务及实际 144 DPI/两族主题；所有时段未获授予。

本轮继续在本族文件整理本域 `PluginMetadataCard`：native component `81548358-cc2f-47cf-856d-39b395bfecfa` / version 1，main 沿用 AI tile `d57fd7f1-bf28-4c97-bc6b-0a52ea4696f1`，其余四 tile 保留原 ID 成为原生实例。全部 40 个原对象保留。它是动态 `module_plugin_list_slot` 的 metadata 组合草稿；没有新建公共 primitive、style、主题或独立 ZUI wrapper，正式导出与实时 status 仍 pending。

首次重开暴露四实例被同步为 AI 主件文字，失败图与 revn12→13 读回保留。已从 Penpot `libraries.cljc::text-change-value` 核实：只有 content-group 不足以保留文字 override，须有 text-content-text 子组。r26 恢复 12 字段、保留字体继承并失效文本缓存；最终两个独立 context 的 API/SVG/UI autosave 后各 20 字段正确，40 个 native shape 对应稳定，无 JS pageerror，最终 revn=15。六张修复图逐图复核，主件/实例右栏身份有效。这只关闭域组件文字覆盖缺陷；公共实例/style、UE 设计、插件/Rust及正常 EXE 验收继续 pending。

## 开放差异

每项正式双端图片当前仍缺失；已保存的域草稿诊断图另列，不关闭这些差异。原因和依赖按已查源码记录，未验证假设明确保留。

| ID | 参考 / 实际差异 | 原因 / 依赖 / 下一步 | 状态 |
| --- | --- | --- | --- |
| SP-001 | UE settings editable 分类/表单；当前单个 `WorkbenchPreferences` 根由 native painter 画内部内容，Penpot 未注册该动态表面 | 已证实源码边界。向 Parent 提交可编辑公共投影/语义节点提案，确定 owner 后再接线，不能展平截图。 | open |
| SP-002 | UE 独立 Editor/Project Nomad tabs；当前一个设置浮层混合 catalog | 已证实入口差异。依赖 Parent 页签/Dock 注册；本族提供 scope 过滤及真实 mutation，保留现有 control identity。 | open |
| SP-003 | UE 搜索建议、筛选和 All Settings；当前无搜索 action 和专用 All 项 | 已证实本族 action/projection 范围。等待真实 Penpot 设计，再实现查询与空结果路径。 | open |
| SP-004 | UE 分节 reset/import/export/Set as Default；初始单项 reset 会清除 resolved source | r28 在原 mutation 入口形成显式 scope/catalog 权限修复候选，保留原调用签名，六项回归未执行。分节原子性、可写来源、格式及禁用 UI 未完成，不扩大单项 reset 语义。 | open |
| SP-005 | UE World Settings 绑定 World Details；Zircon 尚无对应窗口/字段投影 | 搜索未找到对应命名入口，不作为全引擎不存在的证明。依赖场景字段与命令 owner；逐字段映射。 | open |
| SP-006 | UE 动态 Plugin Browser；静态 extension workspace 返回 `Not implemented (preview)` | 已证实固定条目和反馈源码。采用真实 Module Plugins 链路，保留现有 source IDs 及其他消费者；新壳须先在 Penpot 编辑。 | open |
| SP-007 | UE pending-restart banner / Restart Now；当前设置有真实 restart-only job 配额，插件有完整 enablement recovery | 需 desired/active 比较及完整事务投影，不能只看 requires_restart 或 RestartRequired 枚举名。依赖 Parent 正常关闭/重启/dirty-document 协议与产品验证。 | open |
| SP-008 | 全局主题编辑、局部覆盖/reset、两族传播；当前 DesignTokens 类型未见对应可操作 editor | 已证实 painter 分支与 token 安装路径；共享源归 Parent。请求实际 style identity 与 override precedence 决策。 | open |
| SP-009 | 相关 metadata/directories/new-plugin 窗口 | 已核对生成清单不可直接编辑、固定 native 根及真实包脚手架。多根配置和 wizard 接线仍 open；脚手架触及共享 workspace/catalog/app，SP-C7 先交 Parent，不直接调用生成。 | open |
| SP-010 | 四尺寸/DPI、中文长文本、焦点、撤销及每图 ≤1 DIP | 本族文件已分配可读；公共组件/style 未发布，正式设计与产品图缺失，桌面时段未授予。 | open |
| SP-011 | UE plugin tile 的版本/分类/描述/成熟度；Zircon manifest 已有而 UI status report 未带出 | 沿现有 report 发布 metadata/provenance；共享 API 提案 SP-C6。无 icon/URL 来源不补假数据。 | open |
| SP-012 | UE per-World 属性；Zircon 的物理与后处理已有不同作用域的 authority | SP-C5 需明确逐字段归属、命令及序列化，不从名称相近推断等价；具体映射见设计合同。 | open |
| SP-013 | UE wizard 创建/发现反馈；scaffold 缺所需 catalog 标记，report 仅 created_paths，恢复吞错且递归删包根 | r41 接原 owner/msg 的只读合同：现 generation 未接 plugin plan；参数化实际写集为11/13文件及目录。Parent对齐真实catalog结构，原generation owner接 pure plan/逐文件journal/恢复receipt；均未实现/部署，不补旧标记或生成。 | open |
| SP-014 | OVW-27 Localization Dashboard 的 target/culture、采集、PO/编译、字数/冲突报告 | 当前只核实 locale/catalog/localized-text 诊断，未发现等价 target/job 作者合同。语言切换不能关闭此差异；与资源 String Table owner 交接文档打开，Parent 决定真实目标任务能力。 | open |
| SP-015 | OVW-40 Config Editor 的单属性平台/文件层级 | SettingsAuthority 仍是唯一配置来源；r39 已形成原 ResolvedSettingsBatch 同 generation 逐层 raw/presence 的两文件精确提案，未应用。file/writability、页面 scope、提交 fence 与平台层级仍缺；没有 UE INI 层级/导入假兼容。 | open |
| SP-016 | OVW-40 Device Profiles 的命名、继承、CVars/LOD 与实际激活 | 现有 RHI profile 是真实不可变设备事实；未核实可编辑 profile store/selector/undo/apply。需 Parent 与 graphics/platform owner 决定模型，不制造 Desktop/Mobile 示例配置。 | open |
| SP-017 | OVW-40 Keyboard Shortcuts 的专用捕获、冲突再绑定、逐项恢复/持久化 | 单 chord/delta authority 已有；公共 chord-input/focus/IME、同代 preflight 和整 map 原子 mutation/undo 仍待接线。UE 双键槽、INI import/restart 行为不直接套用。 | open |

## 验证与产物状态

截至 r26 的阶段执行源码阅读、TOML/JSON/文档检查、实际 Penpot 域草稿保存重开和隔离无头诊断，没有 Rust 或插件变更。r28 已按 Parent 最小范围修改原 settings_mutation.rs；下述初期基础设施观察不覆盖当前正式入口状态。此前 Jenkins `entry.py capability` 返回 `ordinary_entry_unproven`（localhost:18080 connection refused），正式 tray module 的只读 status 返回 degraded / ownerLost；历史结果保留于 `infrastructure-observation.json`。Parent 接管后本族不再尝试启动/恢复或重复探测服务；pending 不作通过。

后续 milestone 的待执行范围：`zircon_editor` 包、`--no-default-features`、`--lib`、无新增 feature、debug/test profile；只选实际变更的 settings projection/mutation/persistence 和 plugin host policy 测试。正常产品为 `zircon_app` 的现有 editor-host profile，实际 feature/dependency closure 由 Parent 现有产品与 Jenkins 复用身份决定，尚未选定；不能以 capture helper 替代正常 EXE。每条编译前另行声明精确过滤器和命令，通过 Jenkins 执行，不使用全仓编译或 all-package 循环。

自有产物根：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins`。清单 `artifacts.json`；无源码树副本、无构建目录写入、无自有长期进程、未删除任何文件，实际释放 **0 字节**。任务未结束，清单保持 active。

## r27/r28 最小源码接续

Parent 的 followup-02 已明确移交 SP-C3/C6 最小源码范围。14 个相关 Rust 文件起始 tracked diff 为空，baseline 哈希保留。当前仅 `zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs` 有本族源码改动：默认 reset 采用 catalog 声明 scope，新增显式 scope 入口；权限仍来自原 catalog，写入仍经原 SettingsMutationCoordinator。确定目标层无覆盖时返回 None；高优先级 Session 可能遮蔽目标层时只清除指定层。没有新建 store/主题或改变其他 setter/宿主。六项回归覆盖无覆盖、继承 User、Session 遮蔽与清除、禁写 scope 和未绑定 Project；当前未执行，不作通过。

r27 已向用户声明并实际调用现有模块入口提交最初五项回归：zircon_editor、defaults off、features=[]、lib、filter `ui::host::editor_event_runtime_access::settings_mutation::settings_scope_reset_tests::`、单 job/thread、Cargo test/adapter dev profile、物理 E 根。原请求 `ue-settings-20261024-scope-reset-01` 在输入捕获返回 `external_control_forbidden: External checkout controls are not compiler inputs`，同一原 identity 的 QueryOnly 返回 `Current session authorization is pending`。完整日志在 `jenkins-settings-projection-r27`。没有编译/执行通过回执；补充第六项无覆盖兼容边界后源 hash 已变化，原 identity/失败保持原样，当前候选未重投。输入保护和原请求收件授权交 Jenkins owner，不运行本地 Cargo 或恢复服务。

公共 DTO 精确提案在 `scoped-reset-metadata-r28/plugin-public-contract.proposed.patch`，未应用；missing package=None、same snapshot/live clone 和原 enablement tuple 保留。官方 `parent-queue-11.json` 投递 exit 1，错误为“queue cannot contain more than 100 submissions”；没有 ACK，不称已排队或已批准，不改聊天数据库或重复塞队列。公共字段、正式导出、真实产品与逐图验收继续 pending，整个族 passed=false/accepted=false。自有客户端均已等待退出，失败和当前证据保留，释放 0 B；清单保持 active。

## r30 显式作用域写入接续

接收 Parent 的较早 10:20 初始文件交接后，已核对当前自有文档没有“尚未分配”条目；该历史 root-only/revn0 记录保留。后续实际 API 观察以 r29 的 2026-10-24T12:01:09Z 为准：本族 file revn15、四页、一个 PluginMetadataCard 主件与四 copies，公共 file revn0、component/color/typography=0、无发布/外链。八个 board 和保存关闭重开证据仍见组件合同/r26，不把初始观察覆盖新进展。

r30 在同一已移交的 settings_mutation.rs 增加 `set_setting_in_scope(key, scope, SettingValue)`；现有 bool、number-step、enum、color、string、chord 六类生产入口保留原签名和 catalog 默认 scope，均调用该路径。先核注册与 scope 权限，再用原 schema 校验，最后经原 SettingsMutationCoordinator 写入；不增加 store/主题或改公共 DTO。写入/重置仍返回原 receipt，SessionApplied/Unchanged/queued/failed 不改名为 durable。

当前源 SHA-256 `4966571d0e75e2a6f7294d8794d14812623c62278c124f077569ddaf0d8cc54a`，最小 patch SHA-256 `5dc7eea1c4919a836b3c504dafdb8d52456600145c10d1d97ff7e9f17ad28eaa`，均在 `scoped-set-contract-r30`。新增六项 set 回归，加原六项 reset 共十二项，统一私有模块/filter 改为 `ui::host::editor_event_runtime_access::settings_mutation::settings_scope_mutation_tests::`。旧 r28 候选、原五项失败 identity/filter 不改写。格式、diff、change guard 通过（0 errors/warnings/suppressions），没有重新提交已知输入捕获失败的编译。

这些是可被真实页面消费的域 mutation 候选。四独立宿主/语义子树、scope 来源、逐层 presence、事件与 project-binding generation 校验、raw numeric text commit、撤销/持久化重开仍未接通或验证；函数本身没有 expected-generation CAS 参数，不将其宣称为同代 UI 事务闸。公共样式/官方导出/插件/托管 Rust/正常 EXE/实际 DPI/双端逐图验收继续 open，passed=false、accepted=false。

## r33～r35 第二次接续的当前实现与回执

已重新读取 Parent followup-02 并核对原 14 个相关文件：仅本族 `settings_mutation.rs` 的 r30 改动存在，另外 13 个源身份未变化。r33 仍只改该原文件，增加 `set_numeric_setting_in_scope(key, scope, text)`；Int/Float 按原 schema 解析，保留明确 scope、原权限与 coordinator，拒绝非法/非有限/越界值，不 clamp。既有入口/事件未改。新增三项真实 authority/coordinator 回归覆盖类型/真实 Session receipt/重复提交、非法文本不改 generation，以及未知键/非数值/禁写层；连同原十二项共十五项，均未执行。输入控件/文本事件、scope/page、失效准入、undo 与保存重开仍待接线。

当前源 SHA-256 `57dc7a2a36aa883c1f7f1215d3f1ce94d790fb52c4eb9f20221276551cc26602`，r33 最小 patch SHA-256 `8a0cd4b90f5efe9f2550857b347a958e73642b12b383cf828972bbdba9608f8d`。格式/diff/guard 通过，guard errors/warnings/suppressed 全 0；初次格式差异保留。这些是静态结果，托管 Rust、插件、正式资源及画面仍未通过。

r34 完整 SP-C6 公共提案为十个文件、六处 struct literal 迁移；patch SHA-256 `2810e0c2ac85f6ab7e2468d686460ef33c57279acb2b71ae2d52eba35ebb0292`，未应用。metadata 从原 builtin/native 构造使用的包复制，缺包 None，published Arc/live clone 沿原路径；同 ID 的 builtin+native 行保持原 catalog 名称/metadata 优先级，不当作二进制加载版本证明。提案含缺包断言、native 元数据快照独立性及 live 失败保留完整 operation tuple 的回归候选；格式解析和只读 apply-check 通过，没有提案 guard/编译通过。公共 DTO/reexport 仍先交 Parent，未修改公共文件或 UI 资产。

r35 对原五项请求进行一次 QueryOnly：实际 exit 1，`ordinary_entry_unproven: Current session authorization is pending`，两个自有 PID 已退出。随后观察 installed capsule 变为 `cdd14500fa39088ae2c9870f62825acc2e526ba647d24d77cd6287722b5d0f31`，同一 driver/runtime；当前 11 项源码/客户端指纹匹配新 manifest。新 client 只收窄 material 配方，settings 遇到的 `source_preparation.py` 保护源码未变，原 external-control 失败仍需 owner 对账。原 identity/失败不覆盖；十五项候选已准备独立 identity/精确命令但未提交，不借用 material filter/修复身份、不本地 Cargo、不恢复服务。

本轮详细路径、当前测试声明和 Parent 增量见 [交接记录](settings-plugins-parent-handoff.md)。Penpot/正常 EXE/桌面未操作，r26 失败图及全部历史回执保留；passed=false、accepted=false。自有进程已结束，本轮没有删除文件，释放 0 B；任务保持 active。

## r36～r37：恢复态详情候选及实际依赖

12:40Z 认证 API 再核实 own file rev15/四页/一个原生主件，公共 file 仍 rev0/一个根/零组件与样式，见 `dependency-audit-r36/readiness.json`。复用 r26 保存/关闭/重开证据，本轮没有编辑 Penpot、公共资产、服务或桌面。

新增原插件详情最小修复：同插件的 recovery 报告会撤下 Apply intent 和 Tab 项，已聚焦 Apply 时回到 features；显示原事务原因并保留选中和 feature 事实。详情投影、键盘条件及提交复核共用这一原报告。源码及三个新增回归在 Parent 已授予的 `module_plugin_projection/**` 内，不变更公共 DTO、启停事务、operation tuple、原控件/事件或 restart。当前两份域源码合并 patch SHA-256 `6b57b1f4a12cf67610efb622e72f9a93eb8c990be3b0d7540f54b9888f07ea42`，格式/diff/guard 通过；详情八项与 mutation 十五项均未执行。

完整精确 diff/hash/guard、r34 公共提案兼容性和单独的八项详情 filter/未提交 identity 见 `plugin-detail-recovery-r37` 及 [交接记录](settings-plugins-parent-handoff.md)。原 Jenkins 输入保护/收件授权无修复回执，不反复提交或借 material 身份；正式公共实例/导出/family producer/EXE/桌面时段仍待原 owner。所有插件/Rust/产品/双端逐图 gates 保持 open，passed=false、accepted=false；本轮有实际域源码进展，任务不标完成或 blocked。

## r38～r39：逐层值的具体公共提案

r38 依赖复核未解锁产品路径，官方第13次新源码交接仍被100条上限拒收。随后把 SP-C1/C3/Config/快捷键共同缺少的逐层原值变为可审阅的两文件代码提案：`scope-projection-proposal-r39/resolved-scope-values.proposed.patch`，SHA-256 `38f618eb703ce78b97dd314b2f8ab901257d17805a14f6fb678a9c92fd88ee2a`。它补充原 `ResolvedSettingValue::override_in_scope(SettingsScope)`，保留原 key/value/source/batch generation，由原 registry 在一次 batch 捕获中提供真实三层原值及 absence。只读格式解析/apply-check 通过，三项回归未执行；core 两路径超出原写入范围，未应用，待 Parent/core owner 协调。

这是精确 provider 交接的进展，尚无新产品能力：不包含页面 edit scope、旧事件/currentness 校验、文件可写性、UI/reset 禁用态或 undo/持久化重开，也没有新增公共组件/主题/导出或原生画面。r37 两份实际域源码及23项未执行候选保持；r34 metadata 提案和 r39 逐层值提案分别记录，不能当作已接线或已编译。具体源码身份/成本及剩余合同见 [Parent 交接](settings-plugins-parent-handoff.md)，所有族验收继续 open。

## r40 当前接续边界与阻塞审计

2026-10-24T12:59Z 真实 API 仍为 own rev15/四页/原组件、公共 rev0/单根/零组件与样式/未发布；原 Jenkins identity 的再次 QueryOnly 实际 exit1，`ordinary_entry_unproven: Current session authorization is pending`，未起 Cargo。r34/r39 两份精确提案 apply-check 仍通过，但未有 Parent/core 协调决定，均未应用。当前无正式 family source closure/producer/EXE、桌面时段或正常产品图，不能进行所需双端验收。

同一公共接口/库发布/正式导出与原托管请求的阻塞已在 r33～r35、r36～r37、r38～r39、本轮四个连续 goal turn 保留。此前两轮完成了独立源修复和精确提案；当前独立盘点/设计/域候选及必要公共 diff 已完成，继续落公共源码、四宿主/World/restart 或桌面须原 owner 的具体交接。阻塞门槛按同条件连续出现计算，不按 r38/r39 的“无进展次数”重置；本轮已满足 blocked 审计，申请登记 blocked，最终状态以 goal 工具回执为准。整体未完成，passed=false、accepted=false，所有未实现/未验证项保留，释放0 B。完整观察和接续条件为 `readiness-and-goal-audit-r40.json` 及 [交接记录](settings-plugins-parent-handoff.md)。

## r41 SP-C7 只读回执收件

已按原 Codex 消息 `01a16dbc-2adf-7911-8337-3d49d946952c` 核实 generation owner 的原回执，SHA-256 `2b247a0384791c87cfb6fea5920f70aaba82ebdeece6f91b7788b82d7f8a48f4`；current cdd145 capsule/5实现/6clients 与15项源输入均匹配。原 owner `01a12af6-7511-70d1-a452-61b80678eb56` 接 generation authority，Parent 协调真实 scaffold/catalog owner，本族只消费状态。此次没有实现/生成/恢复派发、新业务或 Jenkins 身份；当前 goal 工具实际为 blocked，保持原状态。

设计继续仅展示固定 `<project>/zircon_plugins` 与真实不可用原因，kind 必选无默认，native=false。当前无 plugin plan API，源码标记预检不满足；没有可执行“预览计划”命令或多根/用户项目 SDK 能力。11/13文件写集、pure plan、同业务 request/planRef 的内部阶段、按 postimage 条件恢复及未恢复项 open 的消费合同已补入 [设计合同](settings-plugins-design-contract.md) 和 [组件合同](settings-plugins-component-contracts.md)。本族只读复核及文档结构结果见 `sp-c7-consumer-reconciliation-r41.json`，不计插件/Rust/原生验收；generator/Cargo/job/源或 sealed 修改/服务/桌面操作均0，释放0 B。

## r42 Parent 落地后的实际消费

followup-03 的 12 份实际 after hash 全部匹配，r37 details 的恢复逻辑及 Parent 增补的 `package_metadata: None` 保留。原 r34/r39“未应用”记录是历史状态，原 patch、基线和失败回执不改写。当前 mutation 源 SHA-256 为 `68beb857b9b5de6d87d69d4f6f60eb4922bcbe9dbf0af1c948c8a346c005baea`；本轮仅修改这一份 Rust 源。

reset 读取原 selected-key batch 的 `override_in_scope(edit_scope)`，不再以 winning source 猜目标层。目标层为空，即使 Session 遮蔽，也不进入持久化准入；非空仍清显式目标层，保留 scope 权限、原 receipt 和错误。新增两项缺层回归，并加强“被 Session 遮蔽但真实存在的 Project override 仍须拒绝无绑定写入”的回归。十七项只是未执行候选；原五项 filter 不覆盖当前源。

增量 patch SHA-256 `3e2371728e685ba0712c15cf2f0eb02503865d6d41b83c649b2569684c548005`；完整 mutation candidate/guard digest `7ef8c47926c0d6b27693dab885d8e7bcafc8ea7dcd751acbbb75839841c2e7d8`。rustfmt、diff-check、静态 guard 实际 exit0，guard 0 errors/warnings/suppressed。证据位于 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/scope-consumer-r42`，具体新宿主提案和剩余合同见 [Parent 交接](settings-plugins-parent-handoff.md)。raw 检查与 clear 是分开的操作，没有新增 generation/CAS fence、undo 或 UI 接线。正式插件、Rust、EXE/实际 DPI 及双端逐图验收继续 open，释放0 B。

新交接已通过官方 `codex queue` 实际入队，消息 `01a16e17-0b59-7b83-80e8-5e45f9d83d5c`，exit0；仅证明入队，不证明已读/接口批准。原容量失败记录保留，具体回执为同目录 `parent-queue-14.json`。

## r43 共享 rev1 的最小原生试接

Parent 的新库状态已独立读回，旧 rev0/空库为历史，不再作为当前未发布结论。Shared file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`/page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13` 当前 rev1、41对象、四主件、isShared=true，colors/typographies=0。本族通过 Penpot“管理库→连接库”真实连接，库记录 `isIndirect=false/revn=1`，没有编辑 Shared。

受控验证页 `1ca0591c-2856-49b0-9d7e-eeb076bafe03`、flow board `4829a7b5-8807-4e66-b34d-b94fbb8807fe` 位于原本族 file。8个 copy 来自原生素材 dragstart/dragenter/drop 驱动的 Penpot `instantiate-component`，不是脚本复制 main。每个主件各一个默认与一个文字覆盖；全部 source-file/component/main/shapeRef、原 sourcePath/pluginData、旧域组件均保留，未 detach 或创建同名本地主件。保存、关闭、两次 fresh context 重开后四项文字覆盖均保留；仅证明 Penpot 链接/文字持久化试接，不证明 ZUI/运行时复用。

正常打开原 Preferences 页后曾由 rev15 升到rev16，53个原文字节点 hash 改变；未逐字段重建原因，未回滚未知缓存/来源。此后以实读 rev16 为新基线，原四页对象和原域组件定义逐个 hash 保持，所有新增设计仅在受控第五页。首图发现原生插入后的列布局倒序，assign stored order 后仍需原生 row→column 重排；失败图保留，最终fresh图及API顺序一致。

PropertyEditorRow 的真实 required `value` slot 只接受 Checkbox/Dropdown/Field/NumberField/RangeSlider/Slider/Toggle，四个已发布主件不含这些编辑器；slot原样为空，不能冒充属性值控件。长中文树行/属性名已暴露固定行高和换行溢出，仍需公共 owner 扩充原组件；canonical owner/sourceHash/slot-export、跨文件正式导出与override-reset、两族共享样式A/B、正式UI和EXE均open。本轮无Rust/产品资产/服务/桌面改动。完整身份及逐图记录在 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-linked-consumption-r43`。

已将此轮真实试接、最小slot/export合同和失败图通过官方queue交回Parent，消息 `01a16e33-70ea-7112-86f3-28214b1626d0`，exit0；入队不是已读/批准。保留证据及可复用驱动，全部自有浏览器/Node已退出，任务未关闭，释放0 B。

## r44：原 Jenkins owner 准入审阅收件

实际读取[准入审阅](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/ue-settings-20261024-scope-reset-01/admission-review.json)，SHA-256 `8d925b8df67b2e9887963fdeb301144790d1932e161d5febbc1eb338271b40ea` 与来信一致。Parent 十二 Rust after hash 及当前13份相关 Rust 均匹配；原13路径 guard 含已变化的 Parent note，不能把旧完整 diff 继续标为当前全量通过。相同源码对应的 format/static 证据可复用。

mutation 当前 `68beb857b9b5de6d87d69d4f6f60eb4922bcbe9dbf0af1c948c8a346c005baea`，静态17定义；details 当前 `e63b13022af1fb4d33590ec64327e7fdc5e3294b3b550fbf237b6f872850a1a6`，精确 `details::tests::` 为8定义，不能称 harness13通过。raw3/native3/唯一live1为其他独立过滤范围，上述均未执行；不借原reset或两个prepared身份代替收件，更广projection16定义不自动纳入。

原 owner 的 `candidate-input-r35-current.json`（SHA `a2f79cba79acf888db3adbed47c72dde3539b6ca105381501afeacd5bf3cd14c`）和 `candidate-input-r37-current.json`（SHA `5f59d8113e38078227e56d51af94fd1dc26b736fc72b4615f708c620b350e0ad`）逐字段核对，仅 sourceHashes/dependencyHashes 与原身份不同；原 request/operation/session、策略和其余字段保留。所有列出的源码/依赖当前匹配。这是 prepared current-source revision，尚无服务端/source/resource/native writer准入、Job/EOF或执行结果，不重写本域旧 inputs。

旧 external control plan 冲突仍无正式解决回执，source_preparation 仍 SHA `2a38f55962b5e7c9f9914f7e6dbe637ec41522b1852d1be824ceada1c246ff77`。不删除/排除 plan，不静默选择 full-file-index-v2，不重投或停写全checkout。原 owner 待实际收件/完整输入与writer准入后，另行声明五条最小模块命令；adapter dev 元数据不等于实际 Cargo profile，原计划 argv 未显式 `--profile`。

本轮只更新本域两份记录和[消费核对回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-admission-consumer-r44.json)，无 Rust/ZUI、服务、QueryOnly、Jenkins提交、Cargo、生成、锁、缓存或Penpot操作；按来信不发例行ACK。SP-C7六共享输入hash仍一致，执行保持不可用。r43 Penpot试接及全部正式UI/Rust/EXE/DPI gate原样保留，`passed=false`、`accepted=false`；释放0 B。

## r45：Jenkins恢复后本族原请求只读对账

已读Parent“Jenkins最新恢复结果”及[恢复交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/jenkins-recovery-handoff-02.json)，r2原生终止receipt的实读hash一致，root0/active0/childrenGone/双EOF/readersJoined/jobClosedAfterZero均成立。恢复只属于Jenkins原请求，原P21/Runtime11、产品回归及画面gate不据此关闭。Parent历史20b3胶囊正确保留；本轮preflight与查询时installed已为 `9c684aef96b8b1c94319aa8619207f8b07aefffa71d571d4f58708d2c3408233`，manifest和5server/6client全部实读匹配，同driver `fc553228…` / runtime `start-1792836412672596900`。状态文件实读healthy、desired completed，未启停服务或核改Home。

只运行一次原r27 `ue-settings-20261024-scope-reset-01` / `validate-settings-scope-reset-01` 的正常模块入口 `-QueryOnly`，沿原session/driver/runtime/identity SHA `19981c9704944d591c8e58443a9c151a62da72a92a3dcf580964f6c9753a278a`。原source `96b264…`及旧5项reset filter原样保留；本轮exit1、stdout为pending/ordinary_entry_unproven/Current session authorization is pending、stderr空。客户端已join及双EOF，没有Cargo/Job/新source preparation/收件或当前17项测试结果。实际[查询回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/original-jenkins-query-r45.json) SHA-256 **`f17559c9dac8c86b5fafeca3c4df8f7866af8053c2f785544bb74627d8882701`**；旧r27/r40结果保留。

当前13份Rust与r44一致，mutation17/details8及raw3/native3/live1仍未执行；两prepared inputs未修改、未提交，不借旧身份验新范围。新查询输出已按要求一次性交原Jenkins owner `01a12af6-7511-70d1-a452-61b80678eb56`，官方queue exit0、消息 `01a16e5b-d30d-7011-babe-265024c11aa4`，入队不代表已读/准入。待原owner核对automatic receiver、原收件/外部control-plan/source准备及resource/native writer，给真实receipt后再按最小实际Rust变更范围另行声明编译；不手工grant/换ID/写库/盲重投或全checkout停写。

本轮仅本域文档和预登记证据写入，Rust/ZUI/公共组件/主题/SP-C7/scaffold/锁/服务/桌面/DPI/Cargo均0。自有查询/queue客户端与helpers四PID独立复核均缺席；必要失败、回执及复验驱动保留，释放0 B。typed reset/scope、真实restart、Dialog body/actions、长标签公共变体、正式导出和正常EXE未证明项保持open，`passed=false`、`accepted=false`。

## r46：插件详情报告刷新后的焦点恢复

沿followup-02已授canonical `module_plugin_projection/**` 范围，仅改原 `details.rs`。原代码只在recovery且焦点为Apply时归还features；feature移除/变为enabled+required/owner失配、列表清空或分页消失时仍可能保留失效焦点。这是源码控制流定位，尚无运行复现，不称画面已修复。新逻辑先按原identity更新选中feature/page，再以同份report检查Back/Previous/Next/features/Apply；有效焦点保持，失效回features，空列表回Back。旧report/changed/一次性intent fence和真实enablement tuple保持。

新源SHA **`ea903d46098963df14f446a3f9d67f4e6471cd90b4de6d1917ae8e3715b67f14`**；before为Parent/r44/r45的 `e63b1302…`。精确增量patch SHA **`6f7af9e658b2be8f440aaea3e121ee98d2cfc320ae4dc2a4d364a18de04edf29`**；相对HEAD完整候选/guard digest **`827462fd3912e984dc1f74da6bd2638c214a200a5fd1ab69375ad483e0c3aa01`**。反向移除本轮增量后内存重建精确得到原e63b1302，不写源码副本；r37与Parent改动逐字保留。rustfmt check、diff-check、guard均exit0，guard 0 errors/0 warnings/0 suppressed。

3个新增真实Rust测试定义覆盖：Apply资格撤销三种变化并保留其他插件/原operation；四种原焦点下列表清空并可Back返回原package；Previous/Next失效后保留稳定feature身份和新page。均断言旧intent拒绝及稳定projection缓存，原8定义全部保留，当前详情11、mutation17未执行。两份原owner prepared inputs均因details源或依赖hash前移而已旧，原文件和业务identity保持，不静默更新或提交。

下一最小验证是zircon_editor、defaults off/features=[]、Windows lib、locked、jobs1/threads1、filter `ui::retained_host::app::module_plugin_projection::details::tests::`；runtime必要传递feature仍为target-editor-host/test-support。模块wrapper无显式profile，adapter dev不代表实际编译环境；原Jenkins owner先核真实当前输入/自动收件/source/resource/native writer，再由执行方声明原业务链命令。本轮不跑Cargo、QueryOnly、新identity、generator、Penpot写入或桌面；Shared仍独立GET读到rev1/4主件/0styles，own rev22。未证明的正式设计/typed scope/restart/Dialog/全局主题/EXE/DPI保持open。

精确[消费合同及证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-detail-focus-r46/contract.json)，源patch位于同目录。初次反向辅助脚本的空行assert及普通Python写源errno22保留；未造成源截断或改变权限/锁，后按审阅的rustfmt差异用apply_patch完成格式。必要证据继续保留，释放0 B，passed=false、accepted=false。

本轮精确源码交接已一次queue给Parent和原Jenkins owner：Parent native exit1/queue100条已满，失败原文保留，不重试；Jenkins owner exit0，消息 `01a16e8e-6f25-7891-b28a-f8d8bca4ae28`，只证明入队，不是已读/收件准入。完整双目标回执位于同目录 `parent-queue-16.json`；本域文档提供Parent可审阅入口，不修改其共享记录。
