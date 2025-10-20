#!/usr/bin/env bash
#
# Setup script for HashiCorp Vault testing with local dev server
#
# This script:
# 1. Starts a local Vault dev server via Docker
# 2. Configures vault CLI to use the local server
# 3. Exports VAULT_ADDR and VAULT_TOKEN for testing
# 4. Enables KV v2 secrets engine (already enabled in dev mode at "secret/")
#
# Usage: source ./test/setup-vault-test.sh
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up HashiCorp Vault test environment...${NC}"

# Configuration
export VAULT_ADDR="http://localhost:8200"
export VAULT_TOKEN="fnox-test-token"
COMPOSE_FILE="test/docker-compose.vault.yml"

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
	echo -e "${RED}Error: Docker is not running. Please start Docker and try again.${NC}"
	exit 1
fi

# Start Vault dev server
echo -e "${YELLOW}Starting Vault dev server...${NC}"
docker compose -f "$COMPOSE_FILE" up -d

# Wait for server to be ready
echo -e "${YELLOW}Waiting for Vault to be ready...${NC}"
for i in {1..30}; do
	if curl -s "$VAULT_ADDR/v1/sys/health" >/dev/null 2>&1; then
		echo -e "${GREEN}Vault is ready!${NC}"
		break
	fi
	if [ "$i" -eq 30 ]; then
		echo -e "${RED}Error: Vault failed to start${NC}"
		exit 1
	fi
	sleep 1
done

# Check if vault CLI is installed
if ! command -v vault >/dev/null 2>&1; then
	echo -e "${YELLOW}Warning: vault CLI not found. Installing via mise...${NC}"
	if ! command -v mise >/dev/null 2>&1; then
		echo -e "${RED}Error: mise is required to install vault CLI${NC}"
		exit 1
	fi
	mise install vault
fi

# Verify vault CLI can connect
echo -e "${YELLOW}Verifying Vault connection...${NC}"
if ! vault status >/dev/null 2>&1; then
	echo -e "${RED}Error: Failed to connect to Vault${NC}"
	exit 1
fi

echo -e "${GREEN}✓ HashiCorp Vault test environment ready!${NC}"
echo -e "${GREEN}✓ VAULT_ADDR exported: $VAULT_ADDR${NC}"
echo -e "${GREEN}✓ VAULT_TOKEN exported${NC}"
echo ""
echo -e "${YELLOW}Dev mode notes:${NC}"
echo -e "  - KV v2 secrets engine is mounted at 'secret/'"
echo -e "  - Root token: fnox-test-token"
echo -e "  - This is a dev server - data is ephemeral"
echo ""
echo -e "${YELLOW}To run tests:${NC}"
echo -e "  mise run test:bats -- test/vault.bats"
echo ""
echo -e "${YELLOW}To stop the server:${NC}"
echo -e "  docker compose -f $COMPOSE_FILE down"
echo ""
