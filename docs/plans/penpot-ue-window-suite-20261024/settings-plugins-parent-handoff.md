# 设置与插件 → Parent 交接

## 2026-10-25 r96：已落实健康读取失败处的旧值/焦点清理

本轮是授权14路径内的2源实际增量：`app/settings_window_actions.rs` 的健康捕获错误 caller，以及 `dispatch/templates/workbench/settings_window.rs` 的消费方法/2回归定义。读取的是原 bridge 已显示 context，经原 coordinator 同锁 phase/binding 与 authority generation 检查；失败时撤销值和旧设置原生草稿焦点，仍有效时保留原批次。旧 retry 均撤销，正常 health 可重新安装 retry 但不能恢复旧值；不增加 mutation/retry 写操作或新的 authority。

14源严格内存逆增量匹配，旧74定义全保留，现76仅静态；新2覆盖真实项目清理通知期间的 health 拒绝/值失效及原 pending User ticket 保留、有效值分支。源格式与 guard 通过，完整交接和本轮实际静态结果见[handoff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-health-capture-retirement-r96/handoff.json)。未来仅 `health_materialization_tests::capture_failure::` 与原 `native_focus_retirement_tests::` 两独立 Editor filter，各静态2，未提交执行。无新 request/attempt/operation、Cargo、QueryOnly、source capture/freeze 或服务/桌面操作。

r78/旧 r27/r35/r37/r70/r71/r73 输入及失败不覆盖；不需重 apply 本轮源。共享 origin/frame/window/open lifetime 仍待 Parent 公共边界，不能用本片取消焦点替代入队时的来源证明。Provider/完整 typed subtree、normal producer、插件正式导出、EXE/实际DPI/两族样式A/B和 family accepted 仍 open；不以本片静态通过关闭。释放0 B，保留本轮失败/交付证据。

## 2026-10-25 r95：已交付真实三尺寸导航草稿与保留差异

实际产物，非ACK：own file=b4a1e3ce-9e6a-81fe-8008-d69d60e4e402，Project page=49ff1071-f83b-4b72-bb6e-043a3326ec82，rev355/5页2193对象，saved/fresh data=02fc7deebcf7d5d294fae5a3bfb92ff451e6068f08286d0f86afddaa6ad6e59a。

| 新原生board ID | 尺寸 | 导航初始宽 |
| --- | --- | --- |
| 9f550dbe-81f1-804c-8008-d83cad41915d | 640×520 | 128 |
| cd1e5d81-8700-80a8-8008-d841862bb4ed | 900×620 | 180 |
| cd1e5d81-8700-80a8-8008-d841b6a349bf | 1280×800 | 256 |

新366对象沿原TreeRow/SearchInput消费，主体flow、详情fill，All源文案/粗体/大写/留白与叶单行Clip；隐藏cube/叶箭头但保留Hidden占位。source pluginData均等原板，原1826非目标根节点/目标根其他字段及原子项顺序/另四页保持。最终87copies=83Shared+4Card，另1原main；1264shapeRef=原1063+新201，2318非空styleRef均有效。r01把原main算入88绑定节点，r02已按mainInstanceId分离，旧回执保留。

三板真实关闭后新浏览器重开，板内diff均0；23PNG已逐图看。[640并排](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/penpot-before-after-640.png)仅Penpot，13780变化像素含356导航外变化，未解释/判整页安全。[失败清单](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/retained-failures.json)保留旧缓存、编辑器嵌套、迟positionData/hash保护及null引用检查失败。

Shared真实rev5/4main/34色/3typo；syncedAt仍早于modifiedAt且UI提示更新，没有公共Update/ACK/reset。[设计合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/design-contract.json)、[引用回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/native-audit-r02.json)、[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-source-fidelity-r95/image-review.json)可审阅。没有14域Rust/公共源写入、正式export/plugin/Rust/Jenkins请求、桌面或服务动作，不扩旧filter/重放旧终态。根provider/类型树、Ellipsis/tooltip、live splitter、真实fields/origin、normal producer/EXE/150%DPI及两族主题A/B继续open。原r92共享导航提案待源owner审阅；r94两源无需重apply。goal active，`passed=false`、`accepted=false`。

## 2026-10-25 r94：两份域源码已落实原生草稿撤销，无需重apply

实际canonical settings_window_actions.rs f4d93b1d1226c7c13ad18fba63add510fee16349490e3d3d7d92d7ba814ee2d7→8e2c52dca7336552b60ffb437b7ca892b48147867709aab4c3bbe2b19d66ed39 /15279B；surface/option.rs 9ffe4032a25650b7d43500ba0b6a73999977c21af94658829cff02f71e5b2e56→66b429999f4f2ff03513e50e5622926f130543e63333da8f25673d824b5605b1 /13045B。精确两源patch c9c07f6f3dc6905809c1ce6556e66c181cad740f5da98777473db70cede5f98e，严格逆增量精确还原两前像，源LF/CRLF保持。[增量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/source-change.patch)、[current14](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/source-pins.current.json)。

真实open/sync/category/changed receipt调用点现清原native三类设置focus及抽屉返回设置快照，访根窗和原committed model已登记presenter，不提交值，不改原authority/coordinator/operation tuple。其他12源/原72定义保留，新2只是静态74；独立filter ui::retained_host::app::settings_window_actions::native_focus_retirement_tests::，不扩旧请求。[执行意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-native-focus-retirement-r94/regression-contract.json)。format/reverse/diff/两源guard0、0/0/0，原排版/两脚本诊断失败和Git索引EOL提示均保留。

本轮callback macro经实读是同步invoke，未复现queue race。origin/window lease/published semantic batch、关闭不重开、IME platform、typed subtree/provider、normal producer/正式export/normalEXE/150%DPI/两族A-B仍open。公共9路径仅读、5同/4后继不回退，不扩大共享写集。0Cargo/intake/QueryOnly/source capture/freeze/Penpot/服务/桌面/新业务身份，goal active、passed/accepted=false、释放0B。完整新handoff/summary/最终guard见本轮报告目录。

## 2026-10-25 r93：真实直接刷新生命周期已落源，新增2回归意向

已在授权14范围的1份canonical settings_window.rs实际应用，不需Parent重apply：before4e936f57…→afterfc74f80a2257cc781d59d94282784da4822e28961c52efc67a63733c5e3f3f79 /93453B；精确source patch83251265ffafe5204bd3bcf3a8003e4b88c54da08239073f3e612942bd2f5dda。只给三个原值刷新/关编辑器/分类方法增加7生产行含注释：取走原retry，整次property+template成功后原样恢复，错误留None。其余13源未变，旧70定义完整，新2现72静态，严格逆增量还原原前像。[current pins](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-value-category-materialization-r93/source-pins.current.json)及[增量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-value-category-materialization-r93/source-change.patch)。

实际调用区分：sync值刷新与分类回调可带旧retry进入直接重建；refresh_after_settings_receipt本就先清retry，新增实现必须保留None，不声称该既有回调新发现了重试漏洞。新2定义覆盖三transition的成功Some/None、传入旧于最新的batch、分类/关闭noop、原MissingNode模板失败、pending/generation不变及显式health/value各自恢复；不是harness/GUI结果。新filter value_category_state_change::独立交原Jenkins owner；health宽prefix从6到8，不塞入旧r35/r37/r51/r91 prepared或终态。[正常最小验证意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-value-category-materialization-r93/regression-contract.json)。

初次rustfmt失败保留，修正后format/reverse/scoped diff/sourceguard均0、0/0/0；Git LF→CRLF提示原文保留，实际LF没改。0 Cargo/intake/QueryOnly/source capture/freeze/新业务request/桌面/DPI/Penpot/服务/公共源，r92rev330仅历史设计证据。原origin/window lease、各生产provider/typed subtree、正常family producer/export/EXE/150%DPI/两族A-B仍open；goal active、passed/accepted=false、释放0B。完整summary/最终guard/回执同本轮目录。

## 2026-10-25 r92：三尺寸真实单行试验保存重开，根行样式更正未通过

本轮实际新产物：自己文件rev330/5页1827对象、3张原生linked试验板（各122节点），9行保持单行且搜索随父级铺满。保存关闭及新浏览器重开完整SHA 031ce1d0b1ba2da70a84acf7993e74396b6508fb05634d1d4b2de4301b79ecfa一致，三尺寸完整画板0设备像素变化，24张图已逐张看。[summary](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/summary.md)、[native audit](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/native-source-audit.json)、[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/image-review.json)。旧1460节点精确保留，Project zero-root未有全前像例外单列；Shared rev5未变，71 Shared copies/1063 refs/1880样式引用均可解析，未签library ACK。

必须更正本片早期推论：UE默认FTextBlockStyle的Clip不是所有分类的最终样式。实际StarshipCoreStyle ButtonText显式Ellipsis，PrimaryButtonText复制它，故Root/All为Ellipsis+NormalBold+大写及额外上下padding；普通节才沿NormalText Clip。现在三板仍是单行/行裁切试验，根行省略号/角色/字体/图标/tooltip及分栏边界均未通过，不把新画板当正式UE视觉/真实provider。[角色/source合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-category-nowrap-r92/category-source-contract.json)接续既有267f6013 navigation提案；需要公共owner按同一TreeRow/现text_overflow消费角色，不重造主件或主题。公共源本轮0写。

初次审计整content哈希拒绝已保留：仅57个绝对SVG路径原生平移，334坐标/板与float32(原坐标+实际board位移)完全一致；文字/metadata精确。r02唯一ID1823误作对象计数的报告亦保留，r03按5页总计1827。前后640图有少量不变值栅格差异，未推定原因/≤1DIP。原长标签失败/旧r35r37/现请求原件均不覆盖。

0Cargo/新request/attempt/QueryOnly/source capture/freeze/桌面/DPI/服务，14域源仍同r91、70仅静态定义，未提交新的Rustfilter；下一独立域切片继续，不为共享源等完美交付而停工。正式export→产品加载/normal producer/EXE/实际150%DPI/两族A-B仍open，goal active、passed/accepted=false、释放0B。

## 2026-10-25 r91：实际1源弹层/滚动双上下文失败修复已落地

新产物：原 popup toggle、close editor、scroll 三个direct refresh 只撤销values的缺口已修正；6行生产增量并列 take/成功restore原retry，错误留下两者None，不改noop/事务/clock/store/public input。1源 settings_window.rs 3fec0d9c…→4e936f57ebe4ea284e94f79101e31d05d8423891cf8d66ce666e9d9e9ab0d8e4 /82092B；其余13源同r88。原68定义全部保留，2新定义共四真实transition，当前70未执行。

[当前14pins](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-popup-scroll-materialization-r91/source-pins.current.json)、[exact source delta](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-popup-scroll-materialization-r91/source-change.patch)、[独立执行意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-popup-scroll-materialization-r91/regression-contract.json)。source patch/guard fe7819d3a940f4066e24126cfb20754f5c84375955c9d1098a9cd9e0ae9bb989，rustfmt/reverse/diff/guard0，0/0/0；严格内存逆增量还原前像。仅已授权1源，不需Parent重apply；Shared3/r86/r88/旧r35r37不重放。新filter独立交原执行authority，不等同正式收件/准入/Cargo结果。

未有本轮Penpot/服务/桌面/source capture/freeze/新identity；r90真实rev307设计、r89IME未应用候选及所有旧失败保留。当前片不关闭property partial write/其它direct refresh、native originating frame/window语义、4页/provider/typed subtree、正式export/normalEXE/150%DPI/A-B。goal active，family passed/accepted=false，0B。

## 2026-10-25 r90：三受控画板 SearchInput 内部宽度已原生落地并重开

不是 ACK：本域真实 own rev295→307，在原 Project page 三受控画板只改6个 SearchInput 子节点（width/selrect/points/constraintsH/touched），原59 Shared copies和862 refs有效，所有 pluginData 与其余1455对象精确不变；Shared仍rev5未写。原生背景宽640/900/1280、文字604/864/1244、左右锚定，保持图标/内容/色字体引用。保存→关闭 Chrome2376→新 Chrome29060重开完整data SHA一致；三尺寸画板裁剪均0像素变化。15PNG逐张看过，前后/差异图仅是 Penpot 对照，不是 Penpot-EXE 或 actual DPI 验收。

[实际六节点 delta/身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-content-flow-r90/mutation-saved.json)、[fresh reopen](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-content-flow-r90/reopen-verification.json)、[完整引用与样式核对](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-content-flow-r90/native-source-audit.json)、[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-content-flow-r90/image-review.json)。source guard/metadata原样保留，正式导出 owner 可据实际 delta 审阅消费者布局覆盖，不直接套改 Shared main/重签或将发布当库同步；本轮未运行正式 export。

正文仍结构草稿、Property value空槽与新槽迁移尚未接通；Search 的真实筛选/焦点/IME及 native origin、4页/typed subtree/provider、normal EXE/150%DPI/A-B 均留open，原长标签失败板未动。原14 Rust仍r88，68只是静态定义，r89共享IME候选未apply；没有新的 Cargo filter、intake/identity/source capture/freeze/服务/桌面动作。四个自有 Chrome根均已关闭exit0、认证文件未变；必要报告及两重放helper保留，0B，goal active、passed/accepted=false。

## 2026-10-25 r89：交 Parent 的三路径 IME fallback 候选，未抢公共源

补丁 `shared-ime-commit.proposed.patch` SHA `348792fba80cd4cf6d727ff69f40f064961421e3bbe503d1081b83259d530f4a` /4457B，仅3个当前公共路径：event_loop/platform_input.rs 原 `0d1452b1…`→候选 `b8711cf8…`；其 tests/platform_input.rs 原 `6979b7c6…`→候选 `e4b71814…`；原 platform_input_translation.rs `460e9551…`→候选 `b7dcb8a9…`。完整 raw SHA/字节与责任见 [candidate](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-legacy-ime-commit-r89/candidate.json)，无 canonical Rust 写入。

最终旧回退接收结构化IME Commit文字，保留canvas/retained Session先行、原draft与显式Enter提交；Runtime事件语义不改。新增2个pure fallback定义、修正1个旧翻译断言，原14仍r88/68定义；不重apply r86/r88或扩旧请求。仅stdin格式和只读forward --check0，未集成后guard/编译/GUI。Parent如接纳，按实际前像应用后执行常规最小检查；现有r86 painter候选仍独立。

UE composition/OnEnter差别已按只读源码记录；完整composition/preedit/cancel/undo、输入顺序、frame/window/open lifetime/semantic lease、String provider/4页/typed source/export/normal EXE/实际DPI/A-B继续open。无需用户重复grant，不新增正式Jenkins身份或重投；本轮0Cargo/服务/桌面/Penpot/生成，0B，goal active、passed/accepted=false。

## 2026-10-25 r88：两源真实上下文失败边界增量，独立回归意向

通知源只读追踪发现 prepare health 后直接调用 generic bridge，未经过 r83 health wrapper。已在授权 bridge 的 settings-context 生命周期取出原 values/retry，成功原样恢复，任一 Result 失败保持撤销；2个新回归定义覆盖通知准备/无准备两条失败和正常恢复，未执行。两源实际 after：`componentized_window.rs` `4d4c0b35b8a75ac59257dc27f21774bf0351323ce0e4915a1552da1fb9af75f4`；`settings_window.rs` `3fec0d9c28159630da74527252a952a5415a3565cf2f422bd160bd8f86b1260e`。其余12源仍 r86，原66测试名字及全部旧字节逆还原匹配。

源 patch/guard `e1e8809b7a9bc36d95c51a51eebb66b65ef066e6a909c7ec45b9ab240c9f2a8a`；格式/whitespace/reverse/checker 均0，guard 0/0/0；首轮断言格式失败和 Git EOL 提示保留，不改索引。完整原源范围、当前14pins、两个新定义/精确最小 filter 见 [r88 执行意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-prepared-materialization-r88/regression-contract.json)。这不是 Cargo/新请求/attempt/身份/源资源准入，不借旧 r35/r37 或默增 r83 范围。

