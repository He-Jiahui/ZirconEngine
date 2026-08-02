---
title: WOS166 Ravenpost send transport and escrow audit
status: implemented_static_dynamic_validation_deferred
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS166 Ravenpost Send Transport

## Source contract

Source command id 128 is the fire-and-forget client send
`mail_send { to, subject, body, copper, items }`. `OnlineGame.mailSend` sends the
three JavaScript caller strings, its finite number and an array projection of
each `{ itemId, count }` attachment. The server accepts at most three attachments
and finite numbers, floors attachment counts while preserving copper, applies a
64-code-unit subject and 600-code-unit body slice plus a 32-code-unit trimmed
recipient, and performs chat mute/rate-limit/moderation before resolving the
recipient. The source transport hard-caps the complete serialized WebSocket
frame at 16 KiB; it has no separate player-mail field limits beyond that frame
boundary. The UI has stricter 32/64/600 form controls, but those are not transport
validation and must not reject a direct `IWorldMail` call.

## Typed ingress scope

Schema 55 adds `mail_send` using
`u32_le_utf8_to + u32_le_utf8_subject + u32_le_utf8_body + f64_le_copper +
u8_attachment_count + repeated(u32_le_utf8_item_id + f64_le_count)`. Its minimum
is 21 bytes, its aggregate payload maximum is a 16 KiB binary safety bound
derived from the source WebSocket cap, and its collection maximum is the source
`MAIL_MAX_ATTACHMENTS = 3`. This nested binary cap is not byte-equivalent to the
source serialized-frame limit.

`MailSendCommandPayload` and `MailSendAttachment` retain field order, duplicate
item ids, whitespace, valid UTF-8 text and fractional or negative finite values.
They reject only malformed lengths/UTF-8, non-finite numeric values, an attachment
count above three, aggregate payload overflow and trailing bytes. The transport
does not trim or slice text, floor a value, fold duplicate attachments, look up
an item, reject a soulbound/quest item, select a recipient, consume postage, or
mutate inventory. Those are authoritative Ravenpost reducer rules.

## Deferred authority and projection

The source reducer is a durable cross-player transaction: it requires an alive
sender near a mailbox, rejects a full recipient box before escrow, deducts copper
plus 30 postage, removes only fungible copies, books delivery after the fixed
raven delay, updates an unread index and emits `mailResult` events. Online sending
also resolves offline names and block lists through the realm character database.

Current WOC contains mailbox placement and the three post-receipt command
payloads only; it lacks the durable mail book, recipient directory/block lookup,
mailbox snapshot window, `mailUnread` state and `mailResult` projection. WOS166
advances typed ingress only. It must not claim mail gameplay parity or add a
native escrow fallback. The existing Plugins 08 generic structured
snapshot/presentation-output handoff is the execution dependency for eventual
authoritative mail-state projection. Its WOS166 raw JavaScript text and frame
ingress addendum is additionally required for lone-surrogate text and exact
serialized-frame acceptance parity; Rust/Zr UTF-8 strings cannot model those
source inputs locally.

## Verification

- Static source contract validates the client shape, server intake and 16 KiB
  source frame cap.
- Native encode/decode tests preserve raw finite values, duplicate attachments
  and field order; reject non-finite values, malformed/trailing data, four
  attachments and aggregate overflow.
- Client-intent tests map exactly command id 128 without applying mail policy.
- Regenerate and check payload schema/coverage. Do not run Cargo or ZrVM while
  the existing Plugins 08 dynamic runtime handoff remains open.
