# "Same remainder mod 3" on {0, 1, 2, 3, 4, 5}
M = {(0,0), (0,3), (3,0), (3,3),
     (1,1), (1,4), (4,1), (4,4),
     (2,2), (2,5), (5,2), (5,5)}
CHECK PROPERTIES M
VISUALIZE M AS DIGRAPH

# Closures can repair a relation step by step.
RT = {(1,2), (2,2), (3,3)}
S = SYMMETRIC_CLOSURE(RT)
DISPLAY S
CHECK PROPERTIES S
