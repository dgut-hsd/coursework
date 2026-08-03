import { useState, useEffect } from 'react';
import type { JSX } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Button, Space, App as AntApp, Dropdown, Spin, Breadcrumb, Typography } from 'antd';
import { ArrowLeft, FileDown, FileText, BarChart3, ListChecks, RefreshCw, Home, FolderOpen } from 'lucide-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '../../app/queryClient';
import { mockApi } from './api/mockApi';
import { useSSEProgress } from './hooks/useSSEProgress';
import { ResizablePanels } from './components/ResizablePanels';
import { PdfViewer } from './components/PdfViewer';
import { IssuePanel } from './components/IssuePanel';
import { StatsChart } from './components/StatsChart';
import { ProgressSSE } from './components/ProgressSSE';
import { exportAsPDF, exportAsDOCX, exportAsMarkdown, type ExportFormat } from './components/exportReport';
import type { Finding } from './types';

const STATUS_LABELS: Record<string, { text: string; color: string; bg: string; border: string }> = {
  pending: { text: '待审核', color: '#8c8c8c', bg: '#fafafa', border: '#d9d9d9' },
  running: { text: '审核中', color: '#1677ff', bg: '#e6f4ff', border: '#91caff' },
  done: { text: '已完成', color: '#389e0d', bg: '#f6ffed', border: '#b7eb8f' },
  failed: { text: '失败', color: '#cf1322', bg: '#fff1f0', border: '#ffa39e' },
};

const formatSize = (size: number): string => {
  if (size > 1024 * 1024) return `${(size / 1024 / 1024).toFixed(2)} MB`;
  if (size > 1024) return `${(size / 1024).toFixed(0)} KB`;
  return `${size} B`;
};

