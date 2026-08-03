import { createBrowserRouter, Navigate } from 'react-router-dom';
import { lazy, Suspense } from 'react';
import type { JSX } from 'react';
import { Spin } from 'antd';
import { AppLayout } from './AppLayout';

const DashboardPage = lazy(() => import('../features/dashboard/DashboardPage').then((m) => ({ default: m.DashboardPage })));
const AuditPage = lazy(() => import('../features/bidAudit/AuditPage').then((m) => ({ default: m.AuditPage })));
const ReportPage = lazy(() => import('../features/bidAudit/ReportPage').then((m) => ({ default: m.ReportPage })));
const LoginPage = lazy(() => import('../features/auth/LoginPage').then((m) => ({ default: m.LoginPage })));

const PageFallback = (): JSX.Element => (
  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', minHeight: 400 }}>
    <Spin size="large" />
  </div>
);

export const router = createBrowserRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: <Navigate to="/dashboard" replace /> },
      {
        path: 'dashboard',
        element: (
          <Suspense fallback={<PageFallback />}>
            <DashboardPage />
          </Suspense>
        ),
      },
      {
        path: 'projects/:id/audit/:tid',
        element: (
          <Suspense fallback={<PageFallback />}>
            <AuditPage />
          </Suspense>
        ),
      },
      {
        path: 'projects/:id/report/:rid',
        element: (
          <Suspense fallback={<PageFallback />}>
            <ReportPage />
          </Suspense>
        ),
      },
      {
        path: 'login',
        element: (
          <Suspense fallback={<PageFallback />}>
            <LoginPage />
          </Suspense>
        ),
      },
    ],
  },
]);
