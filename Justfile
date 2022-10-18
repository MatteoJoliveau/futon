PROJECT := "futon"

set dotenv-load := false

default:
  @just --list | grep -v "    default"

build *args:
  cargo build {{ args }}

setup:
    docker-compose up -d

teardown:
    docker-compose down -v

test:
    cargo test

fmt: _fmt _clippy

_fmt:
  cargo fmt

_clippy:
  cargo clippy --fix --allow-dirty --allow-staged

check: fmt test
    cargo check

reset: teardown setup
