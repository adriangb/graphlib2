from __future__ import annotations

from typing import Generic, Hashable, Iterable, List, Optional, Tuple, TypeVar

from graphlib2._types import SupportsItems
from graphlib2.graphlib2 import CycleError
from graphlib2.graphlib2 import TopologicalSorter as _TopologicalSorter

T = TypeVar("T", bound=Hashable)


class TopologicalSorter(Generic[T]):
    __slots__ = ("_ts", "_static_order")

    def __init__(self, graph: Optional[SupportsItems[T, Iterable[T]]] = None) -> None:
        self._ts: _TopologicalSorter[T] = _TopologicalSorter()
        self._static_order: Optional[Tuple[T, ...]] = None
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
        if self._static_order is not None:
            return self._static_order
        self.prepare()
        so: List[T] = []
        while self.is_active():
            node_group = self.get_ready()
            for node in node_group:
                so.append(node)
                yield node
            self._ts.done(tuple(node_group))
        self._static_order = tuple(so)

    def copy(self) -> TopologicalSorter[T]:
        new = object.__new__(TopologicalSorter)  # type: ignore  # generic is unknown
        new._ts = self._ts.copy()
        new._static_order = self._static_order
        return new  # type: ignore  # generic is unknown


__all__ = ("TopologicalSorter", "CycleError")
