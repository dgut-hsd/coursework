import type { JSX } from 'react';
import { Outlet, Link, useNavigate, useLocation } from 'react-router-dom';
import { Layout, Menu, Space, Button, Avatar, Dropdown, Typography, Badge } from 'antd';
import {
  LogOut,
  FileText,
  LayoutDashboard,
  User,
  Sparkles,
  Bell,
  ShieldCheck,
} from 'lucide-react';
import { useAppDispatch, useAppSelector } from './store';
import { selectAuth, logout } from '../features/auth/authSlice';

const { Header, Content } = Layout;

const navItems = [
  { key: '/dashboard', icon: <LayoutDashboard size={16} />, label: '任务列表' },
];

export function AppLayout(): JSX.Element {
  const auth = useAppSelector(selectAuth);
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const location = useLocation();

  const handleLogout = (): void => {
    dispatch(logout());
    navigate('/');
  };

  const selectedKey = navItems.find((item) => location.pathname.startsWith(item.key))?.key ?? '/dashboard';

  return (
    <Layout style={{ minHeight: '100vh', background: '#f0f2f5' }}>
      <Header
        style={{
          display: 'flex',
          alignItems: 'center',
          padding: '0 32px',
          background: 'linear-gradient(95deg, #0a1834 0%, #143055 45%, #1d3a72 100%)',
          gap: 28,
          position: 'sticky',
          top: 0,
          zIndex: 100,
          boxShadow: '0 6px 24px -10px rgba(10, 24, 52, 0.6)',
        }}
      >
        <Link
          to="/dashboard"
          style={{
            color: '#fff',
            display: 'flex',
            alignItems: 'center',
            gap: 12,
            textDecoration: 'none',
          }}
        >
          <div
            style={{
              width: 38,
              height: 38,
              borderRadius: 10,
              background: 'linear-gradient(135deg, #1677ff 0%, #722ed1 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 4px 14px rgba(22, 119, 255, 0.4)',
            }}
          >
            <ShieldCheck size={22} color="#fff" strokeWidth={2.4} />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', lineHeight: 1.1 }}>
            <Typography.Text style={{ color: '#fff', fontSize: 16, fontWeight: 600, letterSpacing: 0.3 }}>
              AI 标书审核工作台
            </Typography.Text>
            <Typography.Text style={{ color: 'rgba(255,255,255,0.55)', fontSize: 11, marginTop: 2 }}>
              Bid Audit Workbench
            </Typography.Text>
          </div>
        </Link>

        <div
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 6,
            padding: '4px 12px',
            borderRadius: 20,
            background: 'rgba(255, 255, 255, 0.08)',
            border: '1px solid rgba(255, 255, 255, 0.12)',
          }}
        >
          <Sparkles size={12} color="#ffd54f" />
          <Typography.Text style={{ color: 'rgba(255,255,255,0.85)', fontSize: 12 }}>
            AI 智能审核 · v1.0
          </Typography.Text>
        </div>

        <Menu
          theme="dark"
          mode="horizontal"
          selectedKeys={[selectedKey]}
          items={navItems.map((item) => ({
            key: item.key,
            icon: item.icon,
            label: <Link to={item.key}>{item.label}</Link>,
          }))}
          style={{ flex: 1, minWidth: 0, background: 'transparent', borderBottom: 'none' }}
        />

        <Space size={16}>
          <Badge dot color="#52c41a" offset={[-2, 2]}>
            <Button
              type="text"
              shape="circle"
              icon={<Bell size={18} color="rgba(255,255,255,0.85)" />}
            />
          </Badge>
          {auth.isAuthenticated ? (
            <Dropdown
              menu={{
                items: [
                  {
                    key: 'logout',
                    icon: <LogOut size={14} />,
                    label: '退出登录',
                    onClick: handleLogout,
                  },
                ],
              }}
            >
              <Space style={{ cursor: 'pointer', color: '#fff', padding: '4px 10px', borderRadius: 8 }}>
                <Avatar
                  size={30}
                  style={{
                    background: 'linear-gradient(135deg, #1677ff 0%, #722ed1 100%)',
                  }}
                  icon={<User size={15} />}
                />
                <Typography.Text style={{ color: '#fff', fontSize: 14 }}>
                  {auth.user?.name ?? '审核员'}
                </Typography.Text>
              </Space>
            </Dropdown>
          ) : (
            <Button
              type="primary"
              icon={<FileText size={14} />}
              onClick={() => navigate('/')}
              style={{ borderRadius: 8 }}
            >
              登录
            </Button>
          )}
        </Space>
      </Header>
      <Content style={{ flex: 1, padding: 0, overflow: 'auto' }}>
        <Outlet />
      </Content>
    </Layout>
  );
}
