#!/usr/bin/env bash
set -e

# Run migrations
echo "==[ Running migrations ]=="
dbmate -d /app/db/migrations migrate --strict --verbose
echo "==[ Migrations complete ]=="
# Run the main application
exec /app/task-scheduler-rs