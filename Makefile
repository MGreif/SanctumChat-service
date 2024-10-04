
POSTGRES_CONTAINER=docker-sanctumchat-postgres-1


.PHONY: start-dependencies
start-dependencies:
		docker compose -f ./docker/docker-compose-dependencies.yml up -d --remove-orphans

start-service: start-dependencies
		cargo run

run-migration:
		diesel migration run

run-unit-tests:
		cargo test

run-service-tests: run-migration
		python3 tests/websocket/test.py