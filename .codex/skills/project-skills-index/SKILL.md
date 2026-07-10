---
name: project-skills-index
description: Catalog and scaffold repository-local Codex skills with progressive disclosure. Use at the start of work in this repo when Codex needs a shallow inventory of `.codex/skills`, needs to capture or refresh each skill's top-level summary, or needs to create/update a local skill using a parent-skill plus child-skill chunked layout.
---

# Project Skills Index

## Start Here

- Enumerate project skills before reading any deep skill content.
- Use a shallow directory listing only. Do not recursively read the full tree by default.
- On PowerShell, run `.\.codex\skills\project-skills-index\scripts\list-skill-tree.ps1`.
- On WSL/Linux, run `bash ./.codex/skills/project-skills-index/scripts/list-skill-tree.sh`.
- Read only each top-level skill's `SKILL.md` frontmatter unless the current task requires deeper detail.
- Refresh the recorded summaries whenever a local skill is added, renamed, split, or materially repurposed.

## Progressive Disclosure Index

- If you need to enumerate current skills and refresh the recorded summaries, read `enumerate-existing-skills.md`, then `catalog-existing-skills/SKILL.md`.
- If you need to create or refactor a skill into the indexed chunked layout, read `generate-indexed-skill-layout.md`, then `scaffold-indexed-skill/SKILL.md`.

## Rules

- Keep the parent `SKILL.md` short and navigational.
- In repository-local `.codex/skills`, any folder whose primary job is grouping child skills must own a short parent `SKILL.md` that explains the category and indexes the child skills.
- Keep root-level instruction files short enough that their filenames advertise the next branch to read.
- Put deeper, topic-specific guidance in child folders with their own `SKILL.md`.
- Prefer one extra layer of child folders. Add deeper nesting only when the skill would otherwise become hard to scan from a shallow listing.
- Resource/support folders such as `agents/`, `assets/`, `references/`, `scripts/`, or similar non-skill internals are exempt; they are not category skills and do not need parent `SKILL.md`.
- Treat a repository-local skill-group folder without `SKILL.md` as structure debt to fix, not as the preferred end state.
- Treat the project skill catalog as a maintained index, not as the source of truth. Rebuild it from the filesystem when in doubt.
