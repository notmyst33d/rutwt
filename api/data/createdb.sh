#!/bin/bash
rm main.db
for file in data/0000-base-schema.sql data/0001-media-update.sql data/0002-postgres-support.sql; do
    echo "Running schema $file"
    sqlite3 main.db < "$file"
done
