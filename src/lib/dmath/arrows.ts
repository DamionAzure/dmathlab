// Shared arrow geometry for the digraph and graph views.
//
// Rules: edges are straight unless they belong to a reciprocal pair (a→b and
// b→a), in which case each bows to its own right so both stay visible.
// Self-loops are a real circle sitting outside the node. Every arrow tip lands
// just outside the target node (arrowheads are drawn by <ArrowMarker/>, which
// extends `tipExtra` px past the end of the path, so the path stops short).

export interface Pt {
  x: number;
  y: number;
}

export interface EdgeGeom {
  /** SVG path data. */
  d: string;
  /** Visual midpoint, for labels (weight pills). */
  mid: Pt;
}

/** Arrowhead geometry. Must match <ArrowMarker/>: viewBox 10, refX 6. */
export const ARROW_REF_X = 6;
export const ARROW_PATH = "M 0 0.5 L 10 5 L 0 9.5 L 2.6 5 Z";
/** Px the arrowhead extends beyond the path end for a marker of `size` px. */
export const tipExtra = (size: number) => ((10 - ARROW_REF_X) / 10) * size;

export const BEND = 0.2;

const f = (n: number) => n.toFixed(2);

function unit(x: number, y: number): Pt {
  const len = Math.hypot(x, y);
  return len < 1e-9 ? { x: 0, y: -1 } : { x: x / len, y: y / len };
}

export interface EdgeOpts {
  /** Signed bow as a fraction of edge length (0 = straight; + bows to the right of travel). */
  bend?: number;
  /** Gap between path/tip and the node circle. */
  gap?: number;
  /** Pull the path end back by this many px (to make room for an arrowhead). */
  endExtra?: number;
}

/** Edge between two node centres, trimmed to the node circles. Null if the nodes touch. */
export function edgeGeometry(p1: Pt, p2: Pt, r1: number, r2: number, o: EdgeOpts = {}): EdgeGeom | null {
  const { bend = 0, gap = 2, endExtra = 0 } = o;
  const dx = p2.x - p1.x;
  const dy = p2.y - p1.y;
  const len = Math.hypot(dx, dy);
  if (len < r1 + r2 + 2 * gap + endExtra + 4) return null;

  const mid0 = { x: (p1.x + p2.x) / 2, y: (p1.y + p2.y) / 2 };
  // Control point (or the midpoint, for a straight edge).
  const c = {
    x: mid0.x + (-dy / len) * len * bend,
    y: mid0.y + (dx / len) * len * bend,
  };
  const t1 = unit(c.x - p1.x, c.y - p1.y);
  const t2 = unit(c.x - p2.x, c.y - p2.y);
  const s = { x: p1.x + t1.x * (r1 + gap), y: p1.y + t1.y * (r1 + gap) };
  const e = { x: p2.x + t2.x * (r2 + gap + endExtra), y: p2.y + t2.y * (r2 + gap + endExtra) };

  if (bend === 0) {
    return { d: `M ${f(s.x)} ${f(s.y)} L ${f(e.x)} ${f(e.y)}`, mid: { x: (s.x + e.x) / 2, y: (s.y + e.y) / 2 } };
  }
  return {
    d: `M ${f(s.x)} ${f(s.y)} Q ${f(c.x)} ${f(c.y)} ${f(e.x)} ${f(e.y)}`,
    // Quadratic Bézier at t = 0.5.
    mid: { x: 0.25 * s.x + 0.5 * c.x + 0.25 * e.x, y: 0.25 * s.y + 0.5 * c.y + 0.25 * e.y },
  };
}

const TAU = Math.PI * 2;
const mod = (a: number) => ((a % TAU) + TAU) % TAU;

/**
 * Self-loop: the larger arc of a circle of radius `loopR` whose centre lies
 * just outside the node (in direction `outward`). It leaves the node circle
 * on one side and re-enters on the other, ending in an arrowhead when
 * `endExtra` > 0.
 */
export function loopGeometry(
  p: Pt,
  outward: Pt,
  nodeR: number,
  loopR = 11,
  gap = 2,
  endExtra = 0,
): EdgeGeom {
  const out = unit(outward.x, outward.y);
  const perp = { x: -out.y, y: out.x };
  const d0 = nodeR + loopR * 0.55; // centre distance from the node centre
  const r0 = nodeR + gap;
  const c = { x: p.x + out.x * d0, y: p.y + out.y * d0 };

  // Intersection of the node circle (r0) with the loop circle (loopR).
  const a = (d0 * d0 + r0 * r0 - loopR * loopR) / (2 * d0);
  const h = Math.sqrt(Math.max(r0 * r0 - a * a, 0));
  const A = { x: p.x + out.x * a - perp.x * h, y: p.y + out.y * a - perp.y * h };
  const B = { x: p.x + out.x * a + perp.x * h, y: p.y + out.y * a + perp.y * h };

  const ang = (q: Pt) => Math.atan2(q.y - c.y, q.x - c.x);
  const thA = ang(A);
  const thB = ang(B);
  const thO = Math.atan2(out.y, out.x);
  // Travel A → B through the outermost point of the loop.
  const sweep = mod(thO - thA) < mod(thB - thA) ? 1 : 0;

  // Stop the arc short of B so the arrowhead tip lands on it.
  const back = (sweep ? -1 : 1) * (endExtra / loopR);
  const thE = thB + back;
  const E = { x: c.x + loopR * Math.cos(thE), y: c.y + loopR * Math.sin(thE) };

  return {
    d: `M ${f(A.x)} ${f(A.y)} A ${loopR} ${loopR} 0 1 ${sweep} ${f(E.x)} ${f(E.y)}`,
    // Label anchor sits just beyond the loop so pills never cover the arc.
    mid: { x: c.x + out.x * (loopR + 11), y: c.y + out.y * (loopR + 11) },
  };
}
