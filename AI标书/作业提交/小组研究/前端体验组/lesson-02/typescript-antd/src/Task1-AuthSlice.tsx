import { useState } from 'react'
import { Card, Form, Input, Button, Alert, Space, Descriptions } from 'antd'
import { useAppDispatch, useAppSelector } from './store/hooks'
import { login, logout, clearError } from './store/slices/authSlice'

export default function Task1AuthSlice() {
  const dispatch = useAppDispatch()
  const { user, token, isAuthenticated, loading, error } = useAppSelector(s => s.auth)
  const [username, setUsername] = useState('admin')
  const [password, setPassword] = useState('123456')

  const handleLogin = () => dispatch(login({ username, password }))
  const handleLogout = () => dispatch(logout())

  return (
    <Card title="Redux Toolkit auth slice">
      <p style={{ color: '#999' }}>账号 admin / 密码 123456，打开 Redux DevTools 查看 action 流</p>

      {error && (
        <Alert
          message={error}
          type="error"
          closable
          onClose={() => dispatch(clearError())}
          style={{ marginBottom: 16 }}
        />
      )}

      {!isAuthenticated ? (
        <Form layout="inline">
          <Form.Item label="用户名">
            <Input value={username} onChange={e => setUsername(e.target.value)} />
          </Form.Item>
          <Form.Item label="密码">
            <Input.Password value={password} onChange={e => setPassword(e.target.value)} />
          </Form.Item>
          <Form.Item>
            <Button type="primary" loading={loading} onClick={handleLogin}>登录</Button>
          </Form.Item>
        </Form>
      ) : (
        <Space direction="vertical" style={{ width: '100%' }}>
          <Alert
            message={`欢迎，${user!.name}（${user!.role}）`}
            type="success"
            action={<Button size="small" onClick={handleLogout}>退出登录</Button>}
          />
          <Descriptions title="Redux State (auth)" bordered size="small" column={1}>
            <Descriptions.Item label="user">{JSON.stringify(user)}</Descriptions.Item>
            <Descriptions.Item label="token">{token}</Descriptions.Item>
            <Descriptions.Item label="isAuthenticated">{String(isAuthenticated)}</Descriptions.Item>
          </Descriptions>
        </Space>
      )}
    </Card>
  )
}
