from __future__ import annotations

from typing import Generic, Hashable, Iterable, Optional, Tuple, TypeVar, Mapping
from .graphlib2 import TopologicalSorter as _TopologicalSorter, CycleError


T = TypeVar("T", bound=Hashable)



class TopologicalSorter(Generic[T]):
    __slots__ = ("_ts")

    def __init__(self, graph: Optional[Mapping[T, Iterable[T]]] = None) -> None:
        """"""
        self._ts = _TopologicalSorter(graph)
    
    def add(self, node: T, *predecessors: T) -> None:
        self._ts.add(node, predecessors)

    def get_ready(self) -> Tuple[T, ...]:
        """"""
        return self._ts.get_ready()

    def done(self, *nodes: T) -> None:
        """"""
        self._ts.done(nodes)

    def is_active(self) -> bool:
        """"""
        return self._ts.is_active()

    def remove(self, *nodes: T) -> None:
        """"""
        self._ts.remove(nodes)

    def prepare(self) -> None:
        """"""
        self._ts.prepare()

    def static_order(self) -> Iterable[Tuple[T, ...]]:
        """"""
        return self._ts.static_order()



__all__ = ("TopologicalSorter", "CycleError")