未改 Shared3、public masters/catalog/tokens/codec、Runtime/native input、通知源或生产 provider。r87 Penpot rev295/Shared5 为最后实际观察，长标签等失败保留；normal EXE/实际150%DPI/native origin/IME/4 page/typed subtree/两族A-B继续 open。仅实际变更合并交 Parent 与原 Jenkins authority；queue 回执只作投递。goal active，passed/accepted=false，0B。

## 2026-10-25 r87：真实流式板已保存重开，新长文本失败可复现

本片只在自有 Project 页新增4块 native duplicate 可编辑板，rev258→295，5页/1461对象；原973对象除零根成员表外、其余4页/全局数据及 Shared rev5 全部保留。原 TreeRow linked instance 的本地 flex/label fill 在640/900/1280使短标签框横向溢出从旧640约52px变为0；4板保存关闭重开完整数据hash一致，4个设计区域PNG差异0设备像素。59 Shared copies/862 source refs全部有效，488新节点完整插件 metadata 与原对应节点一致，未detach/重签/改 Shared，见 [重开审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-flow-r87/fresh-reopen-audit.json) 和 [源身份审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-flow-r87/metadata-source-audit.json)。

新增明确缺口：同一linked标签在长英文/中文压力板实际换行5/4/4，固定行高下越行重叠；英文最大行81.546875大于文本框约76。失败板 `c79af4cc-85c3-80cc-8008-d81334da9160` 保留，原r84失败也保留；不拿框溢出0关闭长文本。请原 public/source owner 沿既有r73导航/叶节点合同接单行/tooltip/ellipsis及真实export语义，本片不另造组件/主题/codec补丁。具体实例ID、原metadata/geometry覆盖与差异原因在 [differences](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-flow-r87/differences.json)，14图逐张复核在 [visual-review](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-flow-r87/visual-review.json)。

本片0源码/资产/Cargo/intake/QueryOnly/source capture/freeze/服务/桌面/正式export。9源与r84一致，原r85/r86实现/回归准备及painter/IME/origin缺口不变；无需借旧r35/r37或新建validation身份。6属性值继续是设计schema夹具，正常引擎producer、4page scope、实际150%DPI与两族A/B未证明。goal active、passed/accepted=false；0B释放，保留证据和2个helper，queue只计投递。

## 2026-10-25 r86：原 domain2 实际应用与新的输入差异

已只应用原 `domain2.proposed.patch` SHA `d2b44500aa5245ad28872bbd38d7cf853b7900b0793add5fef2369e986f1d997`。host raw `c19f802354e5fffa20c6fe1df10fda0505d852c4e21aa5caa080a7ade188a2e7` / 55087B；surface/edit raw `28fa21401fe60666f34929e8a1b304f51dc2b4b290bc7b29003642d35f52bd8d` / 6007B；原 CRLF 均保持。Parent 三后像 fresh匹配并保持只读。14当前指纹与 raw逆增量保留全部 r85/r83 历史，12源不变；未重 apply Shared3 或旧提案。

原真实 Numeric commit 分支现沿 bridge saved typed context -> schema i64/f64 -> coordinator 同 guard/authority 同锁条件 mutation，保留原 scope/persistence/receipt/noop。两源格式/diff/reverse/精确 guard exit0，0/0/0。原3候选回归现落源，独立单包 defaults-off/features[]/Windows lib/test/locked/jobs1/thread1/E 的最小验证意向在 [regression-contract](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-consumer-r86/regression-contract.json)；未创建或重放 request/operation/attempt，未提交、QueryOnly、source capture/freeze 或 Cargo。

两项具体共享跟进已给精确源证据：Numeric painter 只读快照而不读焦点草稿，未应用小补丁 `ebcef8334fee037480788684051a197467a18695a9654a7288ae1738b68c57c6` / 4024B，涉及 Parent painter commands/tests 两源，原像在 [候选合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-consumer-r86/shared-numeric-draft-painter-candidate.json)；只做 stdin rustfmt 和 forward-check，未写共享源/未跑测试。另 Runtime 当前 `Ime(Commit)` 被 legacy Text-only fallback 拒绝，legacy focus 激活时 retained pane 也不接；Editor 旧测试期待 Text 与 Runtime 实现及其测试不符。初次报告误从旧测试推断映射，已保留原稿、读生产实现并更正，见 [修订证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-consumer-r86/input-lane-correction.json)。无 native/IME 实测回执，不用枚举或3域回归关闭 GUI。

完成应用、当前14源与全部精确回执见 [handoff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/preferences-numeric-consumer-r86/handoff.json)。originating window/lifetime、4 page scope、typed editable source、Field/export/normal producer/EXE/150%DPI/双族A-B 均 open。Penpot 无本片新操作，不把旧 r75 rev152 回退为当前；沿用 r84 rev258 最后观察及其52px失败。goal active、passed/accepted=false，0B释放，保留必要失败和源/输入差异。

## 2026-10-25 r85：已落实 6 源 / 两个原 caller / 两个独立 filter

实际增量 `2008c2c4f2e33e35174f4abe12ad23e9b34717e3c4617a9e8a8cbf4fd172a7cf`，只修改授权内 `core/settings/{authority,mutation}.rs`、原 `core/settings/tests/mutation.rs`、host `settings_projection.rs` 与专属测试、`retained/app/settings_window_actions.rs`。原 Preferences 打开和目录完整刷新消费投影原始 u64；原 authority 同锁比较/读取、原 coordinator 同 guard 捕获项目绑定。另外 8 源（包括 r83 bridge）保持；所有旧 hunks 与测试内容经严格内存逆增量还原保留。

六源 rustfmt、diff、反向 apply check 与 change-guard 都为 exit0，guard 0/0/0；这些是静态证据。新增 core3 + host2 静态回归，原限定14源的58定义保留，没有 harness/Cargo 结果。两个新 filter 及 Windows 单 `zircon_editor`、defaults off、features[]、lib/test/locked/jobs1/threads1、物理 E 根和实际 profile 待执行 owner 声明，完整源/资源/native 准入继续 open。原 r35/r37、已终结 operation、旧失败都不重放。

首个 Python 源文件写入 Err22、14 源均未变，失败原件保留；受控 apply_patch 后 6 源匹配候选。完整前后 SHA、14 当前 pin、结构检查、独立 filter 准备及搜索共享合同见 [机器交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-projection-values-pair-r85/handoff.json)。搜索仅盘点实际 8 源与 UE 回调，无越界补丁或死入口。未改 Parent/public 源、主题/Shared/codec 或任何服务。

未取得 originating frame/window/open lifetime、4 page scope、String provider、Numeric caller、官方 export、normal producer/EXE/150% DPI 或双族 A/B 证据。r84 rev258 仅沿用最后观察，无本片 Penpot 新操作；原 52px 长标签盒失败保留。`passed=false/accepted=false`、goal active、0 B 释放，保留必要证据和 2 个小 helper 待原 owner 生命周期处理。回传只报告本次真实实现和独立准备，queue ACK 不计正式收件/准入。

## 2026-10-25 r84：Project分类显隐覆盖及真实三尺寸可编辑板

own file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` / Project page `49ff1071-f83b-4b72-bb6e-043a3326ec82`，rev236→258；原1280板 `c5edfcc8-00e6-4419-a463-3324207d5335` 不改尺寸。两新板由实际原生duplicate分配：900×620 `342ce826-b591-800d-8008-d801c759fd53`，640×520 `342ce826-b591-800d-8008-d801e8de8b52`；非新组件master。新增244对象，973总对象/5页，43 Shared heads、594refs(562Shared/32Card)全有效；Shared5整hash保持，另四页/global保持，原对象除root成员和12显隐覆盖均精确保留。

真实原生 Layers 隐藏三分类行中12个 eye/lock及panel节点，全部由Penpot记录visibility-group覆盖。原12差异仅hidden/touched；文字、几何、原source metadata/shapeRef、34色alpha与字体均保留。UE分类控件只有expander/label/tooltip，此覆盖消除本域不适用操作的设计画面；公共navigation/navigation-leaf尚未发布，未改Shared或内部几何。

三板保存/关闭/fresh context重开整data hash `1cae08b627751d24b7097167dcd9e7f7a99c32f123a009649455c70e71cc7330` 相同；8原图+2实际裁剪/差异逐张复核，原1280覆盖前后1307设备像素变化，三板各自saved/fresh crop差0。640标签框仍溢出52.00000482472478逻辑像素，仍failed/open；未用短标签可见代替长文本/≤1DIP。库更新banner仍显示，本轮未做整库sync/reset，未证明navigation源guard或正式导出。

实际证据 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-project-navigation-r84/`：final-native-audit/responsive-receipt/responsive-reopened/image-comparison/visual-review/source-assets。Parent继续原r73公共variant/leaf/label+源guard集成；本轮不派公共源码变更、不重投Jenkins旧身份。

属性值仍原schema设计fixture文本，无引擎预览/可写控件证明；正式codec/source export→原ZUI→正常EXE、真实150%DPI、四Toolkit/页scope/主题两族A-B及原11要求仍open。当前14 Rust全同r83；无Cargo/intake/QueryOnly/source capture/freeze/桌面/服务/生成/源码副本。goal active，passed=false/accepted=false；0B释放，证据保留，整体cleanup未关闭。


## 2026-10-25 r83：已落源 health caller 单片交接

原 `refresh_settings_persistence_health` 在首个健康属性写入前暂取原值上下文，仅成功或无变化后恢复同一批次；准备/整树刷新 Result 错误保持 None。沿原 health tick、receipt 与 retry 回调生效，正常值刷新负责恢复编辑；原 pending persistence、Retry 身份、scope/generation/coordinator 均保留。

唯一源 `zircon_editor/src/ui/retained/dispatch/templates/workbench/settings_window.rs`：`ff65034edc6b198f85b605dc67ea749a6f3ce47168e9362f0d028ddae4300cfc` → `5cb7988ab371aba87453e7c0b73f58990cc27ec397d255d9c149bdd32e89c158`；新增片已应用，不重复apply。patch `79dae0d5e5abe6df8b8b59129767922bfa0efd0cbc9395393e75776595efaba0`；完整14pins/两回归定义/有限recipe见 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-health-materialization-r83/`。

新增独立 health_materialization_tests 两定义，覆盖成功/no-op、真实模板缺节点错误、失败后 pending 与 authority 不变、健康恢复不冒造值批次、正常值刷新恢复；未运行 Rust。当前14源仅此1份变化、13份与r82相同，旧30定义完整保留，32是静态定义数。rustfmt/反向应用/差异空白/change-guard 均 exit0，guard 0 error/warning/suppressed。

新 filter `ui::retained_host::callback_dispatch::template_bridge::workbench::settings_window::health_materialization_tests::` 单列交原执行 authority；未扩旧payload、未申请/提交新身份或编译。

r74 Numeric 三公共入口仍未落地，本轮不添加孤立 consumer；公共 Rejected/origin lifetime、四page语义树、provider、正式export→原ZUI→正常EXE/实际150%DPI与族验收继续open。Penpot沿r82实际rev236/Shared5历史观察，本轮无API/设计/画面新证据。0Cargo/intake/source capture/freeze/服务/桌面/生成，goal active，passed=false/accepted=false。


## 2026-10-25 r82：实际rev236 Card字体、失败与修正一并交接

实际原20文本绑定Shared5 body/caption/strong（5/10/5），保留五份manifest原文、原Card UUID/main/4copies/32shapeRef、34色引用与alpha、原domain metadata；原69 DIP缩略图和auto-height布局保持，五Card均77 DIP。 own file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` / page `c6a45a2a-60ca-41fb-a1c4-c5164da3cad3` / board `9d912f75-c86c-4b3a-878d-1007947cbf81`，Shared仍5；保存/关闭/fresh重开rev236整hash `39bd214f2d9b04535323e92a982ed182c839c67c52b9dc45f15914c38521d4f9`。当前460refs全有效、另四页整hash保持。

三尺寸原图均保留。640原content比较还覆盖SVG path坐标而失败，不能升级整页源guard；恢复1280后15段文字positionData仍在x309、shape x533，相差224，三张坏图稳定且已保留。原生相同内容编辑为no-op；实际内容临时追加/恢复的首轮因过早hash稳定判断中断，rev207临时字元已沿原编辑路径在208恢复，再处理剩14。现rev236原文全恢复、20字体引用及当前x缓存核对，关闭/全新context重开整data hash一致；修正当前保存态不代表尺寸失效根因已修。

10原PNG逐张已看，两张before/after及raw diff只比较真实Penpot当前字体；坏图不删，fresh两帧crop0设备像素差不当EXE/1DIP证据。证据根 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-plugin-card-flow-r82/`，final-native-audit/visual-review/native-roundtrip-r2-receipt/summary/handoff。这不是公共源patch、完整native source guard或新Jenkins scope；14源码与r81全同，旧请求/失败保留，不再重复投递执行owner。原thumb空槽/真实启停、operation/restart、repeat prototype/wrapper、正常producer/正式export→原ZUI→EXE、真实150%DPI、两族A/B及原11要求仍open。只是本域可编辑设计，非引擎预览或验收。 6 browser exit0，0Cargo/intake/服务/桌面/生成/源副本，释放0B、cleanup=false，goal active，passed/accepted=false。

## 2026-10-25 r81：三个生产弹层入口已落条件物化处理

实际源码已应用，请勿重复apply：原14中仅 `zircon_editor/src/ui/retained/dispatch/templates/workbench/settings_window.rs` 从 `61ef1d5a2c9c755c68e0fe97004f6209c2cf5d7412af64536c7f5b92057802ab` 到 `ff65034edc6b198f85b605dc67ea749a6f3ce47168e9362f0d028ddae4300cfc`；其余13当前pin与r80一致，全部原hunks保留。打开/关闭/滚动关闭在写属性前take原values context，仅原重建成功后恢复相同capture，失败由原正常刷新恢复；不取最新snapshot，不造新状态字段/clock/authority。

原28定义保留，新独立popup_materialization_tests两定义尚未执行，覆盖成功和真实模板缺节点错误撤权/恢复。未来最小单包filter仅 `ui::retained_host::callback_dispatch::template_bridge::workbench::settings_window::popup_materialization_tests::`；root zircon_editor/lib/defaults-off/features[]/Windows/test/locked/jobs1/threads1/物理E，原runtime test-support保留。wrapper无显式rootEditor profile，实际argv/profile/env须执行owner回执。新filter不塞旧r70/r71/r73/r78或r35/r37，不提交新正式请求、不重投旧operation。

证据根 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-popup-materialization-r81/`：精确增量patch/guard digest `d98b25c85eb7d76550d632e80c2ab3afce9d87a4b754d695f190d0a540d59485`，严格逆核对、fmt/diff/reverse/guard均0，guard 0errors/warnings/suppressed。初次仅格式失败原件保留。传播Result错误已处理，通用Rejected报告仍共享路径外；origin/frame/window/open lifetime/semantic authority和四scope、正式export/normal producer/EXE/150%DPI/两族A-B仍open。Penpot沿r80 rev199/Shared5及原溢出失败，本轮无API/桌面/服务/Cargo/source capture/freeze，passed=false/accepted=false，goal active，释放0B。此为实作交接，队列ACK仅投递。

## 2026-10-25 r79/r80：实际26文本 Shared 字体消费，未改共享源

已实际完成独立设计slice：own file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402`，Project page `49ff1071-f83b-4b72-bb6e-043a3326ec82` / board `c5edfcc8-00e6-4419-a463-3324207d5335`，rev192→**199**，Shared5；26可见自有文本绑定真实body/caption/strong，保留其他四页完整hash、所有linked/generated内容/guard、ID/source/全文与颜色引用。保存关闭/新context重开整data `9e67e30b1926c81acbd06e741fb9df7d86adb477a3f97ca28e3b632b6fd0369e` 一致。8原图逐张复核；初次post-update1280与最终帧不一致保留，恢复/重开/第二fresh两帧crop0设备像素差，正确前后改变17595设备像素，均非产品或1DIP验收。

新实际合同事实：当前TS已存在guarded TreeRow全文重排，且生成leftright/right；旧Shared5 copy仍left固定标签。原Project主板只有domain-contract，无原可逆ZUI asset/document，不能直接借正式asset reflow或重签guard。UE20%drawer保持，900树动作仍挤入详情、640标签框超出52px；同源navigation/nonleaf/leaf、真实expand/select/hit、codec/source/ref/guard资格交原公共owner接续，旧r73三源proposal保持未应用，未提交新公共patch或编辑Shared。

r78 14当前源码14/14同；本轮0Rust源码/执行，不把旧28静态定义当新测试或混入r35/r37。详细当前源pin和入口观察：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-tree-label-flow-r79/source-entry-decision.json`；本片产物在 `.../settings-project-typography-r80/`（handoff/summary/visual-review/source-pins.current/saved/reopened/stable-capture-receipt）。新结果只合并回传Parent，不要求普通ACK、全checkout freeze或新用户grant。0Cargo/intake/QueryOnly/服务/桌面/DPI/生成，两个browser/context退出0；必要失败/helpers保留、释放0B、cleanup未关闭。CJKglyph/Engine字体、正式export/product/normalEXE/150%DPI/两族A-B及原11要求仍open，goal active，passed=false/accepted=false。

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

