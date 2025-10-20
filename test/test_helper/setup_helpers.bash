#!/usr/bin/env bash

# Setup helper functions for fnox bats tests

# Wait for a background process to finish
# Usage: wait_for_process <pid>
wait_for_process() {
	local pid="$1"
	local timeout="${2:-30}"
	local count=0

	while kill -0 "$pid" 2>/dev/null && [[ $count -lt $timeout ]]; do
		sleep 1
		((count++))
	done

	if kill -0 "$pid" 2>/dev/null; then
		kill "$pid" 2>/dev/null
		return 1
	fi

	wait "$pid"
	return $?
}

# Create a temporary file with specific content
# Usage: create_temp_file <filename> <content>
create_temp_file() {
	local filename="$1"
	local content="$2"

	echo "$content" >"$filename"
}

# Clean up any existing fnox processes
# Usage: cleanup_fnox_processes
cleanup_fnox_processes() {
	# Kill any remaining fnox processes from this test
	if command -v pkill >/dev/null 2>&1; then
		pkill -f "$FNOX_BIN" 2>/dev/null || true
	fi
}

# Set up a mock provider for testing
# Usage: setup_mock_provider <provider_name>
setup_mock_provider() {
	local provider_name="$1"

	# Create a mock provider script that just echoes its arguments
	cat >"mock-$provider_name.sh" <<'EOF'
#!/bin/bash
echo "Mock provider called with: $*"
exit 0
EOF
	chmod +x "mock-$provider_name.sh"

	# Add current directory to PATH so the mock can be found
	export PATH=".:$PATH"
}

# Simulate network delay for testing
# Usage: simulate_delay <seconds>
simulate_delay() {
	local seconds="$1"
	sleep "$seconds"
}

# Generate a large secret for testing size limits
# Usage: generate_large_secret <size_in_bytes>
generate_large_secret() {
	local size="$1"
	head -c "$size" </dev/zero | tr '\0' 'A'
}

# Create a directory structure for testing
# Usage: create_test_dir_structure
create_test_dir_structure() {
	mkdir -p test_dir/subdir
	echo "test file" >test_dir/test_file.txt
	echo "subdir file" >test_dir/subdir/sub_file.txt
}
