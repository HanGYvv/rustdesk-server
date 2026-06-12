# RustDesk Server Program

[![version](https://img.shields.io/github/v/tag/HanGYvv/rustdesk-server?label=version)](https://github.com/HanGYvv/rustdesk-server/releases)
[![license](https://img.shields.io/github/license/HanGYvv/rustdesk-server)](LICENSE)
[![ci](https://github.com/HanGYvv/rustdesk-server/actions/workflows/ci.yaml/badge.svg)](https://github.com/HanGYvv/rustdesk-server/actions/workflows/ci.yaml)
[![docker](https://github.com/HanGYvv/rustdesk-server/actions/workflows/docker.yaml/badge.svg)](https://github.com/HanGYvv/rustdesk-server/actions/workflows/docker.yaml)
[![release](https://github.com/HanGYvv/rustdesk-server/actions/workflows/release.yaml/badge.svg)](https://github.com/HanGYvv/rustdesk-server/actions/workflows/release.yaml)
[![debian](https://github.com/HanGYvv/rustdesk-server/actions/workflows/debian.yaml/badge.svg)](https://github.com/HanGYvv/rustdesk-server/actions/workflows/debian.yaml)

[**Download**](https://github.com/HanGYvv/rustdesk-server/releases)

[**Manual**](https://rustdesk.com/docs/en/self-host/)

[**FAQ**](https://github.com/rustdesk/rustdesk/wiki/FAQ)

[**How to migrate OSS to Pro**](https://rustdesk.com/docs/en/self-host/rustdesk-server-pro/installscript/#convert-from-open-source)

Self-host your own RustDesk server, it is free and open source.

## How to build manually

```bash
cargo build --release
```

Three executables will be generated in target/release.

- hbbs - RustDesk ID/Rendezvous server
- hbbr - RustDesk relay server
- rustdesk-utils - RustDesk CLI utilities

You can find updated binaries on the [Releases](https://github.com/HanGYvv/rustdesk-server/releases) page.

If you want extra features, [RustDesk Server Pro](https://rustdesk.com/pricing.html) might suit you better.

If you want to develop your own server, [rustdesk-server-demo](https://github.com/rustdesk/rustdesk-server-demo) might be a better and simpler start for you than this repo.

## Installation

Please follow this [doc](https://rustdesk.com/docs/en/self-host/rustdesk-server-oss/)
