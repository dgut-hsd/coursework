import { useState } from 'react';
import type { JSX } from 'react';
import { Form, Input, Button, Typography, App as AntApp } from 'antd';
import { useNavigate } from 'react-router-dom';
import { LogIn, Mail, Lock, Sparkles, ShieldCheck, Zap, FileSearch } from 'lucide-react';
import { useMutation } from '@tanstack/react-query';
import { useAppDispatch } from '../../app/store';
import { setCredentials } from './authSlice';
import { mockApi } from '../bidAudit/api/mockApi';

interface LoginFormValues {
  email: string;
  password: string;
}

export function LoginPage(): JSX.Element {
  const [loading, setLoading] = useState(false);
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const { message } = AntApp.useApp();

  const loginMutation = useMutation({
    mutationFn: (values: LoginFormValues) => mockApi.login(values.email, values.password),
    onSuccess: (data) => {
      dispatch(
        setCredentials({
          token: data.token,
          user: {
            id: data.userId,
            name: data.name,
            email: data.email,
          },
        }),
      );
      localStorage.setItem('auth_email', data.email);
      localStorage.setItem('auth_name', data.name);
      setLoading(false);
      message.success('登录成功');
      navigate('/dashboard');
    },
    onError: (err: unknown) => {
      setLoading(false);
      const text = err instanceof Error ? err.message : '登录失败';
      message.error(text);
    },
  });

  const onFinish = (values: LoginFormValues): void => {
    setLoading(true);
    loginMutation.mutate(values);
  };

  return (
    <div
      style={{
        minHeight: 'calc(100vh - 64px)',
        display: 'flex',
        background: '#f0f2f5',
        position: 'relative',
        overflow: 'hidden',
      }}
    >
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background:
            'linear-gradient(135deg, #0a1834 0%, #143055 40%, #1d3a72 70%, #722ed1 100%)',
          clipPath: 'polygon(0 0, 55% 0, 45% 100%, 0 100%)',
        }}
      />
      <div
        style={{
          position: 'absolute',
          top: '20%',
          left: '8%',
          width: 220,
          height: 220,
          borderRadius: '50%',
          background: 'radial-gradient(circle, rgba(22, 119, 255, 0.3) 0%, transparent 70%)',
          filter: 'blur(20px)',
        }}
      />
      <div
        style={{
          position: 'absolute',
          bottom: '15%',
          left: '25%',
          width: 180,
          height: 180,
          borderRadius: '50%',
          background: 'radial-gradient(circle, rgba(114, 46, 209, 0.3) 0%, transparent 70%)',
          filter: 'blur(24px)',
        }}
      />

      <div
        style={{
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'center',
          padding: '40px 60px',
          position: 'relative',
          zIndex: 1,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginBottom: 28 }}>
          <div
            style={{
              width: 52,
              height: 52,
              borderRadius: 14,
              background: 'linear-gradient(135deg, #1677ff 0%, #722ed1 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 8px 24px -4px rgba(22, 119, 255, 0.5)',
            }}
          >
            <ShieldCheck size={28} color="#fff" strokeWidth={2.4} />
          </div>
          <div>
            <div style={{ color: '#fff', fontSize: 22, fontWeight: 700, letterSpacing: 0.3 }}>
              AI 标书审核工作台
            </div>
            <div style={{ color: 'rgba(255,255,255,0.6)', fontSize: 13, marginTop: 2 }}>
              Bid Audit Workbench · v1.0
            </div>
          </div>
        </div>

        <h1
          style={{
            color: '#fff',
            fontSize: 38,
            fontWeight: 700,
            lineHeight: 1.3,
            margin: 0,
            marginBottom: 16,
            letterSpacing: 0.5,
          }}
        >
          智能审核 · 高效合规
          <br />
          让每一份招标文件
          <br />
          都能<span style={{ color: '#ffd54f' }}>安心发布</span>
        </h1>
        <p
          style={{
            color: 'rgba(255,255,255,0.75)',
            fontSize: 15,
            lineHeight: 1.7,
            margin: 0,
            marginBottom: 36,
            maxWidth: 480,
          }}
        >
          基于大模型的招标文件智能审核系统,自动识别排斥性条款、倾向性参数、违规设置等合规风险,
          生成结构化审核报告。
        </p>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
          {[
            { icon: Zap, title: '实时进度', desc: '基于 SSE 的审核进度实时推送' },
            { icon: FileSearch, title: '智能识别', desc: '覆盖 50+ 法规条款的语义分析' },
            { icon: Sparkles, title: '结构化报告', desc: '一键导出 PDF / DOCX / Markdown' },
          ].map((f) => {
            const Icon = f.icon;
            return (
              <div
                key={f.title}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 14,
                  padding: '12px 18px',
                  borderRadius: 12,
                  background: 'rgba(255, 255, 255, 0.08)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  backdropFilter: 'blur(8px)',
                  maxWidth: 420,
                }}
              >
                <div
                  style={{
                    width: 36,
                    height: 36,
                    borderRadius: 9,
                    background: 'linear-gradient(135deg, #1677ff 0%, #722ed1 100%)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    boxShadow: '0 4px 12px rgba(22, 119, 255, 0.3)',
                  }}
                >
                  <Icon size={17} color="#fff" />
                </div>
                <div>
                  <div style={{ color: '#fff', fontSize: 14, fontWeight: 600 }}>{f.title}</div>
                  <div style={{ color: 'rgba(255,255,255,0.65)', fontSize: 12, marginTop: 2 }}>
                    {f.desc}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      <div
        style={{
          flex: '0 0 460px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: 40,
          position: 'relative',
          zIndex: 1,
        }}
      >
        <div
          style={{
            width: '100%',
            maxWidth: 400,
            background: '#fff',
            borderRadius: 16,
            padding: '36px 32px',
            boxShadow: '0 20px 60px -16px rgba(15, 35, 95, 0.4)',
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              marginBottom: 6,
            }}
          >
            <div
              style={{
                width: 4,
                height: 18,
                background: 'linear-gradient(180deg, #1677ff, #722ed1)',
                borderRadius: 2,
              }}
            />
            <Typography.Text style={{ color: '#1677ff', fontSize: 13, fontWeight: 500 }}>
              欢迎登录
            </Typography.Text>
          </div>
          <Typography.Title level={3} style={{ marginTop: 4, marginBottom: 8 }}>
            审核员登录
          </Typography.Title>
          <Typography.Paragraph
            type="secondary"
            style={{ marginBottom: 24, fontSize: 13 }}
          >
            请使用审核员账号登录工作台
          </Typography.Paragraph>
          <Form<LoginFormValues>
            name="login"
            layout="vertical"
            initialValues={{ email: 'auditor@dgut.edu.cn', password: 'demo' }}
            onFinish={onFinish}
            autoComplete="off"
            requiredMark={false}
          >
            <Form.Item
              label={<span style={{ fontSize: 13, fontWeight: 500 }}>邮箱</span>}
              name="email"
              rules={[{ required: true, type: 'email', message: '请输入有效邮箱' }]}
              style={{ marginBottom: 18 }}
            >
              <Input
                placeholder="请输入邮箱"
                size="large"
                prefix={<Mail size={15} color="#8c8c8c" style={{ marginRight: 6 }} />}
                style={{ borderRadius: 8 }}
              />
            </Form.Item>
            <Form.Item
              label={<span style={{ fontSize: 13, fontWeight: 500 }}>密码</span>}
              name="password"
              rules={[{ required: true, message: '请输入密码' }]}
              style={{ marginBottom: 22 }}
            >
              <Input.Password
                placeholder="请输入密码"
                size="large"
                prefix={<Lock size={15} color="#8c8c8c" style={{ marginRight: 6 }} />}
                style={{ borderRadius: 8 }}
              />
            </Form.Item>
            <Form.Item style={{ marginBottom: 0 }}>
              <Button
                type="primary"
                htmlType="submit"
                size="large"
                block
                loading={loading}
                icon={<LogIn size={16} />}
                style={{
                  borderRadius: 8,
                  height: 44,
                  fontSize: 14,
                  fontWeight: 500,
                  background: 'linear-gradient(135deg, #1677ff 0%, #4096ff 100%)',
                  border: 'none',
                  boxShadow: '0 6px 20px -4px rgba(22, 119, 255, 0.45)',
                }}
              >
                登录
              </Button>
            </Form.Item>
          </Form>
          <div
            style={{
              marginTop: 18,
              padding: '10px 12px',
              background: 'linear-gradient(90deg, #f0f5ff 0%, #f9f0ff 100%)',
              borderRadius: 8,
              border: '1px dashed #d6e4ff',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
            }}
          >
            <Sparkles size={13} color="#1677ff" />
            <Typography.Text style={{ fontSize: 12, color: '#595959' }}>
              演示账号:auditor@dgut.edu.cn / 任意密码
            </Typography.Text>
          </div>
        </div>
      </div>
    </div>
  );
}
