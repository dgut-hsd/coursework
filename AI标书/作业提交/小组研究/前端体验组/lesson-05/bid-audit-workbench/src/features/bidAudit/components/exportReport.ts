import { saveAs } from 'file-saver';
import jsPDF from 'jspdf';
import { asBlob } from 'html-docx-js-typescript';
import type { AuditReport } from '../types';

export type ExportFormat = 'pdf' | 'docx' | 'md';

export interface ExportOptions {
  fileName?: string;
  title: string;
}

export async function exportAsPDF(
  report: AuditReport,
  options: ExportOptions,
): Promise<void> {
  const doc = new jsPDF({
    unit: 'pt',
    format: 'a4',
  });
  const pageWidth = doc.internal.pageSize.getWidth();
  const pageHeight = doc.internal.pageSize.getHeight();
  const margin = 40;
  const contentWidth = pageWidth - margin * 2;

  let y = margin;

  const ensureSpace = (height: number): void => {
    if (y + height > pageHeight - margin) {
      doc.addPage();
      y = margin;
    }
  };

  doc.setFontSize(20);
  doc.setFont('helvetica', 'bold');
  doc.text(report.title, margin, y);
  y += 30;

  doc.setFontSize(10);
  doc.setFont('helvetica', 'normal');
  doc.setTextColor(120);
  doc.text(`Report ID: ${report.id} | Task: ${report.taskId}`, margin, y);
  y += 14;
  doc.text(
    `Created: ${new Date(report.createdAt).toLocaleString('en-US')}`,
    margin,
    y,
  );
  y += 24;

  doc.setTextColor(0);

  doc.setFontSize(12);
  doc.setFont('helvetica', 'bold');
  doc.text('Summary', margin, y);
  y += 18;
  doc.setFontSize(10);
  doc.setFont('helvetica', 'normal');
  doc.text(
    `Critical: ${report.stats.critical}  |  Warning: ${report.stats.warning}  |  Info: ${report.stats.info}`,
    margin,
    y,
  );
  y += 20;

  const lines = report.markdown.split('\n');
  for (const rawLine of lines) {
    const line = rawLine.replace(/\r$/, '');
    if (!line.trim()) {
      y += 6;
      continue;
    }

    if (line.startsWith('# ')) {
      ensureSpace(28);
      doc.setFontSize(18);
      doc.setFont('helvetica', 'bold');
      doc.text(line.slice(2), margin, y);
      y += 26;
    } else if (line.startsWith('## ')) {
      ensureSpace(22);
      doc.setFontSize(14);
      doc.setFont('helvetica', 'bold');
      doc.text(line.slice(3), margin, y);
      y += 20;
    } else if (line.startsWith('### ')) {
      ensureSpace(18);
      doc.setFontSize(12);
      doc.setFont('helvetica', 'bold');
      doc.text(line.slice(4), margin, y);
      y += 16;
    } else if (line.startsWith('- ')) {
      ensureSpace(14);
      doc.setFontSize(10);
      doc.setFont('helvetica', 'normal');
      const text = `• ${line.slice(2)}`;
      const wrapped = doc.splitTextToSize(text, contentWidth);
      for (const w of wrapped) {
        ensureSpace(12);
        doc.text(w, margin, y);
        y += 12;
      }
    } else if (line.startsWith('|')) {
      ensureSpace(12);
      doc.setFontSize(9);
      doc.setFont('helvetica', 'normal');
      const wrapped = doc.splitTextToSize(line, contentWidth);
      for (const w of wrapped) {
        ensureSpace(11);
        doc.text(w, margin, y);
        y += 11;
      }
    } else if (line.startsWith('> ')) {
      ensureSpace(14);
      doc.setFontSize(10);
      doc.setFont('helvetica', 'italic');
      const wrapped = doc.splitTextToSize(`"${line.slice(2)}"`, contentWidth - 12);
      for (const w of wrapped) {
        ensureSpace(12);
        doc.text(w, margin + 12, y);
        y += 12;
      }
    } else {
      ensureSpace(14);
      doc.setFontSize(10);
      doc.setFont('helvetica', 'normal');
      const wrapped = doc.splitTextToSize(line, contentWidth);
      for (const w of wrapped) {
        ensureSpace(12);
        doc.text(w, margin, y);
        y += 12;
      }
    }
  }

  const blob = doc.output('blob');
  const name = options.fileName ?? `audit-report-${report.id}.pdf`;
  saveAs(blob, name);
}

