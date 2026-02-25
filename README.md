# uwu-admin

Web admin dashboard for [Continuwuity](https://continuwuity.org) Matrix homeservers.

Continuwuity only supports admin commands via messages in a special admin room. uwu-admin provides a proper web UI by connecting to the homeserver as a bot account, sending admin commands to the admin room, and displaying the results.

## Table of Contents

- [Architecture](#architecture)
- [Setup](#setup)
- [Session Persistence](#session-persistence)
- [Container Deployment](#container-deployment)
- [Scripts](#scripts)
- [Testing](#testing)

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

## Scripts

Helper scripts live in the `scripts/` directory.

### `update-command-tree.sh`

Regenerates `shared/command-tree.json` from the [uwu-admin fork of continuwuity](https://github.com/PNRxA/continuwuity). Clones the fork into `../continuwuity` if it doesn't already exist, rebases on upstream, and runs `cargo xtask generate-command-tree`. Build prerequisites are the same as for continuwuity itself (Rust, C/C++ compiler, libclang, liburing, make).

```sh
./scripts/update-command-tree.sh
```

### `quadlet-dev.sh`

Development helper for managing the uwu-admin Podman Quadlet. Builds the container image, installs quadlet unit files to `~/.config/containers/systemd/`, and manages the systemd user service.

```sh
./scripts/quadlet-dev.sh <command>
```

| Command | Description |
|---------|-------------|
| `build` | Build the container image |
| `install` | Copy quadlet files and reload systemd |
| `start` | Build image (if needed), install quadlets (if needed), and start the service |
| `stop` | Stop the service |
| `rebuild` | Stop, rebuild image, and restart |
| `restart` | Restart the service without rebuilding |
| `status` | Show service status and recent logs |
| `logs` | Follow the service journal logs |
| `destroy` | Stop service, remove quadlet files, volume, and image |

### Command Tree

The file `shared/command-tree.json` describes every admin command (names, descriptions, argument types) and powers the console's autocomplete. It is auto-generated from the [continuwuity fork](https://github.com/PNRxA/continuwuity) source code via [`update-command-tree.sh`](#update-command-treesh).

## Testing

### API

```sh
cd api
cargo test                    # Unit tests (no server needed)
cargo test -- --skip integration  # Same, explicitly skipping integration tests
cargo test                    # Full suite including integration tests (needs server)
```

Unit tests cover auth, crypto, input validation, command parsing, and response handling.

Integration tests require a running Continuwuity instance. Set the following environment variables (or add them to `api/.env`):

| Variable | Description |
|----------|-------------|
| `TEST_HOMESERVER` | Homeserver URL (e.g. `https://matrix.example.com`) |
| `TEST_USERNAME` | Bot username |
| `TEST_PASSWORD` | Bot password |
| `TEST_ROOM_ID` | Admin room ID or alias |

The integration suite includes an exhaustive command tree test that walks every leaf command in `shared/command-tree.json`, sends it to the server with appropriate test arguments (matching each arg's type — user IDs, room IDs, event IDs, numbers, etc.), and verifies the command parses successfully. This catches any drift between the generated command tree and the actual server command definitions.

### Web

```sh
cd web
npm test              # Run all tests once
npm run test:watch    # Run in watch mode during development
```

Uses [Vitest](https://vitest.dev/) with jsdom. Tests cover:

- **Lib utilities** — response parser (all 4 output branches), HTML sanitization, query key factories, Tailwind class merging
- **Composables** — command autocomplete suggestions, argument hints, input validation
- **API layer** — token management, auth header injection, error handling, 401 refresh flow, request timeouts
- **Pinia stores** — auth (login/register/logout), command execution and history, server connection management
