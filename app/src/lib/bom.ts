// SPDX-License-Identifier: MIT OR Apache-2.0

// The bill of materials, written out (B-11, design doc §6.2).
//
// The core says what to buy — how many, the value it is sold in, the part
// number, what the linter has to say about it — and this writes it as a
// spreadsheet takes it and as a page reads. The names are the palette's,
// because what a part is called is the app's (ADR 0008): the bill says
// "Resistor" where the palette does.

import type { BillLine } from './core'
import { describe } from './palette'

/** What a line's part is called: the palette's name for its kind. */
function nameOf(line: BillLine): string {
	return describe(line.kind)?.label ?? line.kind
}

/** One CSV field: quoted when it has to be, as RFC 4180 has it. */
function field(text: string): string {
	return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text
}

/**
 * The bill as CSV: a header and a row a line, notes joined, so a spreadsheet
 * opens it as it is. CRLF between rows, which is what RFC 4180 says and what
 * every spreadsheet reads.
 */
export function billToCsv(lines: readonly BillLine[]): string {
	const rows = [
		['Quantity', 'Part', 'Value', 'Part number', 'Datasheet', 'Notes'],
		...lines.map((line) => [
			String(line.quantity),
			nameOf(line),
			line.value,
			line.part ?? '',
			line.datasheet ?? '',
			line.notes.join(' '),
		]),
	]
	return rows.map((row) => row.map(field).join(',')).join('\r\n') + '\r\n'
}

/** One table cell: a pipe would end it, and a line break would end the row. */
function cell(text: string): string {
	return text.replaceAll('|', '\\|').replaceAll('\n', ' ')
}

/**
 * The bill as Markdown: a title, a table, and each line's notes under it, so
 * a warning reads as a sentence rather than squeezed into a cell.
 */
export function billToMarkdown(lines: readonly BillLine[], title: string): string {
	const out = [`# ${title.trim() || 'Parts list'}`, '']
	if (lines.length === 0) {
		out.push('Nothing to buy yet.', '')
		return out.join('\n')
	}
	out.push('| Qty | Part | Value | Part number |', '|---:|---|---|---|')
	for (const line of lines) {
		const number = line.part
			? line.datasheet
				? `[${cell(line.part)}](${line.datasheet})`
				: cell(line.part)
			: ''
		out.push(`| ${line.quantity} | ${cell(nameOf(line))} | ${cell(line.value)} | ${number} |`)
	}
	const noted = lines.filter((line) => line.notes.length > 0)
	if (noted.length > 0) {
		out.push('', '## Notes', '')
		for (const line of noted) {
			out.push(`**${nameOf(line)}, ${line.value}**`, '')
			for (const note of line.notes) out.push(`- ${note}`)
			out.push('')
		}
	}
	out.push('', '_Educational simulation — not accurate enough to design from._', '')
	return out.join('\n')
}
