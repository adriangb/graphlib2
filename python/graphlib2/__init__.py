from __future__ import annotations

from typing import Generic, Iterable, Optional, Tuple, TypeVar

from graphlib2._types import SupportsItems
from graphlib2.graphlib2 import CycleError
from graphlib2.graphlib2 import TopologicalSorter as _TopologicalSorter

_T = TypeVar("_T")


class TopologicalSorter(Generic[_T]):
    __slots__ = ("_ts", "_node_id_factory")

    def __init__(
        self,
        graph: Optional[SupportsItems[_T, Iterable[_T]]] = None,
    ) -> None:
        self._ts: _TopologicalSorter[_T] = _TopologicalSorter(graph)

    def add(self, node: _T, *predecessors: _T) -> None:
        self._ts.add(node, predecessors)

    def get_ready(self) -> Tuple[_T, ...]:
        return self._ts.get_ready()

    def done(self, *nodes: _T) -> None:
        self._ts.done(nodes)

    def is_active(self) -> bool:
        return self._ts.is_active()

    def prepare(self) -> None:
        self._ts.prepare()

    def static_order(self) -> Iterable[_T]:
        return self._ts.static_order()

    def copy(self: TopologicalSorter[_T]) -> TopologicalSorter[_T]:
        new: TopologicalSorter[_T] = object.__new__(TopologicalSorter)
        new._ts = self._ts.copy()
        return new


__all__ = ("TopologicalSorter", "CycleError")
