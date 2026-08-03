import { Card, Tag, Space, Typography } from 'antd'
import { createStyles } from 'antd-style'

const { Text } = Typography

const useStyles = createStyles(({ token, css }) => ({
  card: css`
    border: 1px solid ${token.colorBorderSecondary};
    border-radius: ${token.borderRadiusLG}px;
    padding: ${token.paddingMD}px;
    margin-bottom: 12px;
    &:hover {
      border-color: ${token.colorPrimary};
      box-shadow: 0 2px 8px ${token.colorPrimaryBg};
    }
  `,
  critical: css`
    border-left: 4px solid ${token.colorError};
    background: ${token.colorErrorBg};
  `,
  warning: css`
    border-left: 4px solid ${token.colorWarning};
    background: ${token.colorWarningBg};
  `,
  info: css`
    border-left: 4px solid ${token.colorPrimary};
    background: ${token.colorPrimaryBg};
  `,
  title: css`
    font-weight: 600;
    margin-bottom: 4px;
  `,
  desc: css`
    color: ${token.colorTextSecondary};
    font-size: 14px;
  `,
}))

interface Issue {
  id: string
  title: string
  description: string
  severity: 'critical' | 'warning' | 'info'
}

const mockIssues: Issue[] = [
  { id: '1', title: '投标报价超过限价', description: '报价金额超出招标文件规定的最高限价', severity: 'critical' },
  { id: '2', title: '缺少资质证书', description: '未附上安全生产许可证', severity: 'warning' },
  { id: '3', title: '格式建议调整', description: '目录页码建议连续编号', severity: 'info' },
]

function IssueCard({ issue }: { issue: Issue }) {
  const { styles, cx } = useStyles()
  const sevMap = {
    critical: { label: '严重', color: 'error' as const },
    warning: { label: '警告', color: 'warning' as const },
    info: { label: '信息', color: 'processing' as const },
  }
  const sev = sevMap[issue.severity]

  return (
    <div className={cx(styles.card, styles[issue.severity])}>
      <div className={styles.title}>
        <Space>
          {issue.title}
          <Tag color={sev.color}>{sev.label}</Tag>
        </Space>
      </div>
      <div className={styles.desc}>{issue.description}</div>
    </div>
  )
}

const cssModulesCode = `/* CSS Modules 方式 —— 需要单独维护 .module.css 文件，无类型安全 */
.card { border: 1px solid #d9d9d9; border-radius: 8px; padding: 16px; }
.card:hover { border-color: #1677ff; }
.critical { border-left: 4px solid #ff4d4f; background: #fff2f0; }
/* 暗色模式需额外写一套 */`

export default function Task3IssueCard() {
  return (
    <div>
      <p>antd-style createStyles 重写 IssueCard（critical=红 / warning=金 / info=蓝）：</p>
      {mockIssues.map(i => <IssueCard key={i.id} issue={i} />)}

      <Card title="对比：CSS Modules 方式" size="small" style={{ marginTop: 16 }}>
        <pre style={{ fontSize: 12, overflow: 'auto' }}>{cssModulesCode}</pre>
        <Text type="secondary">
          antd-style 优势：token 类型安全、暗色模式自动跟随、无需单独 .module.css
        </Text>
      </Card>
    </div>
  )
}