## 2026-10-25 r73 实交付：条件重试已落 9 源，公共导航行仅精确提案

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

r55 交 Parent 的新实际源合同为 [semantic-subtree-contract](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-semantic-subtree-r55/semantic-subtree-contract.json)／[调用点与边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-semantic-subtree-r55/canonical-source-callsite-proposal.md)。54 份源输入只读，不是完整受管 capture。原 root 保持，分类用原 TreeRow，行用原 PropertyEditorRow 与实际类型 value／description/source/reset；完整 field key／scope 与 canonical source instancePath 分开。真实 open/refresh/select/receipt 调用方应在 typed 数据序列化前接同一 template_surface，不新增 authority/store 或孤立 wrapper。

需要 Parent／原 owner 接线的范围明确为：同一私有 bridge 的 typed capture/call、混合 Row source prototype/slot、真实页面 scope/document 与 commit currentness、scope 目标层显示/default catalog、原 typed numeric setter 的真实 surface callback。schema 没有通用 units／precision／large_step；plugin_pages 仅 metadata；复杂类型与完整 Input Bindings provider 未完成。旧 string/chord 等入口按 definition.scope，coordinator.set/clear 没有 expected_generation，不能称页面 scope 或 CAS 已贯穿。原 r42 wrapper 不重放，原 mutation/receipt 保留。

行／文字几何目前仍来自手工 SettingsWindowLayout 与 root painter；新逐字段证据须接同一真实 UiSurfaceFrame 的 arranged node、render command/text line、hit/focus、window metrics 及源身份，Parent 负责测量/painter/Settings native case，原 public owner 负责 slots/codec/ACK/export。只读再次确认 source PropertyRow 5ab622…／r392 staged bf29… 不一致；未改 F 盘产品、LevelEditor guard、nonce、公共 Rust、窗口、主题、store 或服务。 own rev54／5页720对象、Shared2／4masters／34colors／3typo 仍真实存在；本轮未改 Penpot，r54九图和 r53 whole-page hash 失败均保留。

本阶段只有文档／能力接入合同，无 Rust／正式插件测试或导出、无 managed resource prepare、正常 EXE／actual150%DPI／双端验收，passed=false、accepted=false，释放0 B。

r54 新交付是 [最小 Card source 桥接提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/source-bridge-proposal.md)／[20 项精确源输入与字段合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/source-bridge-contract.json)，不是已落源补丁。原 module_plugins_body.zui 的 root、ModulePluginListPanel、slot 和 Focus/Details 事件保持；建议真实 plugin_card_prototype 引用唯一业务 Card，沿现有 virtual_rows／UiSurface 池。原 payload_plugins 已有 metadata，但缺六组 action ID/label 和 detail_generation/locale；同份 typed ModulePluginsPanePayload 已具备它们，直接消费，不造 DTO/store 或补平行 TOML 事务。现有 projection 每次局部建 surface／cache 只保留扁平节点的缺口已定位。

明确交 Parent 的共享最小增量是 TemplatePaneNodeData 的 optional repeated_item_binding，携带完整 plugin_id 与现有 UiVirtualListNodeBinding；不是借 Inspector 字段。源路径／实例路径仍由真实调用点展开产生，不把 plugin_id 塞进 source ancestry。原有界 pane cache 保留实际 surface／key 映射，既有 current binding 校验旧槽事件。仅用实际字体、宽度、locale、内容测量出的统一最大 extent 复用现行 fixed-extent 能力；它与 UE desired size 的剩余空白差异公开，不假定 112 或 116 是正确高度。Parent／原 source owner 核对路径与承载后才能落共享文件；本域没有先应用。

已在同 5 页＋4 弹窗板真实重拍 [9 图及逐项复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/visual-review.json)。当前 own rev54／720 objects，Shared rev2；33 copies／451 refs 全有效。一次非 TTY stdin 关闭与一次错误 aria-selected 的 9 个断言失败保留；最终 9 个真实 source SVG／可见边界核对通过，浏览器原生退出0／双EOF／join，未占桌面。新图逐字节等于已逐张看过的原图；白底、模拟 fields、固定 action、中文裁切／空 value 继续 draft。五页与 Shared 捕获前后精确不变，不关闭 r53 原库同步的 whole-page hash 失败。

Parent05 记录的 21 个 master style ref 字段，实测其 ref ID 全为 null，故当前不是 21 处有效绑定；34 colors／3 typographies 的发布存在，原 trial 的两个显式颜色引用也保留。公共 catalog 当前已变 e4bba09c…，三个相关 descriptor 不变；20 源哈希仅 19 当前匹配（外来 catalog 变更保留），详见 [捕获后源／库对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/post-capture-readback.json)。正式 Field／Number master 仍需原 public owner 导入，不能拿源能力存在冒充 Penpot 可用。无新 Rust／插件测试／EXE／150%DPI／generation；SP-r51 原请求与失败保留，无盲重投。passed/accepted=false，释放0 B。

r53 新进展与失败边界：本族真实rev54／5页／720对象，Shared真实rev2／4主件／34colors／3typographies。原受控页`1ca0591c-2856-49b0-9d7e-eeb076bafe03`、board`4829a7b5-8807-4e66-b34d-b94fbb8807fe`仅接入两处原生颜色引用：board用`editor.surface.0`（`5edd281a-cc68-597c-a6fa-1234e164adc8`），默认Button leaf`73b56c83-5fed-80c5-8008-d6ceb6a369dc`用`editor.surface.1`（`599586e3-022c-500b-ac80-5d9c490139af`）。局部黄色填充覆盖会去掉fill引用，重选原共享样式恢复引用；组件身份未detach。保存/重开实际有效，但不宣称master reset、全局A/B或EXE。

本文件原生库更新到Shared rev2后，三业务页全哈希改变；当前`fullPostSyncPreservationAccepted=false`。受控页原ID/metadata/父子关系保留，SVG坐标最大差0.00006103515625、形状标量最大差0.0000095367431640625；文字位置缓存有约0.38像素高度变化。25个标签覆盖、18个旧动作隐藏和419个源引用/metadata逐项仍有效；其余业务页缺完整节点前像，不做泛化归因、不重签guard、不整体回滚。请公共owner审阅此原生同步差异；[实际审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-style-linked-r53/library-sync-delta-audit.json)保留失败及全部页哈希。

最小公共合同增量：34颜色已核对源`editor_tokens.zui` SHA`dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743`及alpha；公共4主件本身未改、未绑定本次新style。正式导出owner需核对`singleSolidFill`的direct fillColorRef拒绝路径及上游解析责任（仅静态观察，未声称实际导出报错），并证明3个`gfont-roboto` typography与现有custom font资源闭包后才消费文字样式。保留源映射/slot/override语义，不能detach、复制main、忽略几何或代替正式源producer。9图在[复核记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-style-linked-r53/visual-review.json)。本轮无源码/服务/桌面/Cargo/scaffold操作，0B释放；Rust/正式插件/EXE/DPI/两族全局样式验收仍open。以下旧库/旧rev状态均为历史。

r52 最新实际设计：本族 file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402`，Plugin Browser page `c6a45a2a-60ca-41fb-a1c4-c5164da3cad3`，rev49／5页／720对象，累计33个 external native copies／419个源引用节点。四原配置板ID未变：Beta `b11c37ed-3ccb-4653-876f-451d6af0fb44`、New `0abd721a-403d-43a6-a802-5c2e2f9be55d`、Directories `af252d36-d7d1-4f0b-91ed-065d36b6b0de`、Properties `858bc131-0a68-47c8-ab3d-8acf82516aa7`。Shared仍rev1，source和其他四页、原业务组件指纹保持。

原生Assets创建的4个最终Button实例为 Yes `31db645b-62ad-8091-8008-d6fbcb904180`、No `31db645b-62ad-8091-8008-d6fbcc8e7ba4`、Create Plugin `31db645b-62ad-8091-8008-d6fc00e56d6a`、只读Properties Close `31db645b-62ad-8091-8008-d6fc096ca18f`；均链接 Shared file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`／component `6ddc248b-bd35-80c5-8008-d6c8c01096f3`／main `6ddc248b-bd35-80c5-8008-d6c8bd9b057f`／library rev1，8个源refs与metadata逐项对上实际source。保存后两次fresh contexts共8个最终case断言完成；没有detach、复制main或修改公共文件。

实际UE Editor `SChoiceDialog` 的YesNo顺序是primary Yes在左、No在右，组右对齐；Wizard只有右对齐Create Plugin并由CanCreatePlugin控制；Directories正文没有Close。本轮已修正原草稿和误加实例，所有初始/失败原图留存，84个原节点语义/ID/内容/metadata/父级不变，旧动作文本只隐藏。Create实例opacity0.5只是不可执行设计覆盖；SP-C7无只读plan API，生成保持不可用。公共owner所需最小后续仍为：原Button的primary/disabled/AutoWidth及标准Dialog尺寸/clipboard/焦点合同、Field/value/长文本和共享主题、原始源wrapper/codec/正常family producer；不申请本轮桌面、不新造主题或向导authority。Properties readonly是当前能力差异，未实现UE编辑提交。

具体 [r52源与重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/dialog-linked-r52/save-reopen.json)、[逐图复核与未完成项](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/dialog-linked-r52/visual-review.json)。本轮没有canonical Rust/ZUI变更；17 mutation/11 details、正式插件/EXE/双端对比与实际150%DPI继续open，passed/accepted均false。r51已排给原Jenkins owner的同链源候选审阅不重投、不改prepared。以下历史rev35/空库/收件失败仅代表原时点。

r52这份新设计交接只调用一次官方`codex queue --thread`；实际exit1，Parent队列已有100条而拒绝新增（-32600），没有message ID、没有收件/阅读证明。双EOF/客户端退出及原始stderr见 [parent-queue-21.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/dialog-linked-r52/parent-queue-21.json)。这不是r49旧失败的重放，也没有盲重试或借其他identity绕队列；交付内容已在本自有记录中保留待Parent读取。

发送方聊天 `01a16d53-33db-7fb2-8f59-11118c3ec933`；目标 Parent `01a16d4e-0491-7353-84fa-038bd7706546`。

r51已实际核对 Jenkins 原owner的 [r45只读对账](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/ue-settings-20261024-scope-reset-01/r45-original-query-reconciliation/result.json)，SHA `d6104cc7…` 及摘要 `89f69d4e…` 均匹配；没有再次QueryOnly。原reset、r35、r37在其2026-10-24T15:49:42Z有界authority观察中没有正式收件/资源回执；`ordinary_entry_unproven` 是历史查询先authorize的门，不要求用户补许可。当前installed `2790fb…` 与旧 `9c684a…` 分别按实际时点保留。

本域已审阅当前r46 `details.rs` SHA `ea903d46098963df14f446a3f9d67f4e6471cd90b4de6d1917ae8e3715b67f14` / 52033B / 11个静态测试定义，复用相同源的r46格式/guard证据，确认失效焦点归还及旧intent失效、原恢复/启停tuple保留。给原owner的 [确切输入修订审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-source-review-r51/candidate-revision-review.json) 只提两处：r35 `dependencyHashes[details.rs]`、r37 `sourceHashes[details.rs]` 从 `e63b1302…` 更新为上述 `ea903d46…`。两份原prepared SHA仍为 `a2f79cba…` / `5f59d811…`，各16/17匹配；仅在内存中投影该一项修改后17/17匹配，不是完整编译源闭包或新提交。未修改原文件/身份/payload，也未重放原r27或在途操作。原owner仍需透明记录同业务链输入修订，不能以本审阅冒充正式准入。

原prepared的sourceEvidencePolicy仍缺省，旧external_control_forbidden及两份tracked plan保留；`full-file-index-v2` 仅为待原authority审阅的受支持选项，完整external Gitlink/控制/编译源闭包和真实准备回执仍缺。按该owner观察，E空闲17,155,186,688B，v11保留37,580,963,840B，至少差20,425,777,152B（19.02GiB，未计recipe）；没有pool/cache/native writer准入。此为原观察值，非本轮新资源测量。**immediatelyExecutableScope=null**；不降保留线、切盘或清理他人产物。raw3/native3/live1及Parent的metadata3各需匹配真实context，不能借旧reset/numeric/details身份。Rust/EXE/全族passed与accepted继续false；Penpot最新实际观察仍为下述r50核实的rev35，本轮没有API/设计变更、源码、构建、服务或桌面操作。

这份新源owner审阅已通过官方queue排入原Jenkins owner，native exit0、消息 `01a16ed7-67b9-7f03-8c63-484da45c3dc6`；[真实回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-source-review-r51/jenkins-queue-20.json) 保留stdout/stderr、双EOF及客户端退出。只证明入队，不证明已读/接纳输入/源码准备/资源准入或测试终态。没有盲重投Parent已满队列、创建新Jenkins请求或重试历史QueryOnly。

最新为 **r49，rev35 / 5页 / 712对象**。在原Preferences和Project业务板消费14个真实Shared原生实例（Search×2、Tree×12），254个源节点shapeRef/metadata全对；全文件29个外链根/411节点。14份标签覆盖在修复flow后关闭并两次fresh重开保留；14个旧plain text ID/content/metadata保留隐藏。所有原schema文字及User/Project scope保留，World/Plugin Browser/受控页、原PluginMetadataCard及Shared全页/四主件不变。

r50明确卡片source阻塞：再次实际API核实上述rev35且Plugin Browser整页不变；Parent followup-04九源after SHA全匹配，当前guard `4ef0bd40…`（`32ce94d8…`为较早记录）。原component `81548358-cc2f-47cf-856d-39b395bfecfa` / main `d57fd7f1-bf28-4c97-bc6b-0a52ea4696f1` 的domain-component仍明确sourceWrapper=null/officialCodecMetadata=false，无canonical-record；不是可直接冒签的canonical主件。原source `zircon_editor/assets/ui/editor/host/module_plugins_body.zui::plugin_list_slot` / control `ModulePluginListPanel` / slot `module_plugin_list_slot` 是无卡片children的动态Container，asset kind=view。需原source/codec owner给出同源重复原型或可审阅canonical wrapper及原component/main映射，保留plugin_id、原action/operation与绑定；不造另一master或用view根冒充Card。

原字段节点与现有TOML的具体消费join已列 [eligibility.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/card-source-eligibility-r50/eligibility.json)：title `7fa43274…`→display_name，version `7cea9849…`→package_metadata.version，description `fafd3fc1…`→description，`7ec7d52b…`→category/maturity。None需清除旧metadata表现，operation-only恢复动作保留；空字符串不改成虚构值。原设计main为832×116，native仍固定112；这是设计/源码尺寸，未做匹配EXE测量，需原owner的真实换行/测高/scroll合同，不强塞长描述。12项只读事实核对不算编译、官方export拒绝或产品通过。本轮没有Penpot/Rust/资产修改或新请求；完整源指纹/最小合同在上述记录，等待的仍是原source/producer/managed/native边界。

| 业务页 | 原board | 新Search copy | 分类copy数 |
| --- | --- | --- | --- |
| Preferences `b4a1e3ce-9e6a-81fe-8008-d69d60e4e403` | `92bc794d-9419-4284-bb9c-282b99afb641` | `2dbe7f19-8969-807b-8008-d6f2b3872939` | 9 |
| Project `49ff1071-f83b-4b72-bb6e-043a3326ec82` | `c5edfcc8-00e6-4419-a463-3324207d5335` | `1a13650e-62c1-8093-8008-d6f332b20f2e` | 3 |

