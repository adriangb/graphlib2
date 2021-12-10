# graphlib2

![CI](https://github.com/adriangb/graphlib2/actions/workflows/python.yaml/badge.svg)

This is a Rust port of Python's stdlib [graphlib].
It passes all of the standard libraries tests and is a drop in replacement.
This also happens to be Python 3.7 compatible, so it can be used as a backport.
Since usage is exactly the same as the standard libraries, please refer to their documentation for usage details.

See this project on [GitHub](https://github.com/adriangb/graphlib2).

## Example

```python
from graphlib2 import TopologicalSorter

graph = {0: [1], 1: [2]}  # 0 depends on 1, 1 depends on 2
ts = TopologicalSorter(graph)
ts.prepare()
while ts.is_active():
    ready_nodes = ts.get_ready()
    ts.done(*ready_nodes)  # all at a time or one by one
```

## Motivation

This was primarily written for [di] and for me to learn Rust.
In other words: please vet the code yourself before using this.

## Differences with the stdlib implementation

1. Added `TopologicalSorter.copy()` which copies a prepared or unprepared graph so that it can be executed multiple times.
1. Pretty solid performance improvements (see [benchmarks]).
1. Misc improvements, like working generics without postponed evaluateion (`ToplologicalSorter[int]` works at runtime).

## Performance

The implementation was designed for the specific use case of adding all nodes, calling `prepare()` then copying and executing in a loop:

```python
from graphlib2 import TopologicalSorter

graph = {0: [1], 1: [2]}
ts = TopologicalSorter(graph)
ts.prepare()
while True:  # hot loop
    t = ts.copy()
    while t.is_active():
        ready_nodes = t.get_ready()
        t.done(*ready_nodes)
```

This means that the focus is on the performance of `TopologicalSorter.get_ready()` and `TopologicalSorter.done()`, and only minimal effort was put into other methods (`prepare()`, `add()` and `get_static_order()`), although these are still quite performant.

## Contributing

1. Clone the repo.
1. Run `make init`
1. Run `make test`
1. Make your changes
1. Push and open a pull request
1. Wait for CI to run.

If your pull request gets approved and merged, it will automatically be relased to PyPi (every commit to `main` is released).

[di]: https://github.com/adriangb/di
[graphlib]: https://docs.python.org/3/library/graphlib.html
[benchmarks]: https://nbviewer.org/github/adriangb/graphlib2/blob/main/bench.ipynb
