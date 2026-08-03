import type { AuditTask, Finding, AuditReport } from '../types';

const FINDINGS_BY_TASK: Record<string, Finding[]> = {
  't-1': [
    {
      id: 'f-1',
      taskId: 't-1',
      severity: 'critical',
      title: '排斥性条款:地域限制',
      description: '条款要求投标人须在深圳市设立总部,可能构成以不合理条件限制或排斥潜在投标人。',
      lawRef: '《政府采购法》第二十二条',
      suggestion: '删除"在深圳市设立总部"的限定,改为"在中华人民共和国境内注册"。',
      bbox: { page: 1, x: 120, y: 220, width: 320, height: 38 },
      page: 1,
      createdAt: '2026-07-31T10:00:00Z',
    },
    {
      id: 'f-2',
      taskId: 't-1',
      severity: 'warning',
      title: '技术参数倾向性',
      description: '技术要求中指定了某品牌的产品型号,可能构成倾向性条款。',
      lawRef: '《招标投标法实施条例》第三十二条',
      suggestion: '将"必须使用 XX 品牌 XX 型号"修改为"或同等性能的其他品牌产品"。',
      bbox: { page: 2, x: 80, y: 360, width: 380, height: 50 },
      page: 2,
      createdAt: '2026-07-31T10:00:05Z',
    },
    {
      id: 'f-3',
      taskId: 't-1',
      severity: 'info',
      title: '格式问题:投标保证金',
      description: '投标保证金金额未使用人民币大写,存在涂改风险。',
      suggestion: '请使用"人民币壹拾万元整(¥100,000.00)"的规范格式。',
      bbox: { page: 3, x: 140, y: 480, width: 260, height: 30 },
      page: 3,
      createdAt: '2026-07-31T10:00:10Z',
    },
    {
      id: 'f-4',
      taskId: 't-1',
      severity: 'critical',
      title: '违规设置最高限价',
      description: '未依法设置最高限价或招标控制价,违反法定程序。',
      lawRef: '《政府采购法实施条例》第四十八条',
      suggestion: '在招标文件中明确列示最高限价,并说明制定依据。',
      bbox: { page: 1, x: 100, y: 600, width: 360, height: 40 },
      page: 1,
      createdAt: '2026-07-31T10:00:15Z',
    },
    {
      id: 'f-5',
      taskId: 't-1',
      severity: 'warning',
      title: '评审因素量化不足',
      description: '商务评分项描述模糊,未给出明确量化标准,可能引发争议。',
      lawRef: '《政府采购货物和服务招标投标管理办法》第五十五条',
      suggestion: '将"丰富的行业经验"等模糊描述替换为"近 3 年承担同类项目不少于 X 个"。',
      bbox: { page: 2, x: 90, y: 540, width: 370, height: 45 },
      page: 2,
      createdAt: '2026-07-31T10:00:20Z',
    },
    {
      id: 'f-6',
      taskId: 't-1',
      severity: 'info',
      title: '履约期限未约定',
      description: '合同履约期限缺失,可能导致后续纠纷。',
      suggestion: '补充"合同签订后 60 日内完成交付并验收"。',
      bbox: { page: 3, x: 110, y: 700, width: 320, height: 28 },
      page: 3,
      createdAt: '2026-07-31T10:00:25Z',
    },
  ],
  't-2': [
    {
      id: 'f-7',
      taskId: 't-2',
      severity: 'critical',
      title: '资格条件以不合理业绩限制',
      description: '要求投标人近 3 年承担过 5 个以上同类项目,可能排斥中小供应商。',
      lawRef: '《政府采购法》第二十二条',
      suggestion: '将业绩门槛下调至"近 3 年同类项目不少于 1 个",或明确允许联合体投标。',
      bbox: { page: 1, x: 110, y: 280, width: 380, height: 45 },
      page: 1,
      createdAt: '2026-07-29T11:25:00Z',
    },
    {
      id: 'f-8',
      taskId: 't-2',
      severity: 'warning',
      title: '评标办法描述不清',
      description: '综合评分法的分值构成未细化到子项,容易引发投诉。',
      lawRef: '《政府采购货物和服务招标投标管理办法》第五十五条',
      suggestion: '将价格、技术、商务三项的分值与子项逐一列明,确保总分=100。',
      bbox: { page: 2, x: 90, y: 420, width: 360, height: 50 },
      page: 2,
      createdAt: '2026-07-29T11:25:10Z',
    },
    {
      id: 'f-9',
      taskId: 't-2',
      severity: 'warning',
      title: '付款条件过于苛刻',
      description: '要求中标人先行垫资 100% 货款,验收后才一次性付款,违背公平原则。',
      lawRef: '《保障中小企业款项支付条例》',
      suggestion: '调整为"合同签订后支付 30% 预付款,验收合格后支付 65%,质保期满后支付 5%"。',
      bbox: { page: 2, x: 100, y: 600, width: 380, height: 42 },
      page: 2,
      createdAt: '2026-07-29T11:25:20Z',
    },
    {
      id: 'f-10',
      taskId: 't-2',
      severity: 'info',
      title: '投标文件份数不明确',
      description: '未明确正本/副本份数及电子版要求。',
      suggestion: '明确"纸质投标文件正本 1 份、副本 4 份、电子版 1 份(加盖公章扫描件)"。',
      bbox: { page: 3, x: 130, y: 350, width: 280, height: 32 },
      page: 3,
      createdAt: '2026-07-29T11:25:30Z',
    },
  ],
  't-4': [
    {
      id: 'f-11',
      taskId: 't-4',
      severity: 'critical',
      title: '串通投标风险条款',
      description: '允许投标人就同一项目向多家代理机构报名,可能引发串标风险。',
      lawRef: '《招标投标法实施条例》第四十条',
      suggestion: '明确"同一投标人在本项目仅能授权一家代理机构办理投标事宜"。',
      bbox: { page: 1, x: 100, y: 200, width: 360, height: 40 },
      page: 1,
      createdAt: '2026-07-28T14:10:00Z',
    },
    {
      id: 'f-12',
      taskId: 't-4',
      severity: 'warning',
      title: '履约担保金额超标',
      description: '履约担保金额设置为合同价的 15%,超过法定 10% 上限。',
      lawRef: '《招标投标法实施条例》第五十八条',
      suggestion: '将履约担保金额调整至不超过合同价的 10%。',
      bbox: { page: 2, x: 110, y: 480, width: 320, height: 38 },
      page: 2,
      createdAt: '2026-07-28T14:10:10Z',
    },
    {
      id: 'f-13',
      taskId: 't-4',
      severity: 'info',
      title: '异议处理时限缺失',
      description: '未规定投标人提出异议的法定期限与受理方式。',
      suggestion: '补充"投标人可在知道或应当知道权益受损之日起 10 日内提出书面异议"。',
      bbox: { page: 3, x: 130, y: 520, width: 320, height: 30 },
      page: 3,
      createdAt: '2026-07-28T14:10:20Z',
    },
  ],
  't-5': [
    {
      id: 'f-14',
      taskId: 't-5',
      severity: 'critical',
      title: '将货物与服务混同招标',
      description: '同一标段同时包含硬件采购与运维服务,未拆分标段,潜在投标人受限。',
      lawRef: '《政府采购法实施条例》第二十四条',
      suggestion: '将硬件采购与运维服务拆分为两个独立标段,或明确允许联合体投标。',
      bbox: { page: 1, x: 110, y: 260, width: 380, height: 48 },
      page: 1,
      createdAt: '2026-07-27T09:30:00Z',
    },
    {
      id: 'f-15',
      taskId: 't-5',
      severity: 'warning',
      title: '样品要求过严',
      description: '要求提供原厂样品,限制了小规模厂商参与竞争。',
      lawRef: '《政府采购法》第二十二条',
      suggestion: '明确"可提供功能等同的样品,或原厂授权样品,评审时一视同仁"。',
      bbox: { page: 2, x: 90, y: 340, width: 360, height: 45 },
      page: 2,
      createdAt: '2026-07-27T09:30:10Z',
    },
    {
      id: 'f-16',
      taskId: 't-5',
      severity: 'warning',
      title: '资质门槛与项目规模不匹配',
      description: '项目预算 80 万,却要求投标人具备 ISO27001 等高级资质,门槛过高。',
      lawRef: '《政府采购法》第二十二条',
      suggestion: '根据项目规模合理设置资质要求,删除与项目无关的认证条款。',
      bbox: { page: 2, x: 100, y: 560, width: 380, height: 50 },
      page: 2,
      createdAt: '2026-07-27T09:30:20Z',
    },
    {
      id: 'f-17',
      taskId: 't-5',
      severity: 'info',
      title: '验收标准描述笼统',
      description: '未列明各阶段验收指标、抽样比例及不合格处理方式。',
      suggestion: '补充详细验收清单及 SOW 对应表,明确每项交付物对应的验收方法。',
      bbox: { page: 3, x: 110, y: 640, width: 340, height: 32 },
      page: 3,
      createdAt: '2026-07-27T09:30:30Z',
    },
  ],
  't-7': [
    {
      id: 'f-18',
      taskId: 't-7',
      severity: 'critical',
      title: '单一来源采购依据不足',
      description: '仅在附件中说明"经市场调研,只有一家供应商",未提供论证材料。',
      lawRef: '《政府采购法》第三十一条',
      suggestion: '补充不少于 3 家供应商的调研对比表,或转为公开招标。',
      bbox: { page: 1, x: 100, y: 240, width: 380, height: 50 },
      page: 1,
      createdAt: '2026-07-25T16:00:00Z',
    },
    {
      id: 'f-19',
      taskId: 't-7',
      severity: 'info',
      title: '合同模板未引用',
      description: '未提供合同范本,投标人无法评估履约风险。',
      suggestion: '在附件中提供合同范本,包含违约责任、知识产权、争议解决等条款。',
      bbox: { page: 2, x: 120, y: 420, width: 300, height: 30 },
      page: 2,
      createdAt: '2026-07-25T16:00:10Z',
    },
  ],
};

