# Fnox Test Temporary Directories

This directory contains temporary test artifacts created during BATS test runs.

## Structure

Each test creates a unique temporary directory with the following pattern:
- `fnox-test-<test-filename>-<test-number>-<random-string>`

Example: `fnox-test-init.bats-1-abc123`

## Cleanup

The temporary directories are automatically cleaned up after each test run.
However, you can preserve them for debugging by setting:
```bash
export BATSLIB_TEMP_PRESERVE=1
```

Or preserve only on test failure:
```bash
export BATSLIB_TEMP_PRESERVE_ON_FAILURE=1
```