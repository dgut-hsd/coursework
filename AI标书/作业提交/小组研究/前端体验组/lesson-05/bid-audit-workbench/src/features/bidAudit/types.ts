export type Severity = 'critical' | 'warning' | 'info';

export interface Bbox {
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Finding {
  id: string;
  taskId: string;
  severity: Severity;
  title: string;
  description: string;
  lawRef?: string;
  suggestion?: string;
  bbox: Bbox;
  page: number;
  createdAt: string;
}

export interface TaskProgress {
  taskId: string;
  percent: number;
  stage: 'uploading' | 'parsing' | 'analyzing' | 'reporting' | 'done';
  message: string;
}

export interface AuditTask {
  id: string;
  fileName: string;
  fileSize: number;
  status: 'pending' | 'running' | 'done' | 'failed';
  createdAt: string;
  findingsCount: number;
}

export interface AuditReport {
  id: string;
  taskId: string;
  title: string;
  markdown: string;
  stats: {
    critical: number;
    warning: number;
    info: number;
  };
  createdAt: string;
}

export interface User {
  id: string;
  name: string;
  email: string;
}

export interface LoginPayload {
  email: string;
  password: string;
}

export interface AuthState {
  token: string | null;
  user: User | null;
  isAuthenticated: boolean;
}
