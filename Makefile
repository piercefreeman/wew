.PHONY: check build package-tests run-tests test-full clean

# Build the main project and wrap_wew
build:
	cargo build --release

build-debug:
	cargo build

# Package tests with wrap_wew (tests directory is already set up as a standalone project)
package-tests: build-debug
	@echo "Packaging tests with wrap_wew..."
	./target/debug/wrap_wew --entrypoint ./tests

# Run the packaged tests
run-tests: package-tests
	@echo "Extracting and running packaged tests..."
	tar -xf wew-tests.tar
	@if [ "$(shell uname)" = "Darwin" ]; then \
		echo "Running macOS app bundle..."; \
		./wew-tests/wew-tests.app/Contents/MacOS/wew-tests; \
	else \
		echo "Running executable..."; \
		./wew-tests/wew-tests; \
	fi
	@echo "Cleaning up extracted files..."
	rm -rf wew-tests/

# Full test pipeline
test-full: run-tests

# Clean up generated files
clean:
	rm -rf wew-tests.tar wew-tests/
	cargo clean

# Standard check (just compilation)
check:
	cargo check 