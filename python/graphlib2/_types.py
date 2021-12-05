from __future__ import annotations

import sys
from typing import AbstractSet, Hashable, Tuple, TypeVar

if sys.version_info < (3, 8):
    from typing_extensions import Protocol
else:
    from typing import Protocol

KT_co = TypeVar("KT_co", covariant=True, bound=Hashable)
VT_co = TypeVar("VT_co", covariant=True)


class SupportsItems(Protocol[KT_co, VT_co]):
    def items(self) -> AbstractSet[Tuple[KT_co, VT_co]]:
        ...
