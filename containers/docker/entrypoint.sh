#!/bin/bash
set -e

# If running as root, fix /data ownership and re-exec as uwu
if [ "$(id -u)" = '0' ]; then
    chown -R uwu:uwu /data
    exec gosu uwu "$0" "$@"
fi

# Start the API in the background
export DATABASE_URL="sqlite:/data/uwu-admin.db?mode=rwc"
export API_LISTEN="${API_LISTEN:-127.0.0.1:3001}"
uwu-admin-api &
API_PID=$!

# Shut down both processes on signal
cleanup() {
    kill "$API_PID" 2>/dev/null
    exit 0
}
trap cleanup TERM INT

# Wait for the API to be ready before starting nginx
API_HOST="${API_LISTEN%%:*}"
API_PORT="${API_LISTEN##*:}"
echo "Waiting for API on ${API_HOST}:${API_PORT}..."
for i in $(seq 1 30); do
    if ! kill -0 "$API_PID" 2>/dev/null; then
        echo "ERROR: API process exited unexpectedly"
        exit 1
    fi
    if (echo >/dev/tcp/"$API_HOST"/"$API_PORT") 2>/dev/null; then
        echo "API is ready"
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "ERROR: API did not become ready in 30s"
        exit 1
    fi
    sleep 1
done

# Start nginx in the foreground
nginx

# If nginx exits, clean up the API
cleanup
