# Rich format version identity review

Date: 2026-08-30

Status: `RRT-P1-024_versioned_format_identity_current_source_consumer_closure_static_complete /
RRT-P1-025_authoring_diagnostic_static_complete /
managed_profile_and_product_validation_pending`

## Scope

This record reviews the public rich-format identity before changing parser or cache behavior. It
covers the naming and identity foundation shared by RRT-P1-024 and RRT-P1-025. It does not claim
that Markdown or HTML compatibility is complete, and static diagnostic completion is not milestone
acceptance without managed behavior, performance, and product evidence.

## Current-source findings

- `RichTextFormat::{Markdown, Html}` and `UiRichTextFormat::{Markdown, Html}` advertise complete
  external formats. The Markdown parser actually recognizes only `**strong**`, `*emphasis*`, and
  inline backtick code; it has no block grammar, links, escaping, nesting, or CommonMark contract.
- `html_subset.rs` already describes itself as a deliberately bounded HTML V1 subset. Its tokenizer
  accepts a whitelist and recovers unsupported or malformed input as literal/ignored content. The
  public `Html` name and the style string `html` hide that boundary.
- BBCode is also an engine-owned grammar. Its blocks, tables, inline objects, decorators, and
  resource policy must evolve under an explicit version rather than silently changing `BbCode`.
- The compiled cache correctly separates today's formats, but its key converts the enum to a
  hand-maintained `u8`. The format's grammar version is not a first-class artifact identity, and a
  newly added variant can be forgotten in adjacent identity code.
- `CompiledRichText` retains the format enum, so a truthful, versioned enum can qualify cache,
  artifact equality, UI style, and renderer dirty identity without adding a second format field.

## Unreal and reference-engine boundary

Local Unreal Slate exposes `IRichTextMarkupParser::Process` and
`FDefaultRichTextMarkupParser`; it does not advertise the default angle-bracket grammar as HTML.
The parser produces stripped output plus line/run parse results, and the marshaller/decorators own
later interpretation. Parser injection and explicit owner names keep syntax capability separate from
layout capability.

Godot similarly exposes BBCode-oriented rich text rather than claiming HTML compatibility. These
references support a versioned engine syntax profile, not an unqualified web-format promise.

## Hard-cut decision

The current public/internal variants and serialized style values are replaced without aliases:

| Old identity | New identity | Serialized/style value | Contract |
|---|---|---|---|
| `Plain` | `Plain` | `plain` | no markup grammar |
| `Markdown` | `MarkdownInlineV1` | `markdown_inline_v1` | strong, emphasis, inline code only |
| `BbCode` | `BbCodeV1` | `bbcode_v1` | current Zircon BBCode grammar |
| `Html` | `HtmlSubsetV1` | `html_subset_v1` | current bounded whitelist/recovery grammar |

Legacy `markdown`, `bbcode`, and `html` style strings are rejected rather than silently selecting a
different promise. Migration belongs at authored-document/project migration boundaries, not inside
the runtime parser. Both runtime and interface enums use explicit serde names so acronym casing
cannot alter the wire value.

`RichTextFormat` becomes hashable and the cache key stores it directly. The manual `u8` conversion is
deleted. Adding a future grammar version must therefore update exhaustive parser/UI conversions and
automatically obtains a distinct cache/artifact identity.

## Authoring diagnostic contract

HTML-subset structural recovery now publishes bounded, non-fatal diagnostics with stable code,
source-markup byte range, warning severity, recovery action, and a separate truncation receipt. They
belong to `RichParseResult` and therefore the canonical compiled artifact; layout continues consuming
admitted stripped text. Fatal capacity/security failures remain `RichTextParseError` and are not
conflated with authoring warnings.

The delivered codes cover unsupported/unmatched/implicitly-closed/unclosed tags, unsupported and
malformed attributes, invalid recognized values, unsupported style properties, malformed tags,
unterminated quoted attributes, malformed entities, and unrecognized entities. Diagnostics
retain no dynamic message or tag string; the editor can slice the qualified source artifact by range
and localize through the stable code/message key. The independent request budget defaults to 256
retained diagnostics, rejects allocation before growth, and sets
`authoring_diagnostics_truncated` after the cap. Cache byte accounting includes vector capacity.

