PROJECT := "futon"

# TESTCONTAINERS_RUNTIMES: docker | podman
TESTCONTAINERS_RUNTIME := "docker"

set dotenv-load := false

default:
  @just --list | grep -v "    default"

build *args:
  cargo build {{ args }}

watch:
  cargo-watch --shell 'just check'

setup:
    docker-compose up -d

teardown:
    docker-compose down -v

test: cleanup-containers
    TESTCONTAINERS=remove cargo nextest run --features test-{{ TESTCONTAINERS_RUNTIME }}

fmt: _fmt _clippy

_fmt:
  cargo fmt

_clippy:
  cargo clippy --fix --allow-dirty --allow-staged

check: fmt test
    cargo check

reset: teardown setup

cleanup-containers:
  {{ TESTCONTAINERS_RUNTIME }} ps -q | xargs -I {} sh -c '{{ TESTCONTAINERS_RUNTIME }} stop {} && {{ TESTCONTAINERS_RUNTIME }} rm {}'