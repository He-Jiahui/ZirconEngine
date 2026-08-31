---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: template-manifest-loader-owner-budget
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/pipeline_report_platform_bundle_template_manifest_schema.py
  - tools/zircon_export/pipeline_report_platform_bundle_template_manifest_loader.py
tests:
  - tools/tests/test_zircon_export_platform_bundle_template_manifest_files_owner_boundaries.py
resolved_at: 2026-08-31
---

# Plugins13 template manifest loader owner budget

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：PlatformBundle template manifest report validation
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns the export template report schema and input admission.

## 失败现象与复现证据

The template-manifest schema owner reached 361 lines while its focused-owner
guard requires fewer than 360. It still resolved filesystem paths, admitted the
manifest file, decoded TOML, and checked the format version before routing
schema validation.

## 最低共享层根因

Manifest input loading had no named leaf owner. The schema orchestrator
therefore combined filesystem and serialization admission with field, path,
bundle, files, and identity schema routing.

## 架构修复验收

- Move path resolution, file admission, TOML decoding, and format-version
  admission into one named loader leaf.
- Keep schema diagnostic ordering and identity routing in the schema owner.
- Preserve all existing admission diagnostics exactly.
- Keep the schema owner below 360 lines and the loader below 100 lines.
- Pass the focused owner boundary and template report behavior suites.

## 禁止临时方案

- Do not raise or remove the schema-owner line budget.
- Do not suppress validation branches or diagnostics to reduce line count.
- Do not introduce a reverse import from the loader into schema orchestration.

## 修复结果与回传

- 根因：Template manifest filesystem and TOML admission had no named leaf owner, so schema orchestration exceeded its owner budget.
- 架构修复：Moved path resolution, file admission, TOML decoding, and format-version admission into the one-way template manifest loader leaf while retaining schema ordering and identity routing.
- 验证：Managed source-sealed ticket 9f945595a0484f4584da4c3c36b40873 passed 33/33 with source manifest 80f0adbc9777824cb6ae53cfb077226447917db9ae4ac65b2e8ef23304d7450c; local py_compile and scoped diff checks passed.
- 回传：Template manifest admission now has a bounded loader owner; the exact behavior and owner-budget suites pass without reverse imports or raised thresholds.