const REPORTS_BY_TASK: Record<string, AuditReport> = {
  't-1': {
    id: 'r-1',
    taskId: 't-1',
    title: '某市政项目招标文件审核报告',
    createdAt: '2026-07-31T10:05:00Z',
    stats: { critical: 2, warning: 2, info: 2 },
    markdown: [
      '# 某市政项目招标文件审核报告',
      '',
      '> 任务编号: `t-1` | 生成时间: 2026-07-31 10:05',
      '',
      '## 一、审核概览',
      '',
      '本次审核共发现 **6** 个问题,其中:',
      '',
      '| 严重程度 | 数量 |',
      '| --- | ---: |',
      '| 严重 (critical) | 2 |',
      '| 警告 (warning) | 2 |',
      '| 提示 (info) | 2 |',
      '',
      '## 二、严重问题',
      '',
      '### 2.1 排斥性条款:地域限制',
      '',
      '- **位置**:第 1 页,资质要求段落',
      '- **法规依据**:`《政府采购法》第二十二条`',
      '- **风险**:可能构成以不合理条件限制或排斥潜在投标人',
      '- **修改建议**:删除"在深圳市设立总部"的限定,改为"在中华人民共和国境内注册"。',
      '',
      '### 2.2 违规设置最高限价',
      '',
      '- **位置**:第 1 页,价格条款',
      '- **法规依据**:`《政府采购法实施条例》第四十八条`',
      '- **风险**:未依法设置最高限价,违反法定程序',
      '- **修改建议**:在招标文件中明确列示最高限价,并说明制定依据。',
      '',
      '## 三、警告问题',
      '',
      '### 3.1 技术参数倾向性',
      '',
      '- **位置**:第 2 页,技术要求段落',
      '- **法规依据**:`《招标投标法实施条例》第三十二条`',
      '- **修改建议**:将"必须使用 XX 品牌 XX 型号"修改为"或同等性能的其他品牌产品"。',
      '',
      '### 3.2 评审因素量化不足',
      '',
      '- **位置**:第 2 页,评审标准段落',
      '- **法规依据**:`《政府采购货物和服务招标投标管理办法》第五十五条`',
      '- **修改建议**:将"丰富的行业经验"等模糊描述替换为"近 3 年承担同类项目不少于 X 个"。',
      '',
      '## 四、提示问题',
      '',
      '| 编号 | 标题 | 位置 | 建议 |',
      '| --- | --- | --- | --- |',
      '| 1 | 格式问题:投标保证金 | 第 3 页 | 使用"人民币壹拾万元整(¥100,000.00)"规范格式 |',
      '| 2 | 履约期限未约定 | 第 3 页 | 补充"合同签订后 60 日内完成交付并验收" |',
      '',
      '## 五、总结',
      '',
      '本次审核共发现 **2 个严重问题**,建议在发布前完成修改,以避免法律风险。',
    ].join('\n'),
  },
  't-2': {
    id: 'r-2',
    taskId: 't-2',
    title: '信息系统采购项目审核报告',
    createdAt: '2026-07-29T11:30:00Z',
    stats: { critical: 1, warning: 2, info: 1 },
    markdown: [
      '# 信息系统采购项目审核报告',
      '',
      '> 任务编号: `t-2` | 生成时间: 2026-07-29 11:30',
      '',
      '## 一、审核概览',
      '',
      '本次审核共发现 **4** 个问题,其中严重问题 1 个、警告问题 2 个、提示问题 1 个。',
      '',
      '## 二、严重问题',
      '',
      '### 2.1 资格条件以不合理业绩限制',
      '',
      '- **位置**:第 1 页,投标人资格段落',
      '- **法规依据**:`《政府采购法》第二十二条`',
      '- **风险**:可能排斥中小型供应商,违反公平竞争',
      '- **修改建议**:将业绩门槛下调至"近 3 年同类项目不少于 1 个",或明确允许联合体投标。',
      '',
      '## 三、警告问题',
      '',
      '### 3.1 评标办法描述不清',
      '',
      '- **位置**:第 2 页,评标办法',
      '- **法规依据**:`《政府采购货物和服务招标投标管理办法》第五十五条`',
      '- **修改建议**:将价格、技术、商务三项的分值与子项逐一列明。',
      '',
      '### 3.2 付款条件过于苛刻',
      '',
      '- **位置**:第 2 页,合同条款',
      '- **法规依据**:`《保障中小企业款项支付条例》`',
      '- **修改建议**:调整为分阶段付款,合同签订后 30% 预付款。',
      '',
      '## 四、提示问题',
      '',
      '- **投标文件份数不明确**:补充正本 1 份、副本 4 份、电子版 1 份的要求。',
      '',
      '## 五、总结',
      '',
      '建议针对 **1 个严重问题** 优先整改,避免招标后被投诉。',
    ].join('\n'),
  },
  't-4': {
    id: 'r-4',
    taskId: 't-4',
    title: '工程总承包招标审核报告',
    createdAt: '2026-07-28T14:15:00Z',
    stats: { critical: 1, warning: 1, info: 1 },
    markdown: [
      '# 工程总承包招标审核报告',
      '',
      '> 任务编号: `t-4` | 生成时间: 2026-07-28 14:15',
      '',
      '## 一、审核概览',
      '',
      '本次审核共发现 **3** 个问题。',
      '',
      '## 二、严重问题',
      '',
      '### 2.1 串通投标风险条款',
      '',
      '- **法规依据**:`《招标投标法实施条例》第四十条`',
      '- **修改建议**:明确"同一投标人在本项目仅能授权一家代理机构办理投标事宜"。',
      '',
      '## 三、警告问题',
      '',
      '### 3.1 履约担保金额超标',
      '',
      '- **法规依据**:`《招标投标法实施条例》第五十八条`',
      '- **修改建议**:将履约担保金额调整至不超过合同价的 10%。',
      '',
      '## 四、提示问题',
      '',
      '- **异议处理时限缺失**:补充 10 个工作日的法定异议受理机制。',
    ].join('\n'),
  },
  't-5': {
    id: 'r-5',
    taskId: 't-5',
    title: '设备与运维综合项目审核报告',
    createdAt: '2026-07-27T09:35:00Z',
    stats: { critical: 1, warning: 2, info: 1 },
    markdown: [
      '# 设备与运维综合项目审核报告',
      '',
      '> 任务编号: `t-5` | 生成时间: 2026-07-27 09:35',
      '',
      '## 一、审核概览',
      '',
      '本次审核共发现 **4** 个问题。',
      '',
      '## 二、严重问题',
      '',
      '### 2.1 将货物与服务混同招标',
      '',
      '- **法规依据**:`《政府采购法实施条例》第二十四条`',
      '- **修改建议**:硬件采购与运维服务应拆分为两个独立标段。',
      '',
      '## 三、警告问题',
      '',
      '### 3.1 样品要求过严',
      '',
      '- **法规依据**:`《政府采购法》第二十二条`',
      '- **修改建议**:允许功能等同或原厂授权样品。',
      '',
      '### 3.2 资质门槛与项目规模不匹配',
      '',
      '- **修改建议**:删除与项目无关的高级资质条款。',
      '',
      '## 四、提示问题',
      '',
      '- **验收标准描述笼统**:补充详细验收清单与抽样比例。',
    ].join('\n'),
  },
  't-7': {
    id: 'r-7',
    taskId: 't-7',
    title: '单一来源采购项目审核报告',
    createdAt: '2026-07-25T16:05:00Z',
    stats: { critical: 1, warning: 0, info: 1 },
    markdown: [
      '# 单一来源采购项目审核报告',
      '',
      '> 任务编号: `t-7` | 生成时间: 2026-07-25 16:05',
      '',
      '## 一、审核概览',
      '',
      '本次审核共发现 **2** 个问题。',
      '',
      '## 二、严重问题',
      '',
      '### 2.1 单一来源采购依据不足',
      '',
      '- **法规依据**:`《政府采购法》第三十一条`',
      '- **修改建议**:补充不少于 3 家供应商的调研对比表。',
      '',
      '## 四、提示问题',
      '',
      '- **合同模板未引用**:在附件中提供合同范本。',
    ].join('\n'),
  },
};

