import { useEffect, useRef, useState, useCallback } from 'react';
import { createMockSSE, type SSEHandle } from '../api/mockSSE';
import type { Finding, TaskProgress } from '../types';

export interface UseSSEProgressOptions {
  taskId: string;
  enabled?: boolean;
  onFinding?: (finding: Finding) => void;
  onComplete?: () => void;
  onError?: (error: unknown) => void;
}

export interface UseSSEProgressResult {
  progress: TaskProgress | null;
  findings: Finding[];
  status: 'idle' | 'connecting' | 'running' | 'done' | 'error';
  error: unknown;
  start: () => void;
  stop: () => void;
  reset: () => void;
}

export function useSSEProgress(options: UseSSEProgressOptions): UseSSEProgressResult {
  const { taskId, enabled = true, onFinding, onComplete, onError } = options;

  const [progress, setProgress] = useState<TaskProgress | null>(null);
  const [findings, setFindings] = useState<Finding[]>([]);
  const [status, setStatus] = useState<UseSSEProgressResult['status']>('idle');
  const [error, setError] = useState<unknown>(null);

  const handleRef = useRef<SSEHandle | null>(null);
  const taskIdRef = useRef(taskId);
  const onFindingRef = useRef(onFinding);
  const onCompleteRef = useRef(onComplete);
  const onErrorRef = useRef(onError);
  const startedRef = useRef(false);

  useEffect(() => {
    taskIdRef.current = taskId;
  }, [taskId]);

  useEffect(() => {
    onFindingRef.current = onFinding;
    onCompleteRef.current = onComplete;
    onErrorRef.current = onError;
  }, [onFinding, onComplete, onError]);

  const stop = useCallback(() => {
    handleRef.current?.close();
    handleRef.current = null;
    startedRef.current = false;
  }, []);

  const start = useCallback(() => {
    if (!taskIdRef.current) return;
    if (handleRef.current) {
      handleRef.current.close();
      handleRef.current = null;
    }
    setStatus('connecting');
    setError(null);

    const handle = createMockSSE(taskIdRef.current, {
      onProgress: (data) => {
        setProgress(data);
        setStatus('running');
      },
      onFinding: (data) => {
        setFindings((prev) => [...prev, data]);
        onFindingRef.current?.(data);
      },
      onComplete: () => {
        setStatus('done');
        onCompleteRef.current?.();
      },
      onError: (err) => {
        setStatus('error');
        setError(err);
        onErrorRef.current?.(err);
      },
    });
    handleRef.current = handle;
    startedRef.current = true;
  }, []);

  const reset = useCallback(() => {
    stop();
    setProgress(null);
    setFindings([]);
    setStatus('idle');
    setError(null);
  }, [stop]);

  useEffect(() => {
    if (enabled && taskId) {
      start();
    }
    return () => {
      stop();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [enabled, taskId]);

  return {
    progress,
    findings,
    status,
    error,
    start,
    stop,
    reset,
  };
}
