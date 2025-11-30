#!/bin/bash

# Setup test database for integration tests
# Usage: ./scripts/setup-test-db.sh [database_url]

set -e

# Database URL (can be overridden via environment or argument)
DATABASE_URL="${1:-${TEST_DATABASE_URL:-postgresql://postgres:postgres@localhost:5432}}"
TEST_DB_NAME="pgadmin_test"
TEST_DATABASE_URL="${DATABASE_URL}/${TEST_DB_NAME}"

echo "🔧 Setting up test database..."
echo "Database URL: $DATABASE_URL"
echo "Test Database: $TEST_DB_NAME"

# Check if psql is available
if ! command -v psql &> /dev/null; then
    echo "❌ psql not found. Please install PostgreSQL client tools."
    exit 1
fi

# Extract connection info from URL
# Format: postgresql://user:password@host:port/database
HOST=$(echo "$DATABASE_URL" | sed -E 's|.*@([^:/]+).*|\1|')
PORT=$(echo "$DATABASE_URL" | sed -E 's|.*:([0-9]+).*|\1|' || echo "5432")
USER=$(echo "$DATABASE_URL" | sed -E 's|.*://([^:]+).*|\1|')

echo "Connecting to PostgreSQL at $HOST:$PORT as $USER..."

# Drop existing test database if it exists
echo "Dropping existing test database (if any)..."
PGPASSWORD="${PGPASSWORD:-postgres}" psql -h "$HOST" -p "$PORT" -U "$USER" -c "DROP DATABASE IF EXISTS $TEST_DB_NAME;" 2>/dev/null || true

# Create test database
echo "Creating test database..."
PGPASSWORD="${PGPASSWORD:-postgres}" psql -h "$HOST" -p "$PORT" -U "$USER" -c "CREATE DATABASE $TEST_DB_NAME;"

echo "✅ Test database setup complete!"
echo ""
echo "To run tests, use:"
echo "  TEST_DATABASE_URL=$TEST_DATABASE_URL cargo test"
echo ""
echo "Or set the environment variable:"
echo "  export TEST_DATABASE_URL=$TEST_DATABASE_URL"
echo "  cargo test"
