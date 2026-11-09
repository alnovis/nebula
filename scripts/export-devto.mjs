#!/usr/bin/env node
// Export a blog post for dev.to: inline SVG figures become PNGs, site-only HTML becomes dev.to markdown.
//
// Usage: node scripts/export-devto.mjs <slug> [--out DIR] [--base-url URL]
//
// Figures are linked where publish-figures.mjs uploads them (override with --base-url);
// local copies go to images/. The cover is rendered to images/ for the dev.to cover upload.
// Needs Google Chrome (override with CHROME).

import { readFileSync, writeFileSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import {
    ROOT, FIGURES_URL, FIGURE_RE, parseFrontMatter, extractAssets, parseFigure, figureName,
    rootCss, renderFigure, screenshot,
} from './figures.mjs';

const SITE = 'https://alnovis.io';

function parseArgs(argv) {
    const args = { slug: null, out: null, baseUrl: FIGURES_URL };
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
    const { meta, body: raw } = parseFrontMatter(readFileSync(src, 'utf8'));
    const { body, css, defs } = extractAssets(raw);
    const assets = { css, defs, rootCss: rootCss() };

    const imgDir = join(args.out, 'images');
    mkdirSync(imgDir, { recursive: true });
    const work = mkdtempSync(join(tmpdir(), 'nebula-devto-'));

    try {
        let n = 0;
        let md = body.replace(FIGURE_RE, (whole, inner) => {
            const fig = parseFigure(inner);
            if (!fig) return whole;
            const name = figureName(args.slug, fig.svg);
            renderFigure(fig, assets, join(imgDir, name), work);
            n++;

            let out = `![${escapeAlt(fig.alt)}](${args.baseUrl}/${name})\n`;
            if (fig.caption) out += fig.caption.includes('*') ? `\n${fig.caption}\n` : `\n*${fig.caption}*\n`;
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
            screenshot(`file://${html}`, 1500, 630, join(imgDir, cover), join(work, 'chrome'));
        }

        md = md
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
            ...(cover ? [`cover_image: images/${cover}`] : []),
            '---',
        ].join('\n');

        const mdPath = join(args.out, `${args.slug}.md`);
        writeFileSync(mdPath, `${fm}\n\n${md}\n`);
        console.log(`${n} figures${cover ? ' + cover' : ''} -> ${imgDir}`);
        console.log(`markdown -> ${mdPath}`);
        const leftover = md.match(/<(svg|style|figure|figcaption)\b/g);
        if (leftover) console.warn(`not converted: ${[...new Set(leftover)].join(' ')}`);
    } finally {
        rmSync(work, { recursive: true, force: true });
    }
}

main();
