#!/bin/sh

cleanup() {
    # comment this while dev on a specific feature if you
    # want your lsp to not detect errors, while working on sql keep it
    rm development.db*
}

trap cleanup SIGINT
trap cleanup SIGTERM

# Full reset of the database for development
sqlx database create
sqlx database setup

mold -run cargo run

{
    cleanup
    sqlx database create
    sqlx database setup
} &> /dev/null
