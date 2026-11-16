# Weighted graphs: shortest path, minimum spanning tree, coloring.
W = GRAPH {1,2,3,4,5} {(1,2,7), (1,3,3), (3,2,2), (2,4,2), (3,4,8), (4,5,3)}
VISUALIZE W AS GRAPH

P = SHORTEST_PATH(W, 1, 4)
DISPLAY P

T = MST(W)
DISPLAY T
VISUALIZE T AS GRAPH

# Greedy vertex coloring: triangle needs 3, a path needs 2.
K = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}
DISPLAY COLOR(K)
PATH4 = GRAPH {1,2,3,4} {(1,2), (2,3), (3,4)}
DISPLAY COLOR(PATH4)
