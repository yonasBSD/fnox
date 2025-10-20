# Test Fixtures

This directory contains test fixtures for fnox integration tests.

## Vaultwarden Test Database

**File**: `vaultwarden-test.db`

A pre-seeded SQLite database for vaultwarden containing a single test account.

### Test Account Credentials

- **Email**: `test@fnox.ci`
- **Password**: `TestCIPassword123!`
- **User ID**: `7fc3a10f-9eac-4a9a-955e-a8c4b1d2aa6e`

### How It Was Created

The database was created using:

```bash
1. Start vaultwarden locally
2. Create account via web interface at http://localhost:8080
3. Extract database with WAL checkpoint:
   ./test/extract-db-with-wal.sh
```

### Usage in CI

The `setup-bitwarden-ci.sh` script automatically:

1. Copies this database into the vaultwarden container
2. Restarts vaultwarden to load the database
3. Logs in with the test account credentials
4. Exports BW_SESSION for tests

### Regenerating

If you need to regenerate the database (e.g., after vaultwarden version changes):

```bash
# 1. Start fresh vaultwarden
docker compose -f test/docker-compose.bitwarden.yml up -d

# 2. Create account
open http://localhost:8080
# Register with: test@fnox.ci / TestCIPassword123!

# 3. Extract database
./test/extract-db-with-wal.sh

# 4. Commit the new database
git add test/fixtures/vaultwarden-test.db
git commit -m "Update vaultwarden test database"
```

### Security Note

This database contains only a test account with publicly known credentials.
It should **never** contain real user data or secrets.
