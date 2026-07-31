# Cross-Session Model Tier Policy

Use only these three model tiers for a cross-Session task. Provider-facing IDs
may include a `gpt-` prefix, but the tier name and ordering are fixed.

| Tier | Relative price | Allowed thinking depth | Use for |
|---|---:|---|---|
| `5.6-sol` | highest | High, Extra High | architecture, cross-plan ownership, production incidents, broad reviews, or irreversible decisions |
| `5.6-terra` | middle | Medium, High | multi-file integration, debugging, validation design, ordinary implementation review |
| `5.6-luna` | lowest | Light, Medium | bounded mechanical edits, narrow documentation, deterministic test/format follow-up |

## Dispatch contract

Every cross-Session task must declare all of the following before work starts:

```text
model_tier: 5.6-sol | 5.6-terra | 5.6-luna
thinking_depth: Light | Medium | High | Extra High
selection_reason: <task-specific reason>
primary_session: <the Session to resume after this detour>
```

- Select the least expensive tier that still satisfies the task's required
  reasoning depth and scope. Do not use `5.6-luna` for High/Extra High work or
  `5.6-terra` for Extra High work.
- Use `5.6-sol` for the controller when resolving cross-plan conflicts or
  deciding a production recovery path. Use `5.6-terra` for standard
  implementer/reviewer work. Use `5.6-luna` only for a fully specified,
  isolated task.
- Do not use, suggest, or silently fall back to `gpt-5.5` or any lower model.
  If the requested allowed tier is unavailable, leave the task queued with
  `model_tier_unavailable` or explicitly re-scope it to another allowed tier.
- If the dispatch API cannot set a model, verify the active runtime model is one
  of these three before dispatching. Do not rely on a provider default or an
  undocumented fallback.
- A reviewer may use the same or a higher allowed tier than the implementer;
  it must never use a lower-generation fallback.

## Examples

- A one-file wording update with a precise diff: `5.6-luna` / Light.
- A bounded control-plane + web contract change: `5.6-terra` / High.
- A duplicate rollover or shared-workflow incident: `5.6-sol` / Extra High.
