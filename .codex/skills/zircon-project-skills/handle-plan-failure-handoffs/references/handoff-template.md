# Plan Failure Handoff Template

Use one artifact for the whole lifecycle. Create it in the fixing child directory as `failure-{YYYY-MM-DD}-{summary}.md`; after acceptance, update it and move it to the origin child directory as `fixed-{YYYY-MM-DD}-{summary}.md`.

## Required Frontmatter

```yaml
---
handoff_kind: failure
status: open
created_at: YYYY-MM-DD
summary_slug: short-lowercase-hyphenated-summary
origin_plan: docs/plans/<family>/<nn-origin-plan>.md
fixing_plan: docs/plans/<family>/<nn-fixing-plan>.md
origin_child_dir: docs/plans/<family>/<nn>
fixing_child_dir: docs/plans/<family>/<nn>
related_code:
  - path/to/owned/source.rs
tests:
  - exact reproduction command
---
```

When the fix is accepted, change `handoff_kind` and `status` to `fixed`, add `resolved_at: YYYY-MM-DD`, preserve `created_at` and `summary_slug`, then move and rename the same artifact.

## Required Body

```markdown
# <Owning plan>: <failure summary>

## 来源执行者

- 来源计划：`docs/plans/.../<nn-origin-plan>.md`
- 来源执行切片：<milestone / slice / testing gate>
- 修复责任计划：`docs/plans/.../<nn-fixing-plan>.md`
- 交接原因：<why the lowest shared cause belongs to the fixing plan>

## 失败现象与复现证据

<Observed result, expected result, exact command, counts, logs, or screenshots.>

## 最低共享层根因

<Known root cause, or the narrowest proven boundary when diagnosis is incomplete.>

## 架构修复验收

- <Focused lower-layer behavior that must pass.>
- <Original reproduction that must pass.>
- <Upward plan gate that must be rerun.>

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

Fixed state must replace the open text with:

- 根因：<final root cause>
- 架构修复：<owners and invariants changed>
- 验证：<commands and exact results>
- 回传：<origin plan gate that can resume>
```

## Plan Link Rules

- While open, both the origin and fixing numbered plan files retain a concise `open`/`待修复` summary and a relative link to the `failure-*` artifact.
- After return, both numbered plan files retain a concise `fixed`/`已修复` summary and a relative link to the moved `fixed-*` artifact.
- The fixing plan's link normally crosses plan families. Compute it from the fixing plan file, never write an absolute filesystem path.