第一次重开仍有旧标签位置造成的空隙，原图已保留；通过原生检查器对原分类flow的row-gap临时1再回到原0，原生布局重新计算位置，不写子控件x/y。修复后两页各经两次fresh的断言核对，六张真实图全部打开，仍是白底schema草稿。25项源身份/保存/flow检查成立，不作UE/产品验收。记录：[save-reopen.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-linked-r49/save-reopen.json)、[visual-review.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-linked-r49/visual-review.json)。

本轮实际发现native路径为`/api/main/methods/update-file`；旧监听只看`/api/rpc/command`。修正后实际捕获5份native POST HTTP200，保留payload hash/bytes及path；没有auth/query/body。原Preferences首次创建和r48逐请求缺口仍缺，旧26项总false不改、不重放。源码9份Parent after hash在本轮入口全部匹配，followup-04仍SHA `1a8c91fd15d48a4afc4761caeb69532520867bc51119c91b6d0e4ba744398915`；未重写Parent共享源，r42 wrapper继续proposal、mutation/details与原managed身份不变。

TreeRow仍有不适用的cube/eye/lock/leaf disclosure且层级未接，Search固定240，value_slot接受的编辑器及长中文变体未发布。仍沿SP-C1同源语义与C3明确page scope/currentness，交原公共owner/Parent，不加props/主件/主题/store。Shared rev1/四主件/0colors0typographies；正式card/codec/source/family producer、World/undo/restart、原Jenkins执行、正常EXE/实际DPI及双端逐图继续open，passed=false、accepted=false，无桌面slot。本轮没有Rust修改/Cargo/生成/服务操作，四个自有Node/Chrome PID均已核实不存在；释放0 B，必要证据和有界driver保留。

r49官方队列只尝试一次，实际exit1/`queue cannot contain more than 100 submissions`，未入队、未重试、未推定已读。新完整交接保留在本文顶部，原文/CLI/进程回执在 [parent-queue-19.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-navigation-linked-r49/parent-queue-19.json)。r48曾成功入队的消息及更早失败仍各自保留；不能把本次失败覆盖成旧消息成功，也不要求用户手工grant或新建身份。

以下r47/r48及更早交接为历史记录。

r47/r48时点：Parent followup-04 的9份metadata承载源after SHA已逐一匹配（含oracle refinement `f7c6eb14…`），当前静态guard digest `4ef0bd4010bf47a26037490aa347b8109f4cea86b1daf36cb82107a139517919`；这是Parent实际回执，非本族Rust执行。r47旧8路径proposal `aa339913…` 未应用，已由Parent实现取代；不得再套用。并发apply-check失败、已被Parent补齐的fixture观察及队列100条未送达均保留于 [r47合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/metadata-view-bridge-r47/contract.json)。r42 wrapper继续等真实page/semantic caller和generation准入，不添加孤立层。r46 details `ea903d46…`/11定义与mutation `68beb857…`/17定义未改，旧prepared输入未重写/重投。

本族真实Penpot已推进至 **rev27 / 5页 / 458对象**：[Plugin Browser](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60e4e402&page-id=c6a45a2a-60ca-41fb-a1c4-c5164da3cad3)，原board `9d912f75-c86c-4b3a-878d-1007947cbf81` 中使用真实Shared rev1的7个native linked copies，原受控页8份保留。7个新根/91个节点全部source ref有效、canonical source metadata未变；文字覆盖及父flow在两次fresh重开后保留。其他4页、原PluginMetadataCard组件身份及Shared全页对象指纹不变。只隐藏5个原文字占位，ID/content/metadata均保留，未复制main或新建公共组件/主题。

新复制根：Add `b0e3d946-12fa-80ec-8008-d6ed088e0411`，Directories `b0e3d946-12fa-80ec-8008-d6ece1eaad0b`，Search `b0e3d946-12fa-80ec-8008-d6ecaa4e0ecb`；四分类根与各label shapeRef/override见 [r48保存重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-browser-linked-r48/save-reopen.json)。25项源身份/保存值检查成立；native mutation监听未收录请求，原26项总结果false保留，逐请求传输回执仍open。API直接保存HTTP200和最终持久化读回是不同证据，不把空监听当成功。

具体公共消费差异已补到 [组件合同](settings-plugins-component-contracts.md)：分类TreeRow需有适用的展开/图标/可见性/锁定控制，Button须支持UE AutoWidth、Search须能随flow增长；当前240固定根不满足。正式source/slot/codec/native适配由原公共owner统一接续，不改Shared或靠新增painter role关闭。原native112行高及正式MetadataCard、四UE host、World/restart、Jenkins实际Rust、官方ZUI/PNG、normal EXE/DPI/逐图继续open，无桌面slot。三张真实Penpot图均已打开复核，仍为白底结构草稿，passed/accepted均false。自有浏览器/Node均已退出；释放0 B，必要证据及可复用小型driver保留。

本次r48实际交接已由官方 `codex queue` 接收，消息 `01a16eb3-3333-7703-8d88-1a63bd3d3a9c`、exit0；这是新业务板实例/差异消息，旧r47队列100条失败保留，未重投。回执 [parent-queue-18.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-browser-linked-r48/parent-queue-18.json)；仅证明入队，未证明Parent已读、服务操作、Rust准入或桌面时段。固定公共owner/原managed请求与所有未通过项不变。

当前接续为 **r46 / 插件详情失效焦点域内修复**：只改已授原details.rs，保留r37/Parent metadata增量；report刷新撤销Apply/分页资格或清空feature时归还有效焦点。新源ea903d46…，格式/diff/guard通过，详情11定义未执行。两prepared inputs的details hash已旧，未覆盖或提交，精确增量及最小验证见本文r46节。r45原r27 QueryOnly exit1/pending与新输出交原owner事实保留，不换ID/手工grant；正式准入/终态/capture-ready与Rust仍open。Penpot状态沿 **r43 / Shared rev1**：已独立确认四主件发布，原生连库并在本族第五页保存8个linked copies，4项文字覆盖关闭/两次fresh重开保留，own rev22/367objects。原r42 raw reset消费保留；新page-scope宿主接口仍未应用。最新source/slot/copy身份及缺口见本文r43节，旧rev0/空库和r34/r39“未应用”均保留为当时状态。

本记录随首次汇报已由用户指定的 `codex queue --thread` 排入 Parent；CLI 回执 ID `01a16d5c-33c7-7371-9498-b7c583c6cb12`，原文及 stdout 保留在本任务 `parent-queue-01.json`。队列回执不证明 Parent 已阅读。完整盘点见 [本族记录](settings-plugins.md)，后续具体设计见 [设计及接线合同](settings-plugins-design-contract.md)。

遵照 [Parent 最新协调](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)：本族继续独立工作，不启动或恢复 Jenkins/Penpot、不占用共享桌面，不重复向用户询问服务/页面信息。实际本族 file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402`、page `b4a1e3ce-9e6a-81fe-8008-d69d60e4e403` 已分配且 API 可读；可独立使用隔离 API/无头浏览器。r43已核实公共库rev1发布四个原生主件，colors/typographies仍0；此前空库观察仅保留历史。最小slot/事件/焦点/文本尺寸合同和真实试接身份见 [公共组件合同](settings-plugins-component-contracts.md)，不伪造UUID或正式接入回执。

历史 revn=1/vern=0 四空页面与 `penpot-page-setup-r7.json`、`penpot-browser-r8/receipt.json` 保留。接续已在本族文件保存四个 native flex 结构草稿，初次读回 revn=3；16 个设置键绑定 scope/action/source，五个插件 tile 使用实际 manifest metadata，均明确设计 fixture，未读运行状态。四板实际身份见 [组件合同](settings-plugins-component-contracts.md)。普通校验的请求/ACK/readback 位于 `penpot-domain-flow-r10`；原 context 关闭后 fresh context 逐一重开四页面，八次访问均无 JS pageerror。

草稿只组织域内容，不创建公共 master/style，不把文字框当作公共实例或真实控件。公共库仍无已发布组件。八张浏览器图是中性结构草稿诊断，不是正常 EXE、官方导出或 UE 画面验收；两设置页的 initial/fresh 文字栅格不同，字体就绪待核验。公共实例/样式、窗口宿主、正式导出和原生接入继续 pending。

已修复本次草稿的 native flex stored/visual order 错误，保留失败图及 r12 HTTP 200 但未生效的请求。r13 经 shape schema 解码后修复 71 个容器，最终 revn=6，所有 ID/域 metadata 不变。r14 fresh context 在 Project 草稿实际编辑三种逻辑尺寸并核对顺序，恢复 1280×800 后关闭；只验证草稿排序，没有升级正式 LayoutReviewCase、公共组件或产品验收。

接续 `penpot-dialog-domain-r16` 保存 New Plugin、Plugin Directories、只读 Plugin Properties、Beta 确认四个域内容板，实际 ID 见组件合同；不创建公共 Dialog master。源码补充：kind 必选且无默认、native=false；Properties 生成来源只读，package_company 不当作者，content_roots 不当 CanContainContent bool。SP-C6 还需同代 metadata 的生成来源/可写性。没有运行生成器、写 workspace/manifest 或创建 native 请求。

逐图发现的空格定位已改六个流式文本组并保留旧 ID；r19 真实画面仍检出旧 positionData 显示过期文案，失败保留。r20 按原生文本更新合同只使六个改文案节点的缓存失效，未做坐标补偿。r21 两个独立 context 均读取 revn=11，五组标题/版本和一组确认文案的位置/内容检查通过；四张原图逐图复核后确认重复消失，无 JS pageerror。Node/Chromium 已退出并独立复核 PID 缺席。此为草稿缺陷修复，不升级公共组件、产品窗口或验收。

## Parent 统筹的输入依赖

依据覆盖表 OVW-27/40，已补 Localization Dashboard、Config Editor、Device Profiles、Keyboard Shortcuts 的实际 UE 注册/布局/行为与 Zircon 能力映射，见设计合同第 9 节和 SP-014～017。请求沿 C1/C2/C4 复用逐层配置投影、chord capture/冲突焦点与整 map mutation 合同；Localization target/job 与可编辑设备 profile authority 仍需领域 owner 决定。没有新建这些窗口的空板或假工作流，没有触碰 i18n/keymap/RHI/公共 source。

1. 真实 Penpot 本轮 runtime/URL、可复用 file/page/library/component/style ID、当前插件发行物及 hash、官方保存/重开/导出入口。避免本族另外启动服务或另建主题。
2. 桌面输入/正常产品启动/原生捕获排队入口、持有者与轮到本族的通知。本族尚未取得输入或捕获 ownership，也未发送键鼠事件。
3. 当前正常 editor 产品与资源加载身份、EXE hash、可复用 Jenkins output/cache 与项目/场景材料。Parent 提供的历史候选实际名为 `zircon_editor.exe`；本族已只读核实字节身份，不采用历史主窗口截图作为新验收，也不据此认定当前本族部署闭包。
4. 以下共享变更的明确 owner、接口和整合时点；本族当前不改 catalog、tokens、Starship、全局 native painter role、菜单和窗口注册。

## 具体提案及受影响消费者

| 提案 | 当前证据与所需合同 | 共享 owner / 消费者 |
| --- | --- | --- |
| SP-C1：设置表面的可编辑语义投影 | `preferences.zui#WorkbenchPreferences` 只有根，内部由 native settings painter 画；Penpot `zui-native-painter-role.ts` 没有该角色。复用分类树/PropertyEditorRow/value slot 的实例，保留 key/scope/source/event。现有 PropertyEditorRow 只有单 value slot、名称列 max=105、固定行高且 clip；请求同一组件的 description/source/reset slot 或 variant、label wrap/内在行高合同。Parent 决定公开语义子节点或 painter projection；不能 detached/bitmap。 | Parent 公共投影/组件 owner；Preferences、Project、World、其他 Details 消费者。 |
| SP-C2：四窗口的页签/停靠入口 | UE EditorSettings/ProjectSettings 为 Nomad，PluginsEditor 为 TabSpawner，WorldSettingsTab 为 LevelEditor tab。请求共享窗口注册和 focus-return 合同；本族只实现真实 body 及 scope/data/action projection。 | Parent Workbench/Dock/menu owner；原 Preferences 入口、Module Plugins、Level Editor。 |
| SP-C3：共享主题设置及两族传播 | 复用 `editor.appearance.design_tokens`、SettingsMutationCoordinator 和现有 token 安装，不创建第二个主题 store。请求可编辑字段、style/token identity、precedence、local override/reset 合同与主工作台同步 A/B 捕获时隙。 | Parent tokens/Starship；本族设置、主工作台至少两族。 |
| SP-C4：通用确认/错误/重启提示及安全返回焦点 | 需要真实依赖启停确认、不可写配置、pending/durable/failed/retry、requires_restart 生命周期。复用已有 Dialog/Notification 组件；Restart Now 必须走正常 dirty-doc close/relaunch 协议。 | Parent 模态/进程生命周期；设置、插件、其他配置消费者。 |
| SP-C5：World 字段及事务边界 | 有 ProjectSceneDocument 与真实 Scene，但尚未找到对应 World Settings 注册/属性模型。需确认场景可编辑字段、undo/save 与运行时应用 owner；UE 专属未实现字段保持具体 gap。 | Parent 场景/属性 owner；World Settings 与活动场景/Inspector。 |
| SP-C6：Plugin status 补带已有 metadata | `PluginPackageManifest` 已有 version/category/description/maturity，`EditorPluginStatus` 未带出；建议沿原状态发布链补齐字段及 catalog/source revision，卡片与分类复用同次快照。图标/文档 URL 无真实字段时保持缺失；生成的 plugin.toml 不可直接写。保留原 enablement report 的完整 operation tuple，不新增平行 enabled/restart store。 | Parent 公共 host report/API owner；Module Plugins、Plugin Browser、打包与功能详情消费者。 |
| SP-C7：外部目录与 New Plugin 的真实 authority | 已确认 `<project>/zircon_plugins` 固定 native 根和 bounded discovery/refresh；任意多根列表 setter 不存在于已查 host 入口。`tools/cargo/src/plugin/scaffold/mod.rs` 有真实 importer/system/editor + native 脚手架，但要求引擎 first-party workspace，写 plugin workspace、两 catalog 和 app 六处共享文件；只尽力回滚。需公共 generation owner 的只读计划、完整变更集/receipt/恢复合同，不能直接从 wizard 调命令。 | Parent 插件架构/Jenkins generation owner；New Plugin、目录、发现/live host、first-party catalogs、zircon_app Cargo。 |

## 本族独占工作范围

