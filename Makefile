PHONY: init build test

.init:
	rm -rf venv
	python -m venv venv
	./venv/bin/pip install -r requirements-dev.txt
	touch .init

.clean:
	rm -rf .init

init: .clean .init

build:
	. ./venv/bin/activate && maturin develop --release --strip

test: build
	./venv/bin/python test_graphlib.py
