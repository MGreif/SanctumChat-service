
POSTGRES_CONTAINER=sanctumchat-postgres


.PHONY: start-dependencies
start-dependencies:
		docker start $(POSTGRES_CONTAINER)

start-service: start-dependencies
		cargo run

start-python-tests: run-migration
		python3 tests/websocket/test.py

run-migration:
	diesel migration run

