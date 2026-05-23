# Zircon Neko Pet Design

## Purpose

Create a Codex-compatible custom pet named `Zircon Neko` based on the user's updated direction: a Japanese anime chibi blonde catgirl with a focused engine-architect personality. The pet should feel cute, polished, and work-oriented, while staying readable inside the Codex pet atlas.

## Context

This supersedes the earlier `Zircon Coresmith` automaton concept for the active pet run. The workspace context is still ZirconEngine: Rust engine/editor/hub development, Slint Hub UI work, runtime boundaries, validation, and architecture-heavy implementation. The new pet should carry that "focused builder" identity through styling and behavior rather than through a mechanical body.

## Approved Direction

Name: Zircon Neko

Concept: A non-sexual chibi anime blonde catgirl architect mascot with cat ears, a fluffy tail, blue-purple eyes, a sailor-uniform-led white/blue/gold outfit, and a small zircon-blue engineer accessory.

Style: Japanese anime chibi, thick clean outline, rounded proportions, bright eyes, soft blush, polished sticker-like rendering, 2.5-head body ratio, high readability at pet size.

Reference handling: The user-provided image is a style reference for chibi proportions, bright anime rendering, white/blue/gold palette, large eyes, cute expression, and decorative polish. Do not copy the reference character's exact hat, staff, outfit layout, mascot companion, pose, or ornamental shapes.

## Visual Identity

Zircon Neko has light blonde hair, visible cat ears, a fluffy blonde tail, blue-purple eyes, and a cheerful but focused expression. Her outfit is primarily a sailor uniform: a compact white sailor blouse, blue sailor collar, blue neck ribbon or tie, pleated blue-and-white skirt, gold trim, and small dark-gray work accents so she still reads as an engine-building companion.

Identity locks:

- light blonde hair, cat ears, and fluffy tail
- blue-purple eyes with a calm focused expression
- white, blue, and gold sailor-uniform palette with small dark-gray engineering accents
- small blue-cyan zircon crystal brooch or badge
- sailor collar, neck ribbon/tie, and pleated skirt as the main outfit read
- compact chibi full-body silhouette, safe inside 192x208 cells
- no readable text, logos, UI screenshots, code snippets, copied reference props, or copied reference companion creature

The character should be cute and expressive, but not sexualized. Avoid cleavage, lingerie styling, suggestive pose language, or adult pinup framing. Keep the mascot suitable for a desktop coding assistant pet.

## Animation Contract

The final pet must use the current Codex pet atlas contract: nine rows, 192x208 cells, final atlas 1536x1872, and package files `pet.json` plus `spritesheet.webp`.

Rows:

- `idle`: gentle blink, tiny body bob, and tail sway.
- `running-right`: light rightward chibi steps with tail balancing the motion.
- `running-left`: mirror `running-right` only if the asymmetric accessory and hair still read correctly; otherwise generate separately.
- `waving`: small friendly hand wave, no wave marks.
- `jumping`: light catlike hop with ears lifting; no floor, dust, shadow, or impact marks.
- `failed`: ears droop, tail lowers, expression becomes disappointed or teary-eyed without floating symbols.
- `waiting`: expectant confirmation pose, holding or hugging a small zircon-blue pointer or badge if already present in the base identity.
- `running`: focused task-processing pose, like concentrating on an invisible build/check, not literal sprinting.
- `review`: leaning in with attentive eyes, as if inspecting output; no papers, magnifiers, UI panels, punctuation, or new props.

## Generation Approach

Use the hatch-pet workflow with the image generation skill as the only visual generation layer. Prepare a fresh run from text, using the approved concept as `pet-notes`, style preset `sticker`, and style notes that specify chibi anime rendering. Do not reuse the earlier `Zircon Coresmith` automaton base or row outputs. Do not reuse the first `Zircon Neko` engineer-coat base if it does not read primarily as a sailor uniform.

Data flow:

1. Prepare the new `zircon-neko` pet run manifest and prompts.
2. Generate a new anime catgirl base image as the canonical identity reference.
3. Generate row strips grounded by the canonical base and layout guide for each state.
4. Copy each selected generated output into the run's decoded paths and mark jobs complete.
5. Derive `running-left` only if `running-right` is visually safe to mirror.
6. Extract frames, inspect components, compose the atlas, validate it, render contact sheet and previews.
7. Package the final files under the local Codex pets directory.

## Error Handling And QA

Reject and repair any row with identity drift, missing frames, copied guide marks, shadows, glows, detached effects, cropped body parts, visible chroma backgrounds, wrong facing direction, stagnant animation, copied reference-specific elements, or row semantics that do not match the state.

For this pet, also reject rows where:

- cat ears, tail, hair color, eye color, outfit palette, or zircon brooch disappear
- the character becomes too tall, too detailed, or unreadable at 192x208
- the outfit becomes sexualized, stops reading primarily as a sailor uniform, or no longer reads as a coding/engine companion
- the model copies the provided reference image's exact hat, staff, companion creature, or costume silhouette

If deterministic extraction produces size popping while the source strip is stable, rerun extraction with the hatch-pet stable-slot method before regenerating art.

Final acceptance requires:

- atlas validation passes
- frame inspection has no errors
- contact sheet is visually consistent across all nine rows
- preview GIFs show state-matching motion without unintended scale jumps
- `pet.json` and `spritesheet.webp` are packaged together

## Out Of Scope

Do not integrate this pet into ZirconEngine source code. Do not modify Hub, runtime, editor, or plugin code. Do not create a logo, copy the reference character, or produce a generic anime girl without catgirl and focused-architect identity.
