import { useState, useEffect, useRef, type ReactNode } from 'react';
import type { JSX } from 'react';
import { createStyles } from 'antd-style';
import { Resizable, type ResizeCallbackData } from 'react-resizable';
import 'react-resizable/css/styles.css';

const useStyles = createStyles(({ css }) => ({
  container: css`
    display: flex;
    width: 100%;
    flex: 1 1 auto;
    min-height: 0;
    min-width: 0;
    overflow: hidden;
  `,
  containerRow: css`
    flex-direction: column;
  `,
  pane: css`
    flex: 0 0 auto;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  `,
  resizableBox: css`
    display: flex;
    flex-direction: column;
    min-height: 0;
  `,
}));

interface ResizablePanelsProps {
  left: ReactNode;
  right: ReactNode;
  defaultRatio?: number;
  minRatio?: number;
  maxRatio?: number;
  stacked?: boolean;
}

const STORAGE_KEY = 'audit-panel-ratio';

export function ResizablePanels({
  left,
  right,
  defaultRatio = 0.55,
  minRatio = 0.25,
  maxRatio = 0.8,
  stacked = false,
}: ResizablePanelsProps): JSX.Element {
  const { styles, cx } = useStyles();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [containerSize, setContainerSize] = useState<{ w: number; h: number }>({ w: 0, h: 0 });

  const [width, setWidth] = useState<number>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const n = Number(saved);
        if (!Number.isNaN(n)) return n;
      }
    }
    return 0;
  });

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const measure = (): void => {
      const rect = el.getBoundingClientRect();
      setContainerSize({ w: rect.width, h: rect.height });
      if (width === 0 && rect.width > 0) {
        setWidth(rect.width * defaultRatio);
      }
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    window.addEventListener('resize', measure);
    return () => {
      ro.disconnect();
      window.removeEventListener('resize', measure);
    };
  }, [defaultRatio, width]);

  useEffect(() => {
    if (typeof window !== 'undefined' && width > 0) {
      localStorage.setItem(STORAGE_KEY, String(width));
    }
  }, [width]);

  const onResize = (_e: React.SyntheticEvent, data: ResizeCallbackData): void => {
    setWidth(data.size.width);
  };

  const onResizeStop = (): void => {
    if (stacked || containerSize.w === 0) return;
    const ratio = width / containerSize.w;
    if (ratio < minRatio) setWidth(containerSize.w * minRatio);
    else if (ratio > maxRatio) setWidth(containerSize.w * maxRatio);
  };

  if (stacked || containerSize.w === 0) {
    return (
      <div ref={containerRef} className={cx(styles.container, styles.containerRow)}>
        <div className={styles.pane} style={{ width: '100%', flex: '0 0 auto' }}>
          {left}
        </div>
        <div className={styles.pane} style={{ width: '100%', flex: '1 1 auto', minHeight: 0 }}>
          {right}
        </div>
      </div>
    );
  }

  const minWidth = containerSize.w * minRatio;
  const maxWidth = containerSize.w * maxRatio;

  return (
    <div ref={containerRef} className={styles.container}>
      <Resizable
        width={width}
        height={containerSize.h}
        axis="x"
        minConstraints={[minWidth, containerSize.h]}
        maxConstraints={[maxWidth, containerSize.h]}
        onResize={onResize}
        onResizeStop={onResizeStop}
      >
        <div className={styles.resizableBox} style={{ width, height: containerSize.h }}>
          {left}
        </div>
      </Resizable>
      <div className={styles.pane} style={{ flex: '1 1 auto', minWidth: 0 }}>
        {right}
      </div>
    </div>
  );
}
