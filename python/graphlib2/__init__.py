from __future__ import annotations

from typing import Generic, Hashable, Iterable, Optional, TypeVar

from graphlib2._types import SupportsItems
from graphlib2.graphlib2 import CycleError
from graphlib2.graphlib2 import TopologicalSorter as _TopologicalSorter

T = TypeVar("T", bound=Hashable)


class TopologicalSorter(Generic[T]):
    __slots__ = ("_ts",)

    def __init__(self, graph: Optional[SupportsItems[T, Iterable[T]]] = None) -> None:
        self._ts: _TopologicalSorter[T] = _TopologicalSorter()
        if graph:
            for node, children in graph.items():
                self._ts.add(node, tuple(children))

    def add(self, node: T, *predecessors: T) -> None:
        self._ts.add(node, predecessors)

    def get_ready(self) -> Iterable[T]:
        return self._ts.get_ready()

    def done(self, *nodes: T) -> None:
        self._ts.done(nodes)

    def is_active(self) -> bool:
        return self._ts.is_active()

    def prepare(self) -> None:
        self._ts.prepare()

    def static_order(self) -> Iterable[T]:
        self.prepare()
        while self.is_active():
            node_group = self.get_ready()
            yield from node_group
            self.done(*node_group)

    def copy(self) -> TopologicalSorter[T]:
        new = object.__new__(TopologicalSorter)
        new._ts = self._ts.copy()
        return new


__all__ = ("TopologicalSorter", "CycleError")
