#!/bin/bash
rm main.db
for file in $(ls -1 data/*.sql); do
    echo "Running schema $file"
    sqlite3 main.db < "$file"
done
