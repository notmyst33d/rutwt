#!/bin/bash
psql -c "DROP DATABASE rutwt;"
psql -c "CREATE DATABASE rutwt;"
for file in data/0000-base-schema-postgres.sql data/0001-media-update-postgres.sql; do
    echo "Running schema $file"
    psql rutwt < "$file"
done