const SAMPLE_TASKS: AuditTask[] = [
  {
    id: 't-1',
    fileName: '某市政项目招标文件.pdf',
    fileSize: 1_245_000,
    status: 'done',
    createdAt: '2026-07-30T15:30:00Z',
    findingsCount: 6,
  },
  {
    id: 't-2',
    fileName: '信息系统采购项目.pdf',
    fileSize: 2_312_000,
    status: 'done',
    createdAt: '2026-07-29T11:20:00Z',
    findingsCount: 4,
  },
  {
    id: 't-3',
    fileName: '监理服务招标公告.pdf',
    fileSize: 645_000,
    status: 'running',
    createdAt: '2026-07-31T08:00:00Z',
    findingsCount: 0,
  },
  {
    id: 't-4',
    fileName: '工程总承包招标文件.pdf',
    fileSize: 3_580_000,
    status: 'done',
    createdAt: '2026-07-28T14:00:00Z',
    findingsCount: 3,
  },
  {
    id: 't-5',
    fileName: '设备与运维综合项目.pdf',
    fileSize: 1_980_000,
    status: 'done',
    createdAt: '2026-07-27T09:25:00Z',
    findingsCount: 4,
  },
  {
    id: 't-6',
    fileName: '软件开发服务采购.pdf',
    fileSize: 1_120_000,
    status: 'pending',
    createdAt: '2026-07-31T16:40:00Z',
    findingsCount: 0,
  },
  {
    id: 't-7',
    fileName: '单一来源采购项目.pdf',
    fileSize: 780_000,
    status: 'done',
    createdAt: '2026-07-25T15:50:00Z',
    findingsCount: 2,
  },
  {
    id: 't-8',
    fileName: '医疗设备紧急采购.pdf',
    fileSize: 920_000,
    status: 'failed',
    createdAt: '2026-07-24T10:15:00Z',
    findingsCount: 0,
  },
  {
    id: 't-9',
    fileName: '物业管理服务招标.pdf',
    fileSize: 540_000,
    status: 'pending',
    createdAt: '2026-07-31T17:05:00Z',
    findingsCount: 0,
  },
];

