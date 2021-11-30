from __future__ import annotations

from typing import *

T = TypeVar("T")

_KT_co = TypeVar("_KT_co", covariant=True)
_VT_co = TypeVar("_VT_co", covariant=True)

class SupportsItems(Protocol[_KT_co, _VT_co]):
    def items(self) -> AbstractSet[Tuple[_KT_co, _VT_co]]: ...

class CycleError(ValueError):
    pass

class TopologicalSorter(Generic[T]):
    def __init__(
        self,
        graph: Optional[SupportsItems[T, Iterable[T]]],
        node_id_factory: Optional[Callable[[T], int]] = ...,
    ) -> None: ...
    def get_ready(self) -> Tuple[T, ...]: ...
    def add(self, node: T, predecessors: Tuple[T, ...]) -> None: ...
    def get_ids(self, nodes: Sequence[T]) -> Sequence[int]: ...
    def done(self, nodes: Tuple[T, ...]) -> None: ...
    def done_by_id(self, nodes: Sequence[int]) -> None: ...
    def is_active(self) -> bool: ...
    def remove_nodes(self, nodes: Iterable[T]) -> None: ...
    def remove_nodes_by_id(self, nodes: Iterable[int]) -> None: ...
    def prepare(self) -> None: ...
    def static_order(self) -> Iterable[T]: ...
    def copy(self) -> TopologicalSorter[T]: ...
