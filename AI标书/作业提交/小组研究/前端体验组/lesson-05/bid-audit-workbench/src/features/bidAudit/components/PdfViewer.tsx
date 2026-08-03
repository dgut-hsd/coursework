import { useState, useEffect, useRef } from 'react';
import type { JSX } from 'react';
import { Document, Page, pdfjs } from 'react-pdf';
import { Spin, Alert, App as AntApp } from 'antd';
import { BboxOverlay } from './BboxOverlay';
import type { Finding } from '../types';
import 'react-pdf/dist/Page/AnnotationLayer.css';
import 'react-pdf/dist/Page/TextLayer.css';

pdfjs.GlobalWorkerOptions.workerSrc = `https://unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;

interface PdfViewerProps {
  fileUrl: string | null;
  fileName: string;
  findings: Finding[];
  activeFindingId: string | null;
  onSelect: (finding: Finding) => void;
  pageWidth?: number;
}

function MockPdfPage({
  pageNumber,
  width,
  height,
  children,
}: {
  pageNumber: number;
  width: number;
  height: number;
  children: React.ReactNode;
}): JSX.Element {
  return (
    <div
      className="pdf-page"
      style={{
        width,
        height,
        padding: '40px 50px',
        fontSize: 13,
        lineHeight: 1.7,
        color: '#333',
        textAlign: 'left',
        position: 'relative',
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: 12,
          right: 18,
          fontSize: 11,
          color: '#999',
        }}
      >
        第 {pageNumber} 页
      </div>
      {children}
    </div>
  );
}

const MOCK_PAGES: ReadonlyArray<{ title: string; paragraphs: ReadonlyArray<string> }> = [
  {
    title: '一、招标项目概况',
    paragraphs: [
      '1.1 项目名称:某市政基础设施建设项目',
      '1.2 项目预算:人民币壹拾万元整(¥100,000.00)',
      '1.3 投标人资格要求:在中华人民共和国境内注册,并在深圳市设立总部的独立法人。',
      '1.4 招标人:某市住建局',
      '1.5 项目地点:深圳市南山区',
    ],
  },
  {
    title: '二、技术要求',
    paragraphs: [
      '2.1 主要设备:必须使用 XX 品牌 XX 型号的变频器,功率不小于 50kW。',
      '2.2 监控系统:采用不低于 1080P 的高清摄像头,支持夜视功能。',
      '2.3 技术方案:投标人应提供完整的技术方案,包括系统架构图、网络拓扑图等。',
      '2.4 商务评分:丰富的行业经验(0-10 分),优秀的项目案例(0-15 分)。',
    ],
  },
  {
    title: '三、合同条款',
    paragraphs: [
      '3.1 投标保证金:人民币壹拾万元整(¥100000.00),须在投标截止时间前缴纳。',
      '3.2 履约期限:合同签订后开始实施。',
      '3.3 验收标准:按照国家相关标准及招标文件要求进行验收。',
      '3.4 付款方式:合同签订后预付 30%,验收合格后支付 65%,质保期满后支付 5%。',
    ],
  },
];

export function PdfViewer({
  fileUrl,
  fileName,
  findings,
  activeFindingId,
  onSelect,
  pageWidth = 720,
}: PdfViewerProps): JSX.Element {
  const { message } = AntApp.useApp();
  const [numPages, setNumPages] = useState<number>(0);
  const [error, setError] = useState<string | null>(null);
  const pageRefs = useRef<Record<number, HTMLDivElement | null>>({});
  const containerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!activeFindingId) return;
    const finding = findings.find((f) => f.id === activeFindingId);
    if (!finding) return;
    const el = pageRefs.current[finding.page];
    if (el && containerRef.current) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }, [activeFindingId, findings]);

  const onLoadSuccess = ({ numPages: n }: { numPages: number }): void => {
    setNumPages(n);
    setError(null);
  };

  const onLoadError = (err: Error): void => {
    setError(err.message);
    message.error('PDF 加载失败,将使用模拟视图');
  };

  if (fileUrl) {
    return (
      <div className="pdf-scroll-container" ref={containerRef}>
        {error && (
          <Alert
            type="warning"
            message="PDF 加载失败"
            description={error}
            showIcon
            style={{ marginBottom: 16 }}
          />
        )}
        <Document
          file={fileUrl}
          onLoadSuccess={onLoadSuccess}
          onLoadError={onLoadError}
          loading={<Spin />}
        >
          {Array.from({ length: numPages }, (_, i) => i + 1).map((pageNum) => {
            const pageFindings = findings.filter((f) => f.page === pageNum);
            return (
              <div
                key={pageNum}
                ref={(el) => {
                  pageRefs.current[pageNum] = el;
                }}
                style={{ position: 'relative', marginBottom: 16 }}
              >
                <Page
                  pageNumber={pageNum}
                  width={pageWidth}
                  renderTextLayer
                  renderAnnotationLayer={false}
                />
                <PageBboxOverlay
                  findings={pageFindings}
                  pageWidth={pageWidth}
                  onSelect={onSelect}
                />
              </div>
            );
          })}
        </Document>
      </div>
    );
  }

  return (
    <div className="pdf-scroll-container" ref={containerRef}>
      <div
        style={{
          background: '#fffbe6',
          border: '1px solid #ffe58f',
          borderRadius: 4,
          padding: '8px 12px',
          fontSize: 12,
          color: '#614700',
          alignSelf: 'flex-start',
        }}
      >
        当前显示模拟 PDF (文件名: {fileName}),上传真实 PDF 后将自动切换至 react-pdf 渲染
      </div>
      {MOCK_PAGES.map((page, idx) => {
        const pageNum = idx + 1;
        const pageFindings = findings.filter((f) => f.page === pageNum);
        return (
          <div
            key={pageNum}
            ref={(el) => {
              pageRefs.current[pageNum] = el;
            }}
            style={{ position: 'relative' }}
          >
            <MockPdfPage
              pageNumber={pageNum}
              width={pageWidth}
              height={pageWidth * 1.3}
            >
              <h3
                style={{
                  fontSize: 16,
                  fontWeight: 600,
                  marginBottom: 16,
                  borderBottom: '1px solid #f0f0f0',
                  paddingBottom: 8,
                }}
              >
                {page.title}
              </h3>
              {page.paragraphs.map((p, i) => (
                <p key={i} style={{ margin: '8px 0' }}>
                  {p}
                </p>
              ))}
            </MockPdfPage>
            <PageBboxOverlay
              findings={pageFindings}
              pageWidth={pageWidth}
              onSelect={onSelect}
            />
          </div>
        );
      })}
    </div>
  );
}

interface PageBboxOverlayProps {
  findings: Finding[];
  pageWidth: number;
  onSelect: (finding: Finding) => void;
}

function PageBboxOverlay({
  findings,
  pageWidth,
  onSelect,
}: PageBboxOverlayProps): JSX.Element {
  if (findings.length === 0) return <></>;
  const NATIVE_WIDTH = 600;
  const NATIVE_HEIGHT = pageWidth * 1.3;
  return (
    <BboxOverlay
      bboxes={findings.map((f) => f.bbox)}
      severities={Object.fromEntries(
        findings.map((f, i) => [i, f.severity] as const),
      )}
      activeIndex={null}
      onSelect={(idx) => {
        const finding = findings[idx];
        if (finding) onSelect(finding);
      }}
      pageWidth={pageWidth - 100}
      pageHeight={NATIVE_HEIGHT - 80}
      pdfNativeWidth={NATIVE_WIDTH}
      pdfNativeHeight={NATIVE_HEIGHT}
    />
  );
}
