#!/usr/bin/env node
// Render the inline SVG figures of all blog posts to PNG, dark and light, and upload the ones Cloudinary
// does not have yet. The RSS feed links them by the same names (src/content/feed.rs).
//
// Usage: node scripts/publish-figures.mjs            (needs CLOUDINARY_CLOUD_NAME, _API_KEY, _API_SECRET)
//        node scripts/publish-figures.mjs --dry-run DIR   (render every figure into DIR, upload nothing)

import { readFileSync, readdirSync, mkdirSync, mkdtempSync, rmSync, copyFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { join, basename } from 'node:path';
import { tmpdir } from 'node:os';
import {
    ROOT, THEMES, FIGURE_RE, parseFrontMatter, extractAssets, parseFigure, figureName, rootCss, renderFigure,
} from './figures.mjs';

const PREFIX = 'nebula/figures';

function collectFigures() {
    const dir = join(ROOT, 'content', 'blog');
    const css = rootCss();
    const figures = [];
    for (const file of readdirSync(dir).filter(f => f.endsWith('.md')).sort()) {
        const slug = basename(file, '.md');
        const { body: raw } = parseFrontMatter(readFileSync(join(dir, file), 'utf8'));
        const { body, css: postCss, defs } = extractAssets(raw);
        for (const [, inner] of body.matchAll(FIGURE_RE)) {
            const fig = parseFigure(inner);
            if (!fig) continue;
            for (const theme of Object.keys(THEMES)) {
                figures.push({ ...fig, theme, name: figureName(slug, fig.svg, theme), assets: { css: postCss, defs, rootCss: css } });
            }
        }
    }
    return figures;
}

async function existing(cloud, auth) {
    const ids = new Set();
    let cursor = null;
    do {
        const url = new URL(`https://api.cloudinary.com/v1_1/${cloud}/resources/image/upload`);
        url.searchParams.set('prefix', `${PREFIX}/`);
        url.searchParams.set('max_results', '500');
        if (cursor) url.searchParams.set('next_cursor', cursor);
        const res = await fetch(url, { headers: { Authorization: auth } });
        if (!res.ok) throw new Error(`Cloudinary list: ${res.status} ${await res.text()}`);
        const json = await res.json();
        for (const r of json.resources) ids.add(r.public_id);
        cursor = json.next_cursor;
    } while (cursor);
    return ids;
}

async function upload(cloud, key, secret, png, publicId) {
    const timestamp = Math.floor(Date.now() / 1000);
    const signature = createHash('sha1').update(`public_id=${publicId}&timestamp=${timestamp}${secret}`).digest('hex');
    const form = new FormData();
    form.set('file', new Blob([readFileSync(png)], { type: 'image/png' }), basename(png));
    form.set('public_id', publicId);
    form.set('timestamp', String(timestamp));
    form.set('api_key', key);
    form.set('signature', signature);
    const res = await fetch(`https://api.cloudinary.com/v1_1/${cloud}/image/upload`, { method: 'POST', body: form });
    if (!res.ok) throw new Error(`Cloudinary upload of ${publicId}: ${res.status} ${await res.text()}`);
}

async function main() {
    const dryRun = process.argv[2] === '--dry-run' ? process.argv[3] : null;
    if (process.argv[2] === '--dry-run' && !dryRun) {
        console.error('Usage: node scripts/publish-figures.mjs [--dry-run DIR]');
        process.exit(1);
    }
    const figures = collectFigures();

    let todo = figures;
    let cloud, key, secret;
    if (!dryRun) {
        ({ CLOUDINARY_CLOUD_NAME: cloud, CLOUDINARY_API_KEY: key, CLOUDINARY_API_SECRET: secret } = process.env);
        if (!cloud || !key || !secret) throw new Error('CLOUDINARY_CLOUD_NAME, CLOUDINARY_API_KEY and CLOUDINARY_API_SECRET must be set');
        const have = await existing(cloud, `Basic ${Buffer.from(`${key}:${secret}`).toString('base64')}`);
        todo = figures.filter(f => !have.has(`${PREFIX}/${basename(f.name, '.png')}`));
    }
    console.log(`${figures.length} figures, ${todo.length} to ${dryRun ? 'render' : 'upload'}`);

    const work = mkdtempSync(join(tmpdir(), 'nebula-figures-'));
    try {
        if (dryRun) mkdirSync(dryRun, { recursive: true });
        for (const fig of todo) {
            const png = join(work, fig.name);
            renderFigure(fig, fig.assets, png, work, fig.theme);
            if (dryRun) {
                copyFileSync(png, join(dryRun, fig.name));
            } else {
                await upload(cloud, key, secret, png, `${PREFIX}/${basename(fig.name, '.png')}`);
            }
            console.log(fig.name);
        }
    } finally {
        rmSync(work, { recursive: true, force: true });
    }
}

main().catch(e => {
    console.error(e.message);
    process.exit(1);
});
