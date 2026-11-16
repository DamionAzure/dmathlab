import { ARROW_PATH, ARROW_REF_X } from "@/lib/dmath/arrows";

/**
 * Notched arrowhead, sized in px (independent of the line's stroke width).
 * Its tip extends `tipExtra(size)` px past the end of the path.
 */
export function ArrowMarker({ id, color, size = 11 }: { id: string; color: string; size?: number }) {
  return (
    <marker
      id={id}
      viewBox="0 0 10 10"
      refX={ARROW_REF_X}
      refY={5}
      markerUnits="userSpaceOnUse"
      markerWidth={size}
      markerHeight={size}
      orient="auto"
    >
      <path d={ARROW_PATH} fill={color} stroke={color} strokeWidth={0.4} strokeLinejoin="round" />
    </marker>
  );
}
