#!/usr/bin/env bash
#
# Create a test account in vaultwarden and extract the database
#
# This script:
# 1. Opens vaultwarden in browser for account creation
# 2. Waits for you to create the account
# 3. Extracts the database with the account
# 4. Saves it for CI use
#

set -e

echo "=== Vaultwarden Test Account Setup ==="
echo ""
echo "Step 1: Create the test account"
echo "--------------------------------"
echo "Opening http://localhost:8080 in your browser..."
echo ""
echo "Please:"
echo "  1. Click 'Create Account'"
echo "  2. Enter email:    test@fnox.ci"
echo "  3. Enter password: TestCIPassword123!"
echo "  4. Complete registration"
echo ""
echo "Press ENTER after you've created the account..."

# Open browser
open http://localhost:8080 2>/dev/null || xdg-open http://localhost:8080 2>/dev/null || echo "(Could not open browser automatically)"

read -r

echo ""
echo "Step 2: Verify account works"
echo "-----------------------------"
echo "Testing login with bw CLI..."

# Configure bw CLI
bw config server http://localhost:8080

# Try to login
if bw login test@fnox.ci TestCIPassword123! --raw >/dev/null 2>&1; then
	echo "✓ Account created and login successful!"
else
	echo "✗ Login failed. Please verify:"
	echo "  - Account was created with correct credentials"
	echo "  - Email: test@fnox.ci"
	echo "  - Password: TestCIPassword123!"
	exit 1
fi

# Logout
bw logout >/dev/null 2>&1 || true

echo ""
echo "Step 3: Extract database"
echo "------------------------"
echo "Extracting vaultwarden database..."

# Create directory for test fixtures
mkdir -p test/fixtures

# Copy database from Docker container
docker cp fnox-vaultwarden-test:/data/db.sqlite3 test/fixtures/vaultwarden-test.db

# Verify database exists and has content
if [ -f test/fixtures/vaultwarden-test.db ]; then
	DB_SIZE=$(wc -c <test/fixtures/vaultwarden-test.db | tr -d ' ')
	echo "✓ Database extracted: test/fixtures/vaultwarden-test.db ($DB_SIZE bytes)"
else
	echo "✗ Database extraction failed"
	exit 1
fi

echo ""
echo "Step 4: Create initialization script"
echo "-------------------------------------"

cat >test/fixtures/init-vaultwarden-db.sh <<'SCRIPT_EOF'
#!/usr/bin/env bash
# Initialize vaultwarden with pre-seeded test database
set -e

DB_PATH="${1:-/data/db.sqlite3}"
SOURCE_DB="$(dirname "$0")/vaultwarden-test.db"

if [ -f "$SOURCE_DB" ]; then
    echo "Initializing vaultwarden database from test fixture..."
    cp "$SOURCE_DB" "$DB_PATH"
    chmod 644 "$DB_PATH"
    echo "✓ Database initialized with test account (test@fnox.ci)"
else
    echo "✗ Test database not found at: $SOURCE_DB"
    exit 1
fi
SCRIPT_EOF

chmod +x test/fixtures/init-vaultwarden-db.sh

echo "✓ Created initialization script: test/fixtures/init-vaultwarden-db.sh"

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Files created:"
echo "  - test/fixtures/vaultwarden-test.db (pre-seeded database)"
echo "  - test/fixtures/init-vaultwarden-db.sh (initialization script)"
echo ""
echo "Test account credentials:"
echo "  Email:    test@fnox.ci"
echo "  Password: TestCIPassword123!"
echo ""
echo "Next steps:"
echo "  1. Review the files to ensure they look correct"
echo "  2. Commit them to the repository"
echo "  3. Update CI to use the pre-seeded database"
