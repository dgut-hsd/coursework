import { EventSource } from 'eventsource';
import type { TaskProgress, Finding } from '../types';

type ProgressHandler = (data: TaskProgress) => void;
type FindingHandler = (data: Finding) => void;
type CompleteHandler = () => void;
type ErrorHandler = (error: unknown) => void;

export interface SSEHandlers {
  onProgress: ProgressHandler;
  onFinding: FindingHandler;
  onComplete: CompleteHandler;
  onError?: ErrorHandler;
}

export interface SSEHandle {
  close: () => void;
}

const STAGES: ReadonlyArray<{ stage: TaskProgress['stage']; weight: number }> = [
  { stage: 'uploading', weight: 10 },
  { stage: 'parsing', weight: 20 },
  { stage: 'analyzing', weight: 50 },
  { stage: 'reporting', weight: 15 },
  { stage: 'done', weight: 5 },
];

const STAGE_MESSAGES: Record<TaskProgress['stage'], string> = {
  uploading: '正在上传招标文件...',
  parsing: '正在解析 PDF 结构与目录...',
  analyzing: '正在基于法规库进行条款分析...',
  reporting: '正在汇总生成审核报告...',
  done: '审核完成',
};

const SAMPLE_FINDINGS_FOR_MOCK: ReadonlyArray<Omit<Finding, 'id' | 'taskId' | 'createdAt'>> = [
  {
    severity: 'critical',
    title: '排斥性条款:地域限制',
    description: '条款要求投标人须在深圳市设立总部,可能构成以不合理条件限制或排斥潜在投标人。',
    lawRef: '《政府采购法》第二十二条',
    suggestion: '删除"在深圳市设立总部"的限定。',
    bbox: { page: 1, x: 120, y: 220, width: 320, height: 38 },
    page: 1,
  },
  {
    severity: 'warning',
    title: '技术参数倾向性',
    description: '技术要求中指定了某品牌的产品型号,可能构成倾向性条款。',
    lawRef: '《招标投标法实施条例》第三十二条',
    suggestion: '将"必须使用 XX 品牌"修改为"或同等性能的其他品牌"。',
    bbox: { page: 2, x: 80, y: 360, width: 380, height: 50 },
    page: 2,
  },
  {
    severity: 'info',
    title: '格式问题:投标保证金',
    description: '投标保证金金额未使用人民币大写,存在涂改风险。',
    suggestion: '请使用"人民币壹拾万元整(¥100,000.00)"规范格式。',
    bbox: { page: 3, x: 140, y: 480, width: 260, height: 30 },
    page: 3,
  },
  {
    severity: 'critical',
    title: '违规设置最高限价',
    description: '未依法设置最高限价或招标控制价,违反法定程序。',
    lawRef: '《政府采购法实施条例》第四十八条',
    suggestion: '在招标文件中明确列示最高限价,并说明制定依据。',
    bbox: { page: 1, x: 100, y: 600, width: 360, height: 40 },
    page: 1,
  },
  {
    severity: 'warning',
    title: '评审因素量化不足',
    description: '商务评分项描述模糊,未给出明确量化标准。',
    lawRef: '《政府采购货物和服务招标投标管理办法》第五十五条',
    suggestion: '将"丰富的行业经验"等模糊描述替换为可量化的条件。',
    bbox: { page: 2, x: 90, y: 540, width: 370, height: 45 },
    page: 2,
  },
  {
    severity: 'info',
    title: '履约期限未约定',
    description: '合同履约期限缺失,可能导致后续纠纷。',
    suggestion: '补充"合同签订后 60 日内完成交付并验收"。',
    bbox: { page: 3, x: 110, y: 700, width: 320, height: 28 },
    page: 3,
  },
];

