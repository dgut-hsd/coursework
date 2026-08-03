import { useNavigate } from 'react-router-dom';
import { Table, Tag, Button, Space, App as AntApp, Card, Row, Col, Typography, Empty, Tooltip } from 'antd';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  FileText,
  Play,
  CheckCircle2,
  AlertTriangle,
  Clock,
  Plus,
  FileSearch,
  Inbox,
} from 'lucide-react';
import { queryKeys } from '../../app/queryClient';
import { mockApi } from '../bidAudit/api/mockApi';
import { useState } from 'react';
import type { JSX } from 'react';
import { Upload, Modal } from 'antd';
import type { UploadProps } from 'antd';
import { InboxOutlined } from '@ant-design/icons';
import type { AuditTask } from '../bidAudit/types';

const STATUS_MAP: Record<AuditTask['status'], { color: string; label: string; bg: string; border: string; dotColor: string }> = {
  pending: { color: '#8c8c8c', label: '待审核', bg: '#fafafa', border: '#d9d9d9', dotColor: '#bfbfbf' },
  running: { color: '#1677ff', label: '审核中', bg: '#e6f4ff', border: '#91caff', dotColor: '#1677ff' },
  done: { color: '#389e0d', label: '已完成', bg: '#f6ffed', border: '#b7eb8f', dotColor: '#52c41a' },
  failed: { color: '#cf1322', label: '失败', bg: '#fff1f0', border: '#ffa39e', dotColor: '#ff4d4f' },
};

type StatusFilter = 'all' | 'pending' | 'running' | 'done' | 'failed';

const STAT_CARDS: ReadonlyArray<{
  key: StatusFilter;
  label: string;
  gradient: string;
  shadow: string;
  valueKey: keyof typeof STAT_DEFAULTS;
  Icon: typeof FileText;
}> = [
  { key: 'all', label: '任务总数', gradient: 'linear-gradient(135deg, #1677ff 0%, #4096ff 100%)', shadow: 'rgba(22, 119, 255, 0.5)', valueKey: 'total', Icon: FileText },
  { key: 'done', label: '已完成', gradient: 'linear-gradient(135deg, #52c41a 0%, #73d13d 100%)', shadow: 'rgba(82, 196, 26, 0.5)', valueKey: 'done', Icon: CheckCircle2 },
  { key: 'pending', label: '审核中 / 待审', gradient: 'linear-gradient(135deg, #fa8c16 0%, #ffa940 100%)', shadow: 'rgba(250, 140, 22, 0.45)', valueKey: 'inProgress', Icon: Clock },
  { key: 'failed', label: '发现问题总数', gradient: 'linear-gradient(135deg, #f5222d 0%, #ff4d4f 100%)', shadow: 'rgba(245, 34, 45, 0.45)', valueKey: 'totalFindings', Icon: AlertTriangle },
];

const STAT_DEFAULTS = {
  total: 0,
  done: 0,
  inProgress: 0,
  totalFindings: 0,
};
const formatSize = (size: number): string => {
  if (size > 1024 * 1024) return `${(size / 1024 / 1024).toFixed(2)} MB`;
  if (size > 1024) return `${(size / 1024).toFixed(0)} KB`;
  return `${size} B`;
};

