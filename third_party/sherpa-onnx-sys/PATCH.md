<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Local security patch, 2026-10-24

Upstream: sherpa-onnx-sys 1.13.8, Apache-2.0.
Original crate: https://crates.io/api/v1/crates/sherpa-onnx-sys/1.13.8/download
Original source and README are retained without modification. This directory is
excluded from first-party workspace formatting. It retains its own LICENSE.

Only Cargo.toml changes: ZIP build dependency 2 → 9, with deflate-only features.
Addresses RUSTSEC-2026-0367, -0368, -0369, without changing Rust FFI, native DLL
version, model adapter or pinned ABI. ZIP is used for upstream iOS framework
archives; Windows uses tar/bzip2. iOS execution remains unverified.
Lockfile TLS updates fix RUSTSEC-2026-0346 through -0349, -0381 and -0382 in
upstream build-time download tooling. No advisory is ignored.
