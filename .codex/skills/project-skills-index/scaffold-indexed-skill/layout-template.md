# Indexed Skill Layout Template

```text
skill-name/
  SKILL.md
  discover-current-state.md
  choose-variant.md
  scripts/
  child-topic/
    SKILL.md
    detailed-reference.md
```

## Parent Skill Rules

- Put `name` and `description` in the parent `SKILL.md` frontmatter only.
- Keep the parent body short and navigational.
- Start the parent body with a progressive disclosure index that says which file or child folder to read next for each branch.

## Root Instruction File Rules

- Keep each root file narrow and obvious from its filename.
- Use imperative wording.
- Keep each file short enough that reading it is cheaper than opening a deep subtree.

## Child Skill Rules

- Put deeper guidance in `child-topic/SKILL.md`.
- Start the child `SKILL.md` with another progressive disclosure index when the child has multiple deeper files.
- Stop nesting when the skill can be scanned comfortably from a shallow listing.

## Catalog Rule

- After adding a new skill, update the project-wide catalog with its shallow tree, summary, and layout note.