export function DashboardPage(): JSX.Element {
  const navigate = useNavigate();
  const { message } = AntApp.useApp();
  const qc = useQueryClient();
  const [uploadOpen, setUploadOpen] = useState(false);
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all');

  const tasksQuery = useQuery({
    queryKey: queryKeys.tasks,
    queryFn: () => mockApi.listTasks(),
  });

  const createMutation = useMutation({
    mutationFn: (file: { name: string; size: number }) => mockApi.createTask(file),
    onSuccess: (task) => {
      qc.invalidateQueries({ queryKey: queryKeys.tasks });
      setUploadOpen(false);
      message.success(`任务已创建: ${task.fileName}`);
      navigate(`/projects/p-1/audit/${task.id}`);
    },
  });

  const openReportMutation = useMutation({
    mutationFn: (taskId: string) => mockApi.getReportIdByTaskId(taskId),
    onSuccess: (reportId, taskId) => {
      navigate(`/projects/p-1/report/${reportId}`);
      void taskId;
    },
    onError: () => {
      message.error('打开报告失败');
    },
  });

  const tasks = tasksQuery.data ?? [];

  const stats = {
    total: tasks.length,
    done: tasks.filter((t) => t.status === 'done').length,
    running: tasks.filter((t) => t.status === 'running').length,
    pending: tasks.filter((t) => t.status === 'pending').length,
    failed: tasks.filter((t) => t.status === 'failed').length,
    totalFindings: tasks.reduce((sum, t) => sum + t.findingsCount, 0),
  };

  const doneRatio = stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0;

  const filteredTasks = (() => {
    if (statusFilter === 'all') return tasks;
    if (statusFilter === 'pending') {
      return tasks.filter((t) => t.status === 'pending' || t.status === 'running');
    }
    return tasks.filter((t) => t.status === statusFilter);
  })();

  const uploadProps: UploadProps = {
    multiple: false,
    accept: '.pdf',
    beforeUpload: (file) => {
      createMutation.mutate({ name: file.name, size: file.size });
      return false;
    },
    showUploadList: false,
  };

  return (
    <div className="dashboard">
      <div
        style={{
          background: 'linear-gradient(120deg, #ffffff 0%, #f0f5ff 100%)',
          borderRadius: 16,
          padding: '28px 32px',
          marginBottom: 24,
          border: '1px solid #e6ebf5',
          boxShadow: '0 2px 8px rgba(15, 35, 95, 0.04)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: 16,
        }}
      >
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 6 }}>
            <FileSearch size={20} color="#1677ff" />
            <Typography.Text style={{ fontSize: 13, color: '#1677ff', fontWeight: 500 }}>
              审核工作台
            </Typography.Text>
          </div>
          <h1
            style={{
              fontSize: 26,
              fontWeight: 700,
              margin: 0,
              color: 'rgba(0,0,0,0.88)',
              letterSpacing: 0.2,
            }}
          >
            欢迎回来,审核员 👋
          </h1>
          <Typography.Text style={{ color: 'rgba(0,0,0,0.55)', fontSize: 14, marginTop: 6, display: 'block' }}>
            今日共有 <span style={{ color: '#1677ff', fontWeight: 600 }}>{stats.pending}</span> 个待审核任务,
            已完成 <span style={{ color: '#52c41a', fontWeight: 600 }}>{stats.done}</span> 个
          </Typography.Text>
        </div>
        <Space>
          <Button
            size="large"
            icon={<Plus size={18} />}
            onClick={() => setUploadOpen(true)}
            style={{
              borderRadius: 10,
              background: 'linear-gradient(135deg, #1677ff 0%, #4096ff 100%)',
              border: 'none',
              boxShadow: '0 6px 20px -4px rgba(22, 119, 255, 0.45)',
              height: 44,
              padding: '0 22px',
              fontSize: 14,
              fontWeight: 500,
            }}
            type="primary"
          >
            上传招标文件
          </Button>
        </Space>
      </div>

      <Row gutter={[20, 20]} style={{ marginBottom: 24 }}>
        {STAT_CARDS.map((card) => {
          const isActive = statusFilter === card.key;
          const value =
            card.valueKey === 'total'
              ? stats.total
              : card.valueKey === 'done'
                ? stats.done
                : card.valueKey === 'inProgress'
                  ? stats.running + stats.pending
                  : stats.totalFindings;
          const hint =
            card.valueKey === 'total'
              ? `点击查看全部任务`
              : card.valueKey === 'done'
                ? `完成率 ${doneRatio}% · 点击查看`
                : card.valueKey === 'inProgress'
                  ? `进行 ${stats.running} · 等待 ${stats.pending}`
                  : stats.failed > 0
                    ? `失败 ${stats.failed} · 点击查看`
                    : `点击查看有问题的任务`;
          const Icon = card.Icon;
          return (
            <Col xs={24} sm={12} md={6} key={card.key}>
              <div
                onClick={() => setStatusFilter((prev) => (prev === card.key ? 'all' : card.key))}
                style={{
                  position: 'relative',
                  height: 140,
                  borderRadius: 14,
                  background: card.gradient,
                  boxShadow: isActive
                    ? `0 16px 32px -10px ${card.shadow}, 0 0 0 3px rgba(255,255,255,0.9) inset`
                    : `0 10px 24px -8px ${card.shadow}`,
                  cursor: 'pointer',
                  overflow: 'hidden',
                  transition: 'transform 0.25s ease, box-shadow 0.25s ease',
                  transform: isActive ? 'translateY(-2px)' : 'none',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-4px)';
                  e.currentTarget.style.boxShadow = `0 16px 32px -10px ${card.shadow}`;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = isActive ? 'translateY(-2px)' : 'none';
                  e.currentTarget.style.boxShadow = isActive
                    ? `0 16px 32px -10px ${card.shadow}, 0 0 0 3px rgba(255,255,255,0.9) inset`
                    : `0 10px 24px -8px ${card.shadow}`;
                }}
              >
                <div
                  style={{
                    position: 'absolute',
                    top: -30,
                    right: -30,
                    width: 120,
                    height: 120,
                    borderRadius: '50%',
                    background: 'radial-gradient(circle, rgba(255,255,255,0.18) 0%, transparent 70%)',
                    pointerEvents: 'none',
                  }}
                />
                <div
                  style={{
                    position: 'relative',
                    padding: 20,
                    height: '100%',
                    display: 'flex',
                    flexDirection: 'column',
                    justifyContent: 'space-between',
                  }}
                >
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                    }}
                  >
                    <div
                      style={{
                        width: 38,
                        height: 38,
                        borderRadius: 10,
                        background: 'rgba(255,255,255,0.2)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                      }}
                    >
                      <Icon size={20} color="#fff" />
                    </div>
                    {isActive && (
                      <div
                        style={{
                          background: 'rgba(255,255,255,0.95)',
                          color: 'rgba(0,0,0,0.75)',
                          fontSize: 11,
                          padding: '2px 8px',
                          borderRadius: 10,
                          fontWeight: 500,
                        }}
                      >
                        已筛选
                      </div>
                    )}
                  </div>
                  <div>
                    <div
                      style={{
                        color: 'rgba(255,255,255,0.85)',
                        fontSize: 13,
                        marginBottom: 4,
                      }}
                    >
                      {card.label}
                    </div>
                    <div
                      style={{
                        color: '#fff',
                        fontSize: 30,
                        fontWeight: 700,
                        lineHeight: 1.1,
                      }}
                    >
                      {value}
                    </div>
                    <div
                      style={{
                        color: 'rgba(255,255,255,0.78)',
                        fontSize: 11,
                        marginTop: 6,
                      }}
                    >
                      {hint}
                    </div>
                  </div>
                </div>
              </div>
            </Col>
          );
        })}
      </Row>

      <Card
        bordered={false}
        style={{
          borderRadius: 14,
          boxShadow: '0 2px 8px rgba(15, 35, 95, 0.04)',
        }}
        styles={{ body: { padding: 0 } }}
        title={
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '4px 0' }}>
            <div
              style={{
                width: 4,
                height: 18,
                background: 'linear-gradient(180deg, #1677ff, #722ed1)',
                borderRadius: 2,
              }}
            />
            <span style={{ fontSize: 16, fontWeight: 600 }}>任务列表</span>
            <Tag color="blue" style={{ marginLeft: 4 }}>
              {statusFilter === 'all' ? `共 ${tasks.length} 项` : `已筛选 ${filteredTasks.length} / ${tasks.length} 项`}
            </Tag>
            {statusFilter !== 'all' && (
              <Button
                type="link"
                size="small"
                onClick={() => setStatusFilter('all')}
                style={{ padding: 0, marginLeft: 8 }}
              >
                清除筛选
              </Button>
            )}
          </div>
        }
      >
        {tasksQuery.isLoading ? (
          <div style={{ padding: 60, textAlign: 'center' }}>
            <Empty description="加载中..." />
          </div>
        ) : (
          <Table<AuditTask>
            rowKey="id"
            dataSource={filteredTasks}
            pagination={false}
            rowClassName={() => 'task-row'}
            columns={[
              {
                title: '文件名',
                dataIndex: 'fileName',
                key: 'fileName',
                render: (name: string, record) => (
                  <Space size={10}>
                    <div
                      style={{
                        width: 32,
                        height: 32,
                        borderRadius: 8,
                        background:
                          record.status === 'done'
                            ? 'linear-gradient(135deg, #f6ffed 0%, #d9f7be 100%)'
                            : record.status === 'running'
                              ? 'linear-gradient(135deg, #e6f4ff 0%, #bae0ff 100%)'
                              : record.status === 'failed'
                                ? 'linear-gradient(135deg, #fff1f0 0%, #ffccc7 100%)'
                                : 'linear-gradient(135deg, #fafafa 0%, #f0f0f0 100%)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                      }}
                    >
                      <FileText
                        size={16}
                        color={
                          record.status === 'done'
                            ? '#52c41a'
                            : record.status === 'running'
                              ? '#1677ff'
                              : record.status === 'failed'
                                ? '#ff4d4f'
                                : '#8c8c8c'
                        }
                      />
                    </div>
                    <div>
                      <div style={{ fontWeight: 500, color: 'rgba(0,0,0,0.88)' }}>{name}</div>
                      <div style={{ fontSize: 11, color: 'rgba(0,0,0,0.45)' }}>ID: {record.id}</div>
                    </div>
                  </Space>
                ),
              },
              {
                title: '大小',
                dataIndex: 'fileSize',
                key: 'fileSize',
                width: 90,
                render: (size: number) => (
                  <span style={{ color: 'rgba(0,0,0,0.65)' }}>{formatSize(size)}</span>
                ),
              },
              {
                title: '状态',
                dataIndex: 'status',
                key: 'status',
                width: 100,
                render: (status: AuditTask['status']) => {
                  const meta = STATUS_MAP[status] ?? { color: '#8c8c8c', label: '未知', bg: '#fafafa', border: '#d9d9d9', dotColor: '#bfbfbf' };
                  return (
                    <div
                      style={{
                        display: 'inline-flex',
                        alignItems: 'center',
                        gap: 6,
                        padding: '3px 10px',
                        borderRadius: 20,
                        background: meta.bg,
                        border: `1px solid ${meta.border}`,
                        color: meta.color,
                        fontSize: 12,
                        fontWeight: 500,
                      }}
                    >
                      <span
                        style={{
                          width: 6,
                          height: 6,
                          borderRadius: '50%',
                          background: meta.dotColor,
                          display: 'inline-block',
                        }}
                      />
                      {meta.label}
                    </div>
                  );
                },
              },
              {
                title: '问题数',
                dataIndex: 'findingsCount',
                key: 'findingsCount',
                width: 100,
                render: (count: number) => {
                  if (count === 0) {
                    return <span style={{ color: 'rgba(0,0,0,0.25)' }}>—</span>;
                  }
                  const color = count >= 5 ? '#ff4d4f' : count >= 3 ? '#fa8c16' : '#1677ff';
                  const bg = count >= 5 ? '#fff1f0' : count >= 3 ? '#fff7e6' : '#e6f4ff';
                  return (
                    <span
                      style={{
                        display: 'inline-block',
                        minWidth: 30,
                        textAlign: 'center',
                        padding: '2px 10px',
                        borderRadius: 12,
                        background: bg,
                        color,
                        fontSize: 12,
                        fontWeight: 600,
                      }}
                    >
                      {count}
                    </span>
                  );
                },
              },
              {
                title: '创建时间',
                dataIndex: 'createdAt',
                key: 'createdAt',
                width: 170,
                render: (t: string) => (
                  <Tooltip title={t}>
                    <span style={{ color: 'rgba(0,0,0,0.65)' }}>{new Date(t).toLocaleString('zh-CN')}</span>
                  </Tooltip>
                ),
              },
              {
                title: '操作',
                key: 'action',
                width: 200,
                render: (_, record) => {
                  if (record.status === 'done') {
                    return (
                      <Space size={4}>
                        <Button
                          type="primary"
                          size="small"
                          onClick={() => openReportMutation.mutate(record.id)}
                          loading={openReportMutation.isPending && openReportMutation.variables === record.id}
                          style={{ borderRadius: 6 }}
                        >
                          查看报告
                        </Button>
                      </Space>
                    );
                  }
                  if (record.status === 'running') {
                    return (
                      <Button
                        type="primary"
                        size="small"
                        icon={<Play size={12} />}
                        onClick={() => navigate(`/projects/p-1/audit/${record.id}`)}
                        style={{ borderRadius: 6 }}
                      >
                        查看进度
                      </Button>
                    );
                  }
                  if (record.status === 'pending') {
                    return (
                      <Button
                        type="primary"
                        size="small"
                        onClick={() => navigate(`/projects/p-1/audit/${record.id}`)}
                        style={{ borderRadius: 6 }}
                      >
                        开始审核
                      </Button>
                    );
                  }
                  return (
                    <Typography.Text type="secondary" style={{ fontSize: 12 }}>
                      审核未通过
                    </Typography.Text>
                  );
                },
              },
            ]}
          />
        )}
      </Card>

      <Modal
        title={
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <Inbox size={18} color="#1677ff" />
            <span>上传招标文件</span>
          </div>
        }
        open={uploadOpen}
        onCancel={() => setUploadOpen(false)}
        footer={null}
        destroyOnHidden
        styles={{ body: { paddingTop: 12 } }}
      >
        <Upload.Dragger
          {...uploadProps}
          disabled={createMutation.isPending}
          style={{
            background: 'linear-gradient(180deg, #f0f7ff 0%, #ffffff 100%)',
            border: '1.5px dashed #91caff',
            borderRadius: 10,
          }}
        >
          <p className="ant-upload-drag-icon">
            <InboxOutlined style={{ color: '#1677ff', fontSize: 48 }} />
          </p>
          <p className="ant-upload-text" style={{ fontSize: 15, fontWeight: 500 }}>
            点击或拖拽 PDF 文件到此区域上传
          </p>
          <p className="ant-upload-hint" style={{ color: 'rgba(0,0,0,0.5)' }}>
            支持单个 PDF 文件,大小不超过 50MB
          </p>
        </Upload.Dragger>
      </Modal>
    </div>
  );
}
