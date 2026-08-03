import type { ReactNode } from 'react';
import type { Bbox, Severity } from '../types';

interface BboxOverlayProps {
  bboxes: Bbox[];
  severities: Record<number, Severity>;
  activeIndex: number | null;
  onSelect: (index: number) => void;
  pageWidth: number;
  pageHeight: number;
  pdfNativeWidth: number;
  pdfNativeHeight: number;
}

export function BboxOverlay({
  bboxes,
  severities,
  activeIndex,
  onSelect,
  pageWidth,
  pageHeight,
  pdfNativeWidth,
  pdfNativeHeight,
}: BboxOverlayProps): ReactNode {
  if (pageWidth === 0 || pageHeight === 0) return null;
  if (pdfNativeWidth === 0 || pdfNativeHeight === 0) return null;

  const scaleX = pageWidth / pdfNativeWidth;
  const scaleY = pageHeight / pdfNativeHeight;

  return (
    <div
      className="bbox-overlay"
      style={{
        width: pageWidth,
        height: pageHeight,
      }}
    >
      {bboxes.map((bbox, idx) => {
        const sev = severities[idx] ?? 'info';
        const cls = `bbox-overlay__rect bbox-overlay__rect--${sev}${
          activeIndex === idx ? ' bbox-overlay__rect--active' : ''
        }`;
        return (
          <div
            key={`${bbox.page}-${idx}`}
            className={cls}
            style={{
              left: bbox.x * scaleX,
              top: bbox.y * scaleY,
              width: bbox.width * scaleX,
              height: bbox.height * scaleY,
            }}
            onClick={() => onSelect(idx)}
            title={`#${idx + 1} (${sev})`}
          />
        );
      })}
    </div>
  );
}