const wait = (ms: number): Promise<void> =>
  new Promise((resolve) => setTimeout(resolve, ms));

let nextTaskSeq = 100;
let nextReportSeq = 100;

const randomId = (prefix: string, seq: number): string => `${prefix}-${seq}`;

const buildMarkdownFromFindings = (findings: Finding[], taskId: string): string => {
  const criticals = findings.filter((f) => f.severity === 'critical');
  const warnings = findings.filter((f) => f.severity === 'warning');
  const infos = findings.filter((f) => f.severity === 'info');
  const lines: string[] = [
    '# 招标文件审核报告',
    '',
    '> 任务编号: `' + taskId + '` | 生成时间: ' + new Date().toLocaleString('zh-CN'),
    '',
    '## 一、审核概览',
    '',
    '本次审核共发现 **' + findings.length + '** 个问题,其中:',
    '',
    '| 严重程度 | 数量 |',
    '| --- | ---: |',
    '| 严重 (critical) | ' + criticals.length + ' |',
    '| 警告 (warning) | ' + warnings.length + ' |',
    '| 提示 (info) | ' + infos.length + ' |',
  ];
  if (criticals.length > 0) {
    lines.push('', '## 二、严重问题', '');
    criticals.forEach((f, i) => {
      lines.push('### 2.' + (i + 1) + ' ' + f.title, '');
      lines.push('- **位置**:第 ' + f.page + ' 页');
      if (f.lawRef) lines.push('- **法规依据**:`' + f.lawRef + '`');
      lines.push('- **风险描述**:' + f.description);
      if (f.suggestion) lines.push('- **修改建议**:' + f.suggestion);
      lines.push('');
    });
  }
  if (warnings.length > 0) {
    lines.push('## 三、警告问题', '');
    warnings.forEach((f, i) => {
      lines.push('### 3.' + (i + 1) + ' ' + f.title, '');
      lines.push('- **位置**:第 ' + f.page + ' 页');
      if (f.lawRef) lines.push('- **法规依据**:`' + f.lawRef + '`');
      lines.push('- **风险描述**:' + f.description);
      if (f.suggestion) lines.push('- **修改建议**:' + f.suggestion);
      lines.push('');
    });
  }
  if (infos.length > 0) {
    lines.push('## 四、提示问题', '');
    lines.push('| 编号 | 标题 | 位置 | 建议 |');
    lines.push('| --- | --- | --- | --- |');
    infos.forEach((f, i) => {
      lines.push('| ' + (i + 1) + ' | ' + f.title + ' | 第 ' + f.page + ' 页 | ' + (f.suggestion ?? '-') + ' |');
    });
  }
  lines.push('', '## 五、总结', '', '本次审核共发现 **' + criticals.length + '** 个严重问题,建议在发布前完成修改。');
  return lines.join('\n');
};

