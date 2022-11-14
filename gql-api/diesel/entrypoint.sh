#!/bin/bash

set -x

diesel database reset --database-url $DATABASE_URL/usersdb --migration-dir ./migrations
diesel migration run --database-url $DATABASE_URL/usersdb --migration-dir ./migrations
