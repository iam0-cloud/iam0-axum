#!/bin/sh

cleanup() {
    rm .env development.db*
}

ln -s .env.development .env

trap cleanup EXIT
trap cleanup SIGINT
trap cleanup SIGTERM

# Full reset of the database for development
sqlx database create
sqlx database setup 
mold -run cargo r

cleanup
