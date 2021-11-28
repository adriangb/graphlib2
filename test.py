from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from collections import defaultdict
from time import time
from typing import *

import igraph

from graphlib2 import TopologicalSorter


def get_branched_graph(n: int) -> Dict[int, List[int]]:
    g = igraph.Graph.Tree_Game(n, directed=True)
    res: Dict[int, List[int]] = defaultdict(list)
    for source, dest in g.get_edgelist():
        res[source].append(dest)
    return res


def f(t: TopologicalSorter[int]) -> None:
    while t.is_active():
        new = t.get_ready()
        t.done(*new)


data = get_branched_graph(1_000_000)
ts1 = TopologicalSorter(data)
ts1.prepare()
ts2 = TopologicalSorter(data)
ts2.prepare()
start = time()
with ThreadPoolExecutor(1) as exec:
    exec.submit(f, ts1)
    exec.submit(f, ts2)
print(time()-start)
