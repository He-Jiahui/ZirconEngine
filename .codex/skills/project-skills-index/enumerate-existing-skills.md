# Enumerate Existing Skills

- Use this file when a repo conversation starts and you need quick awareness of local skills.
- Run the shallow tree script first and inspect only top-level entries under each skill folder.
- Read the top-level `SKILL.md` frontmatter for each listed skill and capture `name` plus `description`.
- In repository-local `.codex/skills`, when a directory groups child skills but has no parent `SKILL.md`, record it as missing structure and prefer fixing the index instead of normalizing the bare folder.
- Do not apply that rule to support folders such as `agents/`, `assets/`, `references/`, `scripts/`, or similar non-skill internals.
- Record or refresh the summaries in `catalog-existing-skills/current-project-skills.md`.
- Open deeper files inside another skill only when the current task explicitly needs that skill.
