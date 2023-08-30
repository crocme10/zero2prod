#!/usr/bin/env bash

# Use --insecure because of self-signed certificates.

curl -v \
  --insecure \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{"username":"xyz","password":"xyz"}' \
  https://localhost:8443/api/v1/login
