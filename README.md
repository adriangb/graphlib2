# graphlib2

This is a Rust port of Python's stdlib [graphlib].
It passes all of the standard libraries tests and is a drop in replacement.
This also happens to be Python 3.7 compatible, so it can be used as a backport.
Since usage is exactly the same as the standard libraries, please refer to their documentation for usage details.

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

1. Additional APIs for removing nodes from the graph (`TopologicalSorter.remove_nodes`) and copying a prepared `TopologicalSorter` (`TopologicalSorter.copy`).
1. A couple factors (~5x) faster for large highly branched graphs.
1. Unlocks the GIL during certain operations, which can considerably speed up multithreaded workloads.

## Development

1. Clone the repo.
1. Run `make init`
1. Run `make test`

[di]: https://github.com/adriangb/di
[graphlib]: https://docs.python.org/3/library/graphlib.html
