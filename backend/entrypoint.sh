#!/bin/sh
set -e

echo "Waiting for PostgreSQL to start..."
until sqlx database setup; do
  echo "Postgres is unavailable - sleeping"
  sleep 2
done

echo "Postgres is up - executing migrations"
sqlx migrate run

exec "$@"