// Inline SVG figures of a post: extraction, PNG names and rendering.
// Shared by export-devto.mjs and publish-figures.mjs; src/content/feed.rs derives the same names.

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

export const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const FIGURES_URL = 'https://res.cloudinary.com/ddkzhz9b4/image/upload/nebula/figures';

const CHROME = process.env.CHROME || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const PAD = 24;

// Dark is the site palette. Light is for white pages (feed readers, dev.to): small text keeps
// >= 5.5:1 against #ffffff, as docs/VISUAL_STYLE.md asks of the dark one.
export const THEMES = {
    dark: { suffix: '', tokens: '', colors: {} },
    light: {
        suffix: '-light',
        tokens: `:root{
--color-bg:#ffffff;--color-bg-secondary:#f4f4f5;--color-text:#18181b;--color-text-muted:#52525b;
--color-accent:#4f46e5;--color-accent-hover:#4338ca;--color-border:#d4d4d8;
--color-success:#15803d;--color-warning:#92400e}`,
        colors: { '#818cf8': '#4f46e5', '#4a4a4a': '#a1a1aa', '#c27d0e': '#b45309' },
    },
};

function recolor(markup, theme) {
    const colors = THEMES[theme].colors;
    return markup.replace(/#[0-9a-f]{6}\b/gi, c => colors[c.toLowerCase()] || c);
}

export function parseFrontMatter(md) {
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

// Split a post body into its figure stylesheet, shared <defs> and the rest.
export function extractAssets(body) {
    const style = body.match(/<style>[\s\S]*?<\/style>\n?/);
    const defs = body.match(/<svg width="0" height="0"[^>]*>[\s\S]*?<\/svg>\n?/);
    for (const m of [style, defs]) if (m) body = body.replace(m[0], '');
    return { body, css: style ? style[0] : '', defs: defs ? defs[0] : '' };
}

export const FIGURE_RE = /<figure[^>]*>\s*([\s\S]*?)<\/figure>/g;

// Figure markup -> its SVG and metadata, or null for a figure without SVG.
export function parseFigure(inner) {
    const svg = inner.match(/<svg[\s\S]*<\/svg>/);
    if (!svg) return null;
    const [, vbW, vbH] = svg[0].match(/viewBox="[\d.-]+ [\d.-]+ ([\d.]+) ([\d.]+)"/).map(Number);
    return {
        svg: svg[0],
        vbW,
        vbH,
        alt: (svg[0].match(/aria-label="([^"]*)"/) || [, ''])[1],
        caption: (inner.match(/<figcaption[^>]*>([\s\S]*?)<\/figcaption>/) || [, ''])[1].trim(),
    };
}

export function figureName(slug, svg, theme) {
    return `${slug}-${createHash('sha256').update(svg).digest('hex').slice(0, 12)}${THEMES[theme].suffix}.png`;
}

export function rootCss() {
    return readFileSync(join(ROOT, 'static', 'css', 'style.css'), 'utf8').match(/:root\s*\{[^}]*\}/)[0];
}

export function screenshot(url, width, height, png, profile) {
    execFileSync(CHROME, [
        '--headless=new', '--disable-gpu', '--hide-scrollbars',
        ...(process.env.CI ? ['--no-sandbox'] : []),
        `--user-data-dir=${profile}`,
        '--force-device-scale-factor=2',
        `--window-size=${width},${height}`,
        `--screenshot=${png}`,
        url,
    ], { stdio: 'ignore' });
    if (!existsSync(png)) throw new Error(`Chrome wrote no ${png}`);
}

// Render one figure in a theme, with the post's stylesheet and defs.
export function renderFigure(fig, assets, png, work, theme) {
    const html = join(work, 'figure.html');
    writeFileSync(html, `<!doctype html><meta charset="utf-8">
<style>${assets.rootCss}
${THEMES[theme].tokens}
html,body{margin:0;background:var(--color-bg)}
.fig{padding:${PAD}px;width:${fig.vbW}px}
.fig svg{display:block;width:100%;height:auto}</style>
${recolor(assets.css, theme)}${recolor(assets.defs, theme)}<div class="fig">${recolor(fig.svg, theme)}</div>`);
    screenshot(`file://${html}`, fig.vbW + 2 * PAD, Math.ceil(fig.vbH) + 2 * PAD, png, join(work, 'chrome'));
}
