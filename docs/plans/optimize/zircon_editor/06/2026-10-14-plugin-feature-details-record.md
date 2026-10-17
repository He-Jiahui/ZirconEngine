# Editor06：当前插件全部 feature 详情、限定操作与稳定焦点

记录时间：2026-10-15T06:55:36.289247+00:00。核心实现：runtime_admission_core / gpt-6.1-sol max；独立复核：batch12_evidence / gpt-6-luna max。

原列表只暴露一个候选 feature 操作，无法查看当前插件的全部功能、分页和逐项诊断。现在同一 pane 可以打开当前 owner 的全部 feature 页面、选择一项并通过原有 qualified action 路由操作，再返回原插件行。

RetainedEditorHost 保存 project/plugin/feature 的稳定身份、页面、语义焦点、意图和详情 UI generation。关闭详情时不生成 feature 页面，原 Arc 摘要与 single-pass selector 保留；打开时只构造当前最多 8 行及选中详情。8 行是页面策略，不是已测性能结果。

desired 来自 resolution.enabled；required、blocked 来自已有权威字段；effective 明示 Unknown。点击或键盘操作重新核对当前 project/owner/feature/selection/intent；删除或切换对象会撤销旧意图。解析时持有的 registry/document 锁在实际修改前释放，不根据过期 UI 重新选另一个 feature。

焦点键盘保持 popup/Game/global 原优先级，详情支持 Tab/ShiftTab、方向键、Home/End、PageUp/PageDown、Enter/Space、Escape/Back；返回恢复合法 owner 焦点。实际 pane 的 builtin_template_runtime 同步 host model 与 UiSurface；绑定前退役旧 pane 的 collection state，避免相同 bare feature ID 跨 owner/project 继承选择。

结构化 collection row 先绘制 label，使用非空 typed ID 和已有 hit geometry；虚拟化使用原 extent，其余使用 frame.height/row_count，不引入 header 偏移。详情 generation 与 locale 参与 cache；共享 tokens、单一 body scroll、长 ID 换行，保留窄宽显示。

25 个源文件（23 个原文件、2 个新增 leaf）已冻结。6 个受影响 leaf 的 ordinary 从 9 增到 20；另外 4 个只读相邻 leaf 有 11 个 ordinary 和 1 个 ignored。合计 required31 = preserved20 + new11。原 B16 Editor ordinary 调用及 ordinal15 均未选中这 31 项，应在同一 Editor invocation 追加完整实际 FQN，ordinary 最低数由 136 增到 167；required names 由 130 增到 161，ignored 不计 ordinary。

补丁采用真实 canonical Git HEAD diff 加 2 个新增文件；before raw 取实际 pre-edit admission。混合换行不能由 HEAD 的 LF/全 CRLF 两候选重造，不宣称 raw before 字节重建或 raw reverse 一致。早期 pipe EOF、metadata wiring 假设和 EOL 重建失败保留在证据；最终源码未因这些 metadata 失败回写。

root scoped change guard 实际 outer0、child0、0 errors/0 warnings；独立 source+packet peer 已完成。V1 peer 发现的 Back 后列表选择失效已在 V2 修复；原 V1 packets、阻断结论和实际失败收据完整保留，V2 累积 patch 仍使用原 23 个 tracked before 和 2 个原先 absent 新文件。Rust scoped rustfmt、Git 子进程及 metadata 失败/纠正收据按实际记录区分。

UI 参考采用批准的 ReactBits App UI List/Support Data Table 结构语义；只读取公开文档壳，preview iframe/DOM/CSS 不可用，未复制 Pro 代码，也未声称已验证参考视觉复刻。

原 Editor248 ignored producer 仍是 31 paired samples、2048 features、每 sample1000 selector calls、optimized P95 <= legacy70%；它测摘要 selector，不能替代本次详情页面 P95。1/100/1000 插件稳定读取的 build/clone 计数也不是详情时延或 RSS。

原生 ordinary、原 Release/ignored producers、真实 selected-plugin en/zh × wide/normal/compact 的截图、输入、布局、scroll、focus、stale/manifest 更新和详情性能仍待同源 Jenkins 回执。现有 generic ModuleDetails drawer 截图与 synthetic ReviewCase 不覆盖这组功能，不扩 tooling 以伪造验收；完整 P1-11 保持 OPEN。

[原计划](../06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md)；[补丁](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor06-plugin-feature-details-v2.patch)；[源码证据与完整 FQN](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor06-plugin-feature-details-v2-evidence.json)；[静态检查](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor06-plugin-feature-details-v2-static.json)；[Astra 完成列表](../../../astra/features/editor/2026-10-14-editor06-plugin-feature-details-completion-list.md)；[冻结 Runtime03 绝对 P95 标准](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json)。

产物：仅预登记补丁、元数据、静态报告和计划记录；未创建编译/cache/临时 helper/native fixture/源码副本。实际释放0B，必要验收和失败证据保留。历史 UNKNOWN 进程/回执不因本记录而关闭。
