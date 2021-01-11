# Attractor detection
./cabean -compositional 2 example.ispl

# OT: one pair of source and target
./cabean -compositional 2 -control OT -sin 6 -tin 1 example.ispl

# OT: all pairs of source and target attractors
./cabean -compositional 2 -control OT -allpairs example.ispl

# OT: one pair of source and target + constraints on perturbations
./cabean -compositional 2 -control OT -sin 6 -tin 1  -rmPert rmPert.txt example.ispl

# OT: all pairs of source and target attractors + constraints on perturbations and attractors
./cabean -compositional 2 -control OT -allpairs -rmID rmID.txt -rmPert rmPert.txt example.ispl


# ASI: one pair of source and target
./cabean -compositional 2 -control ASI -sin 6 -tin 1 example.ispl

# ASI: one pair of source and target + constraints on perturbations
./cabean -compositional 2 -control ASI -sin 6 -tin 1 -rmPert rmPert.txt example.ispl

# ASI: one pair of source and target + constraints
./cabean -compositional 2 -control ASI -sin 6 -tin 1 -rmID rmID.txt example.ispl

# ASI: all pairs of source and target attractors
./cabean -compositional 2 -control ASI -allpairs  example.ispl

# ASI: all pairs of source and target attractors + constraints on perturbations and attractors
./cabean -compositional 2 -control ASI -allpairs -rmID rmID.txt -rmPert rmPert.txt example.ispl

# TTC: temporary target control
./cabean -compositional 2 -control TTC -tin 1 example.ispl 