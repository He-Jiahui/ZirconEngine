---
title: WOS171 Challenge response transport
status: implemented_static_dynamic_validation_deferred
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS171 Challenge Response Transport

## Source contract

Source command id 36 is the client send `challengeResponse { n, r, sig }`.
When a server challenge arrives, the source client uses the literal response
`"42"` and computes `signChallenge(nonce, response, clientSeed)`. The server
only calls `verifyChallenge` if all three fields are strings; it verifies against
the authenticated session client seed and otherwise drops the response. The
shared signer is deterministic JavaScript `cyrb53`, using UTF-16 `charCodeAt`,
32-bit `Math.imul` arithmetic and base-36 serialization.

## Typed ingress scope

Schema 60 will represent the valid UTF-8 typed domain as
`u32_le_utf8_nonce + u32_le_utf8_response + u32_le_utf8_signature`, bounded to
256 bytes per field and 12..780 bytes total. Native
`ChallengeResponseCommandPayload` and `ClientGameplayIntent::SendChallengeResponse`
preserve the three strings exactly within that bounded valid-UTF-8 domain. They
do not choose a response, derive a session seed, sign, verify, authenticate or
alter session state.

The WOS166 raw-text/frame ingress handoff still owns exact JavaScript UTF-16 and
outer-frame behavior. The WOS171 Plugins08 addendum additionally owns source-
equivalent `cyrb53` arithmetic and base-36 semantics in the real ZrVM plugin;
WOS171 must not create a native signer or WOC-specific VM callback.

## Deferred authority and projection

The WOC project lacks an accepted authenticated session capability and a real
ZrVM-compatible challenge signer/verifier. This transport slice does not claim a
complete anti-bot challenge loop.

## Verification

- Static anchors cover the incoming source challenge, `signChallenge` call and
  server string gate/`verifyChallenge` call.
- Native codec tests round-trip each field and reject malformed/oversized input.
- Client intent tests map exactly command id 36 without signature policy.
- Regenerate/check payload schema and coverage. Dynamic signing/verification is
  deferred to Plugins08.
