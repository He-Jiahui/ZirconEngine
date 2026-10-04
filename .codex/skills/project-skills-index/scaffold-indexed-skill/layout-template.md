# Layout template

```text
skill-name/
  SKILL.md                 # Independent trigger and concise workflow
  agents/openai.yaml       # Optional display and discovery metadata
  references/topic.md     # Optional supporting guidance
  scripts/tool.py         # Optional reusable implementation
category/
  index.md                 # Navigation, not an invocable skill
  another-skill/
    SKILL.md
```

Every actual `SKILL.md` needs `name` and `description` frontmatter. Category `index.md` and supporting `guide.md` files do not. Keep permission and verification rules in their canonical owners and link them instead of copying them. Add executable scripts only for useful repeatable operations, with checks appropriate to their risk.