const statsOf = (findings: Finding[]): { critical: number; warning: number; info: number } => ({
  critical: findings.filter((f) => f.severity === 'critical').length,
  warning: findings.filter((f) => f.severity === 'warning').length,
  info: findings.filter((f) => f.severity === 'info').length,
});

export const mockApi = {
  async login(email: string, password: string): Promise<{ token: string; userId: string; name: string; email: string }> {
    await wait(300);
    if (!email || !password) {
      throw new Error('邮箱或密码不能为空');
    }
    return {
      token: `mock-jwt-token-${Date.now()}`,
      userId: 'u-1',
      name: '审核员',
      email,
    };
  },

  async listTasks(): Promise<AuditTask[]> {
    await wait(200);
    return [...SAMPLE_TASKS];
  },

  async getTask(id: string): Promise<AuditTask | null> {
    await wait(150);
    return SAMPLE_TASKS.find((t) => t.id === id) ?? null;
  },

  async createTask(file: { name: string; size: number }): Promise<AuditTask> {
    await wait(400);
    const id = randomId('t', nextTaskSeq++);
    const task: AuditTask = {
      id,
      fileName: file.name,
      fileSize: file.size,
      status: 'pending',
      createdAt: new Date().toISOString(),
      findingsCount: 0,
    };
    SAMPLE_TASKS.unshift(task);
    return task;
  },

  async listFindings(taskId: string): Promise<Finding[]> {
    await wait(250);
    return [...(FINDINGS_BY_TASK[taskId] ?? [])];
  },

  async getReport(reportId: string): Promise<AuditReport> {
    await wait(200);
    const found = Object.values(REPORTS_BY_TASK).find((r) => r.id === reportId);
    if (found) return found;
    if (REPORTS_BY_TASK[reportId]) {
      return REPORTS_BY_TASK[reportId];
    }
    throw new Error('报告不存在');
  },

  async getReportIdByTaskId(taskId: string): Promise<string> {
    await wait(100);
    if (REPORTS_BY_TASK[taskId]) return REPORTS_BY_TASK[taskId].id;
    const created = await this.createReport(taskId);
    return created.id;
  },

  async getReportByTask(taskId: string): Promise<AuditReport> {
    await wait(200);
    const cached = REPORTS_BY_TASK[taskId];
    if (cached) return cached;
    const findings = FINDINGS_BY_TASK[taskId] ?? [];
    return {
      id: randomId('r', nextReportSeq++),
      taskId,
      title: '招标文件审核报告',
      createdAt: new Date().toISOString(),
      stats: statsOf(findings),
      markdown: buildMarkdownFromFindings(findings, taskId),
    };
  },

  async createReport(taskId: string): Promise<AuditReport> {
    await wait(500);
    const findings = FINDINGS_BY_TASK[taskId] ?? [];
    const report: AuditReport = {
      id: randomId('r', nextReportSeq++),
      taskId,
      title: '招标文件审核报告',
      createdAt: new Date().toISOString(),
      stats: statsOf(findings),
      markdown: buildMarkdownFromFindings(findings, taskId),
    };
    REPORTS_BY_TASK[taskId] = report;
    return report;
  },
};

export type MockApi = typeof mockApi;
