# Unreal ↔ ZirconEngine 全模块对照：目录、独立 Draft 与测试推进总览

## 1. 本轮阶段与边界

首轮完成用户要求的“先搜索全部关联模块、再逐个提出有据问题 Draft”阶段：**21 个独立问题 Draft、30 个原列领域 + 10 个补充领域映射**。后续新增独立 Notify #171、Text #174、旧恢复归属 #175、Capture #178 和经典纹理缓存 #180，第26轮实测见第30节，累计 **50 个问题 Draft**。第7–29节记录此前各轮的源码与执行结果。下文首轮表格/CSV保留发布当时快照，不是实时测试通过表。不是引擎整体对齐完成，也不是数千模块逐项行为/产品审计通过。

- 每个问题的六点在各自 PR 正文及独立文档内：不一致；实际影响；根因/UE处理；Rust做法；接口/ECS差异；ZE具体修改草案。
- 每项都有精确 UE/ZE 路径、符号、行范围、UE 文件 SHA256、源码定位的生产调用链及未执行边界、既有测试落点。能复用的 UE 原测试与仅由源码推导的新回归已分列；不虚构迁移。
- **首轮21个问题PR刚发布时均只新增各自Markdown；此时的文档级回执冻结保存，不作为后续禁止测试/修复的条件。本总览仍仅Markdown/CSV元数据；初始阶段未改引擎或执行Rust等价测试/Cargo/UBT/GPU/产品。** 构建选取函数的纯内存见证、四元数独立算术和文件系统只读元数据探针不是引擎回归通过。
- 不发明替代架构；Prefab 保持 Unity 思路，仅属性覆盖/预设参考 Blueprint。报告不是修复授权依据的替代品，后续仍回原 UE 源复核。

## 2. 源身份与完整目录口径

