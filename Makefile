.PHONY: database backend frontend gateway

all: database backend frontend gateway

database:
	docker buildx build --load --tag zero2prod/database:latest  -f services/zero2prod-database/Dockerfile services/zero2prod-database

backend:
	docker buildx build --load --tag zero2prod/backend:latest -f services/zero2prod-backend/docker/Dockerfile .

frontend:
	docker buildx build --load --tag zero2prod/frontend:latest -f services/zero2prod-frontend/Dockerfile services/zero2prod-frontend

gateway:
	docker buildx build --load --tag zero2prod/gateway:latest -f services/zero2prod-gateway/Dockerfile services/zero2prod-gateway

builder:
	docker buildx create --name zero2prod-builder
	docker buildx use zero2prod-builder
