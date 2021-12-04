from __future__ import annotations

from typing import Any, Dict, Generic, Iterable, Optional, TypeVar

from graphlib2._types import SupportsItems
from graphlib2.graphlib2 import CycleError
from graphlib2.graphlib2 import TopologicalSorter as _TopologicalSorter

T = TypeVar("T")


class TopologicalSorter(Generic[T]):
    __slots__ = ("_ts",)

    _ts: _TopologicalSorter[T]

    def __init__(
        self,
        graph: Optional[SupportsItems[T, Iterable[T]]] = None,
        *,
        _topological_sorter: Optional[_TopologicalSorter[T]] = None,
    ) -> None:
        self._ts = _topological_sorter or _TopologicalSorter(graph)

    def add(self, node: T, *predecessors: T) -> None:
        self._ts.add(node, predecessors)

    def get_ready(self) -> Iterable[T]:
        return self._ts.get_ready()

    def done(self, *nodes: T) -> None:
        self._ts.done(nodes)

    def is_active(self) -> bool:
        return self._ts.is_active()

    def __bool__(self) -> bool:
        return self.is_active()

    def prepare(self) -> None:
        self._ts.prepare()

    def static_order(self) -> Iterable[T]:
        return self._ts.static_order()

    def copy(self: TopologicalSorter[T]) -> TopologicalSorter[T]:
        return TopologicalSorter(_topological_sorter=self._ts.copy())

    def __copy__(self: TopologicalSorter[T]) -> TopologicalSorter[T]:
        return self.copy()

    def __deepcopy__(
        self: TopologicalSorter[T], memo: Dict[Any, Any]
    ) -> TopologicalSorter[T]:
        new = self.copy()
        memo[self] = new
        return new


__all__ = ("TopologicalSorter", "CycleError")
