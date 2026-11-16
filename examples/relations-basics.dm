# Relations — a relation is a set of ordered pairs.
R = {(1,1), (1,2), (2,2), (2,3), (3,3), (3,4), (4,4)}
CHECK PROPERTIES R
VISUALIZE R AS DIGRAPH

# Transitive closure (Warshall) completes the order.
TC = TRANSITIVE_CLOSURE(R)
DISPLAY TC
VISUALIZE TC AS MATRIX
CHECK PROPERTIES TC

# TC is now a partial order: draw its Hasse diagram.
VISUALIZE TC AS HASSE
