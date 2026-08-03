import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
      staleTime: 30_000,
    },
    mutations: {
      retry: 0,
    },
  },
});

export const queryKeys = {
  tasks: ['tasks'] as const,
  task: (id: string) => ['tasks', id] as const,
  findings: (taskId: string) => ['tasks', taskId, 'findings'] as const,
  report: (reportId: string) => ['reports', reportId] as const,
  reportByTask: (taskId: string) => ['reports', 'task', taskId] as const,
};
