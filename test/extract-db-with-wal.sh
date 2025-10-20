#!/usr/bin/env bash
#
# Extract vaultwarden database with WAL checkpoint
# This ensures all changes are merged into the main database file
#

set -e

CONTAINER_NAME="${1:-fnox-vaultwarden-test}"
OUTPUT_DB="test/fixtures/vaultwarden-test.db"

echo "Extracting database from $CONTAINER_NAME..."

# First, checkpoint the WAL to merge changes into main database
echo "Checkpointing WAL..."
docker exec "$CONTAINER_NAME" sh -c '
cd /data
if command -v sqlite3 >/dev/null 2>&1; then
    sqlite3 db.sqlite3 "PRAGMA wal_checkpoint(TRUNCATE);"
else
    # sqlite3 not available in vaultwarden image
    # Copy all files instead and checkpoint locally
    echo "sqlite3 not available in container, will checkpoint locally"
fi
'

# Copy the database files
echo "Copying database files..."
mkdir -p test/fixtures
docker cp "$CONTAINER_NAME:/data/db.sqlite3" "$OUTPUT_DB.tmp"

# If WAL files exist, copy them too
docker cp "$CONTAINER_NAME:/data/db.sqlite3-wal" "$OUTPUT_DB.tmp-wal" 2>/dev/null || true
docker cp "$CONTAINER_NAME:/data/db.sqlite3-shm" "$OUTPUT_DB.tmp-shm" 2>/dev/null || true

# Checkpoint locally
echo "Checkpointing WAL locally..."
sqlite3 "$OUTPUT_DB.tmp" "PRAGMA wal_checkpoint(TRUNCATE);" 2>/dev/null || true

# Move to final location
mv "$OUTPUT_DB.tmp" "$OUTPUT_DB"
rm -f "$OUTPUT_DB.tmp-wal" "$OUTPUT_DB.tmp-shm"

# Verify
USER_COUNT=$(sqlite3 "$OUTPUT_DB" "SELECT COUNT(*) FROM users;")
echo "✓ Database extracted: $OUTPUT_DB"
echo "✓ User count: $USER_COUNT"

if [ "$USER_COUNT" -eq 0 ]; then
	echo "✗ Warning: No users found in database!"
	exit 1
fi

ls -lh "$OUTPUT_DB"
