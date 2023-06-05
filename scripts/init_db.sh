#!/usr/bin/env bash
# From https://github.com/LukeMathWalker/zero-to-production/blob/main/scripts/init_db.sh
# DATABASE_URL parse fromhttps://stackoverflow.com/questions/6174220/parse-url-in-shell-script 
# set -x
set -eo pipefail

# This script starts with the environment variable DATABASE_URL
# First we check if the variable is set.
# If it is not, then we look in the current directory for a .env file.
# If this fails, we panic.
# FIXME Need a warning if DATABASE_URL is different between the environment
# variable and the one found in .env. 
# Or echo a message to suggest to the user to set DATABASE_URL according to 
# the value set by this script.
if [[ -z "${DATABASE_URL-}" ]]; then
  # If the variable is not set, look if there is a .env file
  if [[ ! -e "./.env" ]]; then 
    echo "DATABASE_URL is not set, and there is no '.env' file." >&2
    exit 1
  fi
  ## FIXME Need more testing (eg does DATABASE_URL exists in .env?
  export $(grep DATABASE_URL .env | xargs)
fi

# Look for prerequisite: psql and sqlx
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
 fi
 
if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version='~0.6' sqlx-cli --no-default-features --features rustls,postgres"
  echo >&2 "to install it."
  exit 1
fi

# Extract individual components from the DATABASE_URL variable.

# Extract the protocol
PROTO="`echo $DATABASE_URL | grep '://' | sed -e's,^\(.*://\).*,\1,g'`"
# remove the protocol
URL=`echo $DATABASE_URL | sed -e s,$PROTO,,g`

# Extract the user and password (if any)
USERPASSWORD="`echo $URL | grep @ | cut -d@ -f1`"
DB_PASSWORD=`echo $USERPASSWORD | grep : | cut -d: -f2`
if [ -n "$DB_PASSWORD" ]; then
    DB_USER=`echo $USERPASSWORD | grep : | cut -d: -f1`
else
    DB_USER=$USERPASSWORD
fi

# Extract the host -- updated
HOSTPORT=`echo $URL | sed -e s,$USERPASSWORD@,,g | cut -d/ -f1`
DB_PORT=`echo $HOSTPORT | grep : | cut -d: -f2`
if [ -n "$DB_PORT" ]; then
    DB_HOST=`echo $HOSTPORT | grep : | cut -d: -f1`
else
    DB_HOST=$HOSTPORT
fi

# Extract the database name (which is the path of the URL)
DB_NAME="`echo $URL | grep / | cut -d/ -f2-`"


# Allow to skip Docker if a dockerized Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]
then
  # if a postgres container is running, print instructions to kill it and exit
  RUNNING_POSTGRES_CONTAINER=$(docker ps --filter 'name=postgres' --format '{{.ID}}')
  if [[ -n $RUNNING_POSTGRES_CONTAINER ]]; then
    echo >&2 "there is a postgres container already running, kill it with"
    echo >&2 "    docker kill ${RUNNING_POSTGRES_CONTAINER}"
    exit 1
  fi
  # Launch postgres using Docker
  docker run \
      -e POSTGRES_USER=${DB_USER} \
      -e POSTGRES_PASSWORD=${DB_PASSWORD} \
      -e POSTGRES_DB=${DB_NAME} \
      -p "${DB_PORT}":5432 \
      -d \
      --name "postgres_$(date '+%s')" \
      postgres -N 1000
      # ^ Increased maximum number of connections for testing purposes
    HOST=$HOSTPORT
 fi
 
# Keep pinging Postgres until it's ready to accept commands
until PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

# No need to export the DATABASE_URL variable a gain
# export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run
 
>&2 echo "Postgres has been migrated, ready to go!"
