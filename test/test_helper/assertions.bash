#!/usr/bin/env bash

# Common assertion helpers for fnox bats tests

# Assert that fnox command succeeds
# Usage: assert_fnox_success <command> [args...]
assert_fnox_success() {
	run "$FNOX_BIN" "$@"
	assert_success
}

# Assert that fnox command fails
# Usage: assert_fnox_failure <command> [args...]
assert_fnox_failure() {
	run "$FNOX_BIN" "$@"
	assert_failure
}

# Assert that config file contains specific content
# Usage: assert_config_contains <content>
assert_config_contains() {
	local content="$1"
	local config_file="${FNOX_CONFIG_FILE:-fnox.toml}"

	if [[ ! -f $config_file ]]; then
		echo "Config file $config_file does not exist" >&2
		return 1
	fi

	if ! grep -q "$content" "$config_file"; then
		echo "Config file $config_file does not contain: $content" >&2
		echo "Config contents:" >&2
		cat "$config_file" >&2
		return 1
	fi
}

# Assert that config file does not contain specific content
# Usage: assert_config_not_contains <content>
assert_config_not_contains() {
	local content="$1"
	local config_file="${FNOX_CONFIG_FILE:-fnox.toml}"

	if [[ ! -f $config_file ]]; then
		return 0 # File doesn't exist, so it doesn't contain the content
	fi

	if grep -q "$content" "$config_file"; then
		echo "Config file $config_file contains (should not): $content" >&2
		echo "Config contents:" >&2
		cat "$config_file" >&2
		return 1
	fi
}

# Assert that a secret exists in the config
# Usage: assert_secret_exists <secret_name>
assert_secret_exists() {
	local secret_name="$1"
	assert_config_contains "\"$secret_name\""
}

# Assert that a secret does not exist in the config
# Usage: assert_secret_not_exists <secret_name>
assert_secret_not_exists() {
	local secret_name="$1"
	assert_config_not_contains "\"$secret_name\""
}

# Assert that a profile exists in the config
# Usage: assert_profile_exists <profile_name>
assert_profile_exists() {
	local profile_name="$1"
	assert_config_contains "\\[profiles\\.$profile_name\\]"
}

# Assert that a profile does not exist in the config
# Usage: assert_profile_not_exists <profile_name>
assert_profile_not_exists() {
	local profile_name="$1"
	assert_config_not_contains "\\[profiles\\.$profile_name\\]"
}

# Assert that output contains a specific secret value (for testing get command)
# Usage: assert_secret_output <secret_name> <expected_value>
assert_secret_output() {
	local secret_name="$1"
	local expected_value="$2"

	assert_output --partial "$secret_name"
	assert_output --partial "$expected_value"
}

# Create a basic fnox config for testing
# Usage: create_test_config [provider]
create_test_config() {
	local provider="${1:-age}"
	cat >"${FNOX_CONFIG_FILE:-fnox.toml}" <<EOF
root = true

[secrets]

[secrets.test_secret]
value = "test_value"

[providers.test-provider]
type = "age"
recipients = ["age1exampleexampleexampleexampleexampleexampleexampleexampleexampleexample"]

[profiles.test]
EOF
}

# Create a test profile with specific provider
# Usage: create_test_profile <profile_name> <provider>
create_test_profile() {
	local profile_name="$1"
	local provider="$2"

	cat >"${FNOX_CONFIG_FILE:-fnox.toml}" <<EOF
root = true

[providers.$provider-provider]
type = "$provider"

[profiles.$profile_name]
EOF
}

# Generate a simple age key pair for testing
# Usage: generate_test_age_key
generate_test_age_key() {
	if ! command -v age-keygen >/dev/null 2>&1; then
		echo "age-keygen not found, skipping age key generation" >&2
		return 1
	fi

	age-keygen -o key.txt 2>/dev/null
	local public_key
	public_key=$(grep "public key:" key.txt | cut -d' ' -f3)
	echo "$public_key"
}

# Assert that output contains fnox version pattern
# Usage: assert_fnox_version_output
assert_fnox_version_output() {
	assert_output --regexp "^fnox\ [0-9]+\.[0-9]+\.[0-9]+$"
}

# Assert that command output is valid JSON (for export command)
# Usage: assert_json_output
assert_json_output() {
	# shellcheck disable=SC2154
	# Try to parse with python, jq, or node (whichever is available)
	if command -v python3 >/dev/null 2>&1; then
		echo "$output" | python3 -m json.tool >/dev/null
	elif command -v jq >/dev/null 2>&1; then
		echo "$output" | jq . >/dev/null
	elif command -v node >/dev/null 2>&1; then
		echo "$output" | node -e "console.log(JSON.parse(require('fs').readFileSync(0, 'utf8')))" >/dev/null 2>&1
	else
		echo "No JSON parser available to validate output" >&2
		return 1
	fi
}
