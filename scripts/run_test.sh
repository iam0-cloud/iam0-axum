#!/bin/sh

# DO NOT use this while development of sql code without
# some modifications

cleanup() {
    rm development.db*
}

trap cleanup SIGINT
trap cleanup SIGTERM

# Full reset of the database
{
    cleanup
    sqlx database create
    sqlx database setup
} &> /dev/null

cargo t -- --nocapture

{
    cleanup
    sqlx database create
    sqlx database setup
} &> /dev/null