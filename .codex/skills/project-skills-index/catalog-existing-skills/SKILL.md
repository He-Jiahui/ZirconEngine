# Catalog Existing Skills

## Progressive Disclosure Index

- If you only need the currently recorded skill summaries, read `current-project-skills.md`.
- If you need to refresh the catalog, run the shallow tree script from the parent skill and inspect only each top-level `SKILL.md` frontmatter first.

## Workflow

- Start from `.codex/skills`.
- Record the shallow tree before reading any deep content.
- Extract `name` and `description` from each top-level `SKILL.md`.
- In repository-local `.codex/skills`, treat a top-level or category directory without `SKILL.md` as a missing parent index unless it is clearly a support folder such as `agents/`, `assets/`, `references/`, or `scripts/`.
- Record whether a skill tree is flat, parent-indexed, or currently missing a parent category file.
- Add a short layout note when a skill is flat, indexed, or otherwise unusual.
- Leave nested folders unread until a task requires that skill's deeper instructions.
- Update `current-project-skills.md` after any local skill add, rename, split, or major rewrite.
