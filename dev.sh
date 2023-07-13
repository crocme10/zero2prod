#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

(trap 'kill 0' SIGINT; \
 bash -c 'cd services/zero2prod-fakeemail; cargo run -- --addr 0.0.0.0 --port 8083' & \
 bash -c 'cd services/zero2prod-frontend; CARGO_TARGET_DIR=../dist trunk serve --address 0.0.0.0 --port 8082' & \
 bash -c 'cd services/zero2prod-backend; cargo watch -- cargo run -- -c ../../config -s database.require_ssl=false -s application.port=8081 run')
