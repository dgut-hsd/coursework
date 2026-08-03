import type { JSX } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Button, Space, App as AntApp, Spin, Breadcrumb, Typography, Card } from 'antd';
import { ArrowLeft, FileDown, Printer, Home, FolderOpen, FileText, Clock, AlertTriangle, AlertCircle, Info, CheckCircle2 } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import { Dropdown } from 'antd';
import { queryKeys } from '../../app/queryClient';
import { mockApi } from './api/mockApi';
import { exportAsPDF, exportAsDOCX, exportAsMarkdown, type ExportFormat } from './components/exportReport';

const STAT_META = [
  { key: 'critical', label: '严重问题', color: '#ff4d4f', bg: 'linear-gradient(135deg, #ff4d4f 0%, #ff7875 100%)', shadow: 'rgba(255, 77, 79, 0.4)' },
  { key: 'warning', label: '警告问题', color: '#fa8c16', bg: 'linear-gradient(135deg, #fa8c16 0%, #ffc53d 100%)', shadow: 'rgba(250, 140, 22, 0.4)' },
  { key: 'info', label: '提示问题', color: '#1677ff', bg: 'linear-gradient(135deg, #1677ff 0%, #4096ff 100%)', shadow: 'rgba(22, 119, 255, 0.4)' },
] as const;

