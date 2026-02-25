#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

IMAGE_NAME="localhost/uwu-admin:latest"
QUADLET_DIR="${HOME}/.config/containers/systemd"
CONTAINER_UNIT="uwu-admin.container"
VOLUME_UNIT="uwu-data.volume"
SERVICE_NAME="uwu-admin"
VOLUME_NAME="systemd-uwu-data"

usage() {
    cat <<EOF
Usage: $(basename "$0") <command>

Development helper for managing the uwu-admin quadlet.

Commands:
  build      Build the container image
  install    Copy quadlet files to ${QUADLET_DIR}
  start      Build image, install quadlet files, and start the service
  stop       Stop the service
  rebuild    Stop, rebuild image, and restart the service
  restart    Restart the service (no rebuild)
  reset-db   Stop, wipe the database volume, and restart
  test       Rebuild image, wipe DB, and start (fresh environment for E2E tests)
  status     Show service status and logs
  logs       Follow the service journal logs
  destroy    Stop service, remove quadlet files, volume, and image

EOF
}

build() {
    echo "==> Building image ${IMAGE_NAME}"
    podman build -f "${PROJECT_ROOT}/containers/docker/Dockerfile" -t "${IMAGE_NAME}" "${PROJECT_ROOT}"
    echo "==> Build complete"
}

install_quadlet() {
    mkdir -p "${QUADLET_DIR}"
    echo "==> Copying quadlet files to ${QUADLET_DIR}"
    cp "${PROJECT_ROOT}/containers/quadlet/${CONTAINER_UNIT}" "${QUADLET_DIR}/"
    cp "${PROJECT_ROOT}/containers/quadlet/${VOLUME_UNIT}" "${QUADLET_DIR}/"
    echo "==> Reloading systemd user daemon"
    systemctl --user daemon-reload
}

start() {
    if ! podman image exists "${IMAGE_NAME}" 2>/dev/null; then
        build
    fi
    if [ ! -f "${QUADLET_DIR}/${CONTAINER_UNIT}" ]; then
        install_quadlet
    fi
    echo "==> Starting ${SERVICE_NAME}"
    systemctl --user start "${SERVICE_NAME}"
    echo "==> Started — listening on http://localhost:8080"
    systemctl --user status "${SERVICE_NAME}" --no-pager || true
}

stop() {
    echo "==> Stopping ${SERVICE_NAME}"
    systemctl --user stop "${SERVICE_NAME}" 2>/dev/null || true
}

rebuild() {
    stop
    build
    install_quadlet
    echo "==> Starting ${SERVICE_NAME}"
    systemctl --user start "${SERVICE_NAME}"
    echo "==> Rebuilt and started — listening on http://localhost:8080"
    systemctl --user status "${SERVICE_NAME}" --no-pager || true
}

restart() {
    echo "==> Restarting ${SERVICE_NAME}"
    systemctl --user restart "${SERVICE_NAME}"
    systemctl --user status "${SERVICE_NAME}" --no-pager || true
}

status() {
    systemctl --user status "${SERVICE_NAME}" --no-pager || true
    echo ""
    echo "==> Recent logs:"
    journalctl --user -u "${SERVICE_NAME}" -n 20 --no-pager || true
}

logs() {
    journalctl --user -u "${SERVICE_NAME}" -f
}

reset_db() {
    echo "==> Resetting database"
    stop
    echo "  -> Removing volume ${VOLUME_NAME}"
    podman volume rm -f "${VOLUME_NAME}" 2>/dev/null || true
    echo "==> Starting ${SERVICE_NAME}"
    systemctl --user start "${SERVICE_NAME}"
    echo "==> Restarted with fresh database — listening on http://localhost:8080"
    systemctl --user status "${SERVICE_NAME}" --no-pager || true
}

test_env() {
    echo "==> Preparing fresh test environment"
    stop
    build
    install_quadlet
    echo "  -> Removing volume ${VOLUME_NAME}"
    podman volume rm -f "${VOLUME_NAME}" 2>/dev/null || true
    echo "==> Starting ${SERVICE_NAME}"
    systemctl --user start "${SERVICE_NAME}"
    echo "==> Fresh build with clean database — listening on http://localhost:8080"
    systemctl --user status "${SERVICE_NAME}" --no-pager || true
}

destroy() {
    echo "==> Tearing down everything"

    echo "  -> Stopping service"
    systemctl --user stop "${SERVICE_NAME}" 2>/dev/null || true

    echo "  -> Removing quadlet files"
    rm -f "${QUADLET_DIR}/${CONTAINER_UNIT}" "${QUADLET_DIR}/${VOLUME_UNIT}"
    systemctl --user daemon-reload
    systemctl --user reset-failed 2>/dev/null || true

    echo "  -> Removing container (if lingering)"
    podman rm -f "${SERVICE_NAME}" 2>/dev/null || true

    echo "  -> Removing volume ${VOLUME_NAME}"
    podman volume rm -f "${VOLUME_NAME}" 2>/dev/null || true

    echo "  -> Removing image ${IMAGE_NAME}"
    podman rmi -f "${IMAGE_NAME}" 2>/dev/null || true

    echo "==> Destroyed"
}

if [[ $# -lt 1 ]]; then
    usage
    exit 1
fi

case "$1" in
    build)       build ;;
    install)     install_quadlet ;;
    start)       start ;;
    stop)        stop ;;
    rebuild)     rebuild ;;
    restart)     restart ;;
    status)      status ;;
    logs)        logs ;;
    reset-db)    reset_db ;;
    test)        test_env ;;
    destroy)     destroy ;;
    *)
        echo "Unknown command: $1" >&2
        usage
        exit 1
        ;;
esac
