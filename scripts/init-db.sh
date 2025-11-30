#!/bin/bash

# Initialize databases for development and testing
# This script runs automatically when PostgreSQL starts in Docker

set -e

# Create test database
echo "Creating pgadmin_test database..."
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
    CREATE DATABASE pgadmin_test;
    GRANT ALL PRIVILEGES ON DATABASE pgadmin_test TO "$POSTGRES_USER";
EOSQL

echo "✅ Databases initialized successfully"
