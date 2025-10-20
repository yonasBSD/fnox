#!/usr/bin/env bash
#
# Initialize vaultwarden with pre-seeded test database
# This script copies the test database into the vaultwarden data directory
#
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DB="$SCRIPT_DIR/vaultwarden-test.db"
TARGET_DB="${1:-/data/db.sqlite3}"

if [ ! -f "$SOURCE_DB" ]; then
	echo "Error: Test database not found at: $SOURCE_DB"
	exit 1
fi

echo "Initializing vaultwarden with pre-seeded test database..."
cp "$SOURCE_DB" "$TARGET_DB"
chmod 644 "$TARGET_DB"

echo "✓ Database initialized"
echo "✓ Test account available: test@fnox.ci / TestCIPassword123!"