export async function exportAsDOCX(
  report: AuditReport,
  options: ExportOptions,
): Promise<void> {
  const html = renderMarkdownToHtml(report);
  const blob = await asBlob(html);
  const name = options.fileName ?? `audit-report-${report.id}.docx`;
  saveAs(blob, name);
}

export function exportAsMarkdown(
  report: AuditReport,
  options: ExportOptions,
): void {
  const blob = new Blob([report.markdown], {
    type: 'text/markdown;charset=utf-8',
  });
  const name = options.fileName ?? `audit-report-${report.id}.md`;
  saveAs(blob, name);
}

function renderMarkdownToHtml(report: AuditReport): string {
  const lines = report.markdown.split('\n');
  const out: string[] = [];
  let inList = false;
  let inTable = false;
  for (const rawLine of lines) {
    const line = rawLine.replace(/\r$/, '');
    if (line.startsWith('# ')) {
      if (inList) {
        out.push('</ul>');
        inList = false;
      }
      if (inTable) {
        out.push('</table>');
        inTable = false;
      }
      out.push(`<h1>${escapeHtml(line.slice(2))}</h1>`);
    } else if (line.startsWith('## ')) {
      if (inList) {
        out.push('</ul>');
        inList = false;
      }
      if (inTable) {
        out.push('</table>');
        inTable = false;
      }
      out.push(`<h2>${escapeHtml(line.slice(3))}</h2>`);
    } else if (line.startsWith('### ')) {
      if (inList) {
        out.push('</ul>');
        inList = false;
      }
      if (inTable) {
        out.push('</table>');
        inTable = false;
      }
      out.push(`<h3>${escapeHtml(line.slice(4))}</h3>`);
    } else if (line.startsWith('- ')) {
      if (!inList) {
        out.push('<ul>');
        inList = true;
      }
      if (inTable) {
        out.push('</table>');
        inTable = false;
      }
      out.push(`<li>${escapeHtml(line.slice(2))}</li>`);
    } else if (line.startsWith('|')) {
      if (inList) {
        out.push('</ul>');
        inList = false;
      }
      if (!inTable) {
        out.push('<table border="1" cellspacing="0" cellpadding="6">');
        inTable = true;
      }
      const cells = line.split('|').slice(1, -1).map((c) => c.trim());
      out.push('<tr>');
      for (const c of cells) {
        out.push(`<td>${escapeHtml(c)}</td>`);
      }
      out.push('</tr>');
    } else if (line.startsWith('> ')) {
      if (inList) {
        out.push('</ul>');
        inList = false;
      }
      if (inTable) {
        out.push('</table>');
        inTable = false;
      }
      out.push(`<blockquote>${escapeHtml(line.slice(2))}</blockquote>`);
    } else if (!line.trim()) {
      if (inList) {
        out.push('</ul>');
        inList = false;
      }
      if (inTable) {
        out.push('</table>');
        inTable = false;
      }
    } else {
      if (inList) {
        out.push('</ul>');
        inList = false;
      }
      if (inTable) {
        out.push('</table>');
        inTable = false;
      }
      out.push(`<p>${escapeHtml(line)}</p>`);
    }
  }
  if (inList) out.push('</ul>');
  if (inTable) out.push('</table>');

  return `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>${escapeHtml(report.title)}</title>
</head>
<body>
${out.join('\n')}
</body>
</html>`;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
