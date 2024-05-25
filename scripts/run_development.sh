#!/bin/sh

cleanup() {
    rm .env
    # comment this while dev on a specific feature if you
    # want your lsp to not detect errors, while working on sql keep it
    #rm development.db*
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
