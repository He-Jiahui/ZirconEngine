---
title: Managed Cargo direct network validation input
category: validation
date: 2026-09-27
status: pending_validation
---

# Managed Cargo network input

The Batch J pack-bin validation ticket `84ad88bc2af943d7a4f1a617fc2e3c5b`
failed in coordinator materialization before compiling tests. Its pinned Cargo
metadata step could not fetch the `wasip3` index entry because global Git
`http.proxy` points to an unavailable `127.0.0.1:7897` service. A separate
direct HTTPS HEAD request for the same index entry returned 200 on this host.

`cargo-direct-network.toml` is an explicit, sealed Cargo `--config` input for
the next grouped managed validation wave. Its empty `http.proxy` setting
disables proxy use for that command while leaving global Git settings and
coordinator tooling untouched. It is included in the source manifest and
passed by repository-relative path. Command normalization, source capture,
metadata planning and the actual Cargo run must still succeed before this
workaround is considered validated. It does not change product source or
claim any test or performance result.

The [Cargo configuration reference](https://doc.rust-lang.org/cargo/reference/config.html#httpproxy)
describes Cargo's proxy precedence over the global Git value. The
[libcurl proxy contract](https://curl.se/libcurl/c/CURLOPT_PROXY.html) specifies
that an explicit empty proxy string disables proxy use, including inherited
environment proxies. The coordinator rejects inline configuration because
it is not sealed; the repository config file passed command normalization.
