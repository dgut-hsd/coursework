import { useEffect } from 'react';
import type { JSX } from 'react';
import { ConfigProvider, App as AntApp, theme } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import { RouterProvider } from 'react-router-dom';
import { Provider as ReduxProvider } from 'react-redux';
import { QueryClientProvider } from '@tanstack/react-query';
import { router } from './app/router';
import { store, useAppDispatch } from './app/store';
import { queryClient } from './app/queryClient';
import { setUser } from './features/auth/authSlice';
import { GlobalStyle } from './styles/global';

function AuthInitializer({ children }: { children: React.ReactNode }): JSX.Element {
  const dispatch = useAppDispatch();

  useEffect(() => {
    const token = localStorage.getItem('auth_token');
    if (token) {
      const email = localStorage.getItem('auth_email') ?? 'auditor@dgut.edu.cn';
      const name = localStorage.getItem('auth_name') ?? '审核员';
      dispatch(
        setUser({
          id: 'u-1',
          name,
          email,
        }),
      );
    }
  }, [dispatch]);

  return <>{children}</>;
}

export function App(): JSX.Element {
  return (
    <ReduxProvider store={store}>
      <QueryClientProvider client={queryClient}>
        <ConfigProvider
          locale={zhCN}
          theme={{
            algorithm: theme.defaultAlgorithm,
            token: {
              colorPrimary: '#1677ff',
              borderRadius: 6,
            },
          }}
        >
          <AntApp>
            <GlobalStyle />
            <AuthInitializer>
              <RouterProvider router={router} />
            </AuthInitializer>
          </AntApp>
        </ConfigProvider>
      </QueryClientProvider>
    </ReduxProvider>
  );
}
