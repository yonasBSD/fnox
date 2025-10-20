#!/usr/bin/env bats

setup() {
    load 'test_helper/common_setup'
    _common_setup
}

teardown() {
    _common_teardown
}

@test "fnox list shows empty message with no secrets" {
    # Create empty config with root=true to prevent recursion
    echo "root = true" > "${FNOX_CONFIG_FILE:-fnox.toml}"

    assert_fnox_success list
    assert_output --partial "No secrets defined"
}

@test "fnox list displays table with secrets" {
    create_test_config

    # Add more secrets for a richer test
    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.db_password]
default = "default_password"
description = "Database password"

[secrets.api_key]
provider = "test-provider"
value = "api-key-name"
description = "API key from provider"
EOF

    assert_fnox_success list
    assert_output --partial "Key"
    assert_output --partial "Type"
    assert_output --partial "test_secret"
    assert_output --partial "db_password"
    assert_output --partial "api_key"
}

@test "fnox list shows descriptions" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.my_secret]
default = "secret_value"
description = "My test secret"
EOF

    assert_fnox_success list
    assert_output --partial "my_secret"
    assert_output --partial "My test secret"
}

@test "fnox list shows provider information" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.provider_secret]
provider = "test-provider"
value = "provider-key-123"
description = "Secret from provider"
EOF

    assert_fnox_success list
    assert_output --partial "provider_secret"
    assert_output --partial "provider (test-provider)"
    assert_output --partial "provider-key-123"
}

@test "fnox list shows secrets with if_missing defined" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.required_secret]
if_missing = "error"
description = "Required secret"

[secrets.optional_secret]
if_missing = "warn"
description = "Optional secret"
EOF

    assert_fnox_success list
    assert_output --partial "required_secret"
    assert_output --partial "optional_secret"
}

@test "fnox list with specific profile" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[profiles.production]
[profiles.production.secrets]

[profiles.production.secrets.prod_secret]
default = "prod_value"
description = "Production secret"
EOF

    assert_fnox_success list --profile production
    assert_output --partial "prod_secret"
}

@test "fnox list shows values with --values flag" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.visible_secret]
default = "visible_value"
description = "Secret with visible value"
EOF

    assert_fnox_success list --values
    assert_output --partial "visible_secret"
    assert_output --partial "visible_value"
}

@test "fnox list without --values flag does not show values" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.hidden_secret]
default = "hidden_value"
description = "Secret with hidden value"
EOF

    assert_fnox_success list
    assert_output --partial "hidden_secret"
    refute_output --partial "hidden_value"
}

@test "fnox list works with ls alias" {
    create_test_config

    assert_fnox_success ls
    assert_output --partial "test_secret"
}

@test "fnox list shows different source types" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.stored_secret]
value = "stored_value"
description = "Stored secret"

[secrets.default_secret]
default = "default_value"
description = "Default secret"

[secrets.env_secret]
description = "Environment variable secret"

[secrets.provider_secret]
provider = "test-provider"
value = "provider-key"
description = "Provider secret"
EOF

    assert_fnox_success list
    assert_output --partial "stored_secret"
    assert_output --partial "default_secret"
    assert_output --partial "env_secret"
    assert_output --partial "provider_secret"
}

@test "fnox list with --full flag shows full provider keys" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.long_key]
provider = "test-provider"
value = "this-is-a-very-long-provider-key-that-exceeds-forty-characters-in-length"
description = "Secret with long key"
EOF

    # Without --full, should be truncated
    run "$FNOX_BIN" list
    assert_success
    assert_output --partial "this-is-a-very-long-provider-key-that..."

    # With --full, should show complete key
    run "$FNOX_BIN" list --full
    assert_success
    assert_output --partial "this-is-a-very-long-provider-key-that-exceeds-forty-characters-in-length"
}

@test "fnox list --values shows comprehensive information" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.complete_secret]
default = "secret_value_123"
description = "Complete secret"
EOF

    assert_fnox_success list --values
    assert_output --partial "complete_secret"
    assert_output --partial "secret_value_123"
    assert_output --partial "Value"
}

@test "fnox list with --no-color flag works" {
    create_test_config

    # Should work without error
    assert_fnox_success list --no-color
    assert_output --partial "test_secret"
}

@test "fnox list with --sources flag shows source file paths" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.source_test]
default = "test_value"
description = "Test secret for source tracking"
EOF

    assert_fnox_success list --sources
    assert_output --partial "Source File"
    assert_output --partial "source_test"
    assert_output --partial "fnox.toml"
}

@test "fnox list --sources shows correct file path" {
    create_test_config

    # Create a config with secrets
    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.tracked_secret]
default = "tracked_value"
description = "Secret with source tracking"
EOF

    run "$FNOX_BIN" list --sources
    assert_success
    # Should show the full path to the config file
    assert_output --partial "$(pwd)/fnox.toml"
}

@test "fnox list --sources --values shows both source and values" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.combined_test]
default = "combined_value"
description = "Test for combined flags"
EOF

    assert_fnox_success list --sources --values
    assert_output --partial "Source File"
    assert_output --partial "Value"
    assert_output --partial "combined_test"
    assert_output --partial "combined_value"
    assert_output --partial "fnox.toml"
}

@test "fnox list without --sources does not show Source File column" {
    create_test_config

    cat >> "${FNOX_CONFIG_FILE:-fnox.toml}" << EOF

[secrets.no_source]
default = "no_source_value"
EOF

    assert_fnox_success list
    refute_output --partial "Source File"
    assert_output --partial "no_source"
}
