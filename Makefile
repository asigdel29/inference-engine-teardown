.PHONY: check demo bench
check:
	python3 -m unittest -v
demo:
	python3 -m unittest -v test_bench.RunnerTests.test_cli
bench:
	python3 -m inference_teardown.bench $(ARGS)
