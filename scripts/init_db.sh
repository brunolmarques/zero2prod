#!/usr/bin/env bash
set -x
set -eo pipefail

# Check if psql health check tool is installed
if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed." 
    exit 1
fi

# Check if slqx-cli is installed via cargo
if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 "    cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres"
    echo >&2 "to install it."
    exit 1
fi

# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER=${POSTGRES_USER:=postgres}

# Check if a custom password has been set, otherwise default to 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"

# Check if a custom database name has been set, otherwise default to 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"

# Check if a custom port has been set, otherwise default to '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

# Launch postgres using Podman
if [[ $1 ]]
then
    # if a Postgres container is running, print instructions to kill it and exit
    RUNNING_POSTGRES_CONTAINER=$(podman ps --filter 'name=postgres' --format '{{.ID}}')
    if [[ -n $RUNNING_POSTGRES_CONTAINER ]]; then
        echo >&2 "There is a Postgres container already running, kill it with"
        echo >&2 "    podman kill ${RUNNING_POSTGRES_CONTAINER}"
        exit 1
    fi
    
    # Initialize podman VM
    PODMAN_MACHINE_RUNNING=$(podman ps)
    if ! [[ -n $PODMAN_MACHINE_RUNNING ]]; then
        echo >&2 "Starting Podman Machine"
        podman machine init
    fi

    # Start podman machine
    podman machine start

    # Launch postgres using Podman
    podman run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d \
        --name "postgres_$(date '+%s')" \
        docker.io/library/postgres \
        postgres -N 1000
        # ^ Increased maximum number of connections for testing purposes
fi

# Set password fot psql
export PGPASSWORD="${DB_PASSWORD}"

# Keep pinging Postgres until it's ready to accept commands
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do 
    >&2 echo "Postgres is still unavailable - sleeping" 
    sleep 1 
done

>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!" 

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create 
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"