# RustDesk Server

RustDesk Server is a minimal self-hosted RustDesk container image. It bundles
the `hbbs` and `hbbr` binaries in a `scratch`-based runtime without
s6-overlay, leaving service supervision to your own platform or orchestrator.

- **Source repository:** <https://github.com/HanGYvv/rustdesk-server>
- **Docker Hub:** `hangyvv/rustdesk-server`
- **GHCR:** `ghcr.io/hangyvv/rustdesk-server`

> Looking for a single-container image with built-in supervision and health
> checks? Use the s6-overlay variant: `hangyvv/rustdesk-server-s6`.

## Highlights

- Self-hosted RustDesk server (`hbbs` + `hbbr`)
- Minimal `scratch`-based image containing only the server binaries
- No bundled supervisor: run `hbbs` and `hbbr` as you prefer

## Exposed ports

| Port        | Service | Purpose                       |
| ----------- | ------- | ----------------------------- |
| `21115/tcp` | hbbs    | NAT type test                 |
| `21116/tcp` | hbbs    | TCP hole punching / heartbeat |
| `21116/udp` | hbbs    | ID registration / heartbeat   |
| `21117/tcp` | hbbr    | Relay service                 |
| `21118/tcp` | hbbs    | Web client support            |
| `21119/tcp` | hbbr    | Web client support            |

## Quick start

This image has no entrypoint, so you provide the command to run. `hbbs` and
`hbbr` are separate processes and are typically run as two containers:

```bash
# Relay server
docker run -d --name hbbr \
  -p 21117:21117 -p 21119:21119 \
  -v rustdesk-data:/root \
  hangyvv/rustdesk-server:latest hbbr

# ID / rendezvous server
docker run -d --name hbbs \
  -p 21115:21115 -p 21116:21116 -p 21116:21116/udp -p 21118:21118 \
  -v rustdesk-data:/root \
  hangyvv/rustdesk-server:latest hbbs -r <relay-host>:21117
```

### docker-compose

```yaml
services:
  hbbs:
    image: hangyvv/rustdesk-server:latest
    container_name: hbbs
    command: hbbs -r <relay-host>:21117
    ports:
      - "21115:21115"
      - "21116:21116"
      - "21116:21116/udp"
      - "21118:21118"
    volumes:
      - rustdesk-data:/root
    depends_on:
      - hbbr
    restart: unless-stopped

  hbbr:
    image: hangyvv/rustdesk-server:latest
    container_name: hbbr
    command: hbbr
    ports:
      - "21117:21117"
      - "21119:21119"
    volumes:
      - rustdesk-data:/root
    restart: unless-stopped

volumes:
  rustdesk-data:
```

## Data & keys

Server state, including the generated `id_ed25519` / `id_ed25519.pub` key
pair, is written to the working directory (`/root`). Mount a shared volume
there for both services and back it up to preserve your server identity.
