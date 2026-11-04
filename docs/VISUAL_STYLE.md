# Visual Style

How covers and in-article figures are drawn. Both are hand-written SVG and share
one visual language: a dark surface, one accent colour, thin lines, rounded
blocks, and text that stays readable at small sizes.

The reference pieces are `static/images/aggregate-function-cover.svg` (cover) and
the figures in `content/blog/aggregate-is-a-function.md` (in-article). When this
document and an older cover disagree, this document wins.

## Table of Contents

- [Principles](#principles)
- [Colour](#colour)
- [Typography](#typography)
- [Blocks](#blocks)
- [Lines and Arrows](#lines-and-arrows)
- [Covers](#covers)
- [In-Article Figures](#in-article-figures)
- [Formulas](#formulas)
- [Charts](#charts)
- [Checklist](#checklist)
- [Publishing](#publishing)

---

## Principles

1. **Colour carries meaning, never decoration.** Indigo is the main thing or the
   path that works. Red is wrong (a duplicate, a conflict, a bad number). Amber
   is late or breaking. Grey is everything neutral. A colour used for one meaning
   is not reused for another in the same picture.
2. **Readable first.** Small text must reach at least 5.5:1 contrast against the
   background. Dim is allowed for "less important", never for "hard to read".
3. **Labels sit on what they describe.** Put the text on the box, arrow or mark
   it explains. Use a legend only for marks that repeat many times.
4. **Show the mechanism.** A figure earns its place by showing how something
   works (where rows lie, what a retry does, which cells add up). If it only
   restates a sentence, cut it.
5. **One idea per figure.** If a figure needs two captions, it is two figures.

---

## Colour

Covers use literal hex values (a cover is a standalone file and is also served
from Cloudinary). In-article figures use the site's CSS variables from
`static/css/style.css`, so they follow the theme.

Contrast is measured against the background `#0f0f0f`.

### Text

| Role | Cover | Figure | Contrast |
|------|-------|--------|----------|
| Primary text, titles | `#e0e0e0` | `var(--color-text)` | 14.5 |
| Code that is "alive" or emphasised | `#c8c8c8` | - | 11.5 |
| Secondary text, box subtitles, captions | `#a8a8a8` | - | 8.1 |
| Second title line, uppercase labels | `#9a9a9a` | - | 6.8 |
| Footer, dimmed but readable text | `#8a8a8a` | `var(--color-text-muted)` (`#888`) | 5.6 |
| Accent text (labels, highlighted words) | `#818cf8` | `#818cf8` | 6.4 |
| Bad text (conflict, duplicate) | `#f87171` | - | 6.9 |
| Late text | `#fbbf24` | `var(--color-warning)` | 11.5 |

Do not use `#6a6a6a` or darker for text: `#666` is 3.3:1, `#555` 2.6:1, `#444`
2.0:1, `#333` 1.5:1. Older covers did; that is what this document replaced.

### Strokes and fills

| Role | Value |
|------|-------|
| Background | `#0f0f0f` / `var(--color-bg)` |
| Box fill | `#1a1a1a` / `var(--color-bg-secondary)` |
| Main box fill (the one the picture is about) | `#1a1a2e` |
| Accent stroke, accent arrows | `#6366f1` / `var(--color-accent)` |
| Neutral box stroke | `#3a3a3a` (figures: `var(--color-border)`) |
| Neutral lines, feed lines | `#4a4a4a` |
| Neutral arrows | `#6a6a6a` (figures: `var(--color-text-muted)`) |
| Divider under title blocks | `#2a2a2a` |
| Bad stroke | `#ef4444` |
| Late stroke | `#f59e0b` (figures: `var(--color-warning)`) |
| Good / running | `#22c55e` / `var(--color-success)` |

`#6366f1` is 4.3:1: fine for strokes, arrows and large numbers (28px and up),
too dim for small text. Small accent text uses `#818cf8`.

---

## Typography

Every SVG sets a font for all text. Without it, any `<text>` that misses a
class falls back to the browser's serif font:

```css
text { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
.mono { font-family: "SF Mono", "Fira Code", "Cascadia Code", monospace; }
```

| Role | Size | Weight | Notes |
|------|------|--------|-------|
| Cover title | 36px | 600 | `letter-spacing: -0.02em`; second line in `#9a9a9a` |
| Cover subtitle (category) | 16px | 500 | uppercase, `letter-spacing: 0.08em`, `#818cf8` |
| Section label | 13px | 400 | uppercase via `text-transform`, `letter-spacing: 0.1em`, `#9a9a9a` |
| Box title | 15-17px | 500 | `#e0e0e0` |
| Box subtitle, caption | 13px | 400 | `#a8a8a8` |
| Small tags | 12px | 400 | the only place 12px is allowed |
| Big number | 28px | 600 | accent or bad colour |
| Code, types, keys | 12-13px | 400-600 | `.mono` |
| Footer | 13px | 400 | `#8a8a8a` |

In figures, body text inside the SVG is 13px (`var(--font-sans)`), emphasised
labels are `600 13px`.

---

## Blocks

Three kinds, always with rounded corners set as an attribute (`rx="6"`), not
through CSS: the attribute works in every renderer.

| Kind | Fill | Stroke | `rx` | Use |
|------|------|--------|------|-----|
| Box | `#1a1a1a` | `#6366f1` 1.5 | 6 | a step on the main path |
| Main box | `#1a1a2e` | `#6366f1` 2 | 8 | the one thing the picture is about |
| Muted box | `#1a1a1a` | `#3a3a3a` 1 | 6 | context, secondary steps, failed attempts |
| Tag | `#1a1a1a` | `#3a3a3a` 1 | 4 | small labels in a row |
| Group frame | none | `#2a2a2a`-`#3a3a3a` 1.5, dashed `6 4` | 8 | "these happen together" |

A box carries a title line and at most one subtitle line. If it needs more, the
idea belongs in the text.

State is shown by the stroke, not the fill: a red stroke for a bad item, amber
for a late one, the rest of the box unchanged. The word that names the state
("again", "week late", "type conflict") sits inside or next to the box, bold, in
the matching text colour.

---

## Lines and Arrows

- **Weights:** 1.5 for arrows and connectors, 2 for emphasis (main-box outlines,
  loops), 1 for dividers and grid lines.
- **Two markers on covers:** indigo and grey, same shape.

```svg
<marker id="arrow" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
  <polygon points="0 0, 8 3, 0 6" fill="#6366f1"/>
</marker>
<marker id="arrow-muted" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
  <polygon points="0 0, 8 3, 0 6" fill="#6a6a6a"/>
</marker>
```

- **The arrowhead sits on a straight segment.** The last stretch of a path must
  run in the direction the arrowhead points. A curve that still bends at its end
  makes the head look detached.
- **Curves:** use a symmetric S (`C` with horizontal control points) when the
  horizontal run is at least twice the vertical offset. Otherwise use straight
  segments with rounded corners (`L ... Q ... L`), radius about 12-22:

```svg
<!-- S-curve, ends horizontal -->
<path d="M356,93 C398,93 394,58 436,58"/>
<!-- straight segments with rounded corners, ends vertical -->
<path d="M680,76 L680,88 Q680,101 667,101 L133,101 Q120,101 120,114 L120,126"/>
```

- **Mirror pairs.** Two arrows that leave one source for two targets mirror each
  other around the source's centre line.
- **Converging lines** (several sources, one target) meet at one point and
  continue as a single neutral arrow. Coloured sources keep their colour up to
  the meeting point; the shared arrow after it is grey.
- **A label on a line** gets a background rectangle in the page colour so the
  line does not cross the text.
- **Do not leave gaps:** an arrow ends 3-4 units before the box it points at.

---

## Covers

### Canvas

| | Blog post | Project |
|---|---|---|
| Size | `1200x630` | `1200x630` |
| `viewBox` | `75 50 1050 580` | `0 0 1200 630` |
| Shown at | aspect ratio `105 / 58` | `1200 / 630` |

Keep everything that matters inside x 100-1100, y 70-615 for blog covers.

### Layout

| Zone | Position | Content |
|------|----------|---------|
| Left title | x 100, subtitle y 100, title y 148 and 184 | category, then the first half of the headline |
| Right title | anchored end at x 1100, same rows | a second category, the second half or a tagline |
| Dividers | y 210, under each title block | `#2a2a2a`, about 240-320 long |
| Diagram | y 240-560 | the article's idea in one picture |
| Footer | y 610 | `alnovis.io` left, `Topic / Topic / Topic` right |

The diagram tells the article's own story. Two covers in a row must not share a
composition; reuse the parts (blocks, arrows, tags), not the layout.

### Template

Every blog cover starts from this:

```svg
<svg width="1200" height="630" viewBox="75 50 1050 580" xmlns="http://www.w3.org/2000/svg">
  <style>
    text { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
    .bg { fill: #0f0f0f; }
    .text { fill: #e0e0e0; }
    .mono { font-family: "SF Mono", "Fira Code", "Cascadia Code", monospace; }
    .title { font-size: 36px; font-weight: 600; letter-spacing: -0.02em; }
    .title2 { fill: #9a9a9a; }
    .subtitle { font-size: 16px; fill: #818cf8; font-weight: 500; letter-spacing: 0.08em; }
    .label { font-size: 13px; fill: #9a9a9a; letter-spacing: 0.1em; text-transform: uppercase; }
    .boxtext { font-size: 17px; font-weight: 500; }
    .sub { font-size: 13px; fill: #a8a8a8; }
    .accent { fill: #818cf8; }
    .line-v { stroke: #2a2a2a; stroke-width: 1; }
    .box { fill: #1a1a1a; stroke: #6366f1; stroke-width: 1.5; }
    .box-main { fill: #1a1a2e; stroke: #6366f1; stroke-width: 2; }
    .box-muted { fill: #1a1a1a; stroke: #3a3a3a; stroke-width: 1; }
    .arrow { stroke: #6366f1; stroke-width: 1.5; fill: none; marker-end: url(#arrow); }
    .arrow-muted { stroke: #6a6a6a; stroke-width: 1.5; fill: none; marker-end: url(#arrow-muted); }
    .footer { font-size: 13px; fill: #8a8a8a; }
  </style>

  <rect class="bg" width="1200" height="630"/>

  <defs>
    <marker id="arrow" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
      <polygon points="0 0, 8 3, 0 6" fill="#6366f1"/>
    </marker>
    <marker id="arrow-muted" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
      <polygon points="0 0, 8 3, 0 6" fill="#6a6a6a"/>
    </marker>
  </defs>

  <text x="100" y="100" class="subtitle">CATEGORY</text>
  <text x="100" y="148" class="text title">First Half</text>
  <text x="100" y="184" class="text title title2">of the Headline</text>
  <line x1="100" y1="210" x2="380" y2="210" class="line-v"/>

  <text x="1100" y="100" text-anchor="end" class="subtitle">SECOND CATEGORY</text>
  <text x="1100" y="148" text-anchor="end" class="text title">Second Half</text>
  <text x="1100" y="184" text-anchor="end" class="text title title2">or a Tagline</text>
  <line x1="860" y1="210" x2="1100" y2="210" class="line-v"/>

  <!-- diagram, y 240-560 -->

  <text x="100" y="610" class="footer">alnovis.io</text>
  <text x="1100" y="610" text-anchor="end" class="footer">Topic / Topic / Topic</text>
</svg>
```

Project covers are typographic posters (large rotated lettering, one icon) and
keep their own composition; they follow the colour and footer rules above.

---

## In-Article Figures

Figures are inline SVG inside the post's Markdown. They inherit the site theme
through CSS variables and need no image hosting.

### Structure

1. One `<style>` block right after the frontmatter, with every class the post
   uses. Class names carry a short per-post prefix (`agg-` in the reference
   post) so two posts never clash.
2. One hidden `<svg>` with shared `<defs>`: arrow markers and patterns. Markers
   and patterns are referenced by id from every figure in the post.
3. Each figure is a `<figure>` with an `<svg viewBox="0 0 800 H">` (800 is the
   content column) and an optional `<figcaption>`.

Markdown rules that keep the HTML intact:

- Start `<figure>` on its own line, with a blank line before and after.
- No blank lines inside a figure: a blank line ends the HTML block and the rest
  is parsed as Markdown.
- Put `<defs>` on its own line after an opening `<svg ...>`; a tag followed by
  other markup on the same line is wrapped in a paragraph.
- Every figure has `role="img"` and an `aria-label` that states what it shows.

### Base classes

Shown with the prefix `fig-`; replace it with the post's own.

```css
.fig{margin:2rem 0}
.fig svg{display:block;width:100%;height:auto}
.fig-cap{color:var(--color-text-muted);font-size:.9rem;text-align:center;margin-top:.5rem}
.fig-txt{fill:var(--color-text);font:13px var(--font-sans)}
.fig-lbl{fill:var(--color-text-muted);font:13px var(--font-sans)}
.fig-acc{fill:var(--color-accent);font:600 13px var(--font-sans)}
.fig-warn{fill:var(--color-warning);font:600 13px var(--font-sans)}
.fig-box{fill:var(--color-bg-secondary);stroke:var(--color-border);stroke-width:1.5}
.fig-boxa{fill:var(--color-bg-secondary);stroke:var(--color-accent);stroke-width:1.5}
.fig-boxw{fill:var(--color-bg-secondary);stroke:var(--color-warning);stroke-width:1.5}
.fig-grp{fill:none;stroke:var(--color-border);stroke-width:1.5;stroke-dasharray:6 4}
.fig-arr{fill:none;stroke:var(--color-text-muted);stroke-width:1.5;marker-end:url(#fig-arrow)}
.fig-arra{fill:none;stroke:var(--color-accent);stroke-width:1.5;marker-end:url(#fig-arrow-a)}
.fig-arrw{fill:none;stroke:var(--color-warning);stroke-width:1.5;marker-end:url(#fig-arrow-w)}
.fig-arrhead{fill:var(--color-text-muted)}
.fig-arrhead-a{fill:var(--color-accent)}
.fig-arrhead-w{fill:var(--color-warning)}
.fig-bg{fill:var(--color-bg)}
```

```svg
<svg width="0" height="0" style="position:absolute" aria-hidden="true">
<defs>
<marker id="fig-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="fig-arrhead" d="M0,0 L10,5 L0,10 z"/></marker>
<marker id="fig-arrow-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="fig-arrhead-a" d="M0,0 L10,5 L0,10 z"/></marker>
<marker id="fig-arrow-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="fig-arrhead-w" d="M0,0 L10,5 L0,10 z"/></marker>
</defs></svg>
```

### Composition

- **Flows** read left to right or top to bottom, never both in one figure.
- **Before / after** is two parallel rows with row labels (`first backfill`,
  `backfill, fixed`), not "before" and "after", which readers take as the
  article's own before and after.
- **State over a value:** write the value that differs (`recomputed_at = now`)
  inside the box, in `.mono`, rather than describing it on an arrow.
- **Statements, not questions:** "shift view sees them as just recomputed", not
  "recomputed in the last 2 hours?" with "yes" / "no" on the arrows.
- **Data layout pictures** (granules, cells) draw rows as thin stripes in boxes;
  the rows that matter are accent, the rest a muted stripe pattern.
- **Symmetry:** when one element feeds two, centre it between them and mirror
  the arrows.
- **A figure replaces prose, it does not repeat it.** After adding a figure,
  shorten the paragraph next to it to what the figure cannot say.

Mermaid is available, but hand-drawn SVG is preferred: Mermaid's layout stretches
long labels, and its line weights and arrows do not match the rest.

---

## Formulas

Core mechanics are shown as a formula card, then the same logic as highlighted
code right below, introduced by "The same in code:".

- The card is an SVG: `fill: var(--color-bg-secondary)`, a 4px accent bar on
  the left, `rx 8`.
- Formula text: `20px "Times New Roman", Times, serif`. Function names in
  `#818cf8`, single-letter variables italic. Georgia is avoided: its old-style
  digits make `0` read as the letter "o".
- Under each formula, a 13px muted comment says what it means in words.
- **Only symbols every engineer reads at a glance:** the sum sign, element-of,
  `max`, `argmax`, `=`, `>`, set braces, named functions (`hour(t)`). No floor
  brackets, no not-exists, no circled operators, no unions: put the idea into the
  comment line instead.
- A subscript under a big operator needs extra vertical space; leave at least
  30 units between a formula line and the comment below it.

---

## Charts

Charts follow the same colours, with one exception: the site yellow `#eab308`
is too light to read as a chart mark on the dark background. Chart marks for
the "bad" series use `#c27d0e` (5.7:1).

- Every value a reader may want is reachable: direct labels for the key values,
  a `<title>` tooltip on every mark, and the full table in a collapsed
  `<details>` under the chart.
- Use a log scale when values span more than one order of magnitude, and say so
  in the axis label.
- A reference line (a budget, a limit) is dashed and labelled in the legend.
- A missing value (a failure, a timeout) is a hollow mark at the edge with its
  reason as the label, never a fake number.

---

## Checklist

Before a cover or figure goes in:

- [ ] No text darker than `#8a8a8a`; small accent text is `#818cf8`.
- [ ] `text { font-family: ... }` is set; nothing renders in serif by accident.
- [ ] Blocks use `rx` attributes; strokes are 1.5 or 2.
- [ ] Every arrowhead sits on a straight segment and points into its target.
- [ ] Labels do not overlap lines or each other (check at 2x).
- [ ] Each colour means one thing in the picture.
- [ ] Figures: `role="img"` and a real `aria-label`; no blank lines inside.
- [ ] Rendered and looked at, not only written. Headless Chrome is enough:

```bash
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new \
  --disable-gpu --hide-scrollbars --force-device-scale-factor=2 \
  --window-size=1100,4000 --virtual-time-budget=3000 \
  --screenshot=/tmp/page.png http://localhost:3000/blog/my-post
```

For a post that is still `draft: true`, set it to `false` locally to see it
(drafts return 404), and set it back before committing.

---

## Publishing

- New covers reach Cloudinary only through the release workflow (a `v*.*.*`
  tag). A content-only push publishes the post but not its cover.
- Release first, then the post: a content push goes live in about 20 seconds,
  while a release takes minutes. Publishing the post first shows it without its
  cover until the release finishes.
- **Changing an existing cover: new file name, never an overwrite.** Cloudinary
  serves files by `public_id` (the file name without extension), and its CDN,
  like social networks, keeps the old version cached; the release upload does
  not invalidate it. So a reworked cover is a new file with the next version
  suffix (`name-cover-v2.svg`, then `-v3`), the old file is removed from the
  repository, and the post's `cover_image` points to the new name.
- That change ships in two steps, for the same reason as a new post: first a
  release with the new file (it reaches Cloudinary), then a content push that
  switches `cover_image`. Switching first would show a missing image until the
  release finishes.
