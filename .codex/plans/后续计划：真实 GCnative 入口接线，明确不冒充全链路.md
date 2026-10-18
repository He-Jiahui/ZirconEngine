# 后续计划：真实 GC/native 入口接线，明确不冒充全链路

## Summary

现有 function/prototype 元数据足够支撑一个真实但有限的下一步：把 inline frame layout 接入 GC 扫描/重写/释放路径，并把 native inline span API 接到真实 native dispatch context。

但它还不够声称完成全链路：`typeLayoutId` 目前主要是 prototype 索引，不是完整的可序列化 `SZrTypeLayout` 表；VM call/return/tail-call 的 by-value struct payload 迁移、完整 native ABI marshaling 仍要单独推进。

## Key Changes

- 增加 runtime type-layout resolver/cache：
  - 从 function frame slot 的 `typeLayoutId` 找到对应 prototype/member metadata。
  - 能安全证明 POD 的 struct 用 `POD` copy/drop。
  - 含 object/string/function/class/ref、ownership、managed lifecycle、close/destructor 的字段走 field copy/drop/gc visitor。
  - 递归或未知布局无法证明安全时返回失败，保留 boxed/旧路径，不伪装成功。

- 接入 GC 真实路径：
  - 在 VM frame 扫描时，frame layout 存在则用 `ZrCore_Function_VisitInlineFrameGcValues` 扫描 inline struct 内嵌 `SZrTypeValue`。
  - minor GC rewrite 使用同一 layout visitor 改写 forward 后的 embedded value。
  - frame teardown 和 tail-call frame reuse 前调用 inline drop helper，释放 owned embedded values，避免把 raw struct bytes 当普通 slot 扫描。

- 接入 native 真实路径：
  - native dispatch 初始化 `ZrLibCallContext` 时填入当前 VM frame 的 function/base/argument start。
  - `ZrLib_CallContext_InlineArgumentSpan` 在真实 native callback 内可取得 inline struct span。
  - v1 不实现完整 `%extern struct` 平台 ABI 传参；native/FFI 层只能显式读取 inline span，缺元数据或未 inline 时回退旧 boxed 路径。

- 明确边界：
  - 本阶段不改完整 interpreter call/return/tail payload movement。
  - 本阶段不宣称 source-level struct by-value 到 native ABI 已完成。
  - 若 resolver 发现 prototype 元数据不足，先升级为显式 module/function type-layout table，再继续扩大接线范围。

## Test Plan

- layout resolver：
  - POD struct 成功解析为 inline POD。
  - 含 managed/object/owned 字段解析为 field-copy/drop/gc layout。
  - 未知 nested struct、坏 `typeLayoutId`、旧 `.zro` 无 frame layout 时安全失败并 fallback。

- GC：
  - inline struct span 内嵌对象/string，在 `stackTop` 低于 frame storage top 时仍被 mark。
  - raw padding bytes 不被当 `SZrTypeValue` 扫描。
  - minor GC 能 rewrite inline embedded value。
  - frame close/tail reuse 释放 owned embedded value 一次且仅一次。

- native：
  - 真实 known-native callback 内调用 `ZrLib_CallContext_InlineArgumentSpan` 能拿到 expected span/size/type id。
  - 缺 frame layout 或 resolver 失败时 API 返回 unavailable，旧 native call 行为不变。

- 回归：
  - 保持现有 core layout、precall、tail reuse、closure precall、compiler integration、module system、CLI hello_world 测试通过。
  - 更新 acceptance 文档：记录 GC/native 入口已真实接线，同时保留 VM payload/native ABI 全链路未完成边界。

## Assumptions

- `typeLayoutId` 暂按 prototype index 使用，但所有访问必须校验 module/prototype 边界。
- 不硬编码类型名分支；布局判断来自 prototype/member metadata 和 resolver。
- GC 必须按 layout 精确访问 embedded value，不能扫描 inline struct raw bytes。
- native inline span 是真实 dispatch 能力，但在 VM by-value payload 迁移前，它只覆盖已经存在 inline frame payload 的场景。