本族初期只写 `docs/plans/penpot-ue-window-suite-20261024/settings-plugins.md`、`settings-plugins-parent-handoff.md`、`settings-plugins-design-contract.md`、`settings-plugins-component-contracts.md`；证据位于 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins`。没有 claim 全局目录，不修改其他会话的记录。Penpot 后续只编辑实际分配的本族文件；公共文件不写入。

待真实 Penpot 首阶段及共享接口交接完成后，拟接续的明确路径为：

- `zircon_editor/assets/ui/editor/components/workbench/floating/preferences.zui` 和 `zircon_editor/assets/ui/editor/host/module_plugins_body.zui`（原资产身份及消费者保留）。
- `zircon_editor/src/ui/settings/settings_window_projection/**`、`zircon_editor/src/ui/settings/persistence_health_projection.rs`、`zircon_editor/src/ui/settings/action_ids.rs`（本族投影与本地化状态）。
- `zircon_editor/src/ui/host/editor_event_runtime_access/settings_projection.rs`、`settings_mutation.rs`（明确 scope 的页面投影/reset guard，避免误删 inherited User override）。
- `zircon_editor/src/ui/retained/app/module_plugin_projection/**` 与 `plugins/**`（真实 body 和完整事务状态；公共 host report、插件导出与捕获协议仍先交 Parent）。

上述是后续候选范围，不表示已经修改或排他占有；实施前再次核对现有 diff/归属。实际 `EditorPluginStatus` API 扩展、catalog/tokens/Starship、公共 painter、菜单/Dock 与重启协议均由 Parent 先协调。

SP-C7 的具体共享路径为 `zircon_plugins/Cargo.toml`、`zircon_plugins/first_party_runtime_catalog/{Cargo.toml,src/lib.rs}`、`zircon_plugins/first_party_editor_catalog/{Cargo.toml,src/catalog.rs}`、`zircon_app/Cargo.toml`，以及公共 scaffold/发现协议。本族只记录了该写入闭包，未修改或生成这些文件。普通项目 SDK scaffold 与 UE 插件模板并不等价，仍需架构 owner 明确产品范围。

`production/extension_plugin_manager_workspace.zui` 中的固定示例及其 `Not implemented (preview)` 路由已列为 SP-006，不按完成界面提交。当前没有产品资产改动，没有申请 commit/push；正式组件化 Penpot 设计、正常 EXE 和双端截图验收均未完成。

## 本族正常窗口与捕获接续

r25 的具体源闭包依赖及捕获提案已通过官方 CLI 投递，ACK `01a16dac-6e19-7611-adb5-1fba1dab8d27`（`parent-queue-09.json`）；仅证明排队，不证明已读或授予桌面时段。

接续独立设计已形成 `PluginMetadataCard` version 1 / component `81548358-cc2f-47cf-856d-39b395bfecfa`，main 为原 AI tile `d57fd7f1-bf28-4c97-bc6b-0a52ea4696f1`，四 copies 和全部 40 节点 ID 保留，最终 revn=15。它只是本域 metadata 组合；不创建公共 primitive/style/主题，也没有独立 ZUI wrapper。真实来源、copy ID、动态 slot 及 C6 接线边界见组件合同和 `penpot-domain-component-r26`。

保留了重开后文字全变 AI 的实际失败：原生 text partial sync 除 content-group 外还要求 text-content-text 子组。按实际 `libraries.cljc::text-change-value` 修复后，两 context 的 API/SVG/UI autosave 后各 20 字段正确，40 个 shape 对应稳定，无 JS pageerror，六张修复图逐图查看。请公共 pipeline 继续保留原生 text override 子组合同；本族没有修改公共实现。官方导出、真实 engine operation、插件/托管 Rust 和完整验收仍 pending。

本轮独立核实 N57 候选 EXE/DLL、启动环境、producer 配置/模块和 R94 项目/场景，10 项文件哈希与原 descriptor 匹配，已记录 producer 合同的 71 项输入也全部匹配。只复用既有文件作只读身份核查，没有启动产品/producer、复制源码或重放 `--create-project`；当前本族源/资源/数据闭包仍未 admitted。报告位于本任务 `normal-capture-readiness-r25.json`，完整 board→原始资产、节点/control/import/事件/slot 与打开命令映射在 `source-asset-map-r25.json`。

真实入口为 `editor.settings.open`（Always、无默认快捷键，OpenSettingsWindow→SettingsWindowOpenRequested→原 native painter floater）和 `view.plugin_manager.open`（`editor.module_plugins#1`、LeftBottom Drawer、ModulePluginsV1/HybridNativeSlot）。这两个旧入口不等于已新增 UE Preferences/Project/World/Plugins 独立宿主。Preferences 保留原 `WorkbenchPreferences` 身份；Plugin body 的 18 个节点、18 个 control 和五条事件均需保留，动态列表 slot 需实际投影补齐。

请原 pipeline owner 接续具体限制：`zui-layout-real-workbench-input.ts::prepareRealWorkbenchInput` 对 `--source-closure` 仍无条件断言 canonical Level Editor；已部署 `normal-workbench-case-producer.mjs` 同样固定该 owner。请求受支持的本族 source closure/review-host/正常 producer 入口和现行部署身份，不能改写本族 sourcePath 冒充 Level Editor，也不能放宽 nonce/七代数/像素关联条件。此项接续 Parent 已交的公共 pipeline 任务，未改公共工具。

本族捕获请求提案为 `capture-handoff-r25.json`：八个实际 board、原始资产指纹、真实项目候选、三种 96 DPI 逻辑尺寸和 1280×800@144 DPI（物理 1920×1200）、通用/专有交互，以及 25/35/25 分钟分批预估均已列明。请在公共实例/正式导出/产品资源与案例实际就绪、原输入 owner 释放后排具体时段；目前不请求抢占桌面。PID/HWND/case hash/nonce/cursor 留空并说明尚未生成，`caseProduced=false`，所有 `passed/accepted=false`。API/headless 权限不扩展为原生输入或系统 DPI 权限。

## followup-02：源码候选、公共提案与实际失败

已接收 Parent 2026-10-24 11:46 的最小源码移交，14 个相关 Rust 文件起始 tracked diff 为空，baseline 见 `scoped-reset-metadata-r28/baseline.json`。当前除四份本域文档，仅修改原 `zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs`：默认 reset 采用 catalog scope，新增显式 scope 入口；先校验原权限，确定目标层缺失则 None，其余经原 coordinator 清除并返回真实 receipt/error。保留现有调用签名、不新建 settings store/主题。四宿主/逐层 presence/禁用状态/typed numeric commit 未接线，C1/C2/C3 不关闭。

请 Parent 协调的精确公共 diff 尚未应用：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/scoped-reset-metadata-r28/plugin-public-contract.proposed.patch`，SHA-256 `a1e7459fbb6cc059834119fe91cb945a09159e79bb130aaf6e9f17c96d186e06`。新增 `EditorPluginStatus.package_metadata: Option<EditorPluginPackageMetadata>`（version/category/description/PluginMaturity）及 reports/mod.rs、exports/mod.rs、host/mod.rs 既有 reexport；六处 struct literal 迁移路径和 same snapshot/live clone/missing=None 规则见同目录 `plugin-public-contract-proposal.json`。生成来源/可写性尚无已证明字段，不推断，不在 UI 另读 TOML，不造 revision 或丢 enablement tuple。

官方消息第 11 次投递 **失败**，`parent-queue-11.json` 记录 exit 1：`queue cannot contain more than 100 submissions`。没有 ACK、未获公共扩展批准；本节为可独立读取的交接，不重复投递或改聊天数据库。

Jenkins 原请求 `ue-settings-20261024-scope-reset-01` / operation `validate-settings-scope-reset-01`，driver `fc55322804fb53ab3e39f996c848ef5e5f3907497d3f38167b7095244b2a7399` / runtime `start-1792836412672596900`。最初五项源 SHA-256 `96b26461be91c193a5e23bf24eeb1edbd65fc93054afbe3da7795c6f48f2500e`；现有模块入口、zircon_editor、defaults off/features=[]/lib、filter `ui::host::editor_event_runtime_access::settings_mutation::settings_scope_reset_tests::`、单 job/thread、Cargo test/adapter dev profile、E 根均已声明。实际输入捕获返回 `external_control_forbidden: External checkout controls are not compiler inputs`；同一原 identity 的 QueryOnly 返回 `Current session authorization is pending`。完整 scope/identity/stdout/stderr/客户端退出在 `jenkins-settings-projection-r27`。没有编译或测试通过回执，不本地 Cargo、不改输入保护或控制配置、不启动服务。

补充 default 无覆盖兼容边界后当前为六项回归候选，源 hash 已变化；原五项 identity 和失败保持原样，当前候选未重投。请 Jenkins 原 owner 接续外部输入保护及原请求收件授权对账；修复后按新源码身份提交六项，不能把原失败改成新源码。当前格式/diff/静态 guard 结果见 r28；它们不代替托管 Rust、官方导出或正常 EXE/双端验收，整个族 passed=false/accepted=false。

r28 当前源 SHA-256 `26421e53f54c13cd2ca957da459744ca652e046d27076d145a47b604c0dd6cf0`，最小 review patch 为 `scoped-reset-metadata-r28/settings-scope-reset.patch`，SHA-256 `8bbeb8593948c25b17f9591bceff9445eff3f732337f1339d8d3d04added1844`。格式、diff、change guard 与源码/文档结构共七项静态检查通过，不计 Rust 执行。两个提交/查询 helper 和两个客户端均已等待退出，11:59:23Z 独立复核四 PID 均缺席；未结束或杀死任何服务。

输入保护的具体只读观察：实际 installed capsule `baac70c97872d3395c5c422fa44a7cdaa481a8ac4f62820d1cc2ac099ad0ac08` 的 `source_preparation.py:1537` 拒绝不在 managed plan 中的普通外部控制路径。`E:/Git/zr_vm` 当前 tracked 控制路径为 `.codex/plans/20261018-ssa-contract-freeze.md`、`.codex/plans/20261018-ssa-phi-identity.md`；是否与现有 managed plan 对账由 Jenkins owner 确认。完整文件名观察与 guard 指纹在 r27 的 `external-control-path-observation.json`、`input-guard-diagnostic.json`。没有读取这些控制文件内容、删除/移出外部源码或放宽保护。

## r30：接收较早交接后的当前身份与 set 合同

用户转交的 10:20 revn0/root-only 是初始观察，未覆盖 r26/r29 后续真实文件；本域四份文档没有“尚未分配”条目，实际 file/page/八板/component/copies 和历史失败继续保留。r29 于 12:01:09Z 读回 own revn15/四页/一个主件、公共 revn0/0 component/color/typography/无发布外链，未回退设计或另造公共组件。

沿 Parent 最小 SP-C3 交接，本轮仍只改原 `settings_mutation.rs`：增 typed `set_setting_in_scope(key, scope, SettingValue)`，六类旧生产 setter 共用原 catalog/schema/coordinator，既有签名、default scope、action IDs 与 receipt 保留。Project scope 无权写 User-only 的 design_tokens；范围/非有限/错误类型在 mutation 前校验。Session 写入和 reset 保持 inherited User，重复写入不发布新代数。新增六项 set 回归与旧六项 reset 合为十二项，私有模块/filter 为 `ui::host::editor_event_runtime_access::settings_mutation::settings_scope_mutation_tests::`。

当前源 SHA-256 `4966571d0e75e2a6f7294d8794d14812623c62278c124f077569ddaf0d8cc54a`；本域 `scoped-set-contract-r30/settings-scoped-mutations.patch` SHA-256 `5dc7eea1c4919a836b3c504dafdb8d52456600145c10d1d97ff7e9f17ad28eaa`，API/receipt/测试边界在同目录 `typed-scope-contract.json`。格式/diff/静态 guard 通过，0 errors/warnings/suppressions；没有托管 Rust 执行回执或新请求，不改原五项 identity。installed capsule/driver/runtime 未变化，原输入捕获保护失败和 QueryOnly 授权 pending 均保留，不反复重投或本地 Cargo。

当前 API 是域 mutation 入口；页面/语义子树及 generation 校验、presence/reset 禁用态、raw numeric 文本提交、undo/持久化重开仍待真正接线。没有新增 expected-generation CAS，不宣称旧事件闸已经实现。公共 metadata DTO 提案仍未应用，C6/7、共享样式和正式源导出依旧由 Parent 协调；无桌面输入/系统 DPI/原生捕获时段，全部验收继续 open。

本次新增源接续报告的官方 queue 第 12 次投递也实际失败：exit 1，`queue cannot contain more than 100 submissions`；完整新消息及 stdout/stderr 在 `parent-queue-12.json`。没有 ACK，不称 Parent 已收到/批准 r30；不继续重复塞队列。此节与 r30 review patch/API 合同作为文件交接入口。

2026-10-24T12:08:46Z 再次认证 API 读回 own revn15、八板全部存在、原生主组件及四 copy 的 componentId/shapeRef 均有效；公共库仍 revn0、0 component/color/typography。当前设计 revision/对象未变化，复用原 r26 保存/关闭/重开与逐图证据，不重复改动浏览器或将 API 存在性算作产品画面通过。实际观察为 `parent-message-reconciliation-r31.json`。

## r32：Actor 额外覆盖的设置收件

已接收用户转交的 [45 项覆盖分工](parent-window-coverage.md)，并与 [资源 08 RT-E06/RT-E09](08-resource-tools.md) 对账。新增具体消费合同在 [设计合同第 10 节](settings-plugins-design-contract.md#10-额外覆盖的跨域消费合同)：

- 本族牵头 OVW-27 的配置/生产入口及 OVW-40 的 Config Editor、Device Profiles、Input Bindings。已有 locale、coordinator、typed scope set/reset、keymap/when 冲突分析只按各自真实范围复用；target/job、可编辑命名 profile、逐层 raw/currentness、捕获/原子确认及 undo 尚未接通的继续 open。
- OVW-35 Struct Viewer、OVW-38 StringTable 的文档/作者、key/引用事务归资源。本族需要类型/full-path 与真实资产/key/reference 身份回传，保留 revision、字段准入、取消/失效及原焦点；不持久化显示标签或临时字段槽位，不实现类型编译/资源 rename 事务。
- OVW-28 设备/部署、OVW-39 Source Control 主体归 Parent/Jenkins。本族只消费核实后的 profile 引用、来源/可写状态与请求结果，不在设置表单执行部署、checkout、Git 写或 changelist 管理。

两项需要公共 provider 明确的实质边界：`ReflectSchemaResponse` 与 `ReflectSchemaCatalogSnapshot` 只含内容 fingerprint，没有实时 generation；Parent 须关联实际 World/TypeRegistry 上下文，不能从相同 fingerprint 证明旧选择可提交。快捷键须读取**目标 User 层原始 map**再构造整 map 候选；Session winner 的 resolved map 不能写回 User。逐行 reset 删除该 path 以恢复继承，显式解绑/冲突接受写 None tombstone，整 key reset 是不同意图。

r32 仅改原四份本域文档并保留源码输入指纹/结构检查，不改任何 Rust、公共文件、资源 owner 文档、原 ZUI 或 Penpot。现有 Rust 仍为 r30 SHA-256 `4966571d0e75e2a6f7294d8794d14812623c62278c124f077569ddaf0d8cc54a`，十二项候选未执行，原 Jenkins 输入失败和 QueryOnly pending 不改写。本轮文档/源码核查无需重复已知受阻的编译，也不以静态检查代替插件、托管 Rust、正常 EXE/双端逐图验收。

官方 queue 最近两次因 100 条上限 exit 1、无 ACK，未观察到容量恢复，故本次不重复塞队列；本节与 `coverage-provider-contract-r32` 是可读取的增量交接，**不是投递成功/Parent 已读或接口批准回执**。原 file/page/八板/component、旧失败及所有 passed=false/accepted=false 保留；不启动服务、不占桌面、不生成额外空板或虚构 provider。下一项独立工作为既有 projection 的字段/状态接线映射，公共变更仍先交 Parent 对账。

## r33～r35：第二次源码移交的精确增量

已再次读取“设置与插件第二次接续”和 `settings-plugins-followup-02.json`，不将其中历史初始 hash 覆盖后续状态。r33 baseline 对账原 14 文件：本族 `settings_mutation.rs` 为 r30 候选，另外 13 项身份一致、tracked diff 空。当前仅继续修改该原文件，World/Dock/restart/公共 master/catalog/tokens、SP-C7 generator 与正式 UI 资产都未动；保留 r26 原始失败图和 Parent 所阅两图为结构草稿的结论。

### SP-C3 当前已写入的域候选

新增 `EditorHostEventController::set_numeric_setting_in_scope(key, scope, text)`：按真实 schema 解析 Int/Float，再经原 catalog 权限、schema 与 SettingsMutationCoordinator；有效值保留 receipt，非法/溢出/非有限/越界不写入或 clamp。旧 typed set/reset、六个生产 setter 的签名、默认 scope、原事件/action 身份保持；Project reset 只清 Project，不追随 inherited User/Session。三个新回归与旧十二项共十五项，均未执行；UI raw text/input、scope/page/currentness、generation CAS、undo/保存重开继续 open。

- 当前源：`zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs`，SHA-256 **`57dc7a2a36aa883c1f7f1215d3f1ce94d790fb52c4eb9f20221276551cc26602`**。
- 精确相对 HEAD diff：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/numeric-scope-adapter-r33/settings-scoped-mutations.patch`，SHA-256 **`8a0cd4b90f5efe9f2550857b347a958e73642b12b383cf828972bbdba9608f8d`**。
- 格式/diff/change guard 实际 exit 0；guard **0 errors / 0 warnings / 0 suppressed**，分别在 r33 `format-final.json`、`diff-check.json`、`change-guard.json`。初次格式差异保留于 `format-initial.json`；这些不代替编译或测试。

### SP-C6 提交协调的完整提案，尚未应用

精确提案：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-metadata-chain-proposal-r34/plugin-metadata-chain.proposed.patch`，SHA-256 **`2810e0c2ac85f6ab7e2468d686460ef33c57279acb2b71ae2d52eba35ebb0292`**。`proposal-contract.json` 列出全部十个实际路径/基线 hash；涉及公开 `EditorPluginStatus.package_metadata: Option<EditorPluginPackageMetadata>`、三个原 reexport、两处原状态生产文件、三组 fixture 与 native product 测试。保留旧 r28 四文件提案，不把它当完整迁移。

builtin/native-only 各自从同次构造已有 package 复制 version/category/description/PluginMaturity；missing selected native 为 None。原 status Arc/live clone 保留 metadata 与完整启停 tuple、依赖/feature closure，不在 UI 另读 TOML、造 revision 或另存 enabled/restart。mixed builtin+native 保留原 builtin 名称/metadata 优先级，不能认定展示版本就是加载 binary 版本；如需改变该优先级或新增 provenance，由 Parent 协调原 host owner。作者/图标/URL/生成来源及可写性仍无已交字段。

提案扩充缺包断言并增加两个回归：native 构造的快照不随原 manifest 对象后续修改、live 失败保留 metadata/无关行/完整 enablement outcome 和 reverted 文案。十文件候选均经 rustfmt stdin 解析，`git apply --check` 实际 exit 0，原十文件 hash 均未变；这不是已应用 checkout 的 guard/编译或 UI 消费完成。最初 fixture 替换锚点失败在产物中保留，发生于内存准备阶段，没有产品写入。按本次移交的明确边界，**等待 Parent 对该精确公共 DTO/reexport/构造迁移的协调，不自行应用**。

### 最小托管测试声明与实际阻塞

当前源码应跑 **zircon_editor** 的设置 host mutation adapter 与必要依赖，defaults off、features=[]、lib、无 bins/integration/all-targets、单 job/thread，完整 filter `ui::host::editor_event_runtime_access::settings_mutation::settings_scope_mutation_tests::`（十五项）。Cargo test profile，当前 pinned parser 的 recipe profile 默认 dev；无 release，物理 build root **E:/cargo-targets**。精确已准备命令如下，**未执行/未提交**：

```powershell
.\tools\dev\dev-module-build.ps1 -Package zircon_editor -Action test -Features @() -LibTests -TestFilter 'ui::host::editor_event_runtime_access::settings_mutation::settings_scope_mutation_tests::' -Jobs 1 -TestThreads 1 -JenkinsIdentityFile 'E:\cargo-targets\zircon-local\reports\ue-window-suite-20261024\settings-plugins\jenkins-route-reconciliation-r35\current-candidate-identity.json'
```

新准备 identity 是 `ue-settings-20261024-scoped-numeric-01` / `validate-settings-scoped-numeric-01`，绑定当前 r33 源 hash；只是一份未投递输入，不是授权/队列或 Jenkins job 回执。原 `ue-settings-20261024-scope-reset-01` 的五项 hash/filter/identity 和输入失败完整保留，不能用原请求验证新十五项源码。

本轮仅通过原模块入口对**原 identity/原五项 filter** 做一次 QueryOnly，实际 exit 1：`ordinary_entry_unproven: Current session authorization is pending`；客户端 PID 23524 与记录中的辅助 PID 已等待退出、独立复核缺席，原 query 完整 stdout 在 `jenkins-route-reconciliation-r35/original-query-receipt.json`。没有新编译、测试或伪通过回执。

12:26:59Z 观察 installed capsule 已由 Parent route-02 的 `baac70…0ac08` 更新为 **`cdd14500fa39088ae2c9870f62825acc2e526ba647d24d77cd6287722b5d0f31`**；driver `fc55322804fb53ab3e39f996c848ef5e5f3907497d3f38167b7095244b2a7399`、runtime `start-1792836412672596900` 不变。新 manifest/11 项当前源与 pinned client hash 全部核对；只读 diff 显示 client 变更收窄 material manifest 配方及 E 根，**settings 输入保护源码未变化**（`source_preparation.py` SHA-256 `2a38f55962b5e7c9f9914f7e6dbe637ec41522b1852d1be824ceada1c246ff77`）。原外部 `.codex/plans` 捕获失败/管理计划对账仍交原 Jenkins owner，不删控制文件、改保护或借 material 身份绕过；不以入口新版本推断修复已验证。新快照详见 `current-capsule-verification.json`，与较早 route/query 观察分开保留。

正式 Penpot 公共实例/源 owner 导出、四宿主、真实 restart/World、family producer/EXE 与桌面时段继续由 Parent 接续。本轮未改 Penpot/公共 ZUI、未启服务/执行 generator/输入或捕获，passed=false、accepted=false；自有临时客户端已结束，释放 0 B。官方消息最近第 11/12 次仍是上限失败且无容量恢复观察，本次交接写入本节和 r35 delivery 状态，不伪造 ACK/Parent 已读或公共接口批准。

## r36～r37：原插件详情的恢复态修复

继续在 followup-02 明确允许的 `module_plugin_projection/**` 中处理实际域缺陷。修改前 `details.rs` tracked diff 为空、SHA-256 `2159c20fa6800c8d26c8a865aa9eb855f4223a43d5d7d6e1b65690368c993c34`；基线与根因见 `plugin-detail-recovery-r37/baseline.json`、`diagnosis.json`。列表消费同次 enablement recovery 禁用变更，详情却只按 feature flags 生成 Apply intent/Tab 项。现统一详情 action/intent、键盘 Apply 和 take_action 的准入，使用原报告，不增加 DTO/store 或改启停事务。

同插件四种 recovery 都不产生变更 intent，原 Apply 焦点回 features；保留真实选中、feature flags/诊断，追加原 operation.status_message。解除恢复后仅新的一次性 qualified intent 可执行，另一插件不受影响。三个新增回归加五个既有回归共八项；夹具静态复核曾纠正 action 编码和 NotCommitted+RestartRequired 的错误组合，未运行测试，不把这些修正记为 red/green。原十五项 mutation 回归也未执行。

- 当前详情源：`zircon_editor/src/ui/retained/app/module_plugin_projection/details.rs`，SHA-256 **`7a8263ede2cf3efaafb95ef67d9421aaa966718ea1107f7ca36a85b1de153792`**。
- 详情单文件 diff：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-detail-recovery-r37/plugin-detail-recovery.patch`，SHA-256 **`20c3881838c67213029f192d53da92f5e48f73579187747dc72e821bd20b768d`**。
- 当前两份域源码相对 HEAD 的完整 diff：同目录 `current-domain-source.patch`，SHA-256 **`6b57b1f4a12cf67610efb622e72f9a93eb8c990be3b0d7540f54b9888f07ea42`**；mutation 源仍为 r33 `57dc7a2a36aa883c1f7f1215d3f1ce94d790fb52c4eb9f20221276551cc26602`。
- 两文件 rustfmt/diff/guard 实际 exit 0，guard **0 errors / 0 warnings / 0 suppressed**，input.patchDigest 与完整 diff 相等。初次格式差异保留，不代替编译。

r34 公共 metadata 提案 SHA-256 `2810e0c2ac85f6ab7e2468d686460ef33c57279acb2b71ae2d52eba35ebb0292` 的**字节未变、仍未应用**。现 `git apply --check` 仍 exit 0；十项基线中九项未变，只有本域详情由上述 baseline 前移至当前 hash。`proposal-compatibility.json` 保留新旧身份，不能把 r34 原始 baseline 报告改写为新源，也不能把 apply-check 当批准/编译。Parent 可将公共 proposal 与此域 diff 一起审阅。

八项详情回归的精确准备范围为 **zircon_editor lib**，defaults off/features=[]、当前 Windows host、单 job/thread、Cargo test profile/现有 adapter dev 默认、物理 E:/cargo-targets；模块只取 retained plugin detail projection 与必要依赖。命令如下，**仅准备，未提交/未编译**：

```powershell
.\tools\dev\dev-module-build.ps1 -Package zircon_editor -Action test -Features @() -LibTests -TestFilter 'ui::retained_host::app::module_plugin_projection::details::tests::' -Jobs 1 -TestThreads 1 -JenkinsIdentityFile 'E:\cargo-targets\zircon-local\reports\ue-window-suite-20261024\settings-plugins\plugin-detail-recovery-r37\current-candidate-identity.json'
```

准备 identity `ue-settings-20261024-detail-recovery-01` / `validate-settings-detail-recovery-01` 绑定两份当前源码 hash、原 driver/runtime；不是已投递身份或授权回执。十五项 mutation 仍用 r35 自己的窄 filter，不并成整个 ui filter。无 Jenkins owner 输入计划/收件修复新回执，保留原五项失败及 r35 QueryOnly pending，不重放已知失败、改保护、借 material 身份或本地 Cargo。

12:40:07Z 独立 API 再核实 own rev15/四页及原 PluginMetadataCard；公共库仍 rev0/单根/0 component/color/typography。保存/关闭/重开沿用未变化的 r26；未发起新浏览器、编辑公共资产、服务、generator 或桌面输入。Parent 12:39 最新交接未新增本域桌面/DTO许可。官方消息最近两次仍是容量上限失败，无新容量恢复观察，本次文件交接不冒称投递成功/已读。插件、Jenkins Rust、正式导出、四宿主/family EXE、实际 DPI 与逐图对比全部 open，passed=false、accepted=false；本轮有源码进展，保留 goal active。

## r38：正式交付依赖复核，未有新产品进展

上一轮 r37 是实际源码进展，本轮没有新增产品源码、Penpot 设计或验证通过。12:48:57Z 通过官方 `codex queue` 发送一份新的 r33～r37 完整精确 diff/hash/guard/公共提案交接，实际 exit 1，仍因 `queue cannot contain more than 100 submissions` 拒收；不是 ACK。原消息和已退出客户端身份保留于 `parent-queue-13.json`，不重复重放或另建消息通道。

12:50:58Z 认证 API 读回 own rev15/四页/原主件、公共 rev0/单根/零组件/颜色/字体样式/未发布；两份响应哈希与 r36 一致。`dependency-audit-r38.json` 记录实际 camelCase API 字段，初次读取误用 kebab-case 导致页字段缺失的报告单独保留，未把缺页解析误认成文件内容变化。已读取 Parent 更新记录，仍无本域公共 DTO 决定、family 导出或桌面授权。

已复核获准的原 settings projection/actions/template：现 `scope` 是 catalog 默认层，resolved source 不是页面 edit scope；没有同代目标层 presence/raw 与事件 generation 合同，不能直接把现有值套上 Project 标签。公共语义接口与必填 DTO/原宿主接线仍须 Parent 的明确交接；本族不另建平行投影/store。持久化健康已有 notifications 与原 surface action 调用 `prepare_settings_persistence_health`，没有证实新缺陷，故未为继续工作而任意补丁。Jenkins capture guard 源哈希仍为 `2a38f55962b5e7c9f9914f7e6dbe637ec41522b1852d1be824ceada1c246ff77`，原失败/QueryOnly 待收件保留，无新的有效编译 receipt。

下一步的真实准入仍是：原公共 owner 交付可消费的基础实例/style、Preferences 同源语义子树与本族 source closure；Parent 处理 r34 精确公共 metadata 提案/页面 scope 与原事件合同；原 Jenkins owner 对账 capture/session receipt；family EXE/资源与捕获时段就绪后才运行桌面案例。这次归类为 **no progress**，不是已验证的 live-job wait；上轮 progress，不从历史累计无进展次数，本轮保持 goal active，不宣称完成。两份域源码 hash/patch/guard 未变化，不重复编译或插件测试，passed=false、accepted=false，释放0 B。

## r39：交付同源逐层原值的两文件精确提案

r38 是本轮前段的依赖复核，未产生产品进展。后续已把缺失 provider 转成可独立审阅的实际代码 diff，构成具体交接进展，仍不代表产品源码已应用或 UI/验证通过。提案保留原 `ResolvedSettingsBatch`/`ResolvedSettingValue`，拟新增公开只读 `ResolvedSettingValue::override_in_scope(SettingsScope) -> Option<&SettingValue>`，由原 registry 的 pub(super) 层读取在同一 batch/authority lock/generation 捕获 User/Project/Session 原值。其他层 winner 与目标层 absence 分开；不建平行 DTO/store，不在 UI 读取 settings.toml。

- 精确未应用 patch：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/scope-projection-proposal-r39/resolved-scope-values.proposed.patch`，SHA-256 **`38f618eb703ce78b97dd314b2f8ab901257d17805a14f6fb678a9c92fd88ee2a`**。
- 路径1：`zircon_editor/src/core/settings/registry.rs`，实际 baseline SHA-256 `ee3f5a61e06ac8abc0b9eb0abf0c18189096ae6b5f635c0a2dd41a86b19c6a7b`。
- 路径2：`zircon_editor/src/core/settings/catalog/resolved_settings_batch.rs`，实际 baseline SHA-256 `ba72d7d15e3511fc21662bc7111f35aa1a5eeaa7e0f4f5487fa1a4538ca7d118`。

两文件当前 tracked diff 为空，提案仅在内存准备并保存最小 patch，没有源码/模块副本或产品写入。两个候选 rustfmt stdin 格式解析 exit0，`git apply --check` 实际 exit0，结束后源 hash 不变。原 public key/value/source/batch-generation 方法和唯一构造路径保留；字段原本私有，不需要外部 struct literal/reexport/设置文件格式迁移。三层原值仅在存在时额外 clone，扩充私有 DTO/PartialEq，需 Parent/core owner 审核读取成本及公共 getter；这些 core 路径不在 followup-02 写入范围，**等待具体协调，不自行应用**。

提案包含三项有意义的 authority 回归：Session 遮蔽下真实 User/Project 原值、默认/继承没有目标层 override、清除隐藏 Project 后旧批次不变而新批次 generation 前移。最小将来执行范围：`zircon_editor` lib/defaults off/features=[]/当前 Windows host、单 job/thread、Cargo test（adapter dev 默认）、物理 E 根，filter **`core::settings::catalog::resolved_settings_batch::tests::`**。三项未编译/执行，不发新 identity、不借原 mutation 请求验证未应用源；格式解析/apply-check 不充当源码 guard 或 Jenkins receipt。

提案只解决 raw/presence 的精确来源，未包含页面 edit scope、UI/reset 禁用态、旧事件/currentness/project-binding fence、scope health/file/writability、undo 或重开。原 r37 两份域源码/23项未执行回归、r34 metadata 公共提案及其待协调状态均不变。第13次新源码 queue 已实际失败，没有恢复容量，故本新增提案只写本节/原设计与组件合同，**未投递、无 ACK**，不称 Parent 已读/批准。正式公共实例/样式、官方导出、四宿主/family EXE、实际 DPI 与双端逐图验收继续 open，passed=false、accepted=false；无服务/桌面/编译操作，释放0 B。

## r40：当前真实依赖及 blocked 审计

12:58:22Z 沿原 `ue-settings-20261024-scope-reset-01` / `validate-settings-scope-reset-01` identity、原五项 reset filter 再做 QueryOnly，实际 exit1，返回 `status=pending / reasonCode=ordinary_entry_unproven / Current session authorization is pending`；未提交编译，不能验证 r33/r37 后续源或两份未应用提案。完整原始 stdout/argv/PID/结束记录在 `original-jenkins-query-r40.json`；客户端与外层 helper 已等待退出。不是有 live job handle 的 verified wait，也不据此重启或另发测试请求。

12:59Z 认证 API 返回 own rev15/四页/一个原主件、公共 rev0/单根/零 component/color/typography/未发布。再次读 Parent 实际更新记录，设置仍沿 followup-02，未观察到新 DTO/core 源决定、Preferences 公共实例/语义子树、family producer/正式导出或桌面时段。原 r34/r39 patch 均未变且 apply-check exit0；实际两份域源 hash 和原 core/公共 DTO 源保持，未越权应用。第13次官方投递的100条上限失败保留，无本轮 ACK/接口批准。

完整审计为 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/readiness-and-goal-audit-r40.json`：同一外部阻塞在用户 followup-02 的 r33～r35、自动接续 r36～r37、r38～r39 和本轮连续出现，已超过至少三轮；独立域修复/回归候选、源码与具体设计、必要精确公共提案均已完成，当前不能在明确 Parent 边界内独立落下剩余产品工作。r38/r39 的无进展计数不是 blocked 门槛，源修复/提案进展不消除仍存在的相同依赖。本轮完成 blocked 审计，**申请 goal 状态 blocked，最终以工具回执为准**，不标 complete 或验收通过。

恢复接续需要原 owner 的具体变化：公共文件已发布且实际可消费的基础实例/共享样式及 Preferences 同源语义/正式 family 导出；Parent/core 对 r34 metadata 与 r39 逐层值精确 diff 的处理以及页面 scope/generation 合同；原 Jenkins capture/session 收件与有效窄测试 receipt；正式 family EXE/资源闭包和 Parent 的尺寸/实际 DPI 捕获时段。四宿主、World/undo/save、正常 restart、额外 OVW 缺 provider 仍在原归属边界，不能由基础 source patch 局部通过覆盖。真实 UI/插件/Rust/持久化重开/两族主题传播/四尺寸及实际150%DPI/逐图对比全部继续 open，passed=false、accepted=false。完整目标、历史失败、他人改动与所有必要证据保留；无服务/桌面/系统DPI/编译操作，释放0 B。

## r41：原 SP-C7 authority 回执只读对账

已消费用户转交的原 Codex 消息 `01a16dbc-2adf-7911-8337-3d49d946952c`，没有新 implementation/generation/recovery 派发或 Jenkins/Cargo 身份。原回执 `E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/settings-plugins-sp-c7-20261024/generation-authority-reconciliation.json` 的实际 SHA-256 匹配 **`2b247a0384791c87cfb6fea5920f70aaba82ebdeece6f91b7788b82d7f8a48f4`**，简明交接 `generation-authority-handoff.md` 已读。独立核对 current installed/driver/capsule 与5实现+6clients、15项源输入全部匹配；这补齐 r38 未完整发现 selected binding 的读取信息，不改写当时失败或提供编译/执行 receipt。

当前 driver `fc55322804fb53ab3e39f996c848ef5e5f3907497d3f38167b7095244b2a7399` / runtime `start-1792836412672596900` / capsule `cdd14500fa39088ae2c9870f62825acc2e526ba647d24d77cd6287722b5d0f31`。pinned runner.generate 只有 source-index/render-template；没有 plugin plan/完整物理写集/恢复接口。现有 MUTATIONS/writer_hold 保留为接入基础。adapter/authority 仍归原 owner `01a12af6-7511-70d1-a452-61b80678eb56`；Parent 协调实际 first-party scaffold/catalog owner，尚未确认产品源实施归属，不从此消息推导写权。

独立核实 scaffold 无条件 static-manifest-end 预检位于新建包前，实际 catalog SHA-256 `05d54294019ea63e51b71baa64b417470d985e520e13b632b20b8cbfe4952cdc`、对应 literal 数量0。若此前预检/读取成功而进入这一步，将拒绝创建；仅静态推断，未执行 generator，不补旧标记或复制模板。参数化物理写集确为5新包+6共享文件，native额外2文件，共11/13，并含新目录及内容不变的共享重写。原 created_paths 漏共享触碰；当前 rollback 吞逐项错误并递归删包根，不能证明恢复，完整路径及 UI/阶段消费合同已写入 [设计合同](settings-plugins-design-contract.md) 的 SP-C7 段和 [组件合同](settings-plugins-component-contracts.md)。

未来仅提案：原 Rust scaffold 提取 pure public plan，metadata-only/no-spawn/no-Cargo，带 revision、源/生成器哈希、全写集/absence/diff/planDigest/结构错误；明确允许执行后同业务 request/planRef 的服务器内部阶段，不改受理 payload、不重放 pending。逐文件 journal，仅恢复仍等于自身 postimage 的路径，保留 foreign/unknown，逐项 applied/restored/conflicted/unknown 与真实 native/EOF 或未启动，未恢复 open。当前没有已实现/部署的 plan 命令，不能在 UI 暗示可执行。

本族设计继续固定 `<project>/zircon_plugins`、kind 必选无默认、native=false，显示真实 authority/source-layout 不可用原因；没有多根 setter/用户项目 SDK 模板、运行时动作或 Penpot 实例修改。本次只读复核/文档结构证据在 `E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/sp-c7-consumer-reconciliation-r41.json`。仅更新四份自有文档及该回执/清单，实际域 Rust/全部源和 sealed 文件不变；generator/Cargo/job/恢复/服务/桌面操作均0，释放0 B。

goal 工具本轮读回实际 **blocked**；该信息性只读收件不恢复目标执行，不重置或打断在途 managed 请求，SP-C7/nativeLifecycle/formal/其他请求状态原样保持。本族整体 passed=false、accepted=false，历史失败、已完成精确提案和完整未完成范围保留。第13次 queue 的容量失败仍为原观察，本次无新的投递/ACK或 owner 实施批准。

## r42：消费 Parent followup-03 的真实源增量

用户本次明确接续实施；已读 Parent“设置第三次接续”与 `settings-plugins-followup-03.json`（SHA-256 `95acfd72fd0e2fb80bcb6265b386977521d665b3967849b1cdc28158565b5d99`）。12 份 canonical after hash 独立核对全部一致。r39 raw getter 与 r34 metadata **现在已由 Parent 应用**；本域不重放旧补丁、不改写原基线/历史失败。r37 details 现 SHA-256 `e63b13022af1fb4d33590ec64327e7fdc5e3294b3b550fbf237b6f872850a1a6`，恢复逻辑和 Parent 迁移的 `package_metadata: None` 保持。

本轮实际 Rust 写入只有 `zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs`：

- before SHA-256 `57dc7a2a36aa883c1f7f1215d3f1ce94d790fb52c4eb9f20221276551cc26602`；after `68beb857b9b5de6d87d69d4f6f60eb4922bcbe9dbf0af1c948c8a346c005baea`。
- 最小消费增量：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/scope-consumer-r42/reset-raw-scope.patch`，SHA-256 `3e2371728e685ba0712c15cf2f0eb02503865d6d41b83c649b2569684c548005`。这是当前实际修改的交接 diff，**已经落源，不再次应用**。
- 完整 mutation candidate 相对 HEAD `75a76ee963792d5a316f738519e966937f770036`：同目录 `domain-candidate.patch`，SHA-256 `7ef8c47926c0d6b27693dab885d8e7bcafc8ea7dcd751acbbb75839841c2e7d8`，与实际 guard input.patchDigest 一致。包含原域增量，不是 Parent 12 文件的替代补丁。

reset 通过原 authority 的 selected-key batch 读取真实 `override_in_scope(scope)`，不以 winning source 推断或删除层。空目标层直接 `Ok(None)`，即使 Session 正在遮蔽、Project 没绑定或 User 无持久化源，也不触发无意义准入。真实存在的目标层仍走原 coordinator clear/receipt/error，不能跳过权限、绑定或持久化错误；只清显式层，保留其他层。raw 检查与 clear 是分开的操作，未加 generation/CAS fence、undo、可写性或 UI/resetEnabled。

新增 `absent_project_reset_under_session_needs_no_project_binding`、`absent_user_reset_under_session_needs_no_user_persistence_source`，检查真实 resolved batch、generation 与 persistence health 均不变；加强 `unbound_project_reset_rejects_without_deleting_any_override`，在 Session 遮蔽下仍拒绝真实 Project override 的无绑定 reset，移除 Session 后验证 Project 值保留，再移除 Project 验证 User 值保留。mutation 当前共 **17 项**，详情原 **8 项**、raw 原 **3 项**也未运行，不能拿原五项 filter、r35/r37 旧 candidate hash 或静态结果作通过回执。

rustfmt check、diff-check、静态 guard 本轮均实际 exit0，guard **0 errors / 0 warnings / 0 suppressed**；Git 的 LF→CRLF 提示原样留在 diff-check receipt。没有 Cargo、新 identity、提交/重放请求或 QueryOnly 操作。最小未来测试仍是 `zircon_editor` lib、defaults off/features=[]、当前 Windows host、Cargo test/原 adapter dev、jobs=1/testThreads=1、物理 E:/cargo-targets，mutation filter 为 `ui::host::editor_event_runtime_access::settings_mutation::settings_scope_mutation_tests::`。由 Parent 已交接的原 Jenkins owner 核对当前源后声明/执行，不在本域复制请求。

新增宿主接口先交精确未应用 diff：同目录 `settings-scoped-values.proposed.patch`，SHA-256 **`423e5651517502c8d940b272038f06758ef7f4d9d8783c273407b09b64034508`**。仅拟改原 `settings_projection.rs`（实际源仍 SHA-256 `6d64945e9d695954dfa1724aa129a8e090e1327dda95bcfa0136fea6c3225993`），新增显式 `capture_settings_scoped_values_for_category(category_id, edit_scope)`；恰好调用原 capture 一次，给原 batch 配对 category/scope，借用原 row/raw，保留 key/source/value/generation。旧签名、共享必填 DTO、注册/事件不变，没有第二 store/全 registry 扫描/磁盘读。仅 rustfmt stdin/apply-check exit0，未应用、未编译。Parent 接 page-scope ingress/refresh、原 catalog/locale/definition.scope、project binding 与事件 generation admission 后才能做正常窗口消费。

SP-C6 最小剩余合同：真实 status 已有四字段，但原 `layouts/windows/window/host_data.rs::ModulePluginStatusViewData` 无 metadata，`app/pane_data/view_rows.rs` 和 `retained/ui/data/module_plugins.rs` 的原卡片链不能承载。请 Parent 交接可选 typed metadata 的 ViewData/全部构造点与官方 PluginMetadataCard/native 子树；本域随后从同一 report row 复制，不再读 manifest。保留原 action/cache/operation tuple，None 显示不可用，catalog version 不冒充 DLL provenance。不把字段拼入旧 title/诊断绕过共享控件，也不改公共 UI 资产或主题。

SP-C7 继续固定根、原 prepared root/非阻塞 ticket/last-good publication 合同；UI 尚缺 project/root/ticket generation 的正式承载/旧项目丢弃接线，未另造扫描器或同步 IO。New Plugin 没有已发布 pure plan/全写集/逐文件恢复回执，执行保持不可用；无 generator、source preparation、共享 catalog/app、服务/桌面/DPI 操作。

本轮完整基线/消费/提案/静态证据汇总为同目录 `consumer-contract.json`，四份自有文档同步当前状态，原历史报告保留。Penpot 沿 Parent 已确认 rev15/四页/299对象/原组件身份，本轮未作新的 API/样式/保存验证。公共实例/Preferences 同源语义、正式导出、四宿主/family EXE、World/undo/save、restart、OVW provider、持久化重开、两族主题传播、四尺寸与实际150%DPI、并排/差异和逐图复核仍 open，passed=false、accepted=false。本轮新独立源修复构成接续进展，不宣称此前 blocked 条件全部解除。实际释放0 B。

本轮新增实际补丁及新接口提案经一次新的官方 queue 交接，实际 exit0，消息 ID **`01a16e17-0b59-7b83-80e8-5e45f9d83d5c`**；原文/stdout/客户端退出保留于 `scope-consumer-r42/parent-queue-14.json`。这是新的真实入队回执，不代表 Parent 已读、批准新接口或授予桌面；此前13次容量失败保留，不改写成成功，也不重放 managed 测试。

## r43：消费 Shared rev1 四主件，真实连库/试接/重开

已读本轮Parent“公共库最新观测”及 `ai-followup-03.json`，独立认证API确认 Shared `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`/page `...ac13` 为rev1/isShared=true/41objects/四主件，color/typography仍0。本轮不再以旧rev0断言当前未发布；旧观察和失败均保留。原认证state只读复用，hash未变，不复制/打印secret。无服务、共享桌面或DPI操作。

已通过原生“管理库→连接库”连接本族文件，get-file-libraries返回直接库 `isIndirect=false/revn=1`。只在原本族file新增受控page **`1ca0591c-2856-49b0-9d7e-eeb076bafe03`**/flow board **`4829a7b5-8807-4e66-b34d-b94fbb8807fe`**；最终 **rev22/五页/367objects**，local component仍仅原PluginMetadataCard `81548358-cc2f-47cf-856d-39b395bfecfa`。没有编辑Shared/其它族、复制main、detach或制造未发布UUID。

