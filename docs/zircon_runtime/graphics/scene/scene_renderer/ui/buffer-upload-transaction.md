# Screen-Space UI Buffer Upload Transaction

Screen-space UI dynamic buffers participate in the scene renderer's single frame upload
transaction. Generic geometry, image geometry, bitmap-atlas instances, bitmap-atlas viewport
parameters, SDF vertices, and SDF materials prepare immutable `WgpuBufferUpload` payloads instead
of writing through `wgpu::Queue`.

## Ownership

`ScreenSpaceUiRenderer` owns one `ScreenSpaceUiBufferUploadTransactionState`. `record` reserves one
generation, prepares all six buffer domains, records draw commands against the prospective physical
buffers, and returns a move-only `ScreenSpaceUiPreparedBufferUpload`.

The direct scene path appends that preparation to `frame_buffer_uploads` after UI recording and
before the frame upload ticket is accepted. The compiled path appends it to the graph pass-local
batch, moves the commit token through `RecordedGraphPass` and `RenderGraphStageExecution`, and then
merges the graph batch into the same outer frame ticket. Both paths commit the UI generation only
after backend acceptance and submission-ledger registration.

## Failure And Retry

Only one preparation may be outstanding per renderer. Physical buffer growth may happen during
preparation, and child owners may store prospective payload hashes or retained-plan identities, but
those identities are reusable only while the renderer's prepared and committed generations match.
Dropping a preparation leaves the generations different. The next preparation consequently forces
a full upload across every child buffer owner, including buffers whose payload hashes otherwise
match. Successful commit advances the committed generation and releases the reservation.

Attachment validates an unforgeable renderer owner identity before moving batch ownership. Commit
repeats the owner and generation checks. This prevents a preparation from another renderer, or an
older preparation invalidated by a later physical replacement, from publishing reuse state.

## Complexity

Preparation remains `O(payload bytes + invalidated segments)`. Retained segment identity avoids
hashing stable generic/image geometry; invalidated buffers hash or pack one contiguous POD payload.
Batch append moves upload vectors without cloning payloads or native buffer handles. No range sort,
per-vertex registration, or byte-diff scan is part of this path.

## Boundary

This contract covers buffer uploads only. Bitmap and SDF atlas texture uploads still use their
typed texture-upload owners and remain part of the later PFO-4d2 Queue convergence work. Runtime
WGPU, PNG, RenderDoc, profile, VRAM, and power evidence is tracked under
`docs/tests/runtime/render`; source checks are not a substitute for those artifacts.
