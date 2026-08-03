import type { JSX } from 'react';
import { Progress, Space, Typography } from 'antd';
import { CheckCircle2, Loader2, AlertCircle, Upload, FileSearch, BarChart3, FileText } from 'lucide-react';
import type { TaskProgress } from '../types';

interface ProgressSSEProps {
  progress: TaskProgress | null;
  status: 'idle' | 'connecting' | 'running' | 'done' | 'error';
}

const STAGE_META: Record<
  TaskProgress['stage'],
  { color: string; label: string; bg: string; border: string; icon: JSX.Element }
> = {
  uploading: {
    color: '#1677ff',
    label: '上传',
    bg: 'linear-gradient(135deg, #e6f4ff 0%, #bae0ff 100%)',
    border: '#91caff',
    icon: <Upload size={11} />,
  },
  parsing: {
    color: '#13c2c2',
    label: '解析',
    bg: 'linear-gradient(135deg, #e6fffb 0%, #87e8de 100%)',
    border: '#5cdbd3',
    icon: <FileSearch size={11} />,
  },
  analyzing: {
    color: '#722ed1',
    label: '分析',
    bg: 'linear-gradient(135deg, #f9f0ff 0%, #d3adf7 100%)',
    border: '#b37feb',
    icon: <BarChart3 size={11} />,
  },
  reporting: {
    color: '#fa8c16',
    label: '报告',
    bg: 'linear-gradient(135deg, #fff7e6 0%, #ffd591 100%)',
    border: '#ffc069',
    icon: <FileText size={11} />,
  },
  done: {
    color: '#52c41a',
    label: '完成',
    bg: 'linear-gradient(135deg, #f6ffed 0%, #b7eb8f 100%)',
    border: '#95de64',
    icon: <CheckCircle2 size={11} />,
  },
};

const STAGE_ORDER: TaskProgress['stage'][] = ['uploading', 'parsing', 'analyzing', 'reporting', 'done'];

export function ProgressSSE({ progress, status }: ProgressSSEProps): JSX.Element {
  const percent = progress?.percent ?? 0;
  const stage = progress?.stage ?? 'uploading';
  const message = progress?.message ?? '等待开始...';

  const isError = status === 'error';
  const isDone = status === 'done';
  const isRunning = status === 'running' || status === 'connecting';
  const currentIndex = STAGE_ORDER.indexOf(stage);

  return (
    <div className="progress-bar">
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: 8,
          background: isError
            ? 'linear-gradient(135deg, #fff1f0 0%, #ffccc7 100%)'
            : isDone
              ? 'linear-gradient(135deg, #f6ffed 0%, #b7eb8f 100%)'
              : 'linear-gradient(135deg, #e6f4ff 0%, #91caff 100%)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          flexShrink: 0,
        }}
      >
        {isError ? (
          <AlertCircle size={14} color="#ff4d4f" />
        ) : isDone ? (
          <CheckCircle2 size={14} color="#52c41a" />
        ) : (
          <Loader2 size={14} color="#1677ff" className="spin" />
        )}
      </div>
      <Typography.Text
        style={{
          fontSize: 13,
          color: isError ? '#ff4d4f' : isDone ? '#52c41a' : '#1677ff',
          fontWeight: 500,
          minWidth: 200,
        }}
      >
        {message}
      </Typography.Text>
      <Progress
        percent={percent}
        size="small"
        status={isError ? 'exception' : isDone ? 'success' : 'active'}
        strokeColor={
          isError
            ? '#ff4d4f'
            : isDone
              ? '#52c41a'
              : { from: '#1677ff', to: '#722ed1' }
        }
        style={{ flex: 1, minWidth: 200, maxWidth: 320 }}
      />
      <Space size={4} wrap>
        {STAGE_ORDER.map((s, i) => {
          const meta = STAGE_META[s];
          const isCurrent = s === stage && isRunning;
          const isPast = isDone || i < currentIndex;
          return (
            <div
              key={s}
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 4,
                padding: '2px 8px',
                borderRadius: 11,
                background: isCurrent ? meta.bg : isPast ? '#fafafa' : 'transparent',
                border: `1px solid ${isCurrent ? meta.border : isPast ? '#e8e8e8' : '#f0f0f0'}`,
                color: isCurrent ? meta.color : isPast ? '#8c8c8c' : '#bfbfbf',
                fontSize: 11,
                fontWeight: 500,
                transition: 'all 0.3s',
              }}
            >
              {meta.icon}
              {meta.label}
            </div>
          );
        })}
      </Space>
    </div>
  );
}
