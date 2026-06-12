# RustDesk Server S6

RustDesk Server S6 is a self-hosted RustDesk container image built on
[s6-overlay](https://github.com/just-containers/s6-overlay). It bundles
`hbbs` and `hbbr` into a single compact runtime with proper service
supervision.

- **Source repository:** <https://github.com/HanGYvv/rustdesk-server>
- **Docker Hub:** `hangyvv/rustdesk-server-s6`
- **GHCR:** `ghcr.io/hangyvv/rustdesk-server-s6`

## Highlights

- Self-hosted RustDesk server (`hbbs` + `hbbr`) in one image
- Built with s6-overlay for reliable service supervision and health checks
- Automatic key-pair handling via Docker secrets or environment variables
- Persistent data through a single `/data` volume

## Exposed ports

| Port        | Service | Purpose                          |
| ----------- | ------- | -------------------------------- |
| `21115/tcp` | hbbs    | NAT type test                    |
| `21116/tcp` | hbbs    | TCP hole punching / heartbeat    |
| `21116/udp` | hbbs    | ID registration / heartbeat      |
| `21117/tcp` | hbbr    | Relay service                    |
| `21118/tcp` | hbbs    | Web client support               |
| `21119/tcp` | hbbr    | Web client support               |

## Environment variables

| Variable         | Default               | Description                                              |
| ---------------- | --------------------- | -------------------------------------------------------- |
| `RELAY`          | `relay.example.com`   | Public address advertised for the relay (`hbbr`)         |
| `ENCRYPTED_ONLY` | `0`                   | Set to `1` to force encrypted connections (`-k _`)       |
| `KEY_PUB`        | _(unset)_             | Public key content, written to `/data/id_ed25519.pub`    |
| `KEY_PRIV`       | _(unset)_             | Private key content, written to `/data/id_ed25519`       |

Keys can also be supplied as Docker secrets named `key_pub` and `key_priv`.
If no key pair is provided, `hbbs` generates one on first start.

## Quick start

```bash
docker run -d \
  --name rustdesk-server \
  -e RELAY=relay.example.com \
  -p 21115-21119:21115-21119 \
  -p 21116:21116/udp \
  -v rustdesk-data:/data \
  hangyvv/rustdesk-server-s6:latest
```

### docker-compose

```yaml
services:
  rustdesk-server:
    image: hangyvv/rustdesk-server-s6:latest
    container_name: rustdesk-server
    environment:
      - RELAY=relay.example.com
      - ENCRYPTED_ONLY=0
    ports:
      - "21115-21119:21115-21119"
      - "21116:21116/udp"
    volumes:
      - rustdesk-data:/data
    restart: unless-stopped

volumes:
  rustdesk-data:
```

## Data & keys

All persistent state, including the generated `id_ed25519` /
`id_ed25519.pub` key pair, lives under the `/data` volume. Back this volume
up to preserve your server identity across container recreation.
