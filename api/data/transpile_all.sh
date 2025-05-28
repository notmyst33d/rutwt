#!/bin/bash
rm main.db
for file in data/0000-base-schema.sql data/0001-media-update.sql; do
    echo "Running transpile $file"
    python data/transpile.py $file
done
