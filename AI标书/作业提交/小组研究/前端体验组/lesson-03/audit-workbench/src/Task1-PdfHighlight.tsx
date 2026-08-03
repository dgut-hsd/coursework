import { useState } from 'react'
import { Document, Page, pdfjs } from 'react-pdf'

// react-pdf worker 配置（pdfjs v4 用 .mjs）
pdfjs.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.mjs'

// ── 类型 ──
interface BBox {
  id: string
  clause_id: string
  page: number
  x: number      // 左上角 X（CSS px，基于 width=600 的坐标系）
  y: number      // 左上角 Y
  w: number      // 宽
  h: number      // 高
}

// ── Mock bbox 数据：模拟讲师提供的坐标 JSON ──
const BBOXES: BBox[] = [
  { id: '1', clause_id: 'CLAUSE-001', page: 1, x: 60, y: 80, w: 480, h: 30 },
  { id: '2', clause_id: 'CLAUSE-002', page: 1, x: 60, y: 140, w: 400, h: 25 },
  { id: '3', clause_id: 'CLAUSE-003', page: 2, x: 50, y: 100, w: 450, h: 28 },
  { id: '4', clause_id: 'CLAUSE-004', page: 3, x: 70, y: 60, w: 420, h: 32 },
]

const PAGE_WIDTH = 600

export default function Task1PdfHighlight() {
  const [numPages, setNumPages] = useState<number>(0)
  const [selected, setSelected] = useState<string | null>(null)

  return (
    <div style={{ padding: 24 }}>
      <h2>任务 1：react-pdf + bbox 高亮覆盖层</h2>
      <p>点击高亮矩形 → console 打印 clause_id（页面上也会显示）</p>
      {selected && (
        <p style={{ color: '#cf1322', fontWeight: 'bold' }}>
          选中: {selected}
        </p>
      )}

      <Document
        file="/sample.pdf"
        onLoadSuccess={({ numPages }) => setNumPages(numPages)}
        loading={<p>加载 PDF...</p>}
      >
        {Array.from({ length: numPages }, (_, i) => i + 1).map(pageNum => (
          <div
            key={pageNum}
            style={{ position: 'relative', marginBottom: 20 }}
          >
            {/* react-pdf 渲染 Canvas */}
            <Page pageNumber={pageNum} width={PAGE_WIDTH} />

            {/* bbox 高亮覆盖层：绝对定位在 Page 上方 */}
            {BBOXES
              .filter(b => b.page === pageNum)
              .map(b => (
                <div
                  key={b.id}
                  onClick={() => {
                    console.log('clause_id:', b.clause_id)
                    setSelected(b.clause_id)
                  }}
                  style={{
                    position: 'absolute',
                    left: b.x,
                    top: b.y,
                    width: b.w,
                    height: b.h,
                    background: 'rgba(255, 77, 79, 0.15)',
                    border: '1px solid #ff4d4f',
                    cursor: 'pointer',
                    pointerEvents: 'auto',
                  }}
                >
                  <span style={{ fontSize: 10, color: '#cf1322' }}>
                    {b.clause_id}
                  </span>
                </div>
              ))}
          </div>
        ))}
      </Document>
    </div>
  )
}
