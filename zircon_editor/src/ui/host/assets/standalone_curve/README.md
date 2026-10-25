# 独立 Curve：源、投影和编辑边界

本目录在原 `DataAsset` 上提供严格的 JSON 编解码，以及借用文档值的 `CurveModel` 适配器。JSON 是载体；解析成功只证明该内容符合本域格式。

`StandaloneCurveCodec::decode(asset, expected_kind)` 的 `expected_kind` 必须由原宿主从**同一次真实 captured catalog、AssetTypeDefinition 和 Toolkit 资格**取得。这个参数是调用方的语义断言，codec 不查询或签发资产资格。`ResourceKind::Data`、文件扩展名、JSON shape、页面名和当前焦点都不能产生该断言。没有对应语义 publication 时，原宿主应保留来源并返回明确 unsupported。当前公共接线仍待 Parent 实现。

## 源格式

| 项目 | 精确边界 |
| --- | --- |
| 载体 | `DataAssetFormat::Json` 的原 `text`；存在且非 null 的 `canonical_json` 必须与重新解析值一致。其他格式拒绝 |
| 版本 | `version = 1` 是域编码版本，不能当作资产、catalog、publication 或 document revision |
| kind / channels | Float 必须恰有 Value；Vector 恰有 X/Y/Z；LinearColor 恰有 R/G/B/A。集合精确，源通道顺序和各自键集合保留 |
| 键身份 | 原字符串 ID 非空、无两端空白，且在通道内唯一。不同通道可以复用相同键名；显示名和位置不是键身份 |
| 输入 | `Seconds` 或 `Unitless`，有限 f32 且按数值严格递增。`-0.0` 与 `+0.0` 不能占两个相同输入位置 |
| 值 / 精度 | 声明 F32；保留有限负值、HDR 和符号零。数值 token 还须通过原 codec 的 f32 十进制保真检查；f64 声明、非有限值、不可保真 token 拒绝 |
| 未知数据 | 重复字段、未知字段或枚举值拒绝，不删除字段后继续保存 |

## 当前 source / view 矩阵

| 语义 | codec / 原源 | 原 CurveView 投影 |
| --- | --- | --- |
| Constant / Linear | 每键保存；非 Cubic 键不接受 authored tangent mode 或 slope | 同一通道所有键模式一致时支持 Step / Linear，末键也参与检查 |
| Cubic | 显式 User，两侧 finite slope 都存在且数值相等；只支持 linked、unweighted | 支持 Hermite。两个零 slope 的各自符号位仍保留，不从 slope 推导作者 mode |
| 各通道不同模式 | 支持，例如 X Constant、Y Linear、Z Cubic | 支持逐通道模式，不要求整份资产统一 |
| 单通道混合键模式 | 保留全部原键模式 | `UnsupportedSurface`，不取首键覆盖其余键 |
| default / unset | `None`、`Some(0.0)`、`Some(-0.0)` 明确区分；有限显式值可编解码 | 显式 default 返回 `UnsupportedSurface`；空通道不生成 key 或默认零 |
| pre / post extrapolation | 仅显式 Constant | 共享 view 没有外推参数或求值器；保留源元数据不证明域外求值已实现 |
| Auto / Break / weights | 拒绝未知 mode、weight 或 weight-mode 字段；不自动补值 | 不启用相应作者操作 |
| LinearColor | 保留原 RGBA 通道、负值和 HDR，不夹到 0..1 | 只提供标量通道曲线；gradient 和七项颜色调整未实现 |

合法空通道可以投影为空键列表。当前空 view 的 Linear 绘制选择不写回作者模式。动画 private projection 对独立斜率或 `None` 的支持不会扩大本资源格式的 User-linked 准入范围。

## 命令、撤销和保存

`StandaloneCurveDelta` 只描述**已存在、ID 不变**的键：typed channel + 完整 expected key + 完整 replacement key。重复目标、missing/stale key、ID 改写和最终非法或无法投影的结果全部拒绝。先检查完整 candidate，再一次发布并返回完整 inverse；一次 channel 模式变更必须显式提交全部相关键及所需 mode/slopes，不自动补斜率。

Key、Channel、Source 和 Delta 的作者值比较保留 f32 位身份，因此 sign-only 编辑不算 no-op，旧键只差零符号也会拒绝。该比较不替代原 document incarnation/revision、activation、publication 或 CAS。

**完整源编码与结构命令分开。** `encode_into` 能保存完整、合法的源快照，包括不同的合法键列表；它没有为插入、删除或重命名提供 UI 命令、权限或 history。现有 delta 不能创建或删除键，插入/删除菜单仍未接通。

