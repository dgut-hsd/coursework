import type { JSX } from 'react';
import { AlertTriangle, AlertCircle, Info, MapPin, Clock, Lightbulb, Scale, FileText, Inbox } from 'lucide-react';
import type { Finding, Severity } from '../types';

interface IssuePanelProps {
  findings: Finding[];
  activeId: string | null;
  onSelect: (finding: Finding) => void;
}

const SEVERITY_META: Record<
  Severity,
  { color: string; bg: string; gradient: string; label: string; icon: JSX.Element }
> = {
  critical: {
    color: '#ff4d4f',
    bg: 'rgba(255, 77, 79, 0.1)',
    gradient: 'linear-gradient(135deg, #ff4d4f 0%, #ff7875 100%)',
    label: '严重',
    icon: <AlertCircle size={14} color="#fff" />,
  },
  warning: {
    color: '#fa8c16',
    bg: 'rgba(250, 140, 22, 0.1)',
    gradient: 'linear-gradient(135deg, #fa8c16 0%, #ffc53d 100%)',
    label: '警告',
    icon: <AlertTriangle size={14} color="#fff" />,
  },
  info: {
    color: '#1677ff',
    bg: 'rgba(22, 119, 255, 0.1)',
    gradient: 'linear-gradient(135deg, #1677ff 0%, #4096ff 100%)',
    label: '提示',
    icon: <Info size={14} color="#fff" />,
  },
};

export function IssuePanel({
  findings,
  activeId,
  onSelect,
}: IssuePanelProps): JSX.Element {
  if (findings.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state__icon">
          <Inbox size={36} color="#bfbfbf" strokeWidth={1.5} />
        </div>
        <div style={{ fontSize: 14, color: '#595959', fontWeight: 500 }}>暂无审核问题</div>
        <div style={{ fontSize: 12, marginTop: 4, color: '#8c8c8c' }}>
          等待 SSE 推送审核结果...
        </div>
      </div>
    );
  }

  const counts = {
    critical: findings.filter((f) => f.severity === 'critical').length,
    warning: findings.filter((f) => f.severity === 'warning').length,
    info: findings.filter((f) => f.severity === 'info').length,
  };

  return (
    <div>
      <div
        style={{
          marginBottom: 14,
          padding: '10px 12px',
          background: 'linear-gradient(135deg, #fafbff 0%, #f5f7fa 100%)',
          borderRadius: 10,
          border: '1px solid #e8ecf3',
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          flexWrap: 'wrap',
        }}
      >
        {(['critical', 'warning', 'info'] as Severity[]).map((s) => {
          const meta = SEVERITY_META[s];
          return (
            <div
              key={s}
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 6,
                padding: '3px 10px',
                borderRadius: 11,
                background: '#fff',
                border: `1px solid ${meta.color}30`,
                fontSize: 12,
                color: meta.color,
                fontWeight: 600,
              }}
            >
              <div
                style={{
                  width: 16,
                  height: 16,
                  borderRadius: 4,
                  background: meta.gradient,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
              >
                {meta.icon}
              </div>
              {meta.label} {counts[s]}
            </div>
          );
        })}
        <div
          style={{
            marginLeft: 'auto',
            color: '#8c8c8c',
            fontSize: 12,
            display: 'inline-flex',
            alignItems: 'center',
            gap: 4,
          }}
        >
          <FileText size={12} /> 共 {findings.length} 个问题
        </div>
      </div>
      {findings.map((finding) => {
        const meta = SEVERITY_META[finding.severity];
        const isActive = finding.id === activeId;
        return (
          <div
            key={finding.id}
            className={`issue-card issue-card--${finding.severity}${isActive ? ' issue-card--active' : ''}`}
            onClick={() => onSelect(finding)}
          >
            <div className="issue-card__title">
              <div
                style={{
                  width: 22,
                  height: 22,
                  borderRadius: 6,
                  background: meta.gradient,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  flexShrink: 0,
                }}
              >
                {meta.icon}
              </div>
              <span style={{ color: 'rgba(0,0,0,0.88)' }}>{finding.title}</span>
            </div>
            {finding.lawRef && (
              <div className="issue-card__law">
                <Scale size={11} style={{ marginRight: 4, verticalAlign: -1, color: '#1677ff' }} />
                <strong style={{ color: '#1677ff' }}>法规依据:</strong>
                {finding.lawRef}
              </div>
            )}
            <div className="issue-card__desc">{finding.description}</div>
            {finding.suggestion && (
              <div className="issue-card__suggestion">
                <Lightbulb size={11} style={{ marginRight: 4, verticalAlign: -1, color: '#52c41a' }} />
                <strong style={{ color: '#389e0d' }}>修改建议:</strong>
                {finding.suggestion}
              </div>
            )}
            <div className="issue-card__footer">
              <span>
                <MapPin size={12} style={{ marginRight: 4, verticalAlign: -1 }} />
                第 {finding.page} 页
              </span>
              <span>
                <Clock size={12} style={{ marginRight: 4, verticalAlign: -1 }} />
                {new Date(finding.createdAt).toLocaleTimeString('zh-CN')}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