- ZE 问题 PR 的独立共同基线：`e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。本地 `cb495bb1c952a24c86f4513b8099bc4942c3a87d` 的退役流程/工具改动不夹带。引擎 crate 零差异已核；构建工具差异及同一 selector AST 已另证。
- UE 是用户本机参考树 [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L10>) 声明的 **6.0.0 / BranchName UE5 / CL 0**，不是已知发行 tag，也不是独立 Git checkout。用文件指纹而非伪造 UE commit 固定证据。
- UE 本机 E: 路径与本轮 D: 审计记录需要本机访问；后者不随 PR 发布，不冒充 GitHub 可读证据。本 PR 只发布规则/测试路径元数据，不发布 Epic 源码正文。

| 本地范围 | Build.cs 路径 | Target.cs 路径 |
|---|---:|---:|
| Engine/Source | 725 | 132 |
| Engine/Plugins | 1810 | 20 |
| Source + Plugins | 2535 | 152 |
| 补 Engine/Platforms + Shaders | 26 | 0 |
| 全 Engine | 2561 | 152 |
| 补根 Templates + Samples | 27 | 26 |
| **整个本机 UE 参考树** | **2588** | **178** |

规则包括模板、External、平台覆写、测试和样例；**不是 2588 个已注册/已启用/已验收模块**。Source 分类为 Runtime 294、Developer 162、Editor 147、Programs 122；Plugins 89 顶层类别中 80 有 Build、9 为零。全 UE 模板 Build 19 条；38 条显式 TestModuleRules、37 条 TestTargetRules。

目录所有 2766 条 Build/Target 的 `path + NUL + sha256 + LF` 排序集合指纹：`a87fc4061f8ba7c0fbe78cc9484f45725ac49191edbe0a5de04b261f510de820`。本地不存在 Engine/Restricted、Engine/Source/ThirdParty；VisionOS 10 条 ThirdParty 覆写已列，不能推断上游没有第三方库。

CSV 标志口径：`is_template` 只识别精确 `Templates` 路径段，SoundCueTemplates 插件不算模板；`is_platform_extension` 表示 Platforms 路径归属，不表示 25 条都是派生覆写或独立模块；`is_external` 只记录规则内直接赋值 `ModuleType.External` 的词法证据，不计算继承后的有效 Type，false 不能反推不是第三方或 External。

| 完整可检索目录文件 | 内容 |
|---|---|
| [module-rules.csv](<module-rules.csv>) | 2588 条规则路径/类别/class/模板/平台/测试标志与 SHA256 |
| [target-rules.csv](<target-rules.csv>) | 178 条目标规则与 SHA256 |
| [module-categories.csv](<module-categories.csv>) | 112 行分类与零值统计 |
| [domain-map.csv](<domain-map.csv>) | 40 域 UE/ZE owner、首轮限定审查状态、PR、未查清边界 |
| [ue-owner-evidence.csv](<ue-owner-evidence.csv>) | 154 条领域–UE owner 关联证据（119 个不同 UE owner / 源路径，含 SHA256） |
| [ue-test-modules.csv](<ue-test-modules.csv>) | 414 条测试指示规则，非已通过/已迁移数量 |
| [ue-test-source-candidates.csv](<ue-test-source-candidates.csv>) | 2677 个命名/Tests 目录 C++ 候选，23 个无最近祖先规则 |
| [ue-csharp-projects.csv](<ue-csharp-projects.csv>) | 162 个 CSharp 项目，39 个测试指示项目 |

## 3. 首轮已建立的21个独立问题Draft（发布时快照）

所有 PR 独立基于同一 main 基线，不把另一个问题的实现栈进来。下表固定链接和状态只指首轮问题定义快照，后续 #159/#161 状态见第7节历史与第8节当前更新；不是RED/GREEN或产品验收。

| Draft | 问题 | 六点与对照测试内容 |
|---|---|---|
| [#147](https://github.com/He-Jiahui/ZirconEngine/pull/147) | [Animation] 对照 Unreal 修正基础姿态混合的共同半球选择 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/018b5a3e27980f865a6a3c798270a11f9abbe768/docs/plans/unreal-alignment/animation-blend-hemisphere.md>)；回归未新增/未运行 |
| [#148](https://github.com/He-Jiahui/ZirconEngine/pull/148) | [Editor UI] 对照 Slate 保留工具栏命中缓存的裁剪语义 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/a88b2de9559b80dc0d1f835747b3438a748ee503/docs/plans/unreal-alignment/editor-toolbar-clip-cache.md>)；回归未新增/未运行 |
| [#149](https://github.com/He-Jiahui/ZirconEngine/pull/149) | [Script] 对照 Unreal：绑定刷新不应重发未变实例 onStart | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/a0d262e952afd749988ccf9139891b9aca44e95d/docs/plans/unreal-alignment/script-start-projection.md>)；回归未新增/未运行 |
| [#150](https://github.com/He-Jiahui/ZirconEngine/pull/150) | [UI] 对照 Slate：祖先 opacity 应传递到子节点命令 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/9a83fff7fac0091f919fd7f12f0dd90a22215adb/docs/plans/unreal-alignment/ui-parent-opacity.md>)；回归未新增/未运行 |
| [#151](https://github.com/He-Jiahui/ZirconEngine/pull/151) | [Input] Web 按放逻辑键配对与 Unreal 稳定键语义对照 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/66c2f788a2d1a2b6e72a618227535fa643505d5b/docs/plans/unreal-alignment/web-key-release-pairing.md>)；回归未新增/未运行 |
| [#152](https://github.com/He-Jiahui/ZirconEngine/pull/152) | [Materials] 对照 Unreal：拒绝超出位预算的材质选项 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/e8c2fdcb4d73a2777a3c1a167735b3b1fd4ed8f6/docs/plans/unreal-alignment/material-option-bit-budget.md>)；回归未新增/未运行 |
| [#153](https://github.com/He-Jiahui/ZirconEngine/pull/153) | [Particles/VFX] 对照 Niagara：帧内新生粒子不应整帧预老化 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/68f7d77ccad3ce8ab3853471ce5e36968ee18547/docs/plans/unreal-alignment/particle-newborn-age.md>)；回归未新增/未运行 |
| [#154](https://github.com/He-Jiahui/ZirconEngine/pull/154) | [PostProcess] 对照 Unreal：内部 LUT 提前丢失 HDR 高光 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/0ec3dae4371e5bcc218189808252b82bcf2e4f91/docs/plans/unreal-alignment/post-lut-hdr-domain.md>)；回归未新增/未运行 |
| [#155](https://github.com/He-Jiahui/ZirconEngine/pull/155) | [Networking] 对照 Unreal：TCP 接收失败的事件与查询不一致 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/060fb19969033e473d70cc4182f14b1d00d83236/docs/plans/unreal-alignment/tcp-read-error-state.md>)；回归未新增/未运行 |
| [#156](https://github.com/He-Jiahui/ZirconEngine/pull/156) | [Audio] 对照 Unreal：WAV 导入成功不应携带 NaN PCM | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/6ab1533da9188764dd6fd680076c9fd7bbbae03c/docs/plans/unreal-alignment/wav-finite-pcm.md>)；回归未新增/未运行 |
| [#157](https://github.com/He-Jiahui/ZirconEngine/pull/157) | [Project] 对照 Unreal：场景后缀识别应与导入保持一致 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/029b2fbea00c3daa1e2c028aab79a14eeba0a0e9/docs/plans/unreal-alignment/project-scene-suffix-case.md>)；回归未新增/未运行 |
| [#159](https://github.com/He-Jiahui/ZirconEngine/pull/159) | [Core IO] 对照 Unreal：合法长目标名被事务临时名拒绝 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/5f54d124dc15ec830f26e40af4e4c3f459b2dddd/docs/plans/unreal-alignment/io-atomic-sibling-names.md>)；回归未新增/未运行 |
| [#160](https://github.com/He-Jiahui/ZirconEngine/pull/160) | [Core Memory] 对照 Unreal：realloc 两次计账漏记并发峰值 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/1a8ae4ecfc702853a8346b637eb78f075e429d21/docs/plans/unreal-alignment/heap-realloc-peak.md>)；回归未新增/未运行 |
| [#161](https://github.com/He-Jiahui/ZirconEngine/pull/161) | [Build] 对照 UBT：产物选择必须保留显式 target 身份 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/3077476ab75e1023834ab084b8afb2ea8fc4fdfe/docs/plans/unreal-alignment/build-target-artifact-identity.md>)；回归未新增/未运行 |
| [#162](https://github.com/He-Jiahui/ZirconEngine/pull/162) | [AI/BehaviorTree] 对照 Unreal：TimeLimit 超时应撤销活跃子任务 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/d8821907f9a0fbc805ac8d3856447448cb6da60d/docs/plans/unreal-alignment/ai-time-limit-child-abort.md>)；回归未新增/未运行 |
| [#163](https://github.com/He-Jiahui/ZirconEngine/pull/163) | [Reflection] 对照 Unreal：非持久字段应保留初始化默认值 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/cb8bb2fd5e6e2d115bc03c17f1f5f0ef6a81e019/docs/plans/unreal-alignment/reflection-transient-default.md>)；回归未新增/未运行 |
| [#164](https://github.com/He-Jiahui/ZirconEngine/pull/164) | [Skeleton] 对照 Unreal：合法后置父骨槽不应令蒙皮失效 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/f1e5c479f83865baea16eacdd8af08e80f9ea26c/docs/plans/unreal-alignment/skeleton-parent-slot-order.md>)；回归未新增/未运行 |
| [#165](https://github.com/He-Jiahui/ZirconEngine/pull/165) | [Timeline] 对照 Sequencer：暂停序列仍须响应绑定失效 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/63bec523ab175ba3a403bab5dd0c1ca91435a374/docs/plans/unreal-alignment/timeline-paused-binding.md>)；回归未新增/未运行 |
| [#166](https://github.com/He-Jiahui/ZirconEngine/pull/166) | [Physics] 对照 Chaos：旋转 Box 的 AABB fallback 不应缩界漏报 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/a17b845e151f0a054ddbb0030b154f6c2541bb5a/docs/plans/unreal-alignment/physics-rotated-box-bounds.md>)；回归未新增/未运行 |
| [#167](https://github.com/He-Jiahui/ZirconEngine/pull/167) | [Components/ECS] 对照 Mass：销毁须释放回调新增的 sparse 行 | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/d678eda10ee9fafc401587a4787f0b15801510c2/docs/plans/unreal-alignment/ecs-despawn-sparse-cleanup.md>)；回归未新增/未运行 |
| [#168](https://github.com/He-Jiahui/ZirconEngine/pull/168) | [Preview] 对照 Slate：已捕获 Game 手势的释放须回到原 Play | [独立文档](<https://github.com/He-Jiahui/ZirconEngine/blob/20252aa20d5f93b85ebd6b612a3525797f6dd47d/docs/plans/unreal-alignment/preview-captured-pointer-release.md>)；回归未新增/未运行 |

## 4. 全领域覆盖与未提新 PR 的原因

“无新缺陷”仅表示这轮读到的限定契约未证实新独立问题，不代表模块全部正确；“目录映射”不是行为审计。多个领域指向同一问题时只计一个 Draft。

CSV 的 `catalog_status` 与初盘 owner 状态保留目录阶段快照；首轮限定结论以其 `review_status` / `review_note` 为准，第7节记录后续状态；不将任一目录/审查快照误认成当前行为通过。

| 领域 | 本轮状态 | PR/依据 |
|---|---|---|
| 物理 | 专项源码复核/独立Draft | [#166](https://github.com/He-Jiahui/ZirconEngine/pull/166)；另关联现有#145；近似AABB不冒充完整narrow phase |
| RHI | 专项源码复核/无新独立缺陷 | [#132](https://github.com/He-Jiahui/ZirconEngine/pull/132)；已核提交退休、代次、transient等；既有#132测试不重复，未宣称RHI全面正确 |
| 音效 | 专项源码复核/独立Draft | [#156](https://github.com/He-Jiahui/ZirconEngine/pull/156)；只证NaN PCM准入；未试听或验证设备后果 |
| 组件 | 专项源码复核/独立Draft | [#167](https://github.com/He-Jiahui/ZirconEngine/pull/167)；安全回调新增sparse owned行的销毁清理 |
| 脚本 | 专项源码复核/独立Draft | [#149](https://github.com/He-Jiahui/ZirconEngine/pull/149)；既有P1-10的未变绑定onStart连续性子项 |
| 资产 | 专项源码复核/无新独立缺陷 | [#134](https://github.com/He-Jiahui/ZirconEngine/pull/134)；已合#134与probe publisher修正不重做；RecoveryDeferred缺消费者误用实证 |
| UI | 专项源码复核/独立Draft | [#150](https://github.com/He-Jiahui/ZirconEngine/pull/150)；仅祖先alpha累乘，不含离屏group compositing |
| 布局 | 局部源码/测试核对，无新独立布局题 | 未凑空 PR；R13响应式/grid/masonry现有及未合测试分开；不宣称所有布局等价 |
| 交互 | 专项源码复核/独立Draft | [#148](https://github.com/He-Jiahui/ZirconEngine/pull/148)、[#168](https://github.com/He-Jiahui/ZirconEngine/pull/168)；缓存裁剪与已捕获手势终态；非全部工具交互验收 |
| 渲染 | 局部源码复核/既有PR关联 | [#154](https://github.com/He-Jiahui/ZirconEngine/pull/154)、[#164](https://github.com/He-Jiahui/ZirconEngine/pull/164)；另关联#135/#138/#141/#144；不是Renderer全部feature行为审计 |
| 输入 | 专项源码复核/独立Draft | [#151](https://github.com/He-Jiahui/ZirconEngine/pull/151)；Web producer/ingress配对；不改共享manager契约、不接管R12 |
| 后处理 | 专项源码复核/独立Draft | [#154](https://github.com/He-Jiahui/ZirconEngine/pull/154)；内部baked LUT HDR域；不是ACES2算法或全部后处理对齐 |
| 项目 | 专项源码复核/独立Draft | [#157](https://github.com/He-Jiahui/ZirconEngine/pull/157)；Scene后缀接受范围一致性；不扩大为所有项目打不开 |
| prefab | 专项源码复核/无新独立缺陷 | [#133](https://github.com/He-Jiahui/ZirconEngine/pull/133)；维持Unity模型；已合#133数据/cache项不重复，withheld/UNKNOWN不当bug |
| 反射 | 专项源码复核/独立Draft | [#163](https://github.com/He-Jiahui/ZirconEngine/pull/163)；同schema非持久字段的显式默认值往返，非新版本迁移架构 |
| 构建 | 专项源码复核/独立Draft | [#161](https://github.com/He-Jiahui/ZirconEngine/pull/161)；显式target到产物来源身份；原函数纯内存见证不等于真实build/stage |
| 材质 | 专项源码复核/独立Draft | [#152](https://github.com/He-Jiahui/ZirconEngine/pull/152)；u32表示预算；不扩UE完整材质系统 |
| 粒子 | 专项源码复核/独立Draft | [#153](https://github.com/He-Jiahui/ZirconEngine/pull/153)；CPU非边界burst出生时间；continuous phase尚待冻结 |
| 蒙太奇 | 关联目录及播放owner已定位/专门语义仍待深入 | 未凑空 PR；不把缺少Montage高级功能直接判bug；不把时间轴问题算全套Montage审计 |
| 时间轴 | 专项源码复核/独立Draft | [#165](https://github.com/He-Jiahui/ZirconEngine/pull/165)；暂停序列绑定失效；不扩展Sequencer高级轨道 |
| 动画 | 专项源码复核/独立Draft | [#147](https://github.com/He-Jiahui/ZirconEngine/pull/147)；Base混合共同半球，保留ZE reorder/q正负号合同 |
| 骨骼 | 专项源码复核/独立Draft | [#164](https://github.com/He-Jiahui/ZirconEngine/pull/164)；合法后置父骨计算次序；保持原joint槽/ABI |
| VFX | 共享CPU后端源码复核/不重复立题 | [#153](https://github.com/He-Jiahui/ZirconEngine/pull/153)；与粒子同一问题；GPU/VFX graph未冒称全面验证 |
| editor_ui | 专项源码复核/独立Draft | [#148](https://github.com/He-Jiahui/ZirconEngine/pull/148)；retained命中缓存正确性；无FPS/分配优化实测 |
| 优化 | 局部诊断/正确性复核，非性能验收 | [#160](https://github.com/He-Jiahui/ZirconEngine/pull/160)、[#148](https://github.com/He-Jiahui/ZirconEngine/pull/148)；只证诊断峰值漏记/缓存语义；不承诺任何帧率收益 |
| jobs | 专项源码复核/无已证实新缺陷 | 未凑空 PR；阻塞observer候选有轻量通知契约反证，无合法生产调用，不凑Draft |
| 导航 | 专项源码复核/复用已有Draft | [#146](https://github.com/He-Jiahui/ZirconEngine/pull/146)、[#158](https://github.com/He-Jiahui/ZirconEngine/pull/158)；已有#146的Legacy world/local调用者问题与#158生产描述符admission问题，均由原独立PR跟踪，不重开 |
| preview | 专项源码复核/独立Draft | [#168](https://github.com/He-Jiahui/ZirconEngine/pull/168)；真实Slider capture、同窗、原pane仍focused；承认pane-focus/native reset |
| AI | 专项源码复核/独立Draft | [#162](https://github.com/He-Jiahui/ZirconEngine/pull/162)；既有AI-P1-020仅TimeLimit子项；不夹带Parallel/revoke |
| 行为树 | 专项源码复核/独立Draft | [#162](https://github.com/He-Jiahui/ZirconEngine/pull/162)；与AI同一Draft；UE TimeLimit原测试迁移优先 |
| 核心 | 专项源码复核/独立Draft | [#160](https://github.com/He-Jiahui/ZirconEngine/pull/160)；仅启用可选allocator wrapper的堆诊断，不是内存损坏/跨DLL统一记账 |
| IO | 专项源码复核/独立Draft | [#159](https://github.com/He-Jiahui/ZirconEngine/pull/159)；合法长leaf的staging/backup命名；OS只读probe非原子写回归 |
| 平台 | 关联owner及局部源码已核/无新独立缺陷 | 未凑空 PR；NTFS/平台条件已限定；不宣称跨平台产品验收 |
| 网络 | 专项源码复核/独立Draft | [#155](https://github.com/He-Jiahui/ZirconEngine/pull/155)；TCP已发生读取错误的事件/查询一致性，非全部复制/网络功能 |
| 插件 | 候选经反证WITHHOLD/不提新PR | 未凑空 PR；UE pauser只计完成不计成功，ZE best-effort/required政策未清；不把ABI非OK推成必须阻止Stop |
| 测试自动化 | 目录和测试来源元数据完成/未运行 | 未凑空 PR；414测试指示规则、2677候选C++和现有harness映射，不是已移植/已通过数量 |
| 导入烘焙缓存 | 局部来源/既有测试核对，无新独立题 | 未凑空 PR；不重开#134；DDC/cook全行为尚未深入 |
| 文本本地化无障碍 | 目录owner映射/未深入行为 | 未凑空 PR；UI透明度不等于文本布局、本地化或无障碍完整审计 |
| 媒体XR虚拟制作 | 目录owner映射/未深入行为 | 未凑空 PR；规则已枚举；尚无细粒度ZE等价owner和行为结论 |
| 第三方和平台外部库 | 外部规则/平台覆写元数据盘点 | 未凑空 PR；本地Source/ThirdParty不存在不等于上游没有；External不等于启用模块 |

## 5. 反证与去重保留

- 导航 Legacy world→local 写回复用既有 [#146](https://github.com/He-Jiahui/ZirconEngine/pull/146)，没有再开相同根因；并发新增的 [#158](https://github.com/He-Jiahui/ZirconEngine/pull/158) 单独跟踪导航 host admission，AI 真实 host 回归需先核前置，未把注册源码当启动回执。物理的新旋转包围体问题与 [#145](https://github.com/He-Jiahui/ZirconEngine/pull/145) 父子姿态写回不同。已有渲染/切线/夹具 [#135](https://github.com/He-Jiahui/ZirconEngine/pull/135)、[#138](https://github.com/He-Jiahui/ZirconEngine/pull/138)、[#141](https://github.com/He-Jiahui/ZirconEngine/pull/141)、[#144](https://github.com/He-Jiahui/ZirconEngine/pull/144) 不重复。
- 插件 restore 候选在独立反证后 **WITHHOLD**：UE pauser 只统计完成，不能作为“ABI 非 OK 必须阻断 Stop”的成功合同；ZE best-effort/required 分类尚未站稳。没有依原稿直接修复，也没有为凑模块新建 PR。
- jobs 阻塞 observer 候选因正式轻量通知契约及无合法生产阻塞调用被撤回。资产/prefab 不把 withheld/UNKNOWN、既有测试或 RecoveryDeferred 的语义直接当 bug。
- 输入草案已修正为 Web 私有 ingress 配对，保留通用 KeyboardInput 契约、raw UI 与 overflow FocusLost；Preview 已收窄为真实 Slider capture、原 pane focused，承认 pane-focus/native reset 与 UE 捕获模式差异。
- 物理现有 query fixture 有静态 spawn_node 返回值 API drift：它是待独立处理的验证前置，不是已运行的编译失败，也不是几何 RED；不得夹带生产修复或借此声称测试已跑。

## 6. 后续只在各自同一 PR 推进

1. 再核对应 UE 源文件/指纹，冻结等价输入、oracle、负向/边界控制与可达生产入口；先迁移确实存在的 UE 原测试。没有同例原测就显式标源码派生，不发明测试框架。
2. 在 ZE 既有 harness 中补回归并记录原实现真实 RED；编译/API drift/环境前置独立记录，不把编译失败当行为 RED，不恢复 Jenkins/旧协调器。
3. 先评估修复可行性、Rust/ABI/ECS差异及依据；只有站得住才在同 PR 作最小修复，再跑相同回归 GREEN。设备/窗口/产品/性能门保持独立，窄测试不得冒充整引擎验收。
4. 每个可编译命令前声明最小 engine 模块、Cargo 包、features、target/filter、profile、确切命令和覆盖理由。构建/编译缓存只在盘根 D/E/F 的 cargo-targets；不盲跑 workspace/all-features。
5. 当前只授权问题 Draft 与本任务后续流程，不因目录盘点/本地命令通过自动合并、改 CI 或接受未清问题。源代码、索引、他人工作树/未提交内容保持保护。

本轮文档级核验：规则源文件 2766 项 SHA256、领域 owner 154 项 SHA256、计数/模板/路径集合交叉核对及所有问题的六点/源链接/行范围/声明指纹检查；Git 最终 diff 与远端 PR head/file 范围逐一核验。这些仅是文档/来源证据，不是引擎测试。


## 7. 第二轮波次快照：2026-11-17（此时状态，后续见第8节）

此表记录首轮创建后真正发生的范围变化，不重写原始发布/目录指纹回执。8份CSV和2766规则、40领域计数均不变；它们的review/test字段是首轮快照，不是实时运行状态。

| 同一问题PR / 新独立PR | 当前进度 | 已验证与明确未验证 |
|---|---|---|
| [#161 Build](https://github.com/He-Jiahui/ZirconEngine/pull/161) | 原实现6案：4个行为断言FAIL、2控制PASS、0ERROR；可行评估后最小fix；已有2个Python模块18案全PASS；独立最终代码复核Scoped PASS | [当前记录与修改](<https://github.com/He-Jiahui/ZirconEngine/blob/f7ae575c8e016dcaaa60c6df27d3b0a081eb015b/docs/plans/unreal-alignment/build-target-artifact-identity.md>)。仅selector/callers与provenance绑定既有显式T、None兼容、8个聚焦case及既有控制。首轮稀疏Import ERROR不是RED；模拟DLL/PDB非真实PE。没有Cargo/UBT/真实产品发布验收，保持Draft。 |
| [#159 Core IO](https://github.com/He-Jiahui/ZirconEngine/pull/159) | 已在原zr_resource lib harness补6条源码派生回归代码；3次Cargo在索引/依赖前置退出101；**未编译或执行单测，无行为RED，无生产修复** | [当前记录与6条测试](<https://github.com/He-Jiahui/ZirconEngine/blob/6166e2c9911bc979104bb63a398664dc62dd6cd0/docs/plans/unreal-alignment/io-atomic-sibling-names.md>)。目录/缓存探针不当作测试PASS；hash命名不能串恢复、journal orphan删除分类及Windows ReplaceFileW自身名字仍是可行门。 |
| [#171 Animation/Notify](https://github.com/He-Jiahui/ZirconEngine/pull/171) | 第22个独立问题Draft，新增只读Montage/Slot/通知打断深审和双重反证 | [六点及精确见证](<https://github.com/He-Jiahui/ZirconEngine/blob/5cb8d1b1bbedc4b6f837019d9cc9dac7ca0b017e/docs/plans/unreal-alignment/animation-interrupt-notify-once.md>)：root A→B被B→C打断重扫B窗口，layer已有正确起点可复用。UE标准blend单node单次Update防重复，不是全局同名notify消重；AI最终记录可能合并，不声称伤害/感知翻倍。源码派生回归仍未新增/执行，不把缺Montage能力当bug。 |

深审推进不代表Montage整套能力、所有通知模式或其他未审领域通过；文本/媒体/XR/更多布局与导入烘焙等局部/目录域仍须继续。其余首轮问题仍按各自同PR补回归、真实RED、可行评估和最小fix推进。

并发原有导航 [#170](https://github.com/He-Jiahui/ZirconEngine/pull/170) 为独立facing-yaw符号问题，与本轮Notify和 #146/#158 均不同；留原会话跟踪，未重复建PR。**第二轮记录22个本任务问题Draft +1个目录Draft均不因窄单元GREEN自动变Ready/合并；产品门、运行窗口/设备门和性能门不降低。**


## 8. 第三轮当前状态：2026-11-17

**24个独立问题Draft +1个目录Draft；均未合并。** 原始8份CSV/40域和2766规则集合指纹仍冻结不变，目录数量不当行为审计通过。本节覆盖第7节中IO当时的前置失败状态，不抹掉当时回执。

| PR | 实际进展 | 验收边界 |
|---|---|---|
| [#159 原子长名](https://github.com/He-Jiahui/ZirconEngine/pull/159) | [当前修复/回归记录](<https://github.com/He-Jiahui/ZirconEngine/blob/5e6555df3ed914a6249fc20e52b0e5537081a4b3/docs/plans/unreal-alignment/io-atomic-sibling-names.md>)：合规F盘缓存原地复用后，原实现6条长名真实FAIL/21控制PASS；可行评估后4-owner最小命名/识别修复，原6不弱化，atomic48PASS+1原perfIgnored、durable23PASS。路径复核发现新test名超80/128，缩短且原字节保留，相关23重跑通过；独立最终代码复核Scoped PASS。 | 原target/同目录/排他/checked序号/平台原语/同步/wire未改；v2精确角色/owner与窄journal-staging分类。只Windows Resource单测，未测Unix/断电/真实Foundation产品保存/性能；23复跑不重复累加。旧短名归属未夹带。 |
| [#174 文本接续字素](https://github.com/He-Jiahui/ZirconEngine/pull/174) | 新独立[六点问题定义](<https://github.com/He-Jiahui/ZirconEngine/blob/90d0342a9a9d94d85f9fe066877f9ca9d83d9fe1/docs/plans/unreal-alignment/text-combining-edit-intent.md>)：focused可写TextInput的e后提交U+0301，合法UTF8 Insert差量被当成新EGC光标边界而拒绝。路由、精确delta/Undo合同与独立反证已核；保持byte delta与interaction grapheme分离。 | 仅1MD，未写或运行该问题测试，未修生产。Slate原ASCII插入是控制，接缝与Undo/Redo为源码派生；不声称原生按键序列/特定IME或CtrlV已实测。 |
| [#175 旧备份错属主](https://github.com/He-Jiahui/ZirconEngine/pull/175) | 新独立[六点问题定义](<https://github.com/He-Jiahui/ZirconEngine/blob/175cea6727bbe3c12224129c7455fb8a0010cfbf/docs/plans/unreal-alignment/io-recovery-exact-owner.md>)：缺失短config.json时，另一较长源leaf的staging或backup在单候选下被旧prefix+union角色判定采纳；Foundation先移动后JSON。同属主合同明确，独立反证通过。 | 仅1MD，尚无该问题实际RED或生产修复，与159长名/v2 namespace分开。UE精确Name/目录备份仅身份处理对照，不是相同自动恢复或原测试迁移。 |

#161工具层18GREEN和#171 source-only状态不变。当前文本/无障碍/本地化深审只保留#174有证据缺陷，不把缺高级功能当bug；其他未深入或局部域继续。并发导航[#173](https://github.com/He-Jiahui/ZirconEngine/pull/173)为Recast烘焙继承shear丢失，已按实际正文/owner文件去重，与174/175无关，留原会话；不接管#146/#158/#170/#172。下轮仍在各原PR先真实RED、可行评估、最小fix和相同GREEN，不自动合并或降低产品门。


## 9. 第四轮当前状态：2026-11-18

**25个独立问题Draft +1个目录Draft，均未合并。** 原始8CSV/目录计数及第7/8节时间点保留；本节覆盖第8节中#175尚未测试的旧状态。以下单元数按各PR独立输入记录，不相加为全引擎通过数量。

| PR | 同PR实际推进 | 明确限制 |
|---|---|---|
| [#175 旧备份归属](https://github.com/He-Jiahui/ZirconEngine/pull/175) | [当前阶段](<https://github.com/He-Jiahui/ZirconEngine/blob/97d360ff915c9d5415fae5b1abbe7b2c6185244e/docs/plans/unreal-alignment/io-recovery-exact-owner.md>)：在其e893独立基线先补7案，真实Windows **3FAIL/4PASS**；实际staging与故障保留backup都被移错目标，混合候选误歧义。可行评估后只改2个I/O owner的精确完整前缀/whole numeric suffix，旧形状与数字拼写规则不变；原7字节不变，加1合法含marker的owner。整个atomic组 **29PASS/0FAIL/1原性能Ignored**，原失败全转绿；独立最终复核Scoped PASS。 | 原target/同目录/typecheck/歧义/平台移动和同步不变，无API/ABI/wire/配置/CI改动；不叠入159。没有实际Foundation启动/系统崩溃/Unix/两PR集成或性能验收。 |
| [#178 同步截图归属](https://github.com/He-Jiahui/ZirconEngine/pull/178) | 新独立[六点问题定义](<https://github.com/He-Jiahui/ZirconEngine/blob/fa87f9e4086cccf1470b076dd51b3d2e5dcc2337/docs/plans/unreal-alignment/viewport-capture-owner.md>)：已创建未提交A、B实际成功离屏渲染，公共同步A读回无owner检查地回退renderer-last并污染A缓存；随后A直接poll可见错帧。合同/公共submit与capture链、UE同receiver纹理/MediaIO OnTick源绑定、独立源码反证通过。 | 仅1MD；没有新增测试或GPU实测、没有修复。既有HDR双视口是控制而非本例原测；本例为源码派生。动态ABI/mailbox是另一入口，未声称常规App PNG/Play/UI/媒体或XR产品受影响。 |

媒体/XR/虚拟制作有界深审还核对真实still-image importer、resident音频播放及单帧捕获的owner/准入；在已检first-party当前路径中没有建立timed video/图像序列播放、媒体设备/录制/流传输、XR pose/compositor/action、LiveLink/拍摄同步生产路径。缺功能只保留边界记录，不当缺陷或新增架构PR，不宣称覆盖所有外部providers/平台。现有UE媒体/XR/虚拟制作原测试来源已定位，但没有运行或迁移那些无ZE对应的测试。

#159的Windows71PASS/1原perfIgnored与#161工具18GREEN不变；#174/171仍source-only。已核对并发[#177](https://github.com/He-Jiahui/ZirconEngine/pull/177)的Sprite2D仿射顶点正文/owner为另会话，与同步capture来源无关；保留#176Physics及此前全部外部工作。所有后续仍独立问题先Draft、同PR真实RED、可行评估、最小fix/相同GREEN；未达产品门不合并。


## 10. 第五轮当前状态：2026-11-18

**26个独立问题Draft +1个目录Draft，全部未合并。** 原8CSV身份、全部冻结目录计数、旧阶段原始回执保留；本节覆盖第9节中#174尚未写测试及issue总数的旧快照。编译失败、源码反证、真实单元GREEN和产品验收分别列，不互相替代。

| PR | 本阶段已交付 | 尚未完成 / 前置 |
|---|---|---|
| [#174 Text精确Insert](https://github.com/He-Jiahui/ZirconEngine/pull/174) | 在既有harness补4案：UiInputManager仅U0301派发的原始bytes/caret与Insert old1..1/new1..3；同DocUndo/Redo；ASCII/空字段完整分解字素控制；整EGCBackspace。测试源码独立复核通过，仅测试+问题MD两文件，未改文本生产、manifest/lock。 | 两次实际lib编译均未进入4案：最小ui的既有Scene/Project/Plugin测试缺native-host引用，436错误；仅补native-host后剩已独立[#137](https://github.com/He-Jiahui/ZirconEngine/pull/137)的模型夹具E0716。**0执行、没有行为RED/GREEN**；不复制/夹带137、屏蔽模块或造runner，不因源码推导先提交fix。 |
| [#180 经典纹理缓存准入](https://github.com/He-Jiahui/ZirconEngine/pull/180) | 新独立[六点问题定义](<https://github.com/He-Jiahui/ZirconEngine/blob/5beaf53043d0a9f838936bbf3cdee52bd22680d4/docs/plans/unreal-alignment/texture-cache-active-version.md>)：同UUID的PNG经独立Removed→Added，新的active载荷与hash已变但数字revision可同1，prepared-texture快路仍误命中旧GPU对象。源码/原watcher调度/现有Gpu ensure门与UE owner/相邻原测，独立复核通过；仅1MD。 | 无新增测试/GPU执行或fix。**范围限缓存准入；撤回默认Core2D成功错色说法。** 并发独立[#179](https://github.com/He-Jiahui/ZirconEngine/pull/179)实际DX12在光栅前因depth Read声明失败，后续产品texel受该前置门约束；首个计划是现有原GPU fixture实际binding对象oracle，不用CPUkey/ready统计代替。两问题不叠栈修复。 |

资产异步/cook/registry深审还追了真实原生PNG准入、project watcher prepare/publish、资源commit与新载荷identity、Web Raw Fetch、semantic streaming pendingqueue/owner边界以及UE async package/DDC/cook/registry owner。没有把只追到队列的语义产品路线宣称为已接通，没有因缺功能登记第二个问题。UE原测来源明确区分包/texture邻近控制与ZE双watcher缓存准入/texel的源码派生回归；没有宣称测试已移植或运行。

既有独立#159Windows Resource71PASS/1原perfIgnored、#175独立旧备份atomic29PASS/1原perfIgnored、#161工具18PASS范围和源输入不变；不将重叠控制累计成全引擎通过数。#178Capture、#171Notify及其他问题按各自当前Draft阶段继续。保留并发#177/#179图形、#176物理及全部导航/其他会话的源码、锁和回执。全领域收敛与产品门仍未完成，不自动合并。


## 11. 第六轮当前状态：2026-11-18

仍是 **26个独立问题 Draft + 1个目录 Draft，0合并**；没有因一个修复闭环把整个 Networking 域或全引擎标为通过。首轮8份CSV、2766条目录身份及集合指纹完全不变。

| 原问题 / 工作项 | 本轮实证与推进 | 明确边界 |
|---|---|---|
| [#155 Networking](https://github.com/He-Jiahui/ZirconEngine/pull/155) | [同一PR已发布阶段](<https://github.com/He-Jiahui/ZirconEngine/blob/ae06746fc99ce3b0424115e91fa101cbf3e2fd61/docs/plans/unreal-alignment/tcp-read-error-state.md#L105-L135>)：既有net插件harness补4条源码派生回归；原实现实际 **3PASS/1FAIL**，真实回环RST使读取报Winsock10054，已产生同ID Failed事件而立即查询仍Open。独立可行复核后，仅3个内部owner把权威状态与内层读取Result配对，manager先同步状态再传播原Io。原4条字节未改且全GREEN；完整60个叶包case中 **56PASS/0FAIL/4原release性能Ignored/0filtered**。独立最终源码/证据复核Scoped PASS。 | 只Windows原生TCP接收状态+net叶包控制；公共API/ABI/ECS/schema/manifest/lock/feature未改，外层worker请求Io不猜Failed。初次离线缺另一个可选成员的固定Git依赖是0执行前置失败，不是RED；普通在线取得锁定源后实际执行，无workspace编译/VM启用/缓存配置复制。不是UE原测移植，不覆盖其他OS、所有错误、真实HTTP/WS可选后端、RPC/复制全链、App产品或性能，保持Draft。 |
| [#167 Components/ECS](https://github.com/He-Jiahui/ZirconEngine/pull/167) | 独立本地工作树已在现成observer harness薄挂载5条源码派生回归，检查Remove/Despawn回调新增稀疏行、替换/已有行、存活实体和真实Drop所有权；仅2份测试输入，无生产修改。 | **本地未编译/执行/提交或发布跟进，0行为RED/GREEN**。真实owner是Runtime，不虚构独立ECS crate；`core-min,native-host`无Graphics单位harness只是静态可行候选，另有既存公开World integration落点，尚未验证。 |
| [#160 Memory](https://github.com/He-Jiahui/ZirconEngine/pull/160) | 仅检查下一CPU回归的真实包/feature边界；仍为原问题Draft。 | `profiling-memory`的现成libtest闭包还会准入Graphics名称引用；私有counter隔离也不能用外围recorder测试替代。这是静态readiness限制，不是实际编译回执或另立的新问题，未加Graphics/屏蔽模块/新runner。 |
| [#174 Text](https://github.com/He-Jiahui/ZirconEngine/pull/174) / [#180 Texture](https://github.com/He-Jiahui/ZirconEngine/pull/180) | 第10节状态未变：Text4条仍0执行、未改文本生产；Texture仅经典预备缓存准入问题定义。 | #137编译前置与#179默认Core2D产品前置均不复制、叠栈或隐藏；不把来源指纹、CPU key或准备的测试当成行为/GPU产品PASS。 |

#155通过选择真正所属插件叶包，让Runtime仅构建为依赖库而非其libtest，合法避开无关图形测试闭包；没有抽取mini-runner、复制夹具或弱化oracle。首轮来源中的UE TcpMessaging与邻近原测归因不变，UE测试未执行。

#159的71个Windows Resource结果、#175的29个独立atomic结果和#161的18个模拟artifact工具结果保持各自范围，控制可能重叠，不作全引擎累加。下一步优先在#167原PR取得真实CPU RED，再独立评估最小修复；其余域继续按原顺序深审/反证/去重。保留并发#181/#177/#179图形/导航、#176物理及所有其他会话的源码、索引、锁、工作树与回执；全领域收敛、真实产品门和性能门仍未完成，不自动合并。


## 12. 第七轮当前状态：2026-11-18

仍是 **26个独立问题 Draft + 1个目录 Draft，0合并**。§11 中 #167 的未执行状态是当时快照；本轮只推进原 PR，不把一个清理 owner 的修复升格成整个 ECS/Runtime 或全引擎验收。8份 CSV、2766条规则/测试目录身份与集合指纹冻结不变。

| 原问题 / 工作项 | 本轮实际推进 | 限定与未通过项 |
|---|---|---|
| [#167 Components/ECS](https://github.com/He-Jiahui/ZirconEngine/pull/167) | [同一PR已发布阶段](<https://github.com/He-Jiahui/ZirconEngine/blob/21ba7619bc7844e738e7fc8c32436b776d6e62bf/docs/plans/unreal-alignment/ecs-despawn-sparse-cleanup.md#L138-L187>)：既有 Runtime CPU 原5案实际 **2PASS/3FAIL**，成功接纳 Remove/Despawn 插入后 sparse count仍1、拥有值Drop仍0。独立可行复核后仅改 hierarchy owner：通知快照不作最终拥有集合，读取当前 signature 清理；child orphan 前绑定原 InternalEntity，3处原 World/plan guard 之后核对原代际，沿用 MissingEntity 不重指替代实体。原5测试字节不变，新增3安全公开同ID/新代际恢复控制；实际 `cpu167_` **8PASS/0FAIL/0ignored/7775filtered**。最终独立 Scoped PASS。 | 仅 Windows MSVC1.94.1、`core-min,native-host`、既有 lib harness；20选定输入运行前后不变，不是完整源码/进程封印。未启用 Graphics/UI/Script，不改 public API/ABI/schema/feature/manifest/lock，不扫描全 storage、不循环 replay late 通知。不是 UE 原测逐字迁移/执行、所有 callback/OS/产品/性能验收，保持 Draft。 |
| #167 扩展既有控制 | observer 前缀 **31PASS/2FAIL**，identity 前缀 **37PASS/1FAIL**；通知/World-plan控制、两条旧 despawn 成本守卫及新3代际案都实际通过。 | **两组明确 NOT GREEN**。3条未修改源码字符串守卫失败：CRLF多行匹配、已过时 Fn 回调文本、旧 sparse locator 字段文本。消费输入与 e893 规范化一致，是直接源码反证而非已运行宽基线的证明；旧 locator 案后续运行断言未到达。无 skip/gate/弱化，保留所有非零回执，不在本修复夹带处理。聚焦8与宽组重叠，不累加为全域通过数。 |
| [#156 Audio](https://github.com/He-Jiahui/ZirconEngine/pull/156) | 只读复核确认真实 `audio_importer.wav` registry handler 到 SoundAsset IEEE32 decode，合法float32 WAV可成功保留NaN；明确真实 owning leaf 是 `zircon_plugin_audio_importer_runtime`，不是 metadata wrapper。 | 仅 source/readiness，**0编译/执行/行为RED/fix**。UE ordinary factory 的成功float32路径转PCM16，但不能宣称UE全局拒绝NaN输入或某个精确输出样本；未找到同输入 exact UE 原测。既有 libtests 即使过滤 WAV也编译OGG include fixture：E参考树有、owned D无，不复制/overlay/屏蔽/另造runner求过；这是下一步真实前置，不是 Graphics137或glTF142的问题。 |
| [#160 Memory](https://github.com/He-Jiahui/ZirconEngine/pull/160) / [#174 Text](https://github.com/He-Jiahui/ZirconEngine/pull/174) / [#180 Texture](https://github.com/He-Jiahui/ZirconEngine/pull/180) | 各自上一阶段保持：Memory静态 feature/readiness；Text4条已补但0执行；Texture经典缓存准入仍仅源码问题。 | profiling-memory测试闭包、独立137模型夹具和179默认Core2D产品前置均未旁路、复制、叠栈或伪造通过。没有扩大成全Runtime UI/GPU libtests重跑或其他问题生产修复。 |

#167 通过真实 owning CPU feature closure 编译原 Runtime unit harness；不是提取 mini-runner/验证副本，也没有把旧测试编译闭包裁掉。原5保留 count/Drop/存活实体的强 oracle；新增3只证明原目标代际围栏与已接纳替代实体保留，不许把新控制倒写成原实现已经取得过 RED。

#155原4同字节GREEN、net叶包56PASS/4原release性能Ignored；#159Windows Resource71、#175独立atomic29、#161模拟artifact工具18的范围不变。各 PR 的产品/OS/性能门仍独立，不能累加结果形成全引擎通过。保留并发#186/#185/#183/#182/#181导航、#184/#177/#179图形、#176物理及其他会话的源/索引/工作树/锁/回执；下一轮继续各原PR的事实调查、等价测试和有界修复，所有Draft不自动合并。

## 13. 行为树超时撤销与完整基线对照：2026-11-18

[#162 AI / BehaviorTree](https://github.com/He-Jiahui/ZirconEngine/pull/162) 已在原 Draft 修复。此前 TimeLimit 到期会返回 `Failed`，但仍运行的 MoveTo 留下旧导航目标，Selector 也可能在外部子任务收到撤销之前执行后备任务。修复只连接既有子树撤销入口，并保护未调用宿主的静态占位节点。详见[源码、UE 测试依据与四次原生结果](https://github.com/He-Jiahui/ZirconEngine/blob/f7bb754968398a167c5b7220632a525c7ca3ff3c/docs/plans/unreal-alignment/ai-time-limit-child-abort.md#L80-L120)。

| Windows/MSVC 运行范围 | 原实现（仅加测试） | 修复后 |
|---|---|---|
| 相同4条回归：真实导航目标、撤销先于后备任务、第三方目标不受影响、自然完成不误撤销 | 2 通过 / 2 失败 | **4 通过 / 0 失败** |
| 完整 `zircon_plugin_ai_runtime` 库，235 项，无过滤 | 180 通过 / 12 失败 / 43 忽略 | **182 通过 / 10 失败 / 43 忽略** |

同一工作树、同一命令实跑两个版本，逐项配对后只有 #162 的两项失败转绿。其余10项失败的断言位置及内容相同，43项既有 release 性能忽略也相同。两次全库命令均退出101，**不是全库通过**。UE 原测提供同步撤销顺序的对照，不等于在 UE 执行测试或实现其异步撤销确认；没有验证产品导航启动或其他 OS。

下一项 [#163 Reflection](https://github.com/He-Jiahui/ZirconEngine/pull/163) 已定位[既有 World 注册/反射测试入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/reflect/tests/vm_type_backing.rs#L16-L55)：可在源和目标 World 明确注册同一 schema，检查快照排除非持久字段且恢复使用声明默认值，不需伪造脚本 VM 或加载图形模块。**尚未写入或运行该回归，也未修复 #163。**

仍是 26 个独立问题 Draft 加本目录 Draft，未合并。8 份目录 CSV 保持原始内容；#167 两个扩展组的3条旧失败及其他问题的验收边界未变。

## 14. 反射存档：保留省略语义，在目标构造默认值（2026-11-18）

[#163 Reflection / DynamicScene](https://github.com/He-Jiahui/ZirconEngine/pull/163) 已在原 Draft 发布修复。源实例的非持久值 `scratch=99` 正确地没有写入场景；原实现恢复到已注册同一 schema 的 World 时，却因 `scratch` 缺席拒绝整个对象。现在恢复为 `{saved:7,scratch:42}`，其中 `42` 来自目标声明。缺持久字段或无明确默认仍应失败；只有普通描述符的场景不推断 VM 默认。补值前按每个实例计入目标快照预算，充分有限额度与 `usize::MAX` 对同一输入返回相同估算。[修复、UE 对照边界和原生结果](https://github.com/He-Jiahui/ZirconEngine/blob/b7f6130d3fa06d2f81253895fc13fe5ad5d42f52/docs/plans/unreal-alignment/reflection-transient-default.md#L75-L119)

| Windows x64/MSVC，Runtime `core-min,native-host` | 原实现（过 / 败 / 忽略） | 修复后（过 / 败 / 忽略） |
|---|---:|---:|
| 原 6 条反射与预算用例，测试文件不变 | 4 / 2 / 0 | **6 / 0 / 0** |
| Prepared 组，包含第 7 条“限额不改变估算，显式值不被覆盖”控制 | 2 / 2 / 0 | **3 / 1 / 0** |
| 场景事务组 | 23 / 1 / 1 | **23 / 1 / 1** |
| 热重载字节预算组 | 3 / 1 / 1 | **3 / 1 / 1** |
| World 暂存组 | 未单独复跑 | **1 / 0 / 0** |

独立终审只签收 **7 条目标用例**。3 个相邻非零组的 **34** 个同名用例逐项配对，唯一变化是第 7 条转绿；两处过期目标的错误类型与断言不符、一处队列未报告帧预算耗尽，在原实现都可用相同输入复现相同断言。2 项既有性能忽略不变。未因此另开无据问题；没有运行全 Runtime、UE 原测、脚本 Play 或产品验收。

后续两个领域目前只有源码准备，没有测试或修复进展：

- [#157 项目 / Editor](https://github.com/He-Jiahui/ZirconEngine/pull/157)：须区分真实工程 `scan_and_import` 成功、[资源目录投影](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/project_sync/tests/mod.rs#L68-L77)与 picker/authority 打开；手填 `kind=Scene` 的列表不是导入成功的证据。Editor 测试会引入 Runtime `target-editor-host`，不是无图形依赖的 parser 测试。
- [#165 时间轴](https://github.com/He-Jiahui/ZirconEngine/pull/165)：[真实 Level 帧夹具](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/runtime_helpers.rs#L109-L144) 要求特定 D 盘工作目录和 smoke 临时区，与本轮使用的本地构建环境不同。不能把暂停序列的请求计数代替真实帧属性写回，更不能跳过夹具伪称跑通。

本审计仍有 **26 个问题 Draft + 1 个目录 Draft，0 合并**；8 份目录 CSV 未变。

## 15. 项目与编辑器：区分场景导入、目录发布和按原名打开（2026-11-18）

[#157](https://github.com/He-Jiahui/ZirconEngine/pull/157) 的[首次原生测试](https://github.com/He-Jiahui/ZirconEngine/blob/5d2f18ec35bab25a93c836992262a4093a000b8a/docs/plans/unreal-alignment/project-scene-suffix-case.md#L82-L94)证实了差异：同一工程中，小写、混合和全大写文件名的三个 Scene 都 `Ready`，按原 URI 能从磁盘解码出三个不同的标记。Editor 却只肯打开小写源，对另两项报 `SceneTarget`。受控 Open 列表也漏行，Create 请求拒绝混合后缀。首次粗筛另命中旧 Inspector 容量用例：完整为 3 过 / 4 败 / 1 忽略，而不是“4 个后缀缺陷”。真实宿主夹具的第四个失败当时连小写行都未观察到，并非该后缀断言。

[修复和完整回归结果](https://github.com/He-Jiahui/ZirconEngine/blob/7657c511d41cf261f4c78f5cd226bdd373a9090b/docs/plans/unreal-alignment/project-scene-suffix-case.md#L96-L111)：只在 Editor 共用完整 ASCII 后缀判断，不更改 URI 拼写、kind 或文件保护。加强真实宿主前置后，旧引擎与候选用同一精确五案：**1 过 4 败 → 4 过 1 败**。真实场景打开、受控 Open 和 Create 三项由红转绿，负控保持。工程文档组 7/7、显式保存控制 1/1、选择器组 12/13；候选四组按完整名称去重后 21 项、20 过 1 败。余下的是同一真实宿主前置：Runtime 三条精确 `Ready`，但 Editor 初次读取及一次正常刷新后仍是空目录。未验证窗口可见、全部 Editor 或 UE，也没有为尚未定位根因的目录现象另开问题。

[#148 工具栏裁剪](https://github.com/He-Jiahui/ZirconEngine/pull/148) 另有源码边界澄清：普通主窗 Scene 若由 [Workbench 接管](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/contract/pointer/routing/panes/entry/local/document.rs#L18-L47)，并不经过有疑问的 native toolbar 路由。接下来应单独验证未被接管的 pane 或分离窗口，不能把手造矩形当成产品点击；本轮没有跑 #148。

本轮重新搜索本机 UE 整棵参考树：**2588 个 Build 规则、178 个 Target 规则**的完整路径集及文件 SHA 与冻结目录逐项相同；另复核 154 个领域–UE 源码关联指纹。仍是 **26 个问题 Draft + 1 个目录 Draft，0 合并**。8 份 CSV 未改；40 个领域的目录映射不是全部模块通过行为验收。

## 16. Editor 资产：先区分目录缺行与完成状态误报（2026-11-18）

| 独立 Draft | 在共同引擎基线确认的事实 | 下一判据 |
|---|---|---|
| [#202 注册 Editor 的初次目录缺少目标](https://github.com/He-Jiahui/ZirconEngine/pull/202) | 三条未改引擎的原生 Editor libtest 命令：直接投影 5 过；已有真实宿主 UI `.zui` 与 Animation `.zranim` 各 1 败，均在查目标 locator 时失败，尚未分发打开动作。两个失败自身**没有打印目录总行数或 Runtime Ready**。 | 同一批实际源：对齐 active Runtime 输入、注册 Editor 代、订阅、当次 open 日志及至多一次真实 refresh；不造行、不等到绿。只有证据指向某个分支才修。 |
| [#201 未发布目录代也会报告打开完成](https://github.com/He-Jiahui/ZirconEngine/pull/201) | 独立的 producer/consumer 差异：给 formatter 健康 Runtime 统计、持久 settings、Editor revision/epoch 均为零，源码仍产出 `completed`；同一日志的正式门会拒绝零代。`completed` 的存活探针只是进度，不是最终验收。 | 单独验证 0/0、一个身份为零，以及**已发布的空目录**，保留原有失败/设置/转义控制。不要把模板专用最少四行变成通用分类，不改公开格式或刷新服务。 |

[#202 六点与原生回执摘要](https://github.com/He-Jiahui/ZirconEngine/blob/1cb02ef435239e4a5f627e1874d5e0aa558799d8/docs/plans/unreal-alignment/editor-initial-catalog-visibility.md)、[#201 六点与原测迁移边界](https://github.com/He-Jiahui/ZirconEngine/blob/8c4ce2d4b59d1a26f05caf10fbd83e7830348bab/docs/plans/unreal-alignment/project-open-catalog-status.md)。在两个初始 Draft 中，各自只新增本问题的一份 Markdown；与调查并行执行的是基线既有的 7 个原生回归，不是两个新测试套件都通过。两问题不随 [#157](https://github.com/He-Jiahui/ZirconEngine/pull/157) 的场景后缀修复更改：那里实测三条 Scene 为 Ready、同一 Editor 管理器刷新前后均是初始空代，才有零行证据。普通 Shader IDE 生成物位于工程 cache、同字节不重写，不能由此认定 watcher 自激。窗口首帧、完整 staged F1/F5、持久或最终空表，以及究竟哪一个提交分支失效仍未证。

**全域既有测试线索再校验。** 重新打开清单内 2,677 个 UE C++ 测试候选文件，记录逐文件 SHA-256；排序 `path + NUL + SHA256 + LF` 摘要 `c996e2b667f12d77cd5577593860d439dd07e473bd13ceba5c0e640b89b8ecf4`。按 11 个常见注册/测试宏做词法分流：1,363 个文件至少一个匹配不在所识别的字面 `#if 0` / `#elif 0` 分支，14 个文件的匹配全部在该类分支（例如 TEDS `ValidateState`），1,300 个未匹配这 11 种宏。数字是**文件数**，不是已注册、已执行或通过的用例数；词表是行首的 SIMPLE/COMPLEX 和两种 CUSTOM `IMPLEMENT_*_AUTOMATION_TEST`、`BEGIN_DEFINE_SPEC`、`DEFINE_SPEC`、`TEST_CLASS_WITH_FLAGS`、`TEST_CLASS`、`TEST_CASE`、`TEST_CASE_NAMED`、`GTEST_TEST`。这不是 C++ 预处理或全树用例发现：其他配置条件、宏和清单外源码另查。