export function AuditPage(): JSX.Element {
  const { id, tid } = useParams<{ id: string; tid: string }>();
  const navigate = useNavigate();
  const { message } = AntApp.useApp();
  const qc = useQueryClient();
  const [activeFindingId, setActiveFindingId] = useState<string | null>(null);
  const [fileUrl, setFileUrl] = useState<string | null>(null);

  const taskQuery = useQuery({
    queryKey: queryKeys.task(tid ?? ''),
    queryFn: () => mockApi.getTask(tid ?? ''),
    enabled: Boolean(tid),
  });

  const findingsQuery = useQuery({
    queryKey: queryKeys.findings(tid ?? ''),
    queryFn: () => mockApi.listFindings(tid ?? ''),
    enabled: Boolean(tid),
    staleTime: 60_000,
  });

  const sse = useSSEProgress({
    taskId: tid ?? '',
    enabled: Boolean(tid),
    onFinding: () => {
      // finding 通过 setFindings 增量合入
    },
    onComplete: () => {
      qc.invalidateQueries({ queryKey: queryKeys.findings(tid ?? '') });
      qc.invalidateQueries({ queryKey: queryKeys.tasks });
      qc.invalidateQueries({ queryKey: queryKeys.task(tid ?? '') });
    },
  });

  const allFindings: Finding[] = [
    ...(findingsQuery.data ?? []),
    ...sse.findings.filter(
      (f) => !(findingsQuery.data ?? []).some((existing) => existing.id === f.id),
    ),
  ];

  const generateReportMutation = useMutation({
    mutationFn: () => mockApi.createReport(tid ?? ''),
    onSuccess: (report) => {
      qc.setQueryData(queryKeys.report(report.id), report);
      qc.setQueryData(queryKeys.reportByTask(tid ?? ''), report);
      message.success('报告已生成');
      navigate(`/projects/${id}/report/${report.id}`);
    },
  });

  const onUpload = (file: File): void => {
    const url = URL.createObjectURL(file);
    setFileUrl(url);
    message.success(`已加载文件: ${file.name}`);
  };

  useEffect(() => {
    return () => {
      if (fileUrl) URL.revokeObjectURL(fileUrl);
    };
  }, [fileUrl]);

  if (!tid) {
    return <div style={{ padding: 24 }}>任务 ID 缺失</div>;
  }

  const task = taskQuery.data;
  const fileName = task?.fileName ?? '招标文件.pdf';
  const isAuditing = sse.status === 'running' || sse.status === 'connecting';
  const taskStatus = task?.status ?? 'pending';
  const statusMeta: { text: string; color: string; bg: string; border: string } =
    STATUS_LABELS[taskStatus] ?? {
      text: '未知',
      color: '#8c8c8c',
      bg: '#fafafa',
      border: '#d9d9d9',
    };
  const safeStatusMeta = statusMeta;

  const onSelectFinding = (finding: Finding): void => {
    setActiveFindingId(finding.id);
  };

  const onExport = async (format: ExportFormat): Promise<void> => {
    if (allFindings.length === 0) {
      message.warning('暂无问题可导出,请先完成审核');
      return;
    }
    try {
      const report = await mockApi.createReport(tid);
      const baseName = `审核报告_${task?.fileName.replace(/\.pdf$/i, '') ?? tid}`;
      if (format === 'pdf') {
        await exportAsPDF(report, { fileName: `${baseName}.pdf`, title: report.title });
      } else if (format === 'docx') {
        await exportAsDOCX(report, { fileName: `${baseName}.docx`, title: report.title });
      } else {
        exportAsMarkdown(report, { fileName: `${baseName}.md`, title: report.title });
      }
      message.success(`已导出为 ${format.toUpperCase()}`);
    } catch (err) {
      const text = err instanceof Error ? err.message : '导出失败';
      message.error(text);
    }
  };

  const leftPane = (
    <>
      <div
        style={{
          padding: '12px 18px',
          borderBottom: '1px solid #f0f0f0',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          background: 'linear-gradient(180deg, #fafbff 0%, #ffffff 100%)',
          flexShrink: 0,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <div
            style={{
              width: 28,
              height: 28,
              borderRadius: 7,
              background: 'linear-gradient(135deg, #1677ff 0%, #722ed1 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 4px 10px -2px rgba(22, 119, 255, 0.4)',
            }}
          >
            <FileText size={14} color="#fff" />
          </div>
          <Typography.Text style={{ fontWeight: 600, fontSize: 14 }}>PDF 预览</Typography.Text>
        </div>
        <Space size={6}>
          <Button
            size="small"
            icon={<RefreshCw size={12} />}
            onClick={() => {
              sse.reset();
              sse.start();
            }}
            disabled={isAuditing}
            style={{ borderRadius: 6 }}
          >
            重新审核
          </Button>
          <input
            id="upload-input"
            type="file"
            accept="application/pdf"
            style={{ display: 'none' }}
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) onUpload(file);
              e.target.value = '';
            }}
          />
          <Button
            size="small"
            onClick={() => document.getElementById('upload-input')?.click()}
            style={{ borderRadius: 6 }}
          >
            上传 PDF
          </Button>
        </Space>
      </div>
      <ProgressSSE progress={sse.progress} status={sse.status} />
      <div className="audit-pane__body audit-pane__body--padded">
        {taskQuery.isLoading ? (
          <div className="empty-state">
            <Spin />
          </div>
        ) : (
          <PdfViewer
            fileUrl={fileUrl}
            fileName={fileName}
            findings={allFindings}
            activeFindingId={activeFindingId}
            onSelect={onSelectFinding}
            pageWidth={720}
          />
        )}
      </div>
    </>
  );

  const rightPane = (
    <>
      <div
        style={{
          padding: '12px 18px',
          borderBottom: '1px solid #f0f0f0',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          background: 'linear-gradient(180deg, #fafbff 0%, #ffffff 100%)',
          flexShrink: 0,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <div
            style={{
              width: 28,
              height: 28,
              borderRadius: 7,
              background: 'linear-gradient(135deg, #fa8c16 0%, #ffc53d 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 4px 10px -2px rgba(250, 140, 22, 0.4)',
            }}
          >
            <BarChart3 size={14} color="#fff" />
          </div>
          <Typography.Text style={{ fontWeight: 600, fontSize: 14 }}>统计概览</Typography.Text>
        </div>
      </div>
      <div className="audit-pane__body" style={{ background: '#fafbff' }}>
        <StatsChart findings={allFindings} />
      </div>
      <div
        style={{
          padding: '12px 18px',
          borderTop: '1px solid #f0f0f0',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          background: 'linear-gradient(180deg, #fafbff 0%, #ffffff 100%)',
          flexShrink: 0,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <div
            style={{
              width: 28,
              height: 28,
              borderRadius: 7,
              background: 'linear-gradient(135deg, #f5222d 0%, #fa8c16 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 4px 10px -2px rgba(245, 34, 45, 0.4)',
            }}
          >
            <ListChecks size={14} color="#fff" />
          </div>
          <Typography.Text style={{ fontWeight: 600, fontSize: 14 }}>问题列表</Typography.Text>
        </div>
        <Button
          type="primary"
          size="small"
          onClick={() => generateReportMutation.mutate()}
          loading={generateReportMutation.isPending}
          style={{
            borderRadius: 6,
            background: 'linear-gradient(135deg, #1677ff 0%, #4096ff 100%)',
            border: 'none',
            boxShadow: '0 2px 8px rgba(22, 119, 255, 0.3)',
          }}
        >
          生成报告
        </Button>
      </div>
      <div className="audit-pane__body">
        <IssuePanel
          findings={allFindings}
          activeId={activeFindingId}
          onSelect={onSelectFinding}
        />
      </div>
    </>
  );

  return (
    <div className="audit-layout" style={{ background: '#f5f7fa' }}>
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          width: '100%',
        }}
      >
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '14px 28px',
            background: 'linear-gradient(180deg, #ffffff 0%, #fafbff 100%)',
            borderBottom: '1px solid #f0f0f0',
            boxShadow: '0 2px 8px rgba(15, 35, 95, 0.04)',
            flexWrap: 'wrap',
            gap: 12,
          }}
        >
          <Space size={14} align="center" wrap>
            <Button
              icon={<ArrowLeft size={16} />}
              onClick={() => navigate('/dashboard')}
              style={{ borderRadius: 8 }}
              className="audit-back-btn"
            >
              返回
            </Button>
            <div
              style={{
                width: 1,
                height: 20,
                background: 'linear-gradient(180deg, transparent, #d9d9d9, transparent)',
              }}
            />
            <Breadcrumb
              items={[
                {
                  href: '/dashboard',
                  title: (
                    <Space size={4}>
                      <Home size={13} /> 任务列表
                    </Space>
                  ),
                  onClick: (e) => {
                    e.preventDefault();
                    navigate('/dashboard');
                  },
                },
                {
                  href: '#',
                  title: (
                    <Space size={4}>
                      <FolderOpen size={13} />
                      {id ?? '项目'}
                    </Space>
                  ),
                },
                {
                  title: (
                    <Space size={6}>
                      <FileText size={13} color="#1677ff" />
                      <span style={{ color: 'rgba(0,0,0,0.88)', fontWeight: 500 }}>{fileName}</span>
                    </Space>
                  ),
                },
              ]}
            />
            <div
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 6,
                padding: '3px 10px',
                borderRadius: 12,
                background: safeStatusMeta.bg,
                border: `1px solid ${safeStatusMeta.border}`,
                color: safeStatusMeta.color,
                fontSize: 12,
                fontWeight: 500,
              }}
            >
              <span
                className={taskStatus === 'running' ? 'spin' : ''}
                style={{
                  width: 6,
                  height: 6,
                  borderRadius: '50%',
                  background: safeStatusMeta.color,
                  display: 'inline-block',
                }}
              />
              {safeStatusMeta.text}
            </div>
            {task && (
              <Typography.Text type="secondary" style={{ fontSize: 12 }}>
                ID: {task.id} · {formatSize(task.fileSize)}
              </Typography.Text>
            )}
            {allFindings.length > 0 && (
              <div
                style={{
                  display: 'inline-flex',
                  alignItems: 'center',
                  gap: 4,
                  padding: '3px 10px',
                  borderRadius: 12,
                  background: allFindings.some((f) => f.severity === 'critical')
                    ? 'linear-gradient(135deg, #fff1f0 0%, #ffccc7 100%)'
                    : 'linear-gradient(135deg, #e6f4ff 0%, #bae0ff 100%)',
                  border: `1px solid ${allFindings.some((f) => f.severity === 'critical') ? '#ffa39e' : '#91caff'}`,
                  color: allFindings.some((f) => f.severity === 'critical') ? '#cf1322' : '#1677ff',
                  fontSize: 12,
                  fontWeight: 500,
                }}
              >
                共 {allFindings.length} 个问题
              </div>
            )}
          </Space>
          <Space>
            <Dropdown
              menu={{
                items: [
                  { key: 'pdf', label: '导出为 PDF', onClick: () => void onExport('pdf') },
                  { key: 'docx', label: '导出为 DOCX', onClick: () => void onExport('docx') },
                  { key: 'md', label: '导出为 Markdown', onClick: () => void onExport('md') },
                ],
              }}
            >
              <Button
                type="primary"
                icon={<FileDown size={14} />}
                style={{
                  borderRadius: 8,
                  background: 'linear-gradient(135deg, #1677ff 0%, #4096ff 100%)',
                  border: 'none',
                  boxShadow: '0 4px 14px -2px rgba(22, 119, 255, 0.4)',
                }}
              >
                导出报告
              </Button>
            </Dropdown>
          </Space>
        </div>
        <ResizablePanels
          left={<div className="audit-pane" style={{ height: '100%' }}>{leftPane}</div>}
          right={<div className="audit-pane" style={{ height: '100%' }}>{rightPane}</div>}
          defaultRatio={0.55}
          minRatio={0.3}
          maxRatio={0.75}
        />
      </div>
    </div>
  );
}
