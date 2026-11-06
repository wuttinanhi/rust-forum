#!/usr/bin/env bash
set -euo pipefail

# Resolve database connection URL from environment or .env file.
#
# Arguments:
#   None
#
# Example usage:
#   load_database_url
load_database_url() {
  if [ -z "${DATABASE_URL:-}" ] && [ -f .env ]; then
    DATABASE_URL="$(grep -E '^DATABASE_URL=' .env | head -n 1 | cut -d '=' -f2-)"
  fi
  DATABASE_URL="${DATABASE_URL:-postgres://postgres:example@127.0.0.1:5432/postgres}"
  export DATABASE_URL
}

# Execute an SQL file against the PostgreSQL database.
#
# Arguments:
#   $1 - Path to the SQL file to execute.
#
# Example usage:
#   execute_sql_file "mock_db.sql"
execute_sql_file() {
  local sql_file="$1"
  if [ ! -f "$sql_file" ]; then
    echo "Error: SQL file not found: $sql_file" >&2
    exit 1
  fi

  if command -v psql >/dev/null 2>&1; then
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -f "$sql_file"
  elif command -v docker >/dev/null 2>&1 && docker compose ps db >/dev/null 2>&1; then
    docker compose exec -T db psql -U postgres -d postgres -v ON_ERROR_STOP=1 -q < "$sql_file"
  else
    echo "Error: Neither psql nor docker compose db service is available." >&2
    exit 1
  fi
}

# Populate the database using the mock data SQL script.
#
# Arguments:
#   $1 - Path to the SQL file containing mock data statements.
#
# Example usage:
#   mock_database "mock_db.sql"
mock_database() {
  local sql_file="$1"
  execute_sql_file "$sql_file"
  echo "Database seeded with 3 users, 3 posts, and 5 comments per post."
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SQL_FILE="${SCRIPT_DIR}/mock_db.sql"

load_database_url
mock_database "$SQL_FILE"
