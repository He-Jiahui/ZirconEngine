# Unreal ↔ ZirconEngine 全模块对照：目录、独立 Draft 与测试推进总览

## 1. 本轮阶段与边界

首轮完成用户要求的“先搜索全部关联模块、再逐个提出有据问题 Draft”阶段：**21 个独立问题 Draft、30 个原列领域 + 10 个补充领域映射**。后续新增独立 Notify #171、Text #174、旧恢复归属 #175、Capture #178 和经典纹理缓存 #180，当前共 **26 个问题 Draft**；第五轮最新范围见第10节，第7/8/9节保留历史快照。下文首轮表格/CSV保留发布当时快照，不是实时测试通过表。不是引擎整体对齐完成，也不是数千模块逐项行为/产品审计通过。

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
