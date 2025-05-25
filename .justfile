#!/usr/bin/env just --justfile

patch:
    cargo release patch --no-publish --execute

minor:
    cargo release minor --no-publish --execute

major:
    cargo release major --no-publish --execute
