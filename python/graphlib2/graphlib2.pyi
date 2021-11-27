from __future__ import annotations

from typing import *

T = TypeVar("T", bound=Hashable)

class CycleError(ValueError):
    pass


class ReadyNode(Generic[T]):
    id: int
    value: T


class TopologicalSorter(Generic[T]):
    def __init__(self, graph: Optional[Mapping[T, Iterable[T]]]) -> None: ...
    def get_ready(self) -> List[ReadyNode[T]]: ...
    def add(self, node: T, predecessors: Tuple[T, ...]) -> None: ...
    def done(self, ids: Iterable[int]) -> None: ...
    def is_active(self) -> bool: ...
    def remove(self, nodes: Tuple[T, ...]) -> None: ...
    def prepare(self) -> None: ...
    def static_order(self) -> Iterable[Tuple[T, ...]]: ...
    def copy(self) -> TopologicalSorter[T]: ...
