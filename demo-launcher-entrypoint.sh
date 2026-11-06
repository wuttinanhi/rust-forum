#!/usr/bin/env bash
# DigitalOcean droplet startup script: installs Docker, clones the repository, starts services, and seeds mock data.

set -euo pipefail

# Wait until PostgreSQL database is ready to accept connections.
#
# Arguments:
#   $1 - Host of the PostgreSQL database.
#   $2 - Port of the PostgreSQL database.
#   $3 - Database user.
#   $4 - Maximum number of retry attempts.
#
# Example usage:
#   wait_for_postgres "127.0.0.1" "5432" "postgres" 30
wait_for_postgres() {
  local host="$1"
  local port="$2"
  local user="$3"
  local max_attempts="${4:-30}"
  local attempt=1

  until pg_isready -h "$host" -p "$port" -U "$user"; do
    echo "Waiting for PostgreSQL to be ready... attempt $attempt/$max_attempts"

    if [ "$attempt" -ge "$max_attempts" ]; then
      echo "Error: PostgreSQL is not ready after $max_attempts attempts." >&2
      return 1
    fi
    attempt=$((attempt + 1))
    sleep 2
  done
}

# Install prerequisites and Docker
export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y git curl ca-certificates jq postgresql-client
curl -fsSL https://get.docker.com | sh
systemctl enable --now docker

# Clone repository if not already present
REPO_URL="https://github.com/wuttinanhi/rust-forum.git"
CLONE_DIR="rust-forum"

if [ ! -f "docker-compose.yml" ]; then
    if [ ! -d "$CLONE_DIR" ]; then
        git clone "$REPO_URL" "$CLONE_DIR"
    fi
    cd "$CLONE_DIR"
fi

# Launch forum services in background
docker compose up -d

# Wait for PostgreSQL to be ready and seed mock data
chmod +x mock_db.sh
wait_for_postgres "127.0.0.1" "5432" "postgres" 30
./mock_db.sh
