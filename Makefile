
POSTGRES_CONTAINER=some-postgres

.PHONY: start-dependencies
start-dependencies:
		docker start $(POSTGRES_CONTAINER)

start-service: start-dependencies
		cargo run

.PHONY: start-python-tests
start-python-tests:
		python3 tests/websocket/test.py

