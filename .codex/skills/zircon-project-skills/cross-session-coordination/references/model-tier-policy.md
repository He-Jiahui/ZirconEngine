# Model selection

- Inherit the current task's configured model and reasoning effort by default, including GPT-6 Astra when it is active. No generation-specific allowlist or downgrade is required.
- Honor an explicit user model choice. Set an override only when authorized and supported by the actual tool/runtime.
- Where selection is permitted, choose for the subtask's reasoning, context, latency, and cost needs. Do not infer capability or current pricing from a fixed historical tier table.
- Omit unsupported optional overrides. Use the exact model IDs and reasoning-effort values exposed by the current tool; display labels such as "Light" are not API values.
- If an explicit required model is unavailable, report that limitation and keep independent work moving. Do not silently substitute another model.
- Record a selection reason only when an override or material tradeoff needs to be understood. Avoid duplicating model metadata across prompts, task metadata, and session notes.
- This policy governs model choice when delegation is permitted; it does not authorize spawning agents or user-visible tasks.
