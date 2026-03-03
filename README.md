> [!NOTE]
> This project is developed with the assistance of AI code generation tools. AI-generated code is reviewed and tested before being merged, but if you encounter any issues, please feel free to open an issue or submit a pull request.

# uwu-admin

https://github.com/user-attachments/assets/36a10541-87f8-4490-a093-9c431be29436

Web admin dashboard for [Continuwuity](https://continuwuity.org) Matrix homeservers.

[GitHub](https://github.com/PNRxA/uwu-admin) · [GitHub Container Registry](https://ghcr.io/PNRxA/uwu-admin) · [Docker Hub](https://hub.docker.com/r/pnrxa/uwu-admin)

Continuwuity only supports admin commands via messages in a special admin room. uwu-admin provides a proper web UI by connecting to the homeserver as a bot account, sending admin commands to the admin room, and displaying the results.

> [!CAUTION]
> uwu-admin has full admin control over your homeserver. It is intended for private or internal use and should not be exposed to the public internet. If you do need remote access, place it behind a TLS-terminating reverse proxy with additional access controls (e.g. VPN, IP allowlist, HTTP basic auth). See [Production Deployment](#production-deployment) for more details.

## Table of Contents

- [Quick Start (Docker)](#quick-start-docker)
- [Architecture](#architecture)
- [Setup](#setup)
- [Development](#development)
- [Production Deployment](#production-deployment)
- [Container Deployment](#container-deployment)
- [Session Persistence](#session-persistence)
- [Shared](#shared)
- [Scripts](#scripts)
- [Testing](#testing)
- [CI](#ci)
- [Releasing](#releasing)

## Architecture

```
Browser (Vue)  →  uwu-admin-api (Rust/axum :3001)  →  Matrix Homeserver
                                                         ↕
                                                    Admin Room
                                                   (bot sends !admin commands,
                                                    reads server responses)
```

## Quick Start (Docker)

uwu-admin versions follow Continuwuity versions to indicate compatibility. Set the tag to your Continuwuity server version, or `latest` if you're feeling brave.

```sh
docker run -d -p 8080:8080 \
  -e JWT_SECRET=$(openssl rand -hex 32) \
  -e ENCRYPTION_KEY=$(openssl rand -hex 32) \
  -v uwu-data:/data \
  pnrxa/uwu-admin:v0.5.5-1
```

Open `http://localhost:8080`, create an admin account, and add your homeserver.

## Setup

### Prerequisites

- Rust (2024 edition)
- Node.js 22+
- A Continuwuity homeserver with an admin bot account and admin room

### Configuration

Copy the example environment file and generate secrets:

```sh
cp api/.env.example api/.env
```

Generate values for `JWT_SECRET` and `ENCRYPTION_KEY`:

```sh
openssl rand -hex 32
```

Paste a unique value into each field in `api/.env`. Both `JWT_SECRET` and `ENCRYPTION_KEY` are **required** — the server will refuse to start without them.

| Variable | Description | Default |
|----------|-------------|---------|
| `JWT_SECRET` | 32-byte hex key for signing auth tokens | **required** |
| `ENCRYPTION_KEY` | 32-byte hex key for encrypting access tokens at rest | **required** |
| `ADMIN_USERNAME` | Seed an admin account on first start (skips setup screen) | none |
| `ADMIN_PASSWORD` | Password for the seeded admin account | none |
| `DATABASE_URL` | SQLite connection string | `sqlite:uwu-admin.db?mode=rwc` |
| `API_LISTEN` | API bind address | `127.0.0.1:3001` |
| `CORS_ORIGIN` | Allowed CORS origin (enables CSRF protection and credentialed cookies) | none |
| `COOKIE_SECURE` | Set the `Secure` flag on refresh token cookies | `true` |
| `ALLOW_PRIVATE_HOMESERVERS` | Allow adding homeservers that resolve to private/loopback IPs (needed when the homeserver is on the same host or network) | `false` |

## Development

Start the API and web frontend in two terminals:

```sh
# Terminal 1 — API
cd api
cargo run              # Starts on :3001
```

```sh
# Terminal 2 — Web
cd web
npm install
npm run dev            # Vite dev server on :5173, proxies /api → :3001
```

Open `http://localhost:5173`, create an admin account on first launch, then add a homeserver by entering its URL, bot credentials, and admin room ID or alias. Room fields accept either a room ID (`!abc:example.com`) or a room alias (`#admins:example.com`) — aliases are resolved automatically on connect.

Alternatively, use the [`quadlet-dev.sh`](#quadlet-devsh) script to run everything in a Podman container.

### Production Build

```sh
# API
cd api
cargo build --release

# Web
cd web
npm run build          # Output in dist/
```

### Production Deployment

#### Internal without TLS

If uwu-admin is only accessible on a trusted local network (e.g. LAN, tailnet,
Docker bridge), the defaults work as-is. Recommended settings:

| Variable | Recommendation |
|----------|----------------|
| `CORS_ORIGIN` | Can be left unset. All access is same-origin and the network is trusted. |
| `COOKIE_SECURE` | Set to `false` — you are likely serving over plain HTTP. |
| `ALLOW_PRIVATE_HOMESERVERS` | Set to `true` if the homeserver is on the same host or network. |

#### Internal with TLS

If you're serving uwu-admin over HTTPS on a local network (e.g. using a self-signed certificate or a private CA), keep `COOKIE_SECURE` at the default (`true`) since cookies will be sent over TLS.

| Variable | Recommendation |
|----------|----------------|
| `CORS_ORIGIN` | Set to your internal URL (e.g. `https://admin.local:8080`) if accessing from a different origin. Can be left unset for same-origin access. |
| `COOKIE_SECURE` | Leave at the default (`true`). |
| `ALLOW_PRIVATE_HOMESERVERS` | Set to `true` if the homeserver is on the same host or network. |

#### Public (behind access controls)

If you need remote access, place uwu-admin behind a **TLS-terminating reverse
proxy** (e.g. Caddy, nginx, Traefik) with additional access controls such as a
VPN, IP allowlist, or HTTP basic auth. The application serves plain HTTP and
should not be exposed directly to the internet — without TLS, credentials,
tokens, and cookies are sent in plaintext.

| Variable | Recommendation |
|----------|----------------|
| `CORS_ORIGIN` | Set to your external URL (e.g. `https://admin.example.com`). Enables server-side CSRF protection as a defense-in-depth layer on top of `SameSite=Strict` cookies. |
| `COOKIE_SECURE` | Leave at the default (`true`) when behind TLS. |

## Container Deployment

Container images are available on [GitHub Container Registry](https://ghcr.io/PNRxA/uwu-admin) and [Docker Hub](https://hub.docker.com/r/pnrxa/uwu-admin).

See [containers/](containers/) for Docker and Podman Quadlet deployment options.

Both require `JWT_SECRET` and `ENCRYPTION_KEY` to be set as environment variables — see the example compose file and quadlet config.

## Session Persistence

The API stores server connections in a local SQLite database (`uwu-admin.db` by default). Access tokens are encrypted at rest using ChaCha20-Poly1305.

On startup the API restores saved connections, validates each token against its homeserver, and removes any stale sessions automatically.

## Scripts

Helper scripts live in the `scripts/` directory.

### `update-command-tree.sh`

Regenerates `shared/command-tree.json` from the [uwu-admin fork of continuwuity](https://github.com/PNRxA/continuwuity). Clones the fork into `../continuwuity` if it doesn't already exist, fetches upstream (including tags), rebases on `upstream/main`, and runs `cargo xtask generate-command-tree`. When a tag is specified, the script checks out that tag for generation then returns to `main` — the repo always ends up on `main`. Build prerequisites are the same as for continuwuity itself (Rust, C/C++ compiler, libclang, liburing, make).

```sh
./scripts/update-command-tree.sh [-w] [-p] [tag]
```

| Option | Description |
|--------|-------------|
| `tag` | Generate from a specific upstream version tag (e.g. `v0.5.0`). Omit to use `main` |
| `-w` | Push tags and commits to the fork (for maintainers). Without this flag, nothing is pushed |
| `-p` | Create a PR with the updated command tree (requires a version tag). Uses `gh` CLI |

Examples:

```sh
./scripts/update-command-tree.sh              # Fetch + rebase on upstream main, generate command tree
./scripts/update-command-tree.sh v0.5.0       # Fetch + rebase, generate from a specific tag, return to main
./scripts/update-command-tree.sh -w           # Same as above + push tags and main to fork
./scripts/update-command-tree.sh -w v0.5.0    # Same as above with a specific tag
./scripts/update-command-tree.sh -p v0.5.6    # Generate from a specific tag + open a PR with the changes
./scripts/update-command-tree.sh -w -p v0.5.6 # Same as above + push tags and main to fork
```

### `test.sh`

Runs the full test suite against a fresh Quadlet build. Rebuilds the container image, wipes the database, then runs frontend unit tests, backend tests (unit + integration), and E2E tests against the container.

```sh
./scripts/test.sh
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
| `reset-db` | Stop, wipe the database volume, and restart with a fresh DB |
| `test` | Rebuild image, wipe DB, and start (fresh environment for E2E tests) |
| `status` | Show service status and recent logs |
| `logs` | Follow the service journal logs |
| `destroy` | Stop service, remove quadlet files, volume, and image |

## Shared

The `shared/` directory contains data shared between the API and web frontend.

### Command Tree

The file `shared/command-tree.json` describes every admin command (names, descriptions, argument types). It powers the console's autocomplete, and input validation on both the API and frontend. It is auto-generated from the [continuwuity fork](https://github.com/PNRxA/continuwuity) source code via [`update-command-tree.sh`](#update-command-treesh).

## Testing

### API

```sh
cd api
cargo test                    # Unit tests (no server needed)
cargo test -- --skip integration  # Same, explicitly skipping integration tests
cargo test                    # Full suite including integration tests (needs server)
```

Unit tests cover auth, crypto, input validation, command parsing, and response handling.

Integration tests require a running Continuwuity instance. Add the following to `api/.env`:

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

### E2E

End-to-end tests use [Playwright](https://playwright.dev/) with Chromium and run against a real API + Vite dev server stack.

```sh
cd web
npx playwright install --with-deps chromium
npx playwright test
```

The suite requires a running `uwu-admin-api` on `:3001` and the same test variables used by the backend integration tests. Copy the web example env and fill in values:

```sh
cp web/.env.example web/.env
```

| Variable | Description | Default |
|----------|-------------|---------|
| `E2E_BASE_URL` | Base URL to test against (skip to use Vite dev server) | `http://localhost:5173` |
| `TEST_HOMESERVER` | Homeserver URL (e.g. `https://matrix.example.com`) | — |
| `TEST_USERNAME` | Bot username | — |
| `TEST_PASSWORD` | Bot password | — |
| `TEST_ROOM_ID` | Admin room ID or alias | — |

A global setup step handles account creation (or login) and adds a test server, saving session tokens so individual specs start authenticated. Tests run serially (`workers: 1`) because they share server state.

Specs cover:

- **Auth** — registration, login, logout flows
- **Server management** — adding, switching, removing servers
- **Console** — sending admin commands, viewing responses
- **User actions** — user list table actions (profile, ban, deactivate, etc.)
- **Room actions** — room list table actions (details, aliases, bans, etc.)
- **Theme toggle** — light/dark mode switching
- **Copy to clipboard** — copying values from the UI

You can also point the tests at an already-running instance (e.g. a container) by setting `E2E_BASE_URL`:

```sh
E2E_BASE_URL=http://localhost:8080 npx playwright test
```

When testing against a Podman Quadlet, use `http://127.0.0.1:8080` instead of `localhost` — Playwright resolves `localhost` to `[::1]` (IPv6), which the container port binding may not listen on.

## CI

A GitHub Actions workflow (`.github/workflows/test.yml`) runs on every push to `main` and on pull requests.

| Job | Runner | Trigger | What it does |
|-----|--------|---------|--------------|
| **frontend** | `ubuntu-latest` | push + PR | `npm ci`, type-check, Vitest unit tests |
| **backend** | self-hosted (push) / `ubuntu-latest` (PR) | push + PR | `cargo test` — unit tests always, integration tests on push (secrets available) |
| **e2e** | self-hosted | push only | Builds the API, starts it in the background, installs Playwright + Chromium, runs the full e2e suite, uploads the HTML report as an artifact |
| **update-command-tree** | self-hosted | manual (`workflow_dispatch`) | Clones the continuwuity fork, generates `shared/command-tree.json` for the given version tag, and opens a PR |

The **e2e** job only runs on pushes to `main` (not PRs) because it needs repository secrets and a self-hosted runner with access to a live Continuwuity instance. The Playwright HTML report is uploaded as a build artifact and retained for 14 days.

The **update-command-tree** workflow is triggered manually from the Actions tab. It clones the public continuwuity fork over HTTPS — no additional secrets are required beyond the default `GITHUB_TOKEN`.

## Releasing

uwu-admin versions track Continuwuity versions - `v0.5.5` of uwu-admin is compatible with Continuwuity `v0.5.5`. When uwu-admin needs additional releases for the same Continuwuity version (bug fixes, UI improvements, etc.), we append a dash suffix: `v0.5.5-1`, `v0.5.5-2`, and so on.

| Tag | Meaning |
|-----|---------|
| `v0.5.5` | Initial release for Continuwuity v0.5.5 |
| `v0.5.5-1` | First uwu-admin patch for the same Continuwuity version |
| `v0.5.5-2` | Second uwu-admin patch |
| `v0.6.0` | Tracks Continuwuity v0.6.0 |

Pushing a `v*` tag triggers the release workflow (`.github/workflows/release.yml`), which builds the container image, pushes it to ghcr.io, and creates a GitHub Release with auto-generated notes.

```sh
git tag v0.5.5
git push origin v0.5.5
```

This will:

1. Build the container from `containers/docker/Dockerfile`
2. Push to `ghcr.io/PNRxA/uwu-admin` with tags `0.5.5`, `v0.5.5`, and `latest`
3. Create a GitHub Release for the tag with auto-generated release notes
