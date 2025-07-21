#!/bin/sh
set -e

sqlx migrate run

exec "$@"