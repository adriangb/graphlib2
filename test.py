import graphlib as graphlib


graph = {0: [1], 1: [2], 2: [3], 3: [4]}
a = graphlib.TopologicalSorter(graph).static_order()
print(a)
a