四个实际component/main及源root control/catalog逐一核对，详见[组件合同](settings-plugins-component-contracts.md)和 `shared-linked-consumption-r43/source-slot-audit.json`。Button/SearchInput/TreeRow/PropertyEditorRow分别由原生素材dragstart/dragenter/drop驱动原Penpot `instantiate-component`，各生成2个copy，全部66个复制节点（含8个根）的shapeRef指向原主件。copy的sourcePath和pluginData逐字保持，componentFile均为Shared。发布canonical-record没有sourceHash，当前实际源SHA已记录，不能以此声称owner/hash/codec等价已完成；copy保留的control-part-list/painter原辅助ID还需官方exporter证明按shapeRef解析，未自行改签。

| 角色 | 默认copy | 文字覆盖copy |
| --- | --- | --- |
| Button | `73b56c83-5fed-80c5-8008-d6ceb6a369db` | `73b56c83-5fed-80c5-8008-d6ceeaf95079`，恢复本层 |
| SearchInput | `73b56c83-5fed-80c5-8008-d6ceea22f59d` | `73b56c83-5fed-80c5-8008-d6ceeb456572`，搜索设置 / Search settings |
| TreeRow | `73b56c83-5fed-80c5-8008-d6ceea75aa06` | `73b56c83-5fed-80c5-8008-d6ceeb849588`，项目设置 — 长分类名称 |
| PropertyEditorRow | `73b56c83-5fed-80c5-8008-d6ceeabc1cd5` | `73b56c83-5fed-80c5-8008-d6ceebe028f7`，平移步长 (Project) |

