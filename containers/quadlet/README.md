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

A helper script at `scripts/quadlet-dev.sh` wraps all of the above for quick iteration. See the [full command reference](../../README.md#quadlet-devsh) in the main README.
