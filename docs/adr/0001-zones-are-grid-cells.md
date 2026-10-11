# Zones are cells of an integer grid

A Layout declares a grid (`columns` by `rows`) and each Zone is a rectangle of whole cells. The original plan stored Zones as decimal fractions (`x = 0.3333, w = 0.3334`).

Decimal fractions cannot say "a third", so every thirds Layout carried a rounding fudge, and whether two neighbours met exactly depended on floating-point sums landing on the same pixel. With a grid, a Zone's edge is grid line `i` of `n`, at pixel `extent * i / n`. Neighbours share the line, so they share the pixel: no gaps and no overlaps, by construction and for every monitor width.

The Settings UI's Zone editor snaps dividers to a character-cell grid, which is this model drawn on screen.

A split the grid cannot express exactly (61.8%) needs a finer grid. That is the cost.
