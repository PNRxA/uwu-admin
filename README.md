# uwu-admin

Web admin dashboard for [Continuwuity](https://continuwuity.org) Matrix homeservers.

Continuwuity only supports admin commands via messages in a special admin room. uwu-admin provides a proper web UI by connecting to the homeserver as a bot account, sending admin commands to the admin room, and displaying the results.

## Architecture

```
Browser (Vue)  →  uwu-admin-api (Rust/axum :3001)  →  Matrix Homeserver
                                                         ↕
                                                    Admin Room
                                                   (bot sends !admin commands,
                                                    reads server responses)
```

## Setup

### Prerequisites

- Rust (2024 edition)
- Node.js 22+
- A Continuwuity homeserver with an admin bot account and admin room

### Configuration

Copy the example environment file and generate secrets:

```sh
cp .env.example .env
```

Generate values for `JWT_SECRET` and `ENCRYPTION_KEY`:

```sh
openssl rand -hex 32
```

Paste a unique value into each field in `.env`. If left unset the API will generate random secrets on startup, but sessions and encrypted tokens won't survive restarts.

| Variable | Description | Default |
|----------|-------------|---------|
| `JWT_SECRET` | 32-byte hex key for signing auth tokens | random (ephemeral) |
| `ENCRYPTION_KEY` | 32-byte hex key for encrypting access tokens at rest | random (ephemeral) |
| `DATABASE_URL` | SQLite connection string | `sqlite:uwu-admin.db?mode=rwc` |
| `API_LISTEN` | API bind address | `127.0.0.1:3001` |
| `CORS_ORIGIN` | Allowed CORS origin | none |

### API

```sh
cd api
cargo build --release
cargo run --release  # Starts on :3001
```

### Web

```sh
cd web
npm install
npm run dev          # Dev server with hot reload (proxies /api to :3001)
npm run build        # Production build (output in dist/)
```

### Usage

1. Start the API server (`cargo run` in `api/`)
2. Start the web dev server (`npm run dev` in `web/`) or serve the built `web/dist/` directory
3. Open the dashboard and create an admin account on first launch
4. Add a homeserver by entering its URL, bot credentials, and admin room ID or alias
5. Manage your servers through the dashboard

Room fields accept either a room ID (`!abc:example.com`) or a room alias (`#admins:example.com`) — aliases are resolved automatically on connect.

## Session Persistence

The API stores server connections in a local SQLite database (`uwu-admin.db` by default). Access tokens are encrypted at rest using ChaCha20-Poly1305.

On startup the API restores saved connections, validates each token against its homeserver, and removes any stale sessions automatically.

## Container Deployment

See [containers/](containers/) for Docker and Podman Quadlet deployment options.

Both require `JWT_SECRET` and `ENCRYPTION_KEY` to be set as environment variables — see the example compose file and quadlet config.

## Pages

- **Overview** - Connection info, server uptime, and stats
- **Users** - List users, create new users
- **Rooms** - List rooms, view room details
- **Federation** - Federation status
- **Server** - Uptime and statistics
- **Media** - Media management
- **Console** - Raw admin command interface