export function ReportPage(): JSX.Element {
  const { id, rid } = useParams<{ id: string; rid: string }>();
  const navigate = useNavigate();
  const { message } = AntApp.useApp();

  const reportQuery = useQuery({
    queryKey: rid ? queryKeys.report(rid) : ['report', 'none'],
    queryFn: () => mockApi.getReport(rid ?? ''),
    enabled: Boolean(rid),
    staleTime: 5 * 60_000,
  });

  const onExport = async (format: ExportFormat): Promise<void> => {
    if (!reportQuery.data) return;
    try {
      const fileName = `审核报告_${reportQuery.data.taskId}`;
      if (format === 'pdf') {
        await exportAsPDF(reportQuery.data, { fileName: `${fileName}.pdf`, title: reportQuery.data.title });
      } else if (format === 'docx') {
        await exportAsDOCX(reportQuery.data, { fileName: `${fileName}.docx`, title: reportQuery.data.title });
      } else {
        exportAsMarkdown(reportQuery.data, { fileName: `${fileName}.md`, title: reportQuery.data.title });
      }
      message.success(`已导出为 ${format.toUpperCase()}`);
    } catch (err) {
      const text = err instanceof Error ? err.message : '导出失败';
      message.error(text);
    }
  };

  const onPrint = (): void => {
    window.print();
  };

  if (reportQuery.isLoading) {
    return (
      <div className="empty-state" style={{ minHeight: 480 }}>
        <Spin size="large" />
        <div style={{ marginTop: 16, color: '#8c8c8c' }}>报告加载中...</div>
      </div>
    );
  }

  if (reportQuery.isError || !reportQuery.data) {
    return (
      <div className="report-page">
        <Button icon={<ArrowLeft size={16} />} onClick={() => navigate(-1)}>
          返回
        </Button>
        <div className="empty-state" style={{ marginTop: 24 }}>
          报告不存在或加载失败
        </div>
      </div>
    );
  }

  const report = reportQuery.data;
  const total = report.stats.critical + report.stats.warning + report.stats.info;

  return (
    <div
      style={{
        padding: '24px 32px 40px',
        maxWidth: 1080,
        margin: '0 auto',
        minHeight: 'calc(100vh - 64px)',
      }}
    >
      <div style={{ marginBottom: 20 }}>
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
              title: (
                <Space size={4}>
                  <FolderOpen size={13} /> {id ?? '项目'}
                </Space>
              ),
            },
            {
              title: (
                <Space size={4}>
                  <FileText size={13} color="#1677ff" /> 审核报告
                </Space>
              ),
            },
          ]}
        />
      </div>

      <Card
        bordered={false}
        style={{
          borderRadius: 16,
          marginBottom: 20,
          background: 'linear-gradient(120deg, #ffffff 0%, #f0f5ff 50%, #f9f0ff 100%)',
          boxShadow: '0 4px 24px -8px rgba(15, 35, 95, 0.1)',
          overflow: 'hidden',
          position: 'relative',
        }}
        styles={{ body: { padding: '28px 32px' } }}
      >
        <div
          style={{
            position: 'absolute',
            top: -40,
            right: -40,
            width: 200,
            height: 200,
            borderRadius: '50%',
            background: 'radial-gradient(circle, rgba(22, 119, 255, 0.08) 0%, transparent 70%)',
            pointerEvents: 'none',
          }}
        />
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: 16,
            marginBottom: 22,
            position: 'relative',
          }}
        >
          <Space size={14} align="center">
            <Button
              icon={<ArrowLeft size={16} />}
              onClick={() => navigate(-1)}
              style={{ borderRadius: 6 }}
            >
              返回
            </Button>
            <div
              style={{
                width: 48,
                height: 48,
                borderRadius: 12,
                background: 'linear-gradient(135deg, #1677ff 0%, #722ed1 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                boxShadow: '0 6px 20px -4px rgba(22, 119, 255, 0.5)',
              }}
            >
              <FileText size={22} color="#fff" />
            </div>
            <div>
              <Typography.Text
                style={{
                  display: 'block',
                  fontSize: 12,
                  color: '#1677ff',
                  fontWeight: 500,
                  marginBottom: 2,
                }}
              >
                审核报告
              </Typography.Text>
              <h1
                style={{
                  margin: 0,
                  fontSize: 22,
                  fontWeight: 700,
                  color: 'rgba(0,0,0,0.88)',
                  letterSpacing: 0.2,
                }}
              >
                {report.title}
              </h1>
            </div>
          </Space>
          <Space>
            <Button
              icon={<Printer size={14} />}
              onClick={onPrint}
              style={{ borderRadius: 6 }}
            >
              打印
            </Button>
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

        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 12,
            flexWrap: 'wrap',
            position: 'relative',
          }}
        >
          {STAT_META.map((m) => {
            const value = report.stats[m.key];
            const Icon = m.key === 'critical' ? AlertCircle : m.key === 'warning' ? AlertTriangle : Info;
            return (
              <div
                key={m.key}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 12,
                  padding: '12px 18px',
                  borderRadius: 12,
                  background: '#fff',
                  border: '1px solid rgba(0,0,0,0.06)',
                  boxShadow: '0 2px 8px rgba(15, 35, 95, 0.04)',
                  minWidth: 140,
                }}
              >
                <div
                  style={{
                    width: 36,
                    height: 36,
                    borderRadius: 9,
                    background: m.bg,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    boxShadow: `0 4px 12px ${m.shadow}`,
                  }}
                >
                  <Icon size={18} color="#fff" />
                </div>
                <div>
                  <div style={{ fontSize: 11, color: 'rgba(0,0,0,0.5)' }}>{m.label}</div>
                  <div style={{ fontSize: 20, fontWeight: 700, color: m.color, lineHeight: 1.2 }}>
                    {value}
                  </div>
                </div>
              </div>
            );
          })}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              padding: '12px 18px',
              borderRadius: 12,
              background: '#fff',
              border: '1px solid rgba(0,0,0,0.06)',
              boxShadow: '0 2px 8px rgba(15, 35, 95, 0.04)',
              minWidth: 140,
            }}
          >
            <div
              style={{
                width: 36,
                height: 36,
                borderRadius: 9,
                background: 'linear-gradient(135deg, #13c2c2 0%, #36cfc9 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                boxShadow: '0 4px 12px rgba(19, 194, 194, 0.4)',
              }}
            >
              <CheckCircle2 size={18} color="#fff" />
            </div>
            <div>
              <div style={{ fontSize: 11, color: 'rgba(0,0,0,0.5)' }}>问题总数</div>
              <div style={{ fontSize: 20, fontWeight: 700, color: '#13c2c2', lineHeight: 1.2 }}>
                {total}
              </div>
            </div>
          </div>
          <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 6, color: 'rgba(0,0,0,0.5)', fontSize: 12 }}>
            <Clock size={13} />
            生成于 {new Date(report.createdAt).toLocaleString('zh-CN')}
          </div>
        </div>
      </Card>

      <Card
        bordered={false}
        style={{
          borderRadius: 16,
          boxShadow: '0 4px 24px -8px rgba(15, 35, 95, 0.08)',
        }}
        styles={{ body: { padding: '32px 40px' } }}
      >
        <div className="report-page__markdown">
          <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeRaw]}>
            {report.markdown}
          </ReactMarkdown>
        </div>
      </Card>
    </div>
  );
}