覆盖只改原copy text content/positionData失效及原生touched `text-content-text`，保留shapeRef和canonical metadata，不冒充真实query/value/action。经正常校验保存、原context关闭、两次fresh context重开后4/4文字覆盖均保留，flow和8个原生链接保留。Shared独立API响应hash与本轮基线完全一致。完整14项检查和身份回执：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-linked-consumption-r43/save-reopen.json`，SHA-256 **`f582dddd2ba5351e142d14a45faef306d6c844a6ce5982431f8eea2261dbc47c`**。

需要明确保留的实际变化/失败：原Preferences首次正常打开后rev15→16，53个旧文字节点hash变化，未逐字段重建原因，不把它们静默称为全程字节不变；没有回滚未知缓存/源。之后以实读rev16为新基线，原四页全部对象和原local component定义均逐个hash保持。本轮IPC plain-pipe关闭、手写JSON转义失败及原生目录selector超时均保留，相关浏览器/Node已退出；没有把失败记成通过。原生插入导致column几何顺序反转，首次修正stored order仍未重排，已只对验证板通过原生row→column触发布局，最终fresh图和API顺序一致，旧图保留。

最小未交付合同仍具体：PropertyRow的required `value`接受Checkbox/Dropdown/Field/NumberField/RangeSlider/Slider/Toggle，当前四主件不包含这些editor，两个slot保持原样空，未塞入不接受的SearchInput或假值。description/source/reset/可换行行高及显式page scope/事件generation仍需SP-C1/C2/C3原owner接线。三个真实浏览器PNG均已逐图打开：长中文树label及Property名在固定行高下换行溢出，仍open。图为1600×1100 headless/100%canvas与640×520试接板，不是EXE/实际DPI/双端1逻辑像素验收。

本轮只完成原生库连接与受控实例/覆盖持久化试接。跨文件正式ZUI/PNG codec、canonical owner/sourceHash/完整slots、override-reset、公共styles和两族A/B、原设置窗口producer/正式资产、17项mutation/8项详情等原Jenkins聚焦测试、正常EXE与四尺寸/150%DPI/逐图比较仍open；passed=false、accepted=false。本轮未改Rust/产品ZUI、未建Jenkins身份或重放请求，r42源哈希保持。无需新启动服务或桌面slot；后续只消费原owner的正式export/slot/typed editor与page-scope接口。

本轮精确身份、回执、实际差异及最小slot/export合同已通过一次新的官方 `codex queue` 入队，exit0，消息 **`01a16e33-70ea-7112-86f3-28214b1626d0`**；原文/stdout/客户端退出保留于 `shared-linked-consumption-r43/parent-queue-15.json`，不等于已读/批准。继续保留必要原图、失败证据和可复用的本族隔离验证驱动，所有自有浏览器/Node均退出；整体任务及后续托管生命周期未关闭，实际释放0 B。

## r44：消费原 owner 的 followup-03 准入审阅

已实际读取[原审阅JSON](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/ue-settings-20261024-scope-reset-01/admission-review.json)及同目录summary，独立SHA-256为 **`8d925b8df67b2e9887963fdeb301144790d1932e161d5febbc1eb338271b40ea`**。这是原 Jenkins owner 的可审阅候选包，`formalAdmissionIssued=false`、`formalTerminalIssued=false`、`captureReady=false`；不把pure parser/recipe检查、prepared输入或135项本地结构/指纹核对当正式收件或Rust执行。

Parent十二份Rust after hash全部匹配，当前13份相关Rust也全部匹配。mutation为r42 **`68beb857b9b5de6d87d69d4f6f60eb4922bcbe9dbf0af1c948c8a346c005baea`**，对应原guard `7ef8c479…`及format；details为Parent已迁移fixture的 **`e63b13022af1fb4d33590ec64327e7fdc5e3294b3b550fbf237b6f872850a1a6`**。旧r35/r37输入和原hash保留。Parent旧13路径guard `6aa1ba22…`包含已变化的note，只复用相同Rust对应的format/static，不称旧完整diff当前仍全量通过。

原 owner 两份current-source revisions均逐字段核对：

| 准备输入 | SHA-256 | 保留业务身份 | 当前状态 |
| --- | --- | --- | --- |
| `candidate-input-r35-current.json` | `a2f79cba79acf888db3adbed47c72dde3539b6ca105381501afeacd5bf3cd14c` | `ue-settings-20261024-scoped-numeric-01` / `validate-settings-scoped-numeric-01` / 本族session | prepared，未提交/准入 |
| `candidate-input-r37-current.json` | `5f59d8113e38078227e56d51af94fd1dc26b736fc72b4615f708c620b350e0ad` | `ue-settings-20261024-detail-recovery-01` / `validate-settings-detail-recovery-01` / 本族session | prepared，未提交/准入 |

它们仅改变sourceHashes并新增dependencyHashes，其余字段与原proposal内identity/原identity文件一致；列出源码及依赖全部实读匹配。没有覆盖旧文件、改策略、新ID/grant、重复QueryOnly或盲重投。原reset `ue-settings-20261024-scope-reset-01` / `validate-settings-scope-reset-01` 仍留r27 source `96b264…`、external_control_forbidden及r40 exit1/ordinary_entry_unproven；新候选不冒充该请求终态。原 owner 有界records只读未命中不等于完整历史查询，本域不扩大查询。

| 独立过滤范围 | 当前静态Rust定义 | 执行情况 |
| --- | ---: | --- |
| `core::settings::catalog::resolved_settings_batch::tests::` | 3 | 未执行，需匹配范围正常登记 |
| `ui::host::exports::status::native::product_catalog_tests::` | 3 | 未执行，需匹配范围正常登记 |
| `ui::host::exports::status::live::tests::live_result_preserves_package_metadata_and_the_full_enablement_outcome` | 1 | 未执行，需匹配范围正常登记；无`::cases::`段 |
| `ui::host::editor_event_runtime_access::settings_mutation::settings_scope_mutation_tests::` | 17 | 未执行；不能沿旧15/5项或旧hash宣称通过 |
| `ui::retained_host::app::module_plugin_projection::details::tests::` | 8 | 未执行；13口径不等于该filter的harness定义/结果 |

上述32仅为源码静态定义，未运行harness list/零匹配检查。更广module_plugin_projection的16定义不自动纳入。五条精确模块命令仍以原JSON为准：根zircon_editor、defaults off、features=[]、Windows lib、locked、jobs1/threads1、物理E根；adapter dev只是元数据，原argv未显式`--profile`，实际profile须由执行环境/命令/回执证明。未来编译由执行owner另行完整声明，本轮不调用任何wrapper或Cargo。

外部两个`.codex` plan与原capture guard的冲突仍无正式解决回执；现source_preparation实读SHA **`2a38f55962b5e7c9f9914f7e6dbe637ec41522b1852d1be824ceada1c246ff77`**未变。full-file-index-v2只是可显式审阅的输入版本，未选用，不改旧payload/删除控制文件/制造capture-ready或停写全checkout。仍缺真实收件、完整source/control闭包、resource/物理输出/native writer准入和Job/EOF/实际结果；原owner接续原业务链。

SP-C7六共享输入独立hash全部匹配，没有生成请求或已发布plan API，保持执行不可用；不运行generator/改catalog或workspace。消费回执：[jenkins-admission-consumer-r44.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-admission-consumer-r44.json)；文档核对：[jenkins-admission-consumer-r44-checks.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-admission-consumer-r44-checks.json)。

本轮只更新本域两份文档及预登记回执/清单，Rust/ZUI/旧proposal与identity、服务、锁、Cargo、生成、Penpot/桌面、缓存清理均0。按来信不发例行ACK/queue，不操作或打断原managed请求。r43设计状态与正式UI/Rust/EXE/DPI/nativeLifecycle gates不变，passed=false、accepted=false；证据及复验驱动继续保留，任务未关闭，释放0 B。

## r45：原恢复r2与本族原identity只读查询

已读取Parent“Jenkins最新恢复结果”及[恢复交接02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/jenkins-recovery-handoff-02.json)。Parent核实的197/197 immutable及原固定896a工具属于其恢复证据，本轮未冒称重复执行其全部盘点。独立重读原r2 native-terminal实际hash `dcbcd30d3c62e00b72bca867921b8669948312116ae8c389089493faa9534675`，root0、active0、childrenGone、stdout/stderr EOF、readersJoined、jobClosedAfterZero均成立；原r1/root1和旧恢复历史保留。Jenkins恢复不替代产品测试、P21/Runtime11或画面验收。

本轮preflight发现实际installed已继续前进：manifest SHA/胶囊 **`9c684aef96b8b1c94319aa8619207f8b07aefffa71d571d4f58708d2c3408233`**、installed SHA `c41a585f6cea0746fd9cc147a9579079e9f8b65a8139d0f81b1225f67cffdb8d`，5server+6client逐个hash匹配；同原driver `fc55322804fb53ab3e39f996c848ef5e5f3907497d3f38167b7095244b2a7399` / runtime `start-1792836412672596900`。状态文件healthy/controller/agent/plugins/acceptingRequests/nativeTaskContextVerified真、desired completed，属于本轮文件观测。Parent20b3/旧Home快照保留为各自时点，未把wrapper与capsule不同文本误判损坏，当前Home/API安装对账仍由原owner负责。本族没有恢复/重装/改锁或服务操作。

实际入口已核实`QueryOnly`直接走historical_query且禁止resume/dryrun组合。原r27身份为 `jenkins-settings-projection-r27/identity.json`，SHA **`19981c9704944d591c8e58443a9c151a62da72a92a3dcf580964f6c9753a278a`**，request `ue-settings-20261024-scope-reset-01` / operation `validate-settings-scope-reset-01` / 本族session。只执行一次以下原始历史查询，无新attempt、payload改写或重投：

```powershell
& 'E:/Git/ZirconEngine/tools/dev/dev-module-build.ps1' -Package zircon_editor -Action test -Features @() -LibTests -TestFilter 'ui::host::editor_event_runtime_access::settings_mutation::settings_scope_reset_tests::' -Jobs 1 -TestThreads 1 -JenkinsIdentityFile 'E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-settings-projection-r27/identity.json' -QueryOnly
```

范围为根zircon_editor、defaults off/features=[]、Windows host lib、locked、jobs1/threads1；这是QueryOnly，未运行Cargo，旧adapter dev/无显式profile argv不代表实际编译profile。原source `96b264…`和旧5项reset filter未替换成r42当前源/17项。新查询 native exit1，stdout原文：

```json
{"formalAcceptance":false,"m8Accepted":false,"message":"Current session authorization is pending","reasonCode":"ordinary_entry_unproven","status":"pending"}
```

stderr空，query客户端27080/helper36064等待结束，客户端双EOF；不冒充Jenkins执行Job/native产品终态。实际[查询回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/original-jenkins-query-r45.json) SHA **`f17559c9dac8c86b5fafeca3c4df8f7866af8053c2f785544bb74627d8882701`**；[入口/恢复消费](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-recovery-consumption-r45.json) SHA **`cf43eae45e2bce8c76dd31e732426c6acfb2a3a736bbfbbb09695e2d343bd162`**，45项纯指纹/入口检查通过。初次ad hoc脚本错把manifest `clientRoot=clients` 当checkout相对路径，FileNotFoundError保留；读实际runner.client_path后按capsule/clients修正，没有触发任何Jenkins/编译操作，不计产品故障。

当前13份Rust和两prepared inputs与r44匹配。mutation `68beb857…`/17、details `e63b1302…`/8及raw3/native3/live1仍未执行；新三范围没有借用旧reset/两个prepared身份。原r27/r40失败及r35/r37未提交proposal不覆盖，当前prepared不伪装收件成功。下一步需原owner沿原业务链核对automatic receiver/records/delivery、外部control-plan/source准备与resource/物理E根/native writer，回真实receipt及可接续scope；无需用户手工grant，不写授权库或放宽输入guard，不全checkout停写。现阶段不提交新编译；原有 typed reset/scope、restart、Dialog body/actions、长标签公共变体及SP-C7的未证明项保持。

新输出已按本次明确交接要求一次送给Jenkins原owner `01a12af6-7511-70d1-a452-61b80678eb56`，官方queue exit0、消息 **`01a16e5b-d30d-7011-babe-265024c11aa4`**；[发送回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/jenkins-owner-handoff-r45.json) SHA `8d3b525627acc593fdc5a4d5ee89779ac740d7b732afae1329b7f369cfe6aecb`，只证明入队，不证明已读/准入/执行。queue客户端20816/helper37812均已join；本轮四个自有PID独立复核缺席，未终止或领用服务PID。没有向Parent发送例行重复ACK。

仅本域两份文档和预登记证据/manifest写入，Rust/ZUI/主题/公共组件、SP-C7生成、服务、锁、缓存清理、共享桌面/DPI及Cargo均0。原证据与复验驱动继续保留；实际释放0 B，任务未关闭。passed=false、accepted=false，Penpot仍沿r43 rev22/五页/367对象，正式窗口/导出/EXE与原native gates全部open。

## r46：可独立落源的插件详情焦点修复

本轮从实际域接线定位到未处理的焦点失效：原reconcile只在recovery时从Apply返回features；同owner报告移除所选feature、将其改为必需/不匹配owner、清空features或改变分页后，旧焦点可能仍指向无操作的Apply/已失效分页/空列表。现有键盘dispatch直接按state.focus执行，故可按原语义状态修复，不需要公共DTO/窗口/主题扩权。以上为源码控制流依据，未冒称运行复现。

仅实际修改已授canonical `zircon_editor/src/ui/retained/app/module_plugin_projection/details.rs`。先核foreign diff及源恰为Parent/r44/r45 e63b1302，然后保留全部原修改；新逻辑在原feature ID/page reconcile之后，以同report检查Back/Previous/Next/features/Apply资格。有效焦点保持，失效返回features，空列表返回Back；Apply沿原detail_feature_action，原owner限定、recovery禁写、report/currentness与一次性intent、缓存、选中/Back及enablement tuple均保留。没有改任何公用必填DTO、签名、外层注册、ZUI事件或公共控件。

| 精确身份 | SHA-256 |
| --- | --- |
| before details source | `e63b13022af1fb4d33590ec64327e7fdc5e3294b3b550fbf237b6f872850a1a6` |
| 当前details source | **`ea903d46098963df14f446a3f9d67f4e6471cd90b4de6d1917ae8e3715b67f14`** |
| 本轮最小 `focus-refresh.patch` | `6f7af9e658b2be8f440aaea3e121ee98d2cfc320ae4dc2a4d364a18de04edf29` |
| 相对原HEAD完整 `domain-candidate.patch` / guard digest | **`827462fd3912e984dc1f74da6bd2638c214a200a5fd1ab69375ad483e0c3aa01`** |

证据根：[plugin-detail-focus-r46](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-detail-focus-r46/contract.json)。在内存反向去除本轮精确增量可重建原before SHA，没写源码副本；此前r37修复和Parent package_metadata None构造完整保留。其他12份相关Rust（含mutation68beb857…）逐个hash与r44一致。rustfmt check、diff-check与change-guard exit0，guard **0 errors / 0 warnings / 0 suppressed**，精确source/dependency记录在contract.json。普通Python写源曾errno22且未改变文件；按审阅的rustfmt stdout差异用apply_patch完成，仅格式差异，未改权限/锁/alias或Jenkins。辅助reverse初次空行assert失败也保留，不计Rust/Jenkins故障。

新增三个Rust测试定义，详情当前 **11** 而非旧8；三个定义内有3种Apply撤销、4种空列表前焦点及2种分页变化。它们检查失效focus归还、旧intent拒绝、稳定feature ID/page、其他plugin/原operation不变、缓存复用及新的qualified action可执行选择。原8个定义完整保留。当前raw3/native3/live1/mutation17/details11均未运行harness或编译，不将static guard充当Rust通过。

原 owner `candidate-input-r35-current.json` 仍SHA a2f79cba…、`candidate-input-r37-current.json` 仍SHA 5f59d811…；两文件均保持原字节。前者dependencyHashes、后者sourceHashes中的details仍为e63b1302，因此目前都已旧；只这一个路径存在本轮已授域增量，不静默覆写准备输入或将旧payload重放。需原owner按原未提交业务request/operation/session审阅当前source revision，给真实自动收件/source/resource/native writer准入；已受理payload不可改写。r27/r40/r45历史请求与pending失败原样保留。

下一聚焦命令范围仅根zircon_editor、defaults off/features=[]、Windows lib、locked、jobs1/testThreads1、`ui::retained_host::app::module_plugin_projection::details::tests::`，必要runtime传递feature仍target-editor-host/test-support。模块wrapper计划无显式profile，adapter记dev，真实profile须实际argv/环境回执；执行方在编译前重新声明。contract内原prepared文件命令标为runnableNow=false，本轮没有提交/查询/Cargo或借raw/native/live身份扩范围。

独立认证GET再读Shared仍rev1/四主件/0colors/typographies，own rev22/原主件保持；未打开浏览器或修改Penpot。真实公共value editors、长标签行高、Dialog body/actions、page-scope/代次及metadata UI、restart/World/本地化配置provider、两族主题A/B、正式导出、正常EXE/DPI/逐图证据继续open。SP-C7无新authority/receipt，不运行scaffold或改共享workspace。passed=false、accepted=false，必要失败及复验驱动保留，释放0 B。

本轮已沿官方queue各发送一次有新源码的具体交接；Parent native exit1，stderr为 `queue cannot contain more than 100 submissions`，未送达、不重试、不清他人队列或改其记录。Jenkins原owner native exit0、消息 **`01a16e8e-6f25-7891-b28a-f8d8bca4ae28`**，仅入队，不等于已读/新准入/测试终态。[双目标原始回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-detail-focus-r46/parent-queue-16.json) SHA **`2c234ad985feece4dce0ae2a3b93513fb03a0b3d536104ed45e88058c2c479d3`**；两客户端与helper均等待退出，失败和已成功入队的业务事实分别保留。Parent可沿本已知交接路径审阅，不把Jenkins的成功回执写成Parent已收到。
