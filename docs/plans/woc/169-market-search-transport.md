---
title: WOS169 Market search transport
status: implemented_static_dynamic_validation_deferred
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS169 Market Search Transport

## Source contract

Source command id 101 is the fire-and-forget client send `market_search` with
`q`, `itemType`, `subtype`, `rarity` and `page`. The client forwards the current
query fields. The server constructs an untrusted raw query, defaulting a
non-string search to an empty string and a non-number page to zero, then calls
`sanitizeMarketQuery` before `sim.marketSearch`. Sanitization owns the 40-code-
unit search cap, filter-enum fallback and finite, non-negative, floored page;
the market subsystem stores the resulting per-player browse state.

## Typed ingress scope

Schema 58 will represent `market_search` as four length-prefixed UTF-8 strings
and one little-endian finite f64 page:
`q + item_type + subtype + rarity + page`. Each typed string uses the established
256-byte transport bound, producing 24..1048 bytes. The native
`MarketSearchCommandPayload` and `ClientGameplayIntent::SearchMarket` preserve
search/filter text and any finite page value, including unknown filters,
whitespace, fractional and negative pages. They do not truncate text, select
known enums, floor a page, build listings, set client state or resync a view.

The source's JSON WebSocket limit and JavaScript UTF-16 domain remain broader
than this typed transport. The Plugins08 WOS166 raw-text/frame-ingress handoff
continues to own exact outer-frame and lone-surrogate parity; this slice makes no
claim beyond valid UTF-8 bounded ingress.

## Deferred authority and projection

WOC still lacks an accepted fixed-tick authoritative market command reducer and
the corresponding snapshot projection. WOS169 advances only typed client ingress
and must not introduce a native market-query fallback.

## Verification

- Static anchors cover the client send, server raw-query construction and
  `sanitizeMarketQuery` ownership.
- Native codec tests round-trip raw strings and finite pages, reject malformed or
  non-finite inputs, and retain descriptor bounds.
- Client intent tests map exactly command id 101 without local normalization.
- Regenerate/check payload schema and coverage. Cargo and ZrVM dynamic execution
  remain deferred under Plugins08.
