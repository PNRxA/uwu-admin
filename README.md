# uwu-admin

Web admin dashboard for [Continuwuity](https://continuwuity.org) Matrix homeservers.

Continuwuity only supports admin commands via messages in a special admin room. uwu-admin provides a proper web UI by connecting to the homeserver as a bot account, sending admin commands to the admin room, and displaying the results.

## Security Warning

**Do NOT expose the uwu-admin web interface to the public internet.** The dashboard provides full admin control over your homeserver and has no authentication layer of its own - anyone who can reach the UI can connect a bot and manage your server.

Run it on `localhost` only, or behind a VPN/firewall restricted to trusted networks.

## Architecture

```
Browser (Vue)  →  uwu-admin-api (Rust/axum :3001)  →  Matrix Homeserver
                                                         ↕
                                                    Admin Room
                                                   (bot sends !admin commands,
                                                    reads server responses)
```

## Session Persistence

The API server stores the active bot session in a local SQLite database (`uwu-admin.db` by default) so it survives restarts. The database contains a single `sessions` table with one row:

| Column | Description |
|--------|-------------|
| `homeserver` | Matrix homeserver URL |
| `access_token` | Bot account access token |
| `room_id` | Resolved admin room ID |
| `user_id` | Bot user ID |
| `since` | Last sync batch token (for resuming where the bot left off) |

On startup the API checks for a saved session, validates the token against the homeserver, and restores the connection automatically. If the token is invalid the stale session is deleted.

Set `DATABASE_URL` to override the default path:

```sh
DATABASE_URL="sqlite:/path/to/sessions.db?mode=rwc" cargo run --release
```

The room field can be either a room ID (`!abc:example.com`) or a room alias (`#admins:example.com`) — aliases are resolved automatically on connect.

## Setup

### Prerequisites

- Rust (2024 edition)
- Node.js 22+
- A Continuwuity homeserver with an admin bot account and admin room

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
3. Open the dashboard in your browser
4. Enter your homeserver URL, bot credentials, and admin room ID or alias
5. Connect and manage your server through the dashboard

## Pages

- **Overview** - Connection info, server uptime, and stats
- **Users** - List users, create new users
- **Rooms** - List rooms, view room details
- **Federation** - Federation status
- **Server** - Uptime and statistics
- **Media** - Media management
- **Console** - Raw admin command interface