真正 no-op 保留原 text 和 cache，包括空 cache；有变化的编码会重新生成整份 JSON，不能保证未编辑区的排版字节不变。typed inverse 恢复作者字段，精确文件字节撤销须由原 Document/history 保留真实 before/after `DataAsset`。

原 Document 一次 permit/history/CAS 承接整批 delta；`SaveCtx` 负责 dirty、锁、I/O、失败和 durable cancel。codec 只在完整校验后更新内存中的 `DataAsset`，不执行持久保存。

`StandaloneCurveCodec::prepare_edit(&DataAsset, expected_kind, delta)` 将原读取、借用 adapter 和编码串为一个不可变源入口。它先核完整 source、全部 expected key 和最终可投影 candidate，再返回 `Some(DataAsset)`；失败不返回部分候选，真正 no-op 返回 `None`，原 text/cache/URI/format 全部不改。即使 delta 自称 no-op，stale key 或不能表示的源也会拒绝。有变化的候选保持原 URI/format，经过原 codec 重开验证；它仍会重新生成 JSON。原 Document 须保存真实 before/after、一次提交并重验同一 view/document/source revision，不能把候选或字段相等当成写入许可。这个入口不增加结构命令、持久 store、history 或 clock，尚无实际 per-view caller。

## 独立未完成项

| 项目 | 当前状态与责任 |
| --- | --- |
| 菜单、上下文菜单和真实 gesture / 文本输入 | 未接真实 per-view caller；资源与 Parent 原宿主接续 |
| insert / delete | 本域整批源候选已实现；共享结构 delta 与真实 per-view 命令未接通，不能仅凭候选启用菜单 |
| gradient | 未实现作者语义、求值和输入；资源接续 |
| 七项颜色调整 | Hue、Saturation、Brightness、BrightnessCurve、Vibrance、MinAlpha、MaxAlpha 未实现；相应源字段当前拒绝，资源接续 |
| 语义 subtype / captured registry | Parent 接同代真实资格；没有 publication 时保持 unsupported |
| 真实 Curve Toolkit | Parent 接注册与文档宿主，不给全部 Data 换 Toolkit |
| 真实资产 provider | 尚无该 codec 的真实项目资产、源版本和 per-view 数据回执；Parent 与资源接续，不造 UUID 或 producer |
| history / CAS / 保存、关闭和焦点返回 | 原 Document/SaveCtx/retained 宿主接续，字段级 inverse 不证明这些行为完成 |
| 正式 ZUI 往返与画面 | 原 source closure、产品加载、正常 EXE、三尺寸和实际 144DPI 尚未验收 |

独立 Curve 沿用原共享 CurveModel/CurveView，不增加 Timeline、Transport、FPS、第二 clock、history 或 source store。当前回归定义和静态检查都不能代替 Jenkins Rust、真实资产或产品验收。


## 整批结构候选

`StandaloneCurveCodec::prepare_keys_insert(asset, expected_kind, keys)` 和 `prepare_keys_remove(asset, expected_kind, keys)` 均接收 `&[(StandaloneCurveChannelId, StandaloneCurveKey)]`，返回 `Result<Option<DataAsset>, StandaloneCurveError>`。插入项携带调用方提供的完整新键；删除项携带完整 expected key，沿原位比较拒绝 stale（包括零符号差异）。重复目标、已有新键 ID、缺失 channel/key、非法时间或字段拒绝整个批次。跨 X/Y/Z 或 R/G/B/A 的操作只产生一份完整候选，不按通道写 history。

两入口都先 decode 并校验完整原 surface，再验证完整最终 surface 和编码重开。新键按实际 input 排序，既有稳定 ID 和作者值保留；空 channel 不补键或默认值。仅空批次在全预检后返回 None，原 text/cache/URI/format 不变。有变化会沿原 codec 重新生成 JSON；typed 增删互逆不保证恢复原文件排版，原 Document 必须保留完整 before/after DataAsset。

此入口保留当前表面边界：显式 default 或同通道 mixed 插值，即使空批次也拒绝；Auto/Break/weights/f64/非 Constant 外推继续不支持。不发明 key ID、斜率或采样值，不重算作者 User linked 切线，不增加 Timeline/Transport/FPS/clock。`StandaloneCurveDelta` 与共享 `CurveModel` 完全不变，仍只编辑既有键。新的源候选不等于共享结构命令、实际输入或撤销已接通。

原宿主仍须从同一真实 captured semantic catalog 取得 expected_kind，核同一 project/document/view/source revision、当前 publication 和预算，经一次原 Document/permit/history/CAS 提交完整候选与真实原源 inverse；成功且代次仍当前才更新 typed selection/focus。SaveCtx 的锁、I/O、durable cancel 与源候选分开验收。未注册真实 Curve 类型或缺真实 caller 时保持 unsupported，不按 JSON shape、URI 或当前 focus 启用操作。
