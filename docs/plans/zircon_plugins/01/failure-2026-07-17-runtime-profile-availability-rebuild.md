---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: runtime-profile-availability-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/runtime_profile/defaults.rs
  - zircon_runtime/src/plugin/runtime_profile/availability.rs
  - zircon_runtime/src/plugin/package_manifest/builtin_catalog.rs
  - zircon_runtime/src/builtin/runtime_modules/availability.rs
tests:
  - runtime profile availability build-count benchmark
  - 1/100/1000 provider selection scaling test
  - availability report byte-equivalence test
---

# Plugins01：runtime profile availability 重建与二次投影

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP plugin package-manifest/runtime-profile 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：profile、builtin catalog、provider projection 和 bootstrap assembly 必须共享一次冻结结果，不能在单个 helper 内留下第二套缓存 authority。

## 失败现象与复现证据

`runtime_profile_availability` 等 bootstrap assembly helper 每次先调用
`RuntimePluginDescriptor::builtin_catalog()` 重建完整 descriptor catalog。`RuntimeProfileDescriptor::for_id()`
也通过 `builtin_profiles()` 构造全部六个 profile，再只取一个。

registration-report 路径先把 linked/native provider id 分别放入 `Vec<String>`，用线性 `push_provider_id`
去重；随后 availability core 又把两个 Vec clone 成 `HashSet<String>`。project manifest selection 同样通过
`Vec::iter_mut().find` 去重。P 个 provider/selection 时，启动投影包含 O(P²) 查重与至少一轮可避免的 String
复制；该路径当前只在 bootstrap/export assembly 可达，未发现逐帧调用，因此列为启动规模问题而非 frame 热点。

## 最低共享层根因

启动 assembly 没有一次性的 `{profile, descriptor catalog, provider membership}` 冻结 projection。
各 helper 重新生成默认 profile/catalog，再在 Vec 与 HashSet 之间重复投影，导致 build count 和复杂度没有统一预算。

## 架构修复验收

- 单次 bootstrap/export generation 中，builtin profile 与 descriptor catalog 各构建一次并由 consumers 借用。
- linked/native provider membership 从 registration reports 直接建立最终集合，不经过线性去重 Vec 和第二次 String clone。
- manifest selection required 合并使用线性索引，同时保持首次出现顺序和 `required` OR 语义。
- 1/100/1000 provider/selection benchmark 证明 build count 恒定、查重线性；报告分类、顺序、reason 文本逐项等价。
- 产品启动 trace 证明该 projection 不进入 frame/tick 路径；若 editor 需要刷新，按 plugin generation 失效而不是无界全局缓存。

## 禁止临时方案

- 不得分别缓存 `builtin_profiles()` 和每个 availability helper 的结果，形成多个失效源。
- 不得为了 HashSet 查重改变报告首次出现顺序或 required 合并规则。
- 不得把 startup-only 静态形态描述成稳定帧瓶颈；优先级必须由规模 benchmark 与产品 trace决定。

## 修复结果与回传

Open state: `待 Plugins01 建立 bootstrap availability generation projection 与规模化 build-count 回归`。
