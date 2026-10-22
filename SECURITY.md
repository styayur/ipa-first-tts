<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Security reports

Send vulnerability reports privately to [yurstya@gmail.com](mailto:yurstya@gmail.com), the maintainer's published contact. GitHub private vulnerability reporting is not currently enabled for this repository. Do not publish exploit details, personal paths, secrets, voice recordings or unpublished content in a public issue.

Include the affected version/commit, platform, backend, minimal reproduction and expected impact. Share only the data needed to reproduce the issue. Response is best effort; no response-time guarantee or mature security-support period is promised for this MVP.

The HTTP UI is a loopback-only local service with Host/Origin checks, not a public multi-user service. Native model/runtime files are trusted inputs and can crash inference when damaged or mismatched. Use matching published assets, verify their checksums and keep native DLLs together. Stop halts playback, not an in-flight native inference. See the [current limitations](README.md#known-limits-and-next-work).
