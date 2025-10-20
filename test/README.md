# Fnox E2E Test Suite

This directory contains the end-to-end test suite for Fnox, using the [Bats](https://github.com/bats-core/bats-core) testing framework.

## Running Tests

```bash
# Run all e2e tests
bats test/

# Run specific test file
bats test/version.bats

# Run specific test by name
bats test/version.bats --filter "fnox --version prints version"

# Run with verbose output
bats test/ --verbose

# Run with timing information
bats test/ --timing
```

## Writing Tests

Tests are written using Bats syntax. Each test file should:

1. Include the common setup:

```bash
setup() {
    load 'test_helper/common_setup'
    _common_setup
}

teardown() {
    _common_teardown
}
```

2. Define tests using `@test`:

```bash
@test "description of test" {
    # test code here
    run fnox --version
    assert_success
    assert_output --regexp "^fnox\ [0-9]+\.[0-9]+\.[0-9]+$"
}
```

See existing test files for examples of different testing patterns.

## Test Helper Functions

The `test_helper/assertions.bash` file provides custom assertion helpers:

- `assert_fnox_success` - Assert fnox command succeeds
- `assert_fnox_failure` - Assert fnox command fails
- `assert_config_contains` - Assert config file contains content
- `assert_config_not_contains` - Assert config file doesn't contain content
- `assert_secret_exists` - Assert secret exists in config
- `assert_secret_not_exists` - Assert secret doesn't exist in config

## Test Environment

- Each test runs in a temporary directory
- Git repository is initialized for tests that need it
- `$FNOX_BIN` points to the fnox binary under test
- Test configs are isolated per test
