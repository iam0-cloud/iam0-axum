#!/bin/sh

# DO NOT use this while development of sql code without
# some modifications

cleanup() {
    rm .env
    rm development.db*
}

trap cleanup EXIT
trap cleanup SIGINT
trap cleanup SIGTERM

# Full reset of the database
{
    cleanup
    ln -s .env.development .env
    sqlx database create
    database setup
} &> /dev/null

cargo t -- --nocapture

{
    cleanup
    ln -s .env.development .env
    sqlx database create
    database setup
} &> /dev/null