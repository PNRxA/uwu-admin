#!/bin/sh
set -e

# Start the API in the background
DATABASE_URL="sqlite:/data/uwu-admin.db?mode=rwc" uwu-admin-api &
API_PID=$!

# Shut down both processes on signal
cleanup() {
    kill "$API_PID" 2>/dev/null
    exit 0
}
trap cleanup TERM INT

# Start nginx in the foreground
nginx

# If nginx exits, clean up the API
cleanup
