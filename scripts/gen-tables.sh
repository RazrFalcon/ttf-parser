#!/usr/bin/env bash

set -e

mypy --strict gen-tables.py
python3 gen-tables.py > ../src/raw.rs
rustfmt ../src/raw.rs
