#!/usr/bin/env bash

curl -v --header "Content-Type: application/json" \
  --request POST \
  --data '{"username":"xyz","password":"xyz"}' \
  http://localhost:8081/api/v1/login