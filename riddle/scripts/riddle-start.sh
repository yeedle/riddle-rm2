#!/bin/sh
# AppLoad entry: load oracle.env, then run riddle (windowed via QTFB_KEY).
HERE="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$HERE/oracle.env" ]; then
    set -a; . "$HERE/oracle.env"; set +a
fi
exec "$HERE/riddle"
