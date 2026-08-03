// Mock API — 供 react-query 使用

export interface Project {
  id: string
  name: string
  status: 'active' | 'archived'
}

export interface AuditTask {
  id: string
  projectId: string
  title: string
  status: 'PENDING' | 'PROCESSING' | 'COMPLETED'
}

const mockProjects: Project[] = [
  { id: 'p1', name: '某市政工程标书', status: 'active' },
  { id: 'p2', name: '某道路建设标书', status: 'active' },
  { id: 'p3', name: '某桥梁施工标书', status: 'archived' },
]

let mockTasks: AuditTask[] = [
  { id: 't1', projectId: 'p1', title: '资质审核', status: 'COMPLETED' },
  { id: 't2', projectId: 'p1', title: '报价审核', status: 'PENDING' },
  { id: 't3', projectId: 'p2', title: '技术方案审核', status: 'PROCESSING' },
]

export async function fetchProjects(): Promise<Project[]> {
  return new Promise(resolve => setTimeout(() => resolve([...mockProjects]), 500))
}

export async function fetchTasks(projectId: string): Promise<AuditTask[]> {
  return new Promise(resolve =>
    setTimeout(() => resolve(mockTasks.filter(t => t.projectId === projectId)), 500),
  )
}

export async function createTask(projectId: string, title: string): Promise<AuditTask> {
  return new Promise(resolve => {
    setTimeout(() => {
      const task: AuditTask = { id: 't' + Date.now(), projectId, title, status: 'PENDING' }
      mockTasks.push(task)
      resolve(task)
    }, 500)
  })
}