**#201 消费端对照：** 实调已有 `Get-MvpEditorProjectOpenEvidence`，输入人工构造的诊断文本，8 组均符合预期：2 组接受、6 组拒绝，含两种单零、双零、有效 starter、已发布但此模板零行及最后一条记录选取。既未调用 Rust producer，也未启动 staged Editor；不是产品通过。

**同 PR 后续原生实证。** [#202 冻结测试与结果](https://github.com/He-Jiahui/ZirconEngine/blob/1deb8f48423a8d764c920f0dd6e8b2568fd18100/docs/plans/unreal-alignment/editor-initial-catalog-visibility.md)：冻结回归实际 0过1败，exit 101：真实小写 Scene/Ready、605字节制品解码与直接索引通过，注册 Editor 初读及一次显式刷新均为 0 行/零代。同一 activation 的 Runtime 序号 12→19，旧 token 从 Current 变为 Superseded，watch 对账提交计数 9→16；具体内部拒绝分支和 OS 触发源仍未证。原 .zui 的 importer 实测不在该宿主中，实际 opener 已 degraded，并非 #201 的误报现场。无生产修复或窗口验收。

公开调查数更新为 **28 个独立问题 Draft + 1 个目录 Draft；本任务未合并**。全模块目录仍为 2,588 个 Build 规则、178 个 Target 规则及 40 域映射；8 份 CSV 内容未变。既有 [#134](https://github.com/He-Jiahui/ZirconEngine/pull/134) 的 Runtime 注册/读取等价控制不重复迁移，未把 UE 的内存对象查询说成 ZE 已有启动时序验收。

## 17. 诊断误报已窄修；动画短弧取得真实 RED；复合源时间单列（2026-11-18）

| 领域、原 Draft | 本轮证据 | 尚未证明或仍需做什么 |
|---|---|---|
| [#201 项目打开日志](https://github.com/He-Jiahui/ZirconEngine/pull/201) | 原生产 formatter 的 9 个 Rust 用例 **6 过 3 败 → 同 9 案全过**。三份原失败日志 13 token 各只错 `result`；候选另有 2 个各不重名的既有 GUI startup action / 精确文案控制均过。 | 只改私有分类，双非零也不等于证明当前 activation；不保证已构建/运行 staged GUI 或 F1/F5。 |
| [#147 动画 Base 混合](https://github.com/He-Jiahui/ZirconEngine/pull/147) | 真 `blend_graph_base_poses` 叶子：原 4 案保留，另 6 案，共 **6 过 4 败**。跨符号缝的近邻没有得到实际短弧中点；同一旋转 q/−q 和广角重排保护不能替代此回归。 | 不用“首项锚”破坏顺序不变性，也不为转绿引入最大权重换锚的可见跳变。另需处理可接受的跨帧/广角策略与真正的 Level 采样，未改动画生产。 |
| [#204 复合资产生成写回与源时间](https://github.com/He-Jiahui/ZirconEngine/pull/204) | **新增独立调查，只有文档**。同一 compound 描述符既供源 mtime 采样又接收生成结果；本机同参数 Windows 文件替换后目标使用 staging 的 last-write time，不保留原值。UE 已核 `FAutoReimportSimpleModifyTest` 的真实缓存/源变更，不是 ZE 自写回直接复现。 | 不宣称引擎已 RED。下步用一个最小真实 compound Shader，两次同步全扫，首次后不写、不 sleep、不启 watcher；保普通 source touch、真实作者/成员变更正控。 |
| [#202 注册 Editor 首目录](https://github.com/He-Jiahui/ZirconEngine/pull/202) | 上轮同 root/activation 的 12→19、7 次 reconciliation、Ready Scene 与未初始化 Editor 快照仍是原证据，这次没有覆盖或推翻它。 | #204 不证明是哪条文件事件、哪一代变化或内部拒绝造成此次观察；不能改 fence 去消红。 |

[诊断配对源码与回执](https://github.com/He-Jiahui/ZirconEngine/blob/bdb937bf2e964fb4aaa826266f6230072da4c05e/docs/plans/unreal-alignment/project-open-catalog-status.md) · [动画冻结回归、实际轴向与候选反证](https://github.com/He-Jiahui/ZirconEngine/blob/8a0dcbab4f6d4fcadae56445e7149fbf557d69b3/docs/plans/unreal-alignment/animation-blend-hemisphere.md) · [复合时间的六点与 UE 原测/同路径无写入验证门](https://github.com/He-Jiahui/ZirconEngine/blob/c8af83b8c5465f3e108c3848ea39f1ab7e81aaed/docs/plans/unreal-alignment/compound-sidecar-source-time.md)。全模块规则仍是 2,588 个 Build + 178 个 Target、40 个领域；2,677 个 UE 测试候选文件不等于运行用例数，8 份 CSV 均未改。**29 个独立问题 Draft + 1 个目录 Draft；本任务未合并。**

## 18. 实测：资产无变更全扫、WAV 非有限样本与 Editor 首目录（2026-11-18）

| 原 Draft | 本轮验证 | 修复边界 |
|---|---|---|
| [#204 复合资产源时间](https://github.com/He-Jiahui/ZirconEngine/pull/204) | 真实 Runtime 全扫 **3 过 1 败**。首扫后不写输入、不启 watcher；同一根的三份解码制品与两个成员不变，第二扫却将首扫自己写出的文件时间记为源时间，输入代 **13→14**，仅一条 `Modified`。两份落盘描述符只有时间字段不同。 | 成员/描述符显式 touch 均有通过的正控，不能靠忽略描述符取绿；保时也须覆盖故障恢复和真实外部操作。 |
| [#156 WAV 导入](https://github.com/He-Jiahui/ZirconEngine/pull/156) | 真实公共 parser 与注册导入器 **4 过 2 败**，合法 mono 和 stereo 都成功产出保留原位型的 NaN。参考树自带 libsndfile 在文件/虚读两个入口均保留该位型；不是 UE Factory 运行。 | 首次离线缺 `symphonia 0.5.5` 未启动测试，未作 RED；锁定依赖就位后才有上述结果。静音/满幅替换或明确拒绝、已有缓存的升级语义仍需确定，不以任意报错代替。 |
| [#202 注册 Editor 首目录](https://github.com/He-Jiahui/ZirconEngine/pull/202) | 单例 0 过 1 败；同 root/activation，输入代 11→17、1 Modified，两端内容仅两处 mtime 槽位不同。Editor 两端行数 0 / 0。 唯一净变行是 `res://shaders/pbr_shader`，不是 `.zui`。 | 观察的是显式刷新前后两个端点的**净变化**，不是每个 watch 批次或内部 token 决策；不能仅凭 #204 的独立全扫例给真实打开归因。 |

[资产两扫、物理文件与正控](https://github.com/He-Jiahui/ZirconEngine/blob/2213793438bbc772d0aa35c1645b31d7a3f9b753/docs/plans/unreal-alignment/compound-sidecar-source-time.md) · [WAV 逐位输出与依赖/组件证据](https://github.com/He-Jiahui/ZirconEngine/blob/1e74ec6b674d0d7c04d5ee7f1b29c8e55ed03e96/docs/plans/unreal-alignment/wav-finite-pcm.md) · [Editor 原见证与新端点结果](https://github.com/He-Jiahui/ZirconEngine/blob/e892e0545b20b84e374e6a1a31a37216cb84772a/docs/plans/unreal-alignment/editor-initial-catalog-visibility.md)。本轮都在已有独立 Draft 上补测试与证据，没有改生产或验收脚本；[上轮 #201 的日志修复](https://github.com/He-Jiahui/ZirconEngine/pull/201)仍与 #202 分开。**29 个问题 Draft + 1 个目录 Draft，本任务未合并。** 全域枚举（2,588 个 Build、178 个 Target、40 个领域）及 8 个 CSV 均未改变；枚举不是行为通过数。

## 19. 动画通知预算、骨架绘制及 Editor 提交分支（2026-11-18）

| 问题 | 已有证据 | 未涵盖 |
|---|---|---|
| 新 [#207 循环点通知](https://github.com/He-Jiahui/ZirconEngine/pull/207) | 没有打断迁移，仅一个区间。无分页为 80；默认 64 条预算，真实采样器与 LevelSystem 队列都交付 **64+32=96**。实跑 1 过 2 败；短帧/精确周期的控制各为 80。 | UE 普通 Sequence 与 Montage 的区间扫描是源码参照；未运行 UE 或整套动画插件。与 [#171](https://github.com/He-Jiahui/ZirconEngine/pull/171) 的重复生产区间分开。 |
| 原 [#164 骨骼后置父槽](https://github.com/He-Jiahui/ZirconEngine/pull/164) | 257 骨的真实 CPU 蒙皮、DX12 光栅读回：原版父骨在前可画、后置父骨退化为绑定姿态；只修父依赖求值后，同一三帧 0 过 1 败 → 1 过 0 败。 | 两骨 GPU 蒙皮的 Base 错误代理仍在源校验失败；CPU 变形后用 GPU 绘制不等于 GPU 蒙皮通过，未宣称性能或全 renderer 验收。 |
| 原 [#202 注册首目录](https://github.com/He-Jiahui/ZirconEngine/pull/202) | 专用分段观测 0 过 1 败。首次打开拿源版本 **3**、唯一显式刷新拿 **10**；两次都命中 Runtime 令牌过期的真实拒绝分支，没有发布 Editor 目录。 | 同一刷新前后 10→16，仅一条 shader 的两处时钟有净变化；没有逐批证实是谁写入，也不放松提交身份检查。 |

[通知的原生位型与次数](https://github.com/He-Jiahui/ZirconEngine/blob/a7e7a21b2abbeeaf21fedd9659c33987b5887cfe/docs/plans/unreal-alignment/loop-notify-resume.md) · [骨架三帧与不同管线边界](https://github.com/He-Jiahui/ZirconEngine/blob/67553e62febec21be5b7c6ca044bd34f057e1291/docs/plans/unreal-alignment/skeleton-parent-slot-order.md) · [Editor 两个同步 span 的原生样本](https://github.com/He-Jiahui/ZirconEngine/blob/6e25d20e8140059583981a47ddf3fb43bdaffe12/docs/plans/unreal-alignment/editor-initial-catalog-visibility.md)。本轮只在 #164 的私有蒙皮矩阵求值上改生产；其余均为调查/测试。 现有 **30 个独立问题 Draft + 1 个目录 Draft**，本任务无合并；2,588 个 Build、178 个 Target、40 个领域和 8 个冻结 CSV 不变，不能把目录数量当作产品通过数。

## 20. 材质准入、Prefab 保存、动画续页和 Editor 文件事务（第16轮）

| 原问题 | 本轮实证 | 范围 |
|---|---|---|
| [#152](https://github.com/He-Jiahui/ZirconEngine/pull/152) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/aa68636392c8bbd993f4e198621a094ce08df591/docs/plans/unreal-alignment/material-option-bit-budget.md) 材质 | 32 位边界能保留枚举值 2；33 位作者声明与旧缓存均不再把 2 别名为 0，同一份坏缓存载荷（解压后 3,359 B）在重开后由合法源重建。**2 过 3 败 → 5 过 0 败**。 | Surface 文本导入/持久读回；不改 ABI，不是全部 typed 构造、warm prewarm 或 GPU。 |
| [#210](https://github.com/He-Jiahui/ZirconEngine/pull/210) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/96b98b9edb22bc017ac0b80b4b0873dadf81fcfd/docs/plans/unreal-alignment/prefab-null-override-save.md) 新 Prefab/Scene | 普通场景文本已修：两个原生用例从 0 过 2 败变为 2 过 0 败。现有空值覆盖行保留两条路径，只省 `value`；三实例混排没有串行。 | Unity 风格不变；不是 Apply、Revert、嵌套任意 JSON、UE 或窗口操作验收。 |
| [#207](https://github.com/He-Jiahui/ZirconEngine/pull/207) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/12acadade50649f9925a396fe1fa0fc351788488/docs/plans/unreal-alignment/loop-notify-resume.md) 动画 | 同一 filter 扩为 7 案，实际 3 过 4 败。20 轨、同一百毫秒正反向区间，每轨五次；0→0.1 实发104；0.1→0 实发84。另有相邻表示值及字节/span 预算正控。 | 未改生产采样器；未把全插件或极端时刻新语义算成通过。 |
| [#202](https://github.com/He-Jiahui/ZirconEngine/pull/202) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/d8a50d923ee4f6e8a48e34f88f5fa7c6374c0d53/docs/plans/unreal-alignment/editor-initial-catalog-visibility.md) Editor 资产 | 首扫与无编辑对账均改写复合描述符，Shader IDE 生成前后三个源文件不变。独立对照未复现“完整目录仅两处 mtime 变化”的组合条件，并未识别其它具体字段。正常打开留下 14 个描述符版本，13/13 对是上个物理时间进入下个持久时间；14 份事务日志只有 intent 而无 target committed 帧。不能用这些样本确定逐次 watcher 或 fence 写手。 | 区分无 watcher 的独立对照、正常打开的实际文件事务与测试的外部端点，不将原始文件通知虚构为 Runtime 消费的具体批次或 fence 的 current token。 |

累计 **31 个独立问题 Draft + 1 个总索引 Draft，零合并**。八份冻结的 UE 模块、目标、领域及测试定位 CSV 未变。新问题先从共同 main 只提交调查，再在原独立 PR 追加回归；已修的行为不代表整套模块或产品验收。

## 21. 分页通知逐位修复、旧音频缓存及两种菜单输入错误（第17轮）

| 独立 PR | 本轮可复核的结果 | 没有声称的覆盖 |
|---|---|---|
| [#207](https://github.com/He-Jiahui/ZirconEngine/pull/207) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/df8ff07ecbe8d94eb01006ebd735c11d205d3c4c/docs/plans/unreal-alignment/loop-notify-resume.md#L133-L147) 动画 | 原版 13 案 3 过 10 败。第一次候选 12 过 1 败，暴露测试把倒放当十进制等间距。用**未修** 20 轨实际单批核定两向各 100 条和每个位型后，同一份严格 13 案 **3 过 10 败 → 13 过 0 败**。正放 `104→100`，倒放 `84→100`；真实 Level `64+32→64+16`。约 1,677 万次距离的两方向各用 **92/93** 次原生步进调用；整个采样器 `26` 过、`2` 个受管用例忽略。 | 只修同一稳定范围的有界分页；不改变原种子、数学圈数或独立帧拼接，不是完整插件与 release 性能验收。 |
| [#156](https://github.com/He-Jiahui/ZirconEngine/pull/156) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/b2f0070ef62e94c9c18ced4b797ca4acce650e12/docs/plans/unreal-alignment/wav-finite-pcm.md#L161-L176) 音频 | 真实 v1 WAV 插件在两个源上首次各导入 **1** 次；同 PM 再扫及新进程冷重开各 **0** 次，`lib://`/`res://` 均从同一真产物读回 qNaN (`7fc00000`) 与有限对照。另一份独立工程只变测试委托描述符版本即每源重导 **1** 次、动作 key 改变。保留了未交给任何候选的第二份 v1 工程。 | 两项显式历史采样通过是确认风险；并未制定非有限 PCM 的处置/迁移政策、修 parser 或测试声卡。 |
| [#212](https://github.com/He-Jiahui/ZirconEngine/pull/212) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/58160a049ba93aac49abb63f7d825fe30e452a93/docs/plans/unreal-alignment/hidden-fallback-menu.md#L45-L51) 新 · 不可见的后备按钮 | 已在最小 owning libtest 提交四个 Runtime 见证：可见 Start/Retry、被项目 HUD 替代后的两种按钮、界外和相机输入。实跑构建止于独立 #137 的图形测试 E0716，四案均未执行；没有 UI 行为 RED/GREEN、没有生产修复。 | 区分提交 UI 与实际设备画面，不改变可见按钮的 MouseUp 策略。 |
| [#213](https://github.com/He-Jiahui/ZirconEngine/pull/213) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/1a6b8aca6843dc7a819eeece9894b1372a8da47f/docs/plans/unreal-alignment/menu-touch-cancel.md) 新 · 触摸取消 | 与上一项相反，此处按钮确实可见。Wayland/Winit 的同指取消能以原坐标进入 Runtime，被正确记为 Cancelled 后却按一次完成点击写游戏命令。初稿只含六点调查。 | UE 本机移动实现有合并为 Ended 的反例；不声称所有平台或硬件已跑。 |
| [#214](https://github.com/He-Jiahui/ZirconEngine/pull/214) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/0ddc9092b2f8df6c0e56640f3075cc60e169475b/docs/plans/unreal-alignment/paused-transition-event-replay.md) 新 · 暂停帧淡化完成 | A→B 未中断；暂停时根时钟冻结，淡化却按世界 dt 完成。恢复后同一非循环 B 通知窗口可再次提交。建立有限源码见证，尚未把预测算作原生红例。 | 与 #171 的中断同帧重提、#207 的单范围分页不同；不把正常双时钟相位差或缺少 Montage 功能当作本问题。 |

**本轮还核对了没有一一对应关系的领域。** UE 的 Media 播放/时钟、OpenXR 会话、LiveLink 和 VCam 有各自的正式入口。现有 UE 原测如 `System.Media.Assets.Playlist` 验列表边界，`OpenXRRenderModel.Loader.SuccessPath` 用替身验证模型加载，`System.Engine.Animation.LiveLink.Interpolation` 用两帧验证插值；都不是硬件验收。ZE 的[普通相机投影及无效 FOV 原测](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/scene/viewport/tests/projection.rs#L3-L35)、[本地声音的 pause/seek](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/sound/manager/playback.rs#L7-L32)和首帧 PNG 不能替代上述合同。没有以缺少高级功能为由开 PR，nDisplay/Takes 仍是目录级。Montage 的 section/slot 也不能直接等同于 ZE 已有的 clip、状态机或属性 Timeline；对照到的具体行为应按各自的生产者和消费者追踪，而不是新建一个“补完整个 Montage”的缺陷。

累计 **34 个独立问题 Draft + 1 个总索引 Draft，零合并**。本轮没有改动八份冻结目录 CSV；2,588 个模块规则、178 个目标规则是枚举规模，并非已跑相同数量的行为测试。

## 22. 原生触摸取消、关卡菜单翻译与插件测试宿主（第18轮）

| 独立 PR | 此次实际证据与范围 |
|---|---|
| [#212](https://github.com/He-Jiahui/ZirconEngine/pull/212) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/85785b76766ae5ed3b07fe4105366e556715fa1d/docs/plans/unreal-alignment/hidden-fallback-menu.md#L61-L67) | 同一回归 **1过2败 → 3过0败**。已导入的真实 authored HUD 和输入命令的 CPU 对照 3 过、0 败。原版独立 WGPU 首帧进程在 15 分钟上限内没有返回已核验的 framebuffer；这是受限中止，不计作通过或断言失败。没有通过旧按钮/新 HUD 像素前置，不据此认定可见画面。修后只复跑 CPU；没有复用旧 GPU 作为修后结果。 GPU 原执行回执 `81cf649bd0b54a026a8de949d34718cac2af682ac8b6ba26351c09bb456fc229`。 |
| [#213](https://github.com/He-Jiahui/ZirconEngine/pull/213) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/735170f0ae9786eaceaf9b3830edf86fa8f5ecda/docs/plans/unreal-alignment/menu-touch-cancel.md#L102-L108) | 同一回归 **2过2败 → 4过0败**。公开 Runtime 触摸输入：4 过、0 败。实际 Cancelled、没有 Ended；开始与重试槽分别为 none / none，物理对应与真正 Ended、外点和同指重用均已验证。同一测试字节与命令下配对修复。 |
| [#214](https://github.com/He-Jiahui/ZirconEngine/pull/214) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/6d81e87ed8047a53b2133dae41f5c0f046890969/docs/plans/unreal-alignment/paused-transition-event-replay.md#L35-L47) | 四项原生状态机回归已提交并实际编译启动，但全部在插件 SDK 建宿主时退出，0 帧、没有通知行为 RED。对同一可执行文件另跑两个仓库原用例，分别定位 physics.runtime 缺 FoundationModule 与无插件栈 AssetModule 缺 TasksModule；不混作动画失败，未改生产。 |
| [#215](https://github.com/He-Jiahui/ZirconEngine/pull/215) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/07b6ae1711fb8c2e24b774ed1f1a14c86f377bd9/docs/plans/unreal-alignment/editor-file-menu-locale.md#L1-L29) | 在默认组件化关卡编辑器中，命令模型和弹窗叶已消费中文翻译，File 入口本身没有。首个提交仅是六点调查；不把已有字典或者原模板上的 literal 当作实际窗口测试。 |
| [#216](https://github.com/He-Jiahui/ZirconEngine/pull/216) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/6a685dee74b10d8334288a8457235d8d6378998f/docs/plans/unreal-alignment/sdk-default-module-dependencies.md#L1-L31) | 零额外插件的仓库原控制在安装 World 扩展时实报 AssetModule 缺 TasksModule；源码还显示 Scene 要 Time，后者并未作为此次首错误被触发。仅测试 SDK，未误报产品启动。 |
| [#217](https://github.com/He-Jiahui/ZirconEngine/pull/217) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/7d84fb42e1178348cce265d6f26adbdf1c63ae8d/docs/plans/unreal-alignment/sdk-linked-host-dependency.md#L1-L31) | 真实 Foundation 已登记，但 SDK 的 plugin-only 顺序验证看不到它；未改的双插件控制在第一帧前失败。生产的宿主感知组合是另一条图；不能关掉验证或抹掉旧注册 API 的兼容义务。 |

**解释计数。** 新 SDK 两个前置若以同一个动画测试可执行文件作观察，它们是不同的失败位置，不叠加为动画通知的复现次数。源级正常传播与失败控制分列，UE 原测只按各自已核对的主机、文本或插件边界引用，未移植运行。保留 Unity 式 Prefab，不在本轮改动。

累计 **37 个独立问题 Draft + 1 个目录 Draft，零合并**。八份不可变模块/目标/领域/测试定位 CSV 与原来的 Git blobs 完全一致，2,588 个模块规则 + 178 个目标规则仍不是行为通过数。

## 23. 菜单译文、测试宿主与数组层源的缓存身份（第19轮）

| 原 PR | 本轮进展及边界 |
|---|---|
| [#215](https://github.com/He-Jiahui/ZirconEngine/pull/215) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/cc8e45d861618ed5975034f91e5e9189c483db2f/docs/plans/unreal-alignment/editor-file-menu-locale.md#L43-L53) | 真实挂载的工作台实跑：同一可点击 File 标题 File → 文件 → File；同一弹窗叶按语言正常 Open Scene... → 打开场景... → Open Scene...。原版末尾断言失败 → 同一回归修后通过；仅接入一个已有文字键。 |
| [#216](https://github.com/He-Jiahui/ZirconEngine/pull/216) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/733b90396a6fdd9a09efbdcf97adac18f0e6808c/docs/plans/unreal-alignment/sdk-default-module-dependencies.md#L47-L59) | 同一六个 SDK 主用例 2 过 4 败 → 6 过 0 败；同一产物再执行仅选 Foundation 或仅选 Asset 的两项，1 过 1 败 → 2 过 0 败。默认宿主真实推进一个固定步，未选择的根不会被补入。这里只澄清两个互不重叠的运行过滤器，没有改生产或测试。 |
| [#217](https://github.com/He-Jiahui/ZirconEngine/pull/217) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/d2f91601adc4b7854f5a573ef618d7690f8823d8/docs/plans/unreal-alignment/sdk-linked-host-dependency.md#L39-L48) | 同一个 SDK、未改的原四案 3 过 1 败 → 4 过 0 败；另在新入口实测 3 过 0 败。不把未选根登记进去，真正缺失仍拒绝，旧 extra module 及跨宿主排序语义保留。没有在 #217 叠跑 #216 或 #214。 |
| [#219](https://github.com/He-Jiahui/ZirconEngine/pull/219) · [证据](https://github.com/He-Jiahui/ZirconEngine/blob/2f206a7dafba042a40263661ec8fe51080d45d2d/docs/plans/unreal-alignment/texture-array-source-refresh.md#L43-L52) | 已实证普通改色导致同一个父数组动作键对应两个不同内容摘要：旧工程全量重扫从磁盘读到绿/红，独立无制品新工程读到绿/蓝。独立 PNG 自身确实已更新，两工程最终三份作者输入逐一字节相同。尚未改生产。 |

外部平台模块还核对了 UBT External 的 Win64 选库和 ZE 的 dist 命名：Cargo 允许单设库名，但没有查到 ZE 已承诺这一布局；因此未为自定义 DLL 名创建问题 PR。不把本地源码读取或旧工厂已加载数组刷新等价为 UE 全工程冷重开通过。

累计 **38 个独立问题 Draft + 1 个总索引，均未合并**。2,588 个模块规则、178 个目标规则及八份 CSV 是冻结目录，不是行为通过数。[#214](https://github.com/He-Jiahui/ZirconEngine/pull/214) 原动画四例仍要等宿主依赖约束真正在其原输入上解除；独立 SDK 的局部通过数不代替动画帧。

## 24. 纹理源、缓存恢复与编辑器操作所有权（第20轮）

| 独立 PR | 实际范围与结果 |
|---|---|
| [#219](https://github.com/He-Jiahui/ZirconEngine/pull/219) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/482351810f7dd48f627a476c3dc003fb00608359/docs/plans/unreal-alignment/texture-array-source-refresh.md#L69-L86) | 父数组的缓存身份与解码现使用相同源图字节。相同六案 1 过 5 败→6 过；与旧制品相同构建的两案 0 过 2 败→2 过。关项改一图重开从误恢复父数组变为 1 恢复/2 重导；未改仍 3 恢复/0 重导。4 个 Runtime 合同及 4 个既有快照/预算检查通过。插件旧 libtest 被两份未改测试中的 23 个编译诊断挡住，三个额外兼容用例没有执行。 |
| [#221](https://github.com/He-Jiahui/ZirconEngine/pull/221) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/af4503c9aa97abf9780155d5d630555362329692/docs/plans/unreal-alignment/main-close-model-result.md#L41-L55) | 真实 retained libtest 两例：0 过、2 败。未观测到预设的错误行为签名。两例都在切换到新项目的 authoring scene 时遭到同一代际拒绝，尚未发起 OBJ 导入或主窗口关闭；不能把这两次失败当成关闭所有权问题的 RED。 |
| [#222](https://github.com/He-Jiahui/ZirconEngine/pull/222) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/45e26fc73ff112fe321be21c3cfd315cedfda0d0/docs/plans/unreal-alignment/inspector-unedited-enter-redo.md#L39-L53) | 真实 retained libtest 两例：0 过、2 败。未观测到预设的错误行为签名。两例均实际运行，但在绑定启动场景时就遇到代际拒绝；没有聚焦 Details、按 Enter、记录保存点或执行 Redo，不能把本次失败当作精度丢失的 RED。 |
| [#224](https://github.com/He-Jiahui/ZirconEngine/pull/224) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/35db7c3f87741bed4b49570b9c63baecd2b64ce2/docs/plans/unreal-alignment/texture-ktx1-test-context.md#L34-L44) | KTX1 原测试的 2 项编译诊断已消失；同一插件 libtest 从 23 项降为 21 项，剩余均为独立 mipgen 测试的原诊断。仍未生成测试程序、0 行为用例执行；仅修测试，不改生产。 |
| [#225](https://github.com/He-Jiahui/ZirconEngine/pull/225) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/40003b14353185ae7d6d0a4eacdae8ef40167869/docs/plans/unreal-alignment/texture-mip-test-result.md#L34-L44) | mipgen 原测试的 21 项编译诊断已消失；同一插件 libtest 从 23 项降为 2 项，剩余均为独立 KTX1 测试的原诊断。仍未生成测试程序、0 行为用例执行；仅修测试，不改生产。 |

本轮新增项中，主窗口关闭和未编辑数值框是两条 Editor 行为调查；KTX1、mipgen 是同一 Plugins07/G01 验证门下的两处独立测试编译维护，不是另外两项纹理产品错误。纹理修复以恢复计数、生成源回滚、旧工程首次打开和冷工程分别取证。编辑器源码、宿主、桌面 GUI 与 UE 原测分开计算。

累计 **42 个独立问题 Draft + 1 个总索引，零合并**。原 2,588 个模块规则、178 个目标规则与八份目录 CSV 不变；不把同一个回归的重跑或一次目录枚举视作额外问题/全域验收。

## 25. 在真实项目冷启动后验证输入与关闭（第21轮）

| 独立 PR | 原生检查和实际边界 |
|---|---|
| [#168](https://github.com/He-Jiahui/ZirconEngine/pull/168) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/2e8ea0fdd631408a2674343470166fe9efcc29d8/docs/plans/unreal-alignment/preview-captured-pointer-release.md#L63-L71) | 真实 V8 Play 相机已进入右键旋转；同窗 Game 外释放时原会话的 release 计数不增加，回入后 Move 实际送达且仍转动。inside、无主输入和失焦对照先通过。没有把相机观测当作 authored Slider 或 OS 验收。 |
| [#221](https://github.com/He-Jiahui/ZirconEngine/pull/221) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/623216b09fb67d0da0948a983b58196791b7fd9e/docs/plans/unreal-alignment/main-close-model-result.md#L57-L63) | 冷启动的无导入正控制已真正退出并释放项目。导入例已绑定同根 Scene 与 Document，但在首次目录/待处理重载的联合门限前停止，0 次测试 OBJ 提交和 0 次该例关窗；未证实或修复模型结果交接。 |
| [#222](https://github.com/He-Jiahui/ZirconEngine/pull/222) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/8211cbbad4f8570856b4544a05522bdb47863dda/docs/plans/unreal-alignment/inspector-unedited-enter-redo.md#L85-L91) | 同一冻结回归由 3过/1败 变为 4过/0败。空 Enter 和只移动光标均保留原 Scene 的精度、保存点及可重做的 Y；显式重输相同显示值仍形成可撤销事务。另外独立核对设置、文本、搜索、重命名的既有测试，仍以各自实测结果为准。 |
| [#227](https://github.com/He-Jiahui/ZirconEngine/pull/227) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/ae59d575725e1f67b7be90ce0326bad89a886671/docs/plans/unreal-alignment/reopened-project-window-exit.md#L49-L55) | 同一真实主窗 A→Welcome→B→X 已从 1过/1败 转为 2过/0败。保留旧关闭的严格身份校验，A、B 的 guard 均按各自会话释放；不代表一般热切换或进程重启的整体验收。 |

每条 Editor 观察使用自己的真实项目与用户设置。能编译、能打开工程、能将键盘文字放入输入框、能提交场景事务、能在打包程序中重现，是五种不同结果。未执行 UE、GPU 或整套产品验收。

累计 **43 个独立问题及测试维护 Draft + 1 个总索引，零合并**。八份不可变的模块/目标目录仍保持原始版本。

## 26. 跨 Play 输入归属、Editor 会话与导出交接（第22轮）

| 独立 PR | 实测及尚未覆盖的边界 |
|---|---|
| [#168](https://github.com/He-Jiahui/ZirconEngine/pull/168) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/86d72473ba32e3f8f3726915466d8a5d0bf51dd6/docs/plans/unreal-alignment/preview-captured-pointer-release.md#L110-L130) | 修复限同窗真实鼠标右键：原 Play 在视窗外恰好收到一次 release，回入不再空转；A 的旧松键不进 B，已观察到的非 Game Down 不凭空产生 Game 松键。没有先前可证明 Down 的正常窗内松键仍走原路。 |
| [#221](https://github.com/He-Jiahui/ZirconEngine/pull/221) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/e22a1d490f4876a9ce3825f9ede40f783e0670bd/docs/plans/unreal-alignment/main-close-model-result.md#L65-L84) | 新建与真正关项后用新 Core 重开，两次 Runtime 都有同根 Scene；两次 Editor 在入口和超时都是空目录。重开两端均无 pending reload，单是目录缺失已足以挡住导入；依旧没有导入后的关窗证据。 |
| [#222](https://github.com/He-Jiahui/ZirconEngine/pull/222) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/01c6d9cf60a688587498bc5f68087da288bac8ba/docs/plans/unreal-alignment/inspector-unedited-enter-redo.md#L108-L119) | 新增同焦点用例 2/2 通过，真实 Undo 恢复原值、历史干净、仍可重做、磁盘不变。未改的原四例此次 3/4：显式重输、Undo、Redo 已过，末尾 Save 写盘后重导入再次被更新代次拒绝；上次为 4/4，不把两次拼成全绿。 |
| [#227](https://github.com/He-Jiahui/ZirconEngine/pull/227) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/58231036674341f21389381192dc77b6a1b4fbb0/docs/plans/unreal-alignment/reopened-project-window-exit.md#L57-L66) | 新增两例均通过：同一物理工程关闭再开由新操作接管，X 能退出；第二工程真实变脏后，X 显示关闭提示，Cancel 保留原会话、文档、脏状态及磁盘。原 2 个关闭用例也通过。 |
| [#230](https://github.com/He-Jiahui/ZirconEngine/pull/230) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/0671cc1aadd852622d2b3732813f1abd8953bc88/docs/plans/unreal-alignment/same-run-cooked-pack-manifest.md#L64-L84) | 已修正同一次 Cook→Pack 的交接，不改独立 Pack override、Rust writer 或最终发布门。原红测通过；两种清单拼法、两个相同输入入口均实产 3 个资产的同字节包；debug-only 资产未混入。 |

累计 **44 个问题及测试维护 Draft + 1 个总索引，零合并**。历史八份模块目录保持不变。组件测试、实际写包、桌面/硬件和 UE 的原测分开计，不把任何一组结果当成整个引擎验收。

## 27. 暂停序列、滚动范围、单域构建与首次目录比较（第23轮）

| 独立 PR | 实测及尚未覆盖的边界 |
|---|---|
| [#165](https://github.com/He-Jiahui/ZirconEngine/pull/165) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/fc6df989a02e903a3841101c712c8f9e242b4523/docs/plans/unreal-alignment/timeline-paused-binding.md#L138-L164) | 冻结原六例后补了混合绑定回归。共 4 过 / 4 败：混合序列中原轨保留外部编辑，但新建目标仍为 X=0、没有获得 X=2；另一个显式 seek 和真实 Ready 增版的正控通过。两份候选因不能证明资源重新驻留后的归属均未进入生产。 |
| [#202](https://github.com/He-Jiahui/ZirconEngine/pull/202) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/83a85364c7d462fee4e37b80d8e8219d3547f534/docs/plans/unreal-alignment/editor-initial-catalog-visibility.md#L137-L146) | 核实原来真正运行的 Editor 测试 EXE 及其配对 PDB。首次 open / 显式刷新可分别确认 expected=3 / 10，而且两次首次比较都拒绝；比较内部 current 仍不可观测。没有再开工程、附加调试器、关闭 watcher 或把 #204 当作已归因。 |
| [#232](https://github.com/He-Jiahui/ZirconEngine/pull/232) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/3b03a94d63f184930ff6c2c88bcad01ecbd47e9b/docs/plans/unreal-alignment/scrollbox-padding-range.md#L79-L106) | 已修正普通滚动范围：同一组 5 例从 1 过 4 败到 5/5。内容刚好放下时偏移为 0，窗口变大后旧偏移归零；横轴只扣左右内距，嵌套滚轮正确交给外层；零内距行为保持不变。 独立的旧 ReactBits 文档对照前后仍均 0/1，未到三个视口尺寸。 |
| [#233](https://github.com/He-Jiahui/ZirconEngine/pull/233) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/2c04b37fd2e23c08e206e6e0d6fc992b8f2f5ea8/docs/plans/unreal-alignment/native-ui-domain-dependency.md#L55-L68) | 已修正 Windows 的共享 cooked 资产编译依赖：同样的 `ui` 和 `graphics` 单特性、x64 MSVC `--lib` 都从 E0433 变为通过，未启用 native-host。未改锁文件、CI、codec 或虚拟目标配置；同一组 4 个 Build/依赖控制通过。 |

累计 **46 个问题及测试维护 Draft + 1 个总索引，零合并**。历史八份模块目录保持不变。组件测试、实际写包、桌面/硬件和 UE 的原测分开计，不把任何一组结果当成整个引擎验收。

## 28. 旋转扫掠、暂停绑定、响应式控件与 DX12 窗口调整（第24轮）

| 独立 PR | 实测及尚未覆盖的边界 |
|---|---|
| [#165](https://github.com/He-Jiahui/ZirconEngine/pull/165) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/57740aa0160feeddc52e43996436c66eaad927bf/docs/plans/unreal-alignment/timeline-paused-binding.md#L166-L181) | 第九个真实磁盘用例已越过全部前置：同源、同 ID、同字节和修订号的 sequence 释放最后租约后重新驻留，active key 变了。暂停时刻不变，原 A 的外部编辑保留，新 B 没有按 t=0.5 采样；九例 4 过 5 败，未提交生产修复。 |
| [#166](https://github.com/He-Jiahui/ZirconEngine/pull/166) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/6a887ccf6b13f5cd45a346f21a2d004b04080e1f/docs/plans/unreal-alignment/physics-rotated-box-bounds.md#L193-L234) | 已修正公共 CPU 正距离 Box 扫掠，旋转目标和旋转查询现在都在 2.65 命中，三组对应 Chaos 的输入均得到预期距离；20 个同一冻结用例全部通过。没有改变原生 Jolt、静止重叠或普通射线。 |
| [#234](https://github.com/He-Jiahui/ZirconEngine/pull/234) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/e9a92788433102813d3227ed725cac150962f9ca/docs/plans/unreal-alignment/responsive-root-control-lookup.md#L54-L68) | 仅修正原测试的身份查找。未改的同一工作区夹具真正执行 640、900、1280 三个视口及后续样式断言，原测试与独立身份对照 2/2；无生产改动。 |
| [#235](https://github.com/He-Jiahui/ZirconEngine/pull/235) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/d541af1d6738249445e6c0676559a2128b31502d/docs/plans/unreal-alignment/dx12-retired-frame-resize.md#L1-L41) | 正常 Windows DX12 resize：历史实包非零尺寸调整后退出 1，主线仍先 configure 才回收旧引用。历史邻近修复非主线祖先，两次历史 test blob 不同；本轮只有独立调查，没有在这份 Draft 上运行 GPU，未移入私有补丁。 |

累计 **48 个问题及测试维护 Draft + 1 个总索引，零合并**。历史八份模块目录保持不变。公共 CPU 查询、真实资产驻留与桌面/GPU 产品、UE 原测的执行分别计，不把组件结果当成整个引擎验收。

## 29. DX12 窗口对照、暂停序列及脚本激活回调（第25轮）

| 独立 PR | 实测及尚未覆盖的边界 |
|---|---|
| [#149](https://github.com/He-Jiahui/ZirconEngine/pull/149) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/4a4269b938a32eb66953172033a81d16021825a7/docs/plans/unreal-alignment/script-start-projection.md#L125-L149) | 脚本的六个用例实测 6 过、0 败。只改 A 后，B 只执行 onUpdate。真正重装、同行改值再还原及不同 Core 的首帧也分别取得正确的启动次数。 |
| [#165](https://github.com/He-Jiahui/ZirconEngine/pull/165) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/7bd76527b6b808747a7dc863a9172c112bd8f378/docs/plans/unreal-alignment/timeline-paused-binding.md#L212-L226) | 暂停序列现在记录实际加载的资源版本以及各播放器尚未成功写入的轨道。相同的 14 个正式帧场景由 5 过 9 败变为 14 个全过，两项投影缓存库测试也通过。 |
| [#235](https://github.com/He-Jiahui/ZirconEngine/pull/235) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/60505c1da9e8a86d8a3826d46b966d52be4c8534/docs/plans/unreal-alignment/dx12-retired-frame-resize.md#L49-L61) | 真实非零窗口、WGPU 实例和 surface 均已建成；同一 EXE 在 60 秒和 180 秒均未完成兼容适配器选择。未到呈现或 resize，无本机重配 RED；没有提交推测性的引擎修复。 |
| [#236](https://github.com/He-Jiahui/ZirconEngine/pull/236) · [源码和证据](https://github.com/He-Jiahui/ZirconEngine/blob/da0f224b4ed0a9e768014708e6b9ab796ecdb261/docs/plans/unreal-alignment/rhi-profile-admission-order.md#L56-L70) | 提前校验已选适配器与调用方提供的 profile。相同测试由 2 过 1 败变为 3 过 0 败。不匹配时请求调用和首次轮询均为 0，匹配时各 1 次，结果及错误均原样交付。 |

累计 **49 个问题及测试维护 Draft + 1 个总索引，零合并**。八份模块目录沿用已核验版本。

## 30. DX12 窗口、UI 透明度、CPU 粒子与动画库测试（第26轮）

| 模块调查 | 本轮实际结果 |
|---|---|
| [#150](https://github.com/He-Jiahui/ZirconEngine/pull/150) · [源码与验证](https://github.com/He-Jiahui/ZirconEngine/blob/871d26696eadbf4a8704b0478741546dc303a0cf/docs/plans/unreal-alignment/ui-parent-opacity.md#L76) | 同一份六例全部通过。两层 0.5 透明度下，局部透明度 0.8 的叶节点得到 0.2，随后绘制的兄弟节点保留 0.625。单独修改父节点后，工作区发布新的子节点绘制结果。 |
| [#153](https://github.com/He-Jiahui/ZirconEngine/pull/153) · [源码与验证](https://github.com/He-Jiahui/ZirconEngine/blob/bb3b01c0baf536a8e5956338f695fa721e14b66b/docs/plans/unreal-alignment/particle-newborn-age.md#L205) | 同一份六例在 CPU 定时 burst 修复后全部通过。0.375 秒出生的粒子到首帧末移动到 x=1，年龄 0.125 秒，短寿命粒子先进入一次渲染提取，再按时到期。 |
| [#235](https://github.com/He-Jiahui/ZirconEngine/pull/235) · [源码与验证](https://github.com/He-Jiahui/ZirconEngine/blob/38cbb4c860169c2aaf8b7fc41a3702451b63eada/docs/plans/unreal-alignment/dx12-retired-frame-resize.md#L98) | 同一台 RTX 3060 上，正常回收对照和保留旧引用的用例均完成 192×128 到 256×144 的窗口调整，并成功取得、绘制和呈现下一帧。原先失败的用例在重配前仍有 98,304 字节的已完成帧资源。 同一程序的三项既有回归也通过，覆盖窗口收尾与提交队列并发执行、重复票据结算，以及已提交资源的保留。 |
| [#237](https://github.com/He-Jiahui/ZirconEngine/pull/237) · [源码与验证](https://github.com/He-Jiahui/ZirconEngine/blob/eed698b826a8f89d448a156a30b18ee65be5f46e/docs/plans/unreal-alignment/animation-portable-libtests.md#L54) | 关闭动画包默认特性时，原先阻断库测试构建的 18 条诊断已消除。使用仓库现有的 `tests::` 筛选，实际通过 89 项。显式开启 `native-host` 后，注册、物理同步后的求值顺序和分发清单三项原测试全部通过。 |

本次核对 50 个问题及测试维护 Draft 和 1 个总索引。8 份 UE 模块目录继续指向相同的 Git 对象。
