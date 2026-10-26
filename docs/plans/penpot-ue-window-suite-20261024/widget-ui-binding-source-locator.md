# Widget binding 本地源定位候选 r01

接续 Actor [AB-DEF-01～08](actor-blueprint-shared-contracts.md) 与 Parent [WG-C07](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)。本候选已写入原 checkout 的 Widget 域源码，提供精确 binding 声明范围和受修订保护的本地 Source 导航 API。尚未编译、执行或接 GUI；不代表外部 Definition、Graph、handler 完成或整族验收。增量、输入指纹与检查状态见 [候选回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/binding-source-locator-r01/candidate.json)。

## 原实现与改动

原 [Source outline](../../../zircon_editor/src/ui/asset_editor/source/source_sync.rs) 的 entry_for_node 仍是节点块。本次在该缓存的不可变索引上按需保留 binding 索引，复用 current / last-valid 的现有缓存寿命及共享 Arc；未增加独立 SourceStore。构建节点 outline 的既有路径和计数保持。

[binding_locator](../../../zircon_editor/src/ui/asset_editor/source/binding_locator/mod.rs) 使用 Cargo.lock 当前锁定的 toml 1.1.4+spec-1.1.0 的 DeTable / DeValue / ValueDeserializer。从一次解析的同一语法树取得原 binding 类型和字段 span，保留原 ID、event、component_event、mode、route、action、payload、targets。没有按文本首个同名词定位，也没有声明新产品 schema。缓存中的绑定必须与 session 当前 last-valid v2 绑定逐项相等，源 digest 也须一致。

inline array 的声明范围保留括号。array-of-table 的解析器 span 只含表头，候选会合并该条声明所有后代 key/value span，覆盖嵌套 payload，停止于本条声明的最后语法值；不吞并下一条 binding 或尾部注释。id、event、component_event 各自保留独立值范围，包含其 TOML 字符串引号。所有范围为半开区间。

[ownership](../../../zircon_editor/src/ui/asset_editor/source/binding_locator/ownership.rs) 从真实 root、components.*.root 和 children mount 关系确定声明归属，循环遍历有去重。控件 component 字段单独作为 control_component 返回；它不冒充 owning component。调用本地组件不会将其内部声明归给调用者。重复实例的 invocation ancestry、repeat stable key 和编译代际仍由实际 Runtime/Parent 来源合同提供；本地声明定位不产生这些身份。

## 域内消费接口

[session 适配](../../../zircon_editor/src/ui/asset_editor/session/binding_source_navigation.rs) 提供 crate 内 API，类型从既有 asset_editor 入口重导出。尚无公共 SourceLocator provider 或新的 UI command。

| API / 数据 | 候选行为 |
| --- | --- |
| locate_binding_source(selector, Current) | selector 包含原 node_id、binding_id、event、component_event，可明确指定实际 owner。要求 current 文本已经成为同一 last-valid 文本；不从无效 current 猜 span。预览编译诊断不等同于源解析失败，不应遮住仍合法的本地声明。 |
| locate_binding_source(selector, LastValid) | 显式定位 last-valid 文本，并标记 LastValid。range_source_digest 属于范围所用的旧文本；request_buffer_revision 属于发起查询时的当前编辑缓冲，不能混为一个版本。 |
| UiAssetBindingSourceLocation | 字段私有，返回声明、scope、request_buffer_revision、last_valid_generation、range_source_digest 的只读访问；同时保留真实 buffer Arc allocation 与 route 身份。相同 ID / 同文本关闭重开也不复用旧 ticket。 |
| select_binding_source_location(location) | 先核 buffer allocation/revision、route、last-valid generation，再核当前有效性与原 authoring projection 的完整 binding；成功才改变本节点/binding 选择及精确源 byte cursor。拒绝旧 ticket 和 LastValid，不 clamp 旧偏移到新文本。 |
| UiAssetSourcePosition | byte_offset 是 UTF-8 字节，line 为 1-based，utf16_column 为 0-based。转 LSP 时行须减一。中文、非 BMP 字符及 CRLF 保留语义；UTF-8 中间字节、CRLF 中间位置拒绝。 |

节点无 owner、owner 歧义、owner 不匹配、重复 binding ID（即使 event 不同）、缺 binding、event 变化、源不匹配、无效 current、过期及 authoring selection 不可用分别返回具体错误。多个 owner 只接受调用者已知的实际 owner，不取第一个。legacy layout 本轮返回 UnsupportedSchema。

导航沿原 selection / binding payload reconciliation；源文本、save dirty 和 document undo 不因定位而改变。接口只移动本地 byte cursor，尚未提供源编辑器的整段文本 selection 或逐行行框验收。原 outline 无精确 entry 时的节点块降级仍须由宿主明确标为 node block。没有把本候选的 binding 范围写成外部函数/Graph 定义。

## 已写用例与验证门槛

已写 12 项 Rust 用例，未编译或执行。七项 [session 消费用例](../../../zircon_editor/src/tests/ui/ui_asset_editor/binding_source_locator.rs) 覆盖 inline / array-table、中文引号节点键、CRLF、实际源 cursor 与原绑定、源不变和 undo 不增、无效 current / 显式 last-valid、编辑→undo 后旧 ticket、同 ID 重开、缺值 / event / owner 错误、重复组件调用的声明归属，以及真实预览编译失败时仍定位合法声明。四项 index 私有用例覆盖重复 ID、多个 / 缺失 owner、旧文本 / 改变的绑定投影、嵌套 payload / typed event / targets；另有一项 positions 用例覆盖 UTF-16 / CRLF / 字节边界。

最小后续托管范围为 zircon_editor，default features 关闭，features=[]，lib tests，test profile，jobs=1、test-threads=1；精确过滤器分别为 tests::ui::ui_asset_editor::binding_source_locator::（7）、ui::asset_editor::source::binding_locator::index::tests::（4）、ui::asset_editor::source::binding_locator::positions::tests::（1）。只在原 Jenkins owner 完成 source / 正式准入及 scope 对账后执行，并在执行前声明实际命令。不得将这些过滤器悄悄写入原 layout / Canvas identity。

原 widget-ui-penpot-r01-layout / validate-editor-layout-01 与 Canvas 请求均保留。最近正式只读对账 captureReady=false、serverReceipt=null；历史 QueryOnly 的 authorization pending 不证明服务端已排队等人工批权。本轮不发 source freeze、不做全量 capture、不改 grants / identity，也不新开请求绕过原失败。静态格式/guard/夹具解析不等于 Rust 通过。

Parent 仍持有实际 source_path / URI、target document、请求取消、导航历史和焦点/provider 的宿主合同。当前 route.asset_id 不是文件路径，本候选没有据其猜 URI。Actor/VM 的 GetDefinition Rust 出口、callback→URI/span 桥、Graph 后端、外部 handler 完成/错误/取消继续 open。产品 ZUI、公共 Runtime/interface、动画 schema、主题、catalog、导出与捕获协议未改；本轮无 Penpot、插件、正常 EXE、桌面或 DPI 操作，passed=0 / accepted=0。
