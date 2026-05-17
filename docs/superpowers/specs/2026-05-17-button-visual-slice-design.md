# Button Visual Slice Design

## Goal

Make `Button` the first polished retained UI component slice: its authored Material metadata, native painter pixels, GPU/softbuffer command stream, and pointer hit region must agree for the common editor states.

## Scope

This slice is limited to `zircon_editor` retained-host Button rendering and validation. It does not change `zircon_hub`, Slint, public `ChromeCommandStream` command formats, or the runtime UI component schema.

## Component Contract

`Button` keeps the existing compact editor sizing contract: 32px control height for showcase/editor controls, 10px default radius, and pill radius only when the asset explicitly requests it. The visible states are default, hover, pressed, focus-visible, disabled, primary/contained, secondary/outlined, text, and danger/error.

State priority remains deterministic: disabled wins over all interactive states, then validation danger/error, then pressed, then focused/selected, then hover, then variant defaults. Pressed must be visibly stronger than hover. Focus must use a clearly sampled ring/border. Disabled must mute background and foreground and must not dispatch a click.

## Rendering And Input

The retained native painter owns Button pixels through `host_contract/painter/template_nodes.rs`; `.zui` assets only author stable metadata and representative examples. The rendered rectangle and hit-test frame both come from the projected layout frame, so a click inside a visible Button activates the same control and a click outside does not.

## Verification

The focused gate is editor-only:

- native painter tests sample primary, outlined/text, hover, pressed, focus, disabled, and danger pixels.
- projection tests prove `ButtonDemo` keeps variant/state/padding/input metadata.
- native pointer tests prove the projected showcase Button routes clicks and disabled Button nodes do not activate.
- profile/screenshot validation can be run after the code gate to confirm no software fallback and to record batching metrics.

