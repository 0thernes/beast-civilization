<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Operations, Backups, Archival & Offsite Mirroring (`A35`)

`beast-civilization` is a local bare-metal research engine and CLI (no persistent hosted database or production SaaS deployment). Its operational integrity centers on deterministic builds, cryptographic checksums, independent archival mirrors, and RFC 3161 trusted timestamps.

## 1. Local & Offsite Repository Mirroring (`git bundle` + Software Heritage + Master Archive)
- **Canonical Remote:** `https://github.com/0thernes/beast-civilization`
- **Master Research Archive Pin (`0TH-BUILD-017`):** `https://github.com/0thernes/0thernes-research-archive` pins the exact release commit SHA, frozen PDF, arXiv tarball, verification ledgers, and SHA-256 digests.
- **Permanent Offsite Mirror (Software Heritage):** Every release tag is archived via Software Heritage Save Code Now (`https://archive.softwareheritage.org`) with permanent `swh:1:snp:*` SWHID recorded in `VERIFICATION.md` and `docs/run-log.md`.
- **Offline Single-File Backup (`git bundle`):** Create a verifiable offline mirror snapshot at any time:
  ```powershell
  git bundle create ..\beast-civilization-v1.0.0.bundle --all
  git bundle verify ..\beast-civilization-v1.0.0.bundle
  ```

## 2. Cryptographic Checksums, RFC 3161 TSA & SLSA v4 Provenance
- **Root Checksums:** [`CHECKSUMS.sha256`](../CHECKSUMS.sha256) covers the frozen preprint PDF, clean-room arXiv tarball, Lean audit receipts, citation audit, paired statistical audit, and key manifests.
- **Data Checksums:** [`data/MANIFEST.sha256`](../data/MANIFEST.sha256) and [`results/MANIFEST.sha256`](../results/MANIFEST.sha256) pin every archived measurement log.
- **RFC 3161 DigiCert Timestamp:** [`data/timestamps/CHECKSUMS.sha256.tsr`](../data/timestamps/CHECKSUMS.sha256.tsr) provides independent third-party cryptographic proof-of-existence over `CHECKSUMS.sha256` verifiable via:
  ```bash
  openssl ts -reply -in data/timestamps/CHECKSUMS.sha256.tsr -text
  ```
- **SLSA v4 Attestations:** `.github/workflows/ci.yml` and `.github/workflows/release.yml` emit GitHub `actions/attest-build-provenance@v4` attestations over frozen release artifacts.

## 3. Local HTTP Server Observability (`beast serve`)
- Health/readiness endpoints: `GET /health` and `GET /healthz` return HTTP `200 OK` JSON containing `status`, `version`, `model`, `architecture`, `gguf_bytes`, `gguf_tensors`, and `c2c_layers`.
- Strict guardrails: 5-second read/write timeouts, 256 KiB request body / `Content-Length` cap (`413 Payload Too Large`), `405 Method Not Allowed`, `404 Not Found`, `400 Bad Request`, loopback binding (`127.0.0.1`) by default.
