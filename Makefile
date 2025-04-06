TESTS = $(shell find examples/ -maxdepth 1 -type f -printf '%P\n')
TARGET = expected
tests:
	for f in $(TESTS); do \
		cargo run -- -f examples/$$f > examples/$(TARGET)/$$f ;\
	done

test:
	mkdir -p examples/.actual
	make TARGET=.actual tests
	for f in $(TESTS); do \
		d=$$(diff examples/expected/$$f examples/.actual/$$f) ;\
		if [[ "$$d" != "" ]]; then \
			echo -e "\n\n$$f\n $$d" ;\
		fi \
	done