const STREAM_POOL: Record<string, ReadonlyArray<Omit<Finding, 'id' | 'taskId' | 'createdAt'>>> = {
  default: SAMPLE_FINDINGS_FOR_MOCK,
  't-2': [
    {
      severity: 'critical',
      title: '资格条件以不合理业绩限制',
      description: '实时流:发现新的业绩门槛条款,可能排斥中小供应商。',
      lawRef: '《政府采购法》第二十二条',
      suggestion: '将业绩门槛下调至 1 个同类项目。',
      bbox: { page: 1, x: 110, y: 280, width: 380, height: 45 },
      page: 1,
    },
    {
      severity: 'warning',
      title: '评标办法描述不清',
      description: '实时流:综合评分法的分值构成未细化到子项。',
      lawRef: '《政府采购货物和服务招标投标管理办法》第五十五条',
      suggestion: '将价格、技术、商务三项的分值逐一列明。',
      bbox: { page: 2, x: 90, y: 420, width: 360, height: 50 },
      page: 2,
    },
    {
      severity: 'info',
      title: '投标文件份数不明确',
      description: '实时流:正本/副本份数未明确。',
      suggestion: '明确正本 1 份、副本 4 份、电子版 1 份。',
      bbox: { page: 3, x: 130, y: 350, width: 280, height: 32 },
      page: 3,
    },
  ],
  't-4': [
    {
      severity: 'critical',
      title: '串通投标风险条款',
      description: '实时流:允许投标人就同一项目向多家代理机构报名。',
      lawRef: '《招标投标法实施条例》第四十条',
      suggestion: '明确仅能授权一家代理机构。',
      bbox: { page: 1, x: 100, y: 200, width: 360, height: 40 },
      page: 1,
    },
    {
      severity: 'warning',
      title: '履约担保金额超标',
      description: '实时流:履约担保设置为合同价的 15%。',
      lawRef: '《招标投标法实施条例》第五十八条',
      suggestion: '将履约担保金额调整至不超过 10%。',
      bbox: { page: 2, x: 110, y: 480, width: 320, height: 38 },
      page: 2,
    },
  ],
  't-5': [
    {
      severity: 'critical',
      title: '将货物与服务混同招标',
      description: '实时流:同一标段同时包含硬件采购与运维服务。',
      lawRef: '《政府采购法实施条例》第二十四条',
      suggestion: '硬件采购与运维服务应拆分为独立标段。',
      bbox: { page: 1, x: 110, y: 260, width: 380, height: 48 },
      page: 1,
    },
    {
      severity: 'warning',
      title: '样品要求过严',
      description: '实时流:要求提供原厂样品,限制竞争。',
      lawRef: '《政府采购法》第二十二条',
      suggestion: '允许功能等同或原厂授权样品。',
      bbox: { page: 2, x: 90, y: 340, width: 360, height: 45 },
      page: 2,
    },
    {
      severity: 'info',
      title: '验收标准描述笼统',
      description: '实时流:未列明各阶段验收指标。',
      suggestion: '补充详细验收清单与抽样比例。',
      bbox: { page: 3, x: 110, y: 640, width: 340, height: 32 },
      page: 3,
    },
  ],
  't-7': [
    {
      severity: 'critical',
      title: '单一来源采购依据不足',
      description: '实时流:仅在附件中说明"只有一家供应商"。',
      lawRef: '《政府采购法》第三十一条',
      suggestion: '补充不少于 3 家供应商的调研对比表。',
      bbox: { page: 1, x: 100, y: 240, width: 380, height: 50 },
      page: 1,
    },
  ],
};

// 真实 SSE 连接 (依赖 eventsource package 以支持自定义 Header)
export function createRealSSEConnection(url: string, handlers: SSEHandlers): SSEHandle {
  const es = new EventSource(url, {
    headers: { Authorization: 'Bearer mock-token' },
  });

  es.addEventListener('progress', (ev) => {
    const data = parseData<TaskProgress>(ev);
    if (data) handlers.onProgress(data);
  });

  es.addEventListener('finding', (ev) => {
    const data = parseData<Finding>(ev);
    if (data) handlers.onFinding(data);
  });

  es.addEventListener('complete', () => {
    handlers.onComplete();
  });

  es.onerror = (ev) => {
    handlers.onError?.(ev);
  };

  return {
    close: () => es.close(),
  };
}

// Mock 模式:用 setInterval 模拟进度/finding 推送
export function createMockSSE(taskId: string, handlers: SSEHandlers): SSEHandle {
  let elapsed = 0;
  const totalMs = 6_000;
  const tickMs = 300;
  let findingIndex = 0;
  let closed = false;
  const samples = STREAM_POOL[taskId] ?? STREAM_POOL.default ?? SAMPLE_FINDINGS_FOR_MOCK;

  const interval = setInterval(() => {
    if (closed) return;
    elapsed += tickMs;
    const percent = Math.min(100, Math.round((elapsed / totalMs) * 100));
    const stage = pickStage(percent);

    const progress: TaskProgress = {
      taskId,
      percent,
      stage,
      message: STAGE_MESSAGES[stage],
    };
    handlers.onProgress(progress);

    if (stage === 'analyzing' && findingIndex < samples.length) {
      const sample = samples[findingIndex];
      if (sample) {
        const finding: Finding = {
          id: `f-${taskId}-${findingIndex}`,
          taskId,
          createdAt: new Date().toISOString(),
          ...sample,
        };
        handlers.onFinding(finding);
        findingIndex += 1;
      }
    }

    if (percent >= 100) {
      clearInterval(interval);
      handlers.onComplete();
      closed = true;
    }
  }, tickMs);

  return {
    close: () => {
      closed = true;
      clearInterval(interval);
    },
  };
}

function parseData<T>(ev: unknown): T | null {
  if (ev && typeof ev === 'object' && 'data' in ev) {
    const data = (ev as { data: string }).data;
    try {
      return JSON.parse(data) as T;
    } catch {
      return null;
    }
  }
  return null;
}

function pickStage(percent: number): TaskProgress['stage'] {
  let acc = 0;
  for (const s of STAGES) {
    acc += s.weight;
    if (percent <= acc) return s.stage;
  }
  return 'done';
}
