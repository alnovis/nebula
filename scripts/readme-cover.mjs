#!/usr/bin/env node
// Draw the README cover: loose cells condensing into an indigo core, beside the wordmark.
// The layout is generated from fixed seeds, so every run gives the same picture.
// Writes docs/images/cover.svg and renders docs/images/cover.png (2560x1280) with Chrome.
//
// Usage: node scripts/readme-cover.mjs

import { writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { ROOT, screenshot } from './figures.mjs';

const W = 1280, H = 640;

function rng(seed) {
    return () => {
        seed |= 0; seed = (seed + 0x6d2b79f5) | 0;
        let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
        t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
}

// Smooth pseudo-noise: a few random sine waves, range ~[-1, 1].
function noise2(seed) {
    const r = rng(seed);
    const waves = Array.from({ length: 6 }, () => ({
        fx: (r() - 0.5) * 0.02, fy: (r() - 0.5) * 0.02, p: r() * Math.PI * 2,
    }));
    return (x, y) => waves.reduce((s, w) => s + Math.sin(x * w.fx + y * w.fy + w.p), 0) / 3;
}

const STYLE = `
  <style>
    text { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
    .word { fill: #e0e0e0; font-size: 92px; font-weight: 700; letter-spacing: 0.14em; }
    .sub { fill: #818cf8; font-size: 20px; font-weight: 500; letter-spacing: 0.12em; }
    .rule { stroke: #2a2a2a; stroke-width: 1; }
  </style>`;

const WORDMARK = `
  <text class="word" x="88" y="318">NEBULA</text>
  <line class="rule" x1="92" y1="350" x2="520" y2="350"/>
  <text class="sub" x="92" y="388">SELF-HOSTED BLOG ENGINE</text>`;

function wrap(body, defs = '') {
    return `<svg width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" xmlns="http://www.w3.org/2000/svg">${STYLE}
  <defs>${defs}</defs>
  <rect width="${W}" height="${H}" fill="#0f0f0f"/>
${body}${WORDMARK}
</svg>
`;
}

function cover() {
    const r = rng(7), n = noise2(11);
    const cx = 930, cy = 320, pitch = 24, s = 16;
    const cells = [];
    for (let gy = -14; gy <= 14; gy++) {
        for (let gx = -18; gx <= 18; gx++) {
            const x0 = cx + gx * pitch, y0 = cy + gy * pitch;
            // Elliptic distance, warped by noise so the cloud is not a disc.
            const d = Math.hypot(gx / 13, gy / 9.5) + 0.35 * n(x0, y0);
            if (d > 1.15) continue;
            const core = Math.abs(gx) <= 3 && Math.abs(gy) <= 2;
            const p = core ? 1 : Math.max(0, 1.05 - d) ** 1.3 + 0.08;
            if (r() > p) continue;
            const loose = core ? 0 : Math.min(1, Math.max(0, (d - 0.25) / 0.8));
            const x = x0 + (r() - 0.5) * 22 * loose, y = y0 + (r() - 0.5) * 22 * loose;
            const rot = (r() - 0.5) * 70 * loose;
            const size = s * (1 - 0.35 * loose * r());
            let fill, stroke;
            if (core) {
                const c = Math.hypot(gx, gy);
                fill = c < 1.5 ? '#818cf8' : '#6366f1';
                stroke = 'none';
            } else if (d < 0.45) {
                fill = '#1a1a2e'; stroke = '#6366f1';
            } else if (d < 0.75) {
                fill = '#1f1f1f'; stroke = '#4a4a4a';
            } else {
                fill = '#181818'; stroke = '#333333';
            }
            const op = core ? 1 : (1 - 0.55 * loose).toFixed(2);
            cells.push(`  <rect x="${(x - size / 2).toFixed(1)}" y="${(y - size / 2).toFixed(1)}" width="${size.toFixed(1)}" height="${size.toFixed(1)}" rx="3" fill="${fill}" stroke="${stroke}" stroke-width="1.2" opacity="${op}" transform="rotate(${rot.toFixed(1)} ${x.toFixed(1)} ${y.toFixed(1)})"/>`);
        }
    }
    const defs = `
    <radialGradient id="glow" cx="50%" cy="50%" r="50%">
      <stop offset="0" stop-color="#6366f1" stop-opacity="0.28"/>
      <stop offset="1" stop-color="#6366f1" stop-opacity="0"/>
    </radialGradient>`;
    const glow = `  <ellipse cx="${cx}" cy="${cy}" rx="260" ry="190" fill="url(#glow)"/>\n`;
    return wrap(glow + cells.join('\n') + '\n', defs);
}

const dir = join(ROOT, 'docs', 'images');
const svg = cover();
writeFileSync(join(dir, 'cover.svg'), svg);

const work = mkdtempSync(join(tmpdir(), 'nebula-cover-'));
try {
    const html = join(work, 'cover.html');
    writeFileSync(html, `<!doctype html><meta charset="utf-8"><style>html,body{margin:0;background:#0f0f0f}svg{display:block}</style>${svg}`);
    const png = join(dir, 'cover.png');
    rmSync(png, { force: true });
    screenshot(`file://${html}`, W, H, png, join(work, 'chrome'));
} finally {
    rmSync(work, { recursive: true, force: true });
}
console.log(`wrote ${join(dir, 'cover.svg')} and cover.png`);
