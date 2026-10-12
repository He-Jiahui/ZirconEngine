# Product Receipt Interoperability Fixture

The fixed public test key uses Ed25519 seed byte 07 repeated 32 times. It is never a product trust root.

Node 22.13.1 / OpenSSL 3.0.15 generated and independently verified this fixture using the existing cargo-zircon ProductReceipt v1 canonical field order. Runtime tests consume the static bytes and verify with ring 0.17.14. No runtime signing/canonical helper generated the stored signature.

- Receipt identity: 20C28D5AF8C448C86F30095D7A1BF0DA8BCC15EBA139FD729EF9EE47F54C0DCB
- Toolchain identity: 1A15D846E8810546D756F418881A546812109B1163E60949D615A335727226BA
- Public key: EA4A6C63E29C520ABEF5507B132EC5F9954776AEBEBE7B92421EEA691446D22C

The executable and native DLL artifact payloads are literal UTF-8 `exe` and `dll`; these test bytes are authenticated metadata fixtures and are never loaded as code.

Producer compatibility evidence is emitted by the isolated `evidence/verify-producer-interop.ps1` against an existing cargo-zircon executable: it issues the same draft, compares the deterministic receipt identity and signature, then invokes the existing verifier against the independent fixture and materialized bytes. Until that command runs successfully, producer interoperability remains pending.
