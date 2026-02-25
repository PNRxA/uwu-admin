# Podman Quadlet Setup

Run uwu-admin as a systemd service using Podman Quadlet files.

## Build the image

From the repository root:

```sh
podman build -f containers/docker/Dockerfile -t uwu-admin .
```

## Install the Quadlet files

For rootless (recommended):

```sh
mkdir -p ~/.config/containers/systemd
cp containers/quadlet/*.container containers/quadlet/*.volume ~/.config/containers/systemd/
systemctl --user daemon-reload
```

For rootful (`/etc/containers/systemd/`), omit `--user` from systemctl commands.

## Configure secrets

Before starting the service, edit `uwu-admin.container` and replace the
placeholder values for `JWT_SECRET` and `ENCRYPTION_KEY` with real 32-byte hex
keys:

```sh
openssl rand -hex 32
```

Set each generated value in the `Environment=` lines. The service will fail to
start if these are left as the defaults.

## Start the service

```sh
systemctl --user start uwu-admin
```

## Check status

```sh
systemctl --user status uwu-admin
```

## View logs

```sh
journalctl --user -u uwu-admin
```

## Enable on boot

```sh
systemctl --user enable uwu-admin
loginctl enable-linger $USER   # keep user services running after logout
```

## Stop the service

```sh
systemctl --user stop uwu-admin
```

The admin panel will be available at `http://localhost:8080`.

## Development script

A helper script at `scripts/quadlet-dev.sh` wraps all of the above for quick
iteration:

```sh
./scripts/quadlet-dev.sh start    # build image, install quadlet files, start service
./scripts/quadlet-dev.sh stop     # stop the service
./scripts/quadlet-dev.sh restart  # restart the service
./scripts/quadlet-dev.sh reset-db # stop, wipe the database volume, and restart
./scripts/quadlet-dev.sh status   # show service status and recent logs
./scripts/quadlet-dev.sh logs     # follow journal logs
./scripts/quadlet-dev.sh destroy  # stop service, remove quadlet files, volume, and image
```

`reset-db` wipes the database volume and restarts the service with a fresh
SQLite database, without rebuilding the image. Useful for E2E test runs that
need a clean slate.

`destroy` tears down everything: stops the service, deletes the quadlet unit
files from `~/.config/containers/systemd/`, removes the container, volume, and
image so you can start fresh.
