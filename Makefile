.PHONY: \
	database \
	gateway \
	frontend \
	backend

all: builder database gateway frontend backend

database:
	docker buildx build --load --tag zero2prod/postgres:latest -f services/zero2prod-database/Dockerfile services/zero2prod-database

gateway:
	docker buildx build --load --tag zero2prod/gateway:latest -f services/zero2prod-gateway/Dockerfile services/zero2prod-gateway

frontend:
	docker buildx build --load --tag zero2prod/frontend:latest -f services/zero2prod-frontend/Dockerfile services/zero2prod-frontend

backend:
	docker buildx build --load --tag zero2prod/backend:latest -f Dockerfile .

builder:
	docker buildx create --name zero2prod-builder
	docker buildx use zero2prod-builder