Attribute-name issues are accumulated during the existing tokenizer scan; value and style-property
issues are accumulated where the existing projection already parses them. Malformed tag/quote state
is classified by the tokenizer and entity state by the existing decoder. The parser publishes at
most one diagnostic per source segment and code, so no second document or attribute pass was
introduced. Malformed markup/entity source remains visible verbatim; ordinary less-than text is not
treated as markup, and EOF recovery keeps diagnostic publication in source order.

## Validation and performance gates

This identity cut is allocation-neutral: the cache replaces one scalar format field with the enum
itself and adds no parser pass. Static contracts must reject the old variants/strings, require the
versioned variants in parser dispatch/UI conversion, and require the cache key to own
`RichTextFormat` directly. Managed Rust tests remain required before acceptance. Parser timing, RSS,
power, WGPU, and PNG claims are neither needed nor made for this naming/identity-only slice; the
future attribute/value diagnostic materialization path requires its own bounded corpus and profile
before shipping.

The implementation contract now passes in the complete 45/45 Runtime Text static suite. Exact old
variant and authored-value scans return zero matches; focused Rustfmt and scoped diff-check pass.
Wire round-trip and legacy-rejection Rust tests are present but remain unrun under the managed Cargo
blocker, so this is not an accepted milestone.

A later current-source closure audit found that the public enum hard cut had not reached every Rust
consumer. Seven internal/UI/renderer/transport source files still held 67 references to the deleted
`RichTextFormat` or `UiRichTextFormat` `Html`, `Markdown`, or `BbCode` variants. The admission fixture
also constructed the deleted `LinkRef::href` field. Those consumers now use `HtmlSubsetV1`,
`MarkdownInlineV1`, `BbCodeV1`, and `UiRichLinkTarget`; the original 9-byte decorator and 12-byte
retained-link budget expectations remain unchanged. Word-boundary contracts cover both enum owners,
and the complete Runtime/RuntimeInterface source scan returns zero deleted variants. The scoped
Runtime Text infrastructure suite passes 40/40, but managed Rust compilation remains required before
acceptance.

The same closure audit found 11 artifact-identity fixtures using the non-existent `[link=...]` tag
although BBCode V1 registers `[url=...]`. They were comparing unknown-markup/source identity instead
of a compiled link run. Both fixture owners now use `[url=...]`, and the composite artifact test
asserts that the compiled target is the canonical `res://docs` target before registration. One
inline-widget product fixture also still authored `rich_text_format = "bbcode"`; it now uses
`bbcode_v1`, and the Runtime authored-value scan returns zero legacy values. The static contract
rejects both regressions; the final complete rerun passes 47/47 in 1.744 s, including the adjacent
inline-widget typed-slot and measure-owner structure guards.

The subsequent structural diagnostic contract passes in the complete 46/46 static suite. Focused
Rust behavior tests cover deterministic code/range order, truncation, EOF recovery, and stable code/
catalog-key uniqueness but remain unrun. No Cargo, WGPU, PNG, timing, RSS, or power claim is made.

The attribute/style follow-up keeps the same 46/46 suite green and adds focused Rust behavior for
unsupported/malformed attributes, invalid values, unsupported style properties, and source-range
qualification. The final static follow-up adds focused behavior for literal malformed markup,
unterminated quotes with and without a close delimiter, malformed/unrecognized entities, ordinary
less-than text, and EOF source ordering. Diagnostics helpers and the active-tag stack moved into
`parser/html_diagnostics.rs` and `parser/active_tags.rs`; the parser root/children are 723/108/123
lines, restoring the 800-line source gate. These tests also remain unrun under the managed Cargo
blocker; no timing, RSS, power, WGPU, PNG, or product claim is made.
