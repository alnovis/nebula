#!/usr/bin/env node
// Export a blog post for dev.to: inline SVG figures become PNGs, site-only HTML becomes dev.to markdown.
//
// Usage: node scripts/export-devto.mjs <slug> [--out DIR] [--base-url URL]
//
// Without --base-url the images are referenced as images/<name>.png; upload them
// in the dev.to editor and replace the links. Needs Google Chrome (override with CHROME).

import { readFileSync, writeFileSync, mkdirSync, mkdtempSync, rmSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const SITE = 'https://alnovis.io';
const CHROME = process.env.CHROME || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const PAD = 24;

function parseArgs(argv) {
    const args = { slug: null, out: null, baseUrl: null };
    for (let i = 0; i < argv.length; i++) {
        if (argv[i] === '--out') args.out = argv[++i];
        else if (argv[i] === '--base-url') args.baseUrl = argv[++i].replace(/\/$/, '');
        else args.slug = argv[i];
    }
    if (!args.slug) {
        console.error('Usage: node scripts/export-devto.mjs <slug> [--out DIR] [--base-url URL]');
        process.exit(1);
    }
    args.out = resolve(args.out || join(ROOT, 'target', 'devto', args.slug));
    return args;
}

function parseFrontMatter(md) {
    const m = md.match(/^---\n([\s\S]*?)\n---\n/);
    if (!m) throw new Error('no front matter');
    const meta = {};
    for (const line of m[1].split('\n')) {
        const kv = line.match(/^(\w+):\s*(.*)$/);
        if (!kv) continue;
        const raw = kv[2].trim();
        meta[kv[1]] = raw.startsWith('[') ? JSON.parse(raw) : raw.replace(/^"(.*)"$/, '$1');
    }
    return { meta, body: md.slice(m[0].length) };
}

// Cut the first match out of the body and return it.
function take(state, re) {
    const m = state.body.match(re);
    if (!m) return '';
    state.body = state.body.replace(m[0], '');
    return m[0];
}

function screenshot(url, width, height, png, profile) {
    execFileSync(CHROME, [
        '--headless=new', '--disable-gpu', '--hide-scrollbars',
        `--user-data-dir=${profile}`,
        '--force-device-scale-factor=2',
        `--window-size=${width},${height}`,
        `--screenshot=${png}`,
        url,
    ], { stdio: 'ignore' });
    if (!existsSync(png)) throw new Error(`Chrome wrote no ${png}`);
}

function escapeAlt(s) {
    return s.replace(/[\[\]]/g, '\\$&').replace(/\s+/g, ' ').trim();
}

function devtoTags(tags) {
    const clean = tags.map(t => t.toLowerCase().replace(/[^a-z0-9]/g, ''));
    if (clean.length > 4) console.warn(`dev.to takes 4 tags, dropped: ${clean.slice(4).join(', ')}`);
    return clean.slice(0, 4);
}

function main() {
    const args = parseArgs(process.argv.slice(2));
    const src = join(ROOT, 'content', 'blog', `${args.slug}.md`);
    const { meta, body } = parseFrontMatter(readFileSync(src, 'utf8'));
    const state = { body };

    const css = take(state, /<style>[\s\S]*?<\/style>\n?/);
    const defs = take(state, /<svg width="0" height="0"[^>]*>[\s\S]*?<\/svg>\n?/);
    const rootCss = readFileSync(join(ROOT, 'static', 'css', 'style.css'), 'utf8').match(/:root\s*\{[^}]*\}/)[0];

    const imgDir = join(args.out, 'images');
    mkdirSync(imgDir, { recursive: true });
    const work = mkdtempSync(join(tmpdir(), 'nebula-devto-'));
    const profile = join(work, 'chrome');
    const link = name => (args.baseUrl ? `${args.baseUrl}/${name}` : `images/${name}`);

    try {
        let n = 0;
        state.body = state.body.replace(/<figure[^>]*>\s*([\s\S]*?)<\/figure>/g, (_, inner) => {
            const svg = inner.match(/<svg[\s\S]*<\/svg>/);
            if (!svg) return _;
            const [, vbW, vbH] = svg[0].match(/viewBox="[\d.-]+ [\d.-]+ ([\d.]+) ([\d.]+)"/).map(Number);
            const alt = (svg[0].match(/aria-label="([^"]*)"/) || [, ''])[1];
            const caption = (inner.match(/<figcaption[^>]*>([\s\S]*?)<\/figcaption>/) || [, ''])[1].trim();

            const name = `${args.slug}-fig-${String(++n).padStart(2, '0')}.png`;
            const html = join(work, `fig-${n}.html`);
            writeFileSync(html, `<!doctype html><meta charset="utf-8">
<style>${rootCss}
html,body{margin:0;background:var(--color-bg)}
.fig{padding:${PAD}px;width:${vbW}px}
.fig svg{display:block;width:100%;height:auto}</style>
${css}${defs}<div class="fig">${svg[0]}</div>`);
            screenshot(`file://${html}`, vbW + 2 * PAD, Math.ceil(vbH) + 2 * PAD, join(imgDir, name), profile);

            let out = `![${escapeAlt(alt)}](${link(name)})\n`;
            if (caption) out += caption.includes('*') ? `\n${caption}\n` : `\n*${caption}*\n`;
            return out;
        });

        // dev.to shows covers at 1000x420; pad ours to that ratio instead of letting it crop the titles
        let cover = null;
        if (meta.cover_image) {
            const svgPath = join(ROOT, 'static', 'images', meta.cover_image);
            const html = join(work, 'cover.html');
            writeFileSync(html, `<!doctype html><body style="margin:0;background:#0f0f0f;display:flex;justify-content:center">
<img src="file://${svgPath}" width="1200" height="630"></body>`);
            cover = `${args.slug}-cover.png`;
            screenshot(`file://${html}`, 1500, 630, join(imgDir, cover), profile);
        }

        state.body = state.body
            .replace(/<details[^>]*>\s*<summary>([\s\S]*?)<\/summary>/g, (_, s) => `{% details ${s.trim()} %}`)
            .replace(/<\/details>/g, '{% enddetails %}')
            .replace(/\]\(\//g, `](${SITE}/`)
            .replace(/href="\//g, `href="${SITE}/`)
            .replace(/\n{3,}/g, '\n\n')
            .trim();

        const fm = [
            '---',
            `title: ${JSON.stringify(meta.title)}`,
            'published: false',
            `description: ${JSON.stringify(meta.description)}`,
            `tags: ${devtoTags(meta.tags || []).join(', ')}`,
            `canonical_url: ${SITE}/blog/${args.slug}`,
            ...(cover ? [`cover_image: ${link(cover)}`] : []),
            '---',
        ].join('\n');

        const mdPath = join(args.out, `${args.slug}.md`);
        writeFileSync(mdPath, `${fm}\n\n${state.body}\n`);
        console.log(`${n} figures${cover ? ' + cover' : ''} -> ${imgDir}`);
        console.log(`markdown -> ${mdPath}`);
        const leftover = state.body.match(/<(svg|style|figure|figcaption)\b/g);
        if (leftover) console.warn(`not converted: ${[...new Set(leftover)].join(' ')}`);
    } finally {
        rmSync(work, { recursive: true, force: true });
    }
}

main();
