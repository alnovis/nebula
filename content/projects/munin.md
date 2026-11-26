---
title: "Munin"
slug: "munin"
description: "Local memory for AI coding agents: one Rust binary, one SQLite file, shared by every MCP client on the machine"
date: "2026-11-27T12:00:00Z"
tags: ["rust", "mcp", "sqlite", "ai-agents", "cli"]
status: "active"
github_url: "https://github.com/alnovis/munin"
featured: true
cover_image: "munin-project-cover.svg"
---

<style>
.mun-fig{margin:2rem 0}
.mun-fig svg{display:block;width:100%;height:auto}
.mun-cap{color:var(--color-text-muted);font-size:.9rem;text-align:center;margin-top:.5rem}
.mun-txt{fill:var(--color-text);font:13px var(--font-sans)}
.mun-lbl{fill:var(--color-text-muted);font:13px var(--font-sans)}
.mun-acc{fill:var(--color-accent);font:600 13px var(--font-sans)}
.mun-warn{fill:var(--color-warning);font:600 13px var(--font-sans)}
.mun-mono{fill:var(--color-text);font:12px var(--font-mono)}
.mun-box{fill:var(--color-bg-secondary);stroke:var(--color-border);stroke-width:1.5}
.mun-boxa{fill:var(--color-bg-secondary);stroke:var(--color-accent);stroke-width:1.5}
.mun-boxw{fill:var(--color-bg-secondary);stroke:var(--color-warning);stroke-width:1.5}
.mun-arr{fill:none;stroke:var(--color-text-muted);stroke-width:1.5;marker-end:url(#mun-arrow)}
.mun-arrw{fill:none;stroke:var(--color-warning);stroke-width:1.5;marker-end:url(#mun-arrow-w)}
.mun-arrhead{fill:var(--color-text-muted)}
.mun-arrhead-w{fill:var(--color-warning)}
.mun-dot{fill:var(--color-text-muted)}
</style>

<svg width="0" height="0" style="position:absolute" aria-hidden="true">
<defs>
<marker id="mun-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="mun-arrhead" d="M0,0 L10,5 L0,10 z"/></marker>
<marker id="mun-arrow-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="mun-arrhead-w" d="M0,0 L10,5 L0,10 z"/></marker>
</defs></svg>

Munin is local memory for AI coding agents. An agent searches it before unfamiliar work and records what it learned afterwards: decisions, how the system is built, bugs and their fixes. Everything lives in one SQLite file, and every MCP client on the machine (Claude Code, Codex, Cursor, OpenCode) reads and writes the same memory.

## Why

Agents start every session from zero. Instructions files such as `AGENTS.md` hold what people write down; they do not hold what an agent found out an hour ago while chasing a bug.

Munin reimplements the ideas of [larchanka/memo-mcp](https://github.com/larchanka/memo-mcp) in Rust: one SQLite file, a search that returns compact cards first and the full entry only on request, and entries that fade with age by kind. It changes what does not hold up over months of use:

| memo-mcp | Munin |
|----------|-------|
| Entries can never change | An entry is superseded by a newer one or retracted with a reason |
| No links between entries | `supersedes` links both ends of the chain |
| Project only as a prefix in the summary | Project is a column; search stays inside it by default |
| Decay applied after a top-30 cut by raw BM25 | Decay applied to every match before the limit |
| Invalid input silently becomes a `note` | Invalid input is rejected with the field and what is allowed |
| `npx -y` fetches the latest version on every start | One pinned binary |

## How It Works

<figure class="mun-fig">
<svg viewBox="0 0 760 250" role="img" aria-label="Claude Code, Codex and Cursor each start munin serve and talk to it over MCP on stdio. A person uses the Munin command line from the shell. Both the server and the command line read and write the same SQLite file, ~/.munin/munin.db.">
<text class="mun-lbl" x="200" y="16" text-anchor="middle">MCP, stdio</text>
<rect class="mun-box" x="20" y="24" width="130" height="36" rx="6"/>
<text class="mun-txt" x="85" y="47" text-anchor="middle">Claude Code</text>
<rect class="mun-box" x="20" y="70" width="130" height="36" rx="6"/>
<text class="mun-txt" x="85" y="93" text-anchor="middle">Codex</text>
<rect class="mun-box" x="20" y="116" width="130" height="36" rx="6"/>
<text class="mun-txt" x="85" y="139" text-anchor="middle">Cursor</text>
<path class="mun-arr" d="M150,42 L250,78"/>
<path class="mun-arr" d="M150,88 L250,88"/>
<path class="mun-arr" d="M150,134 L250,98"/>
<rect class="mun-box" x="250" y="66" width="150" height="44" rx="6"/>
<text class="mun-txt" x="325" y="93" text-anchor="middle">munin serve</text>
<rect class="mun-box" x="20" y="190" width="130" height="36" rx="6"/>
<text class="mun-txt" x="85" y="213" text-anchor="middle">You, in a shell</text>
<path class="mun-arr" d="M150,208 L250,208"/>
<rect class="mun-box" x="250" y="186" width="150" height="44" rx="6"/>
<text class="mun-txt" x="325" y="213" text-anchor="middle">Munin CLI</text>
<path class="mun-arr" d="M400,88 L520,136"/>
<path class="mun-arr" d="M400,208 L520,160"/>
<rect class="mun-boxa" x="520" y="120" width="220" height="56" rx="6"/>
<text class="mun-mono" x="630" y="143" text-anchor="middle">~/.munin/munin.db</text>
<text class="mun-lbl" x="630" y="163" text-anchor="middle">SQLite + FTS5</text>
</svg>
<figcaption class="mun-cap">Agents and people work on the same database file.</figcaption>
</figure>

On connect, the server sends the agent its instructions: search before unfamiliar work, record after a decision or a fix, reuse existing tags. Nothing has to be copied into the agent's own instructions file.

## Entries

| Kind | For | Fades with age |
|------|-----|----------------|
| `decision` | a choice or convention that holds until replaced | no |
| `architecture` | how the system is built | no |
| `bugfix` | a problem, its cause and the fix | half its weight every 30 days |
| `note` | anything else | half its weight every 30 days |

Ranking is BM25 text relevance times the age factor of the kind, computed by a function registered in SQLite. It covers every match before the limit is applied, so one old `decision` is not pushed out by a pile of stale fixes.

An entry is never edited. Its status only moves away from `active`, and either way it stays readable with its history:

<figure class="mun-fig">
<svg viewBox="0 0 760 200" role="img" aria-label="A new entry starts as active and is found by search. A newer entry can supersede it, after which it points to its replacement. Or it can be retracted, keeping the reason it is wrong. Neither state leads back to active.">
<circle class="mun-dot" cx="30" cy="100" r="5"/>
<text class="mun-lbl" x="70" y="90" text-anchor="middle">append</text>
<path class="mun-arr" d="M36,100 L108,100"/>
<rect class="mun-boxa" x="110" y="72" width="160" height="56" rx="6"/>
<text class="mun-acc" x="190" y="96" text-anchor="middle">active</text>
<text class="mun-lbl" x="190" y="116" text-anchor="middle">found by search</text>
<path class="mun-arr" d="M270,90 L498,46"/>
<text class="mun-lbl" x="360" y="38" text-anchor="middle">a newer entry supersedes it</text>
<path class="mun-arrw" d="M270,110 L498,154"/>
<text class="mun-lbl" x="360" y="172" text-anchor="middle">retracted with a reason</text>
<rect class="mun-box" x="500" y="16" width="240" height="56" rx="6"/>
<text class="mun-txt" x="620" y="40" text-anchor="middle">superseded</text>
<text class="mun-lbl" x="620" y="60" text-anchor="middle">points to its replacement</text>
<rect class="mun-boxw" x="500" y="128" width="240" height="56" rx="6"/>
<text class="mun-warn" x="620" y="152" text-anchor="middle">retracted</text>
<text class="mun-lbl" x="620" y="172" text-anchor="middle">keeps the reason it is wrong</text>
</svg>
<figcaption class="mun-cap">An entry leaves active once and is never deleted.</figcaption>
</figure>

Triggers in the database refuse edits and deletes, so this holds for any client that opens the file, not only for Munin's own code.

## Projects and Groups

Every entry belongs to a project, named after the git root the agent works in. Projects can be grouped, for example the services of one microservice system. A project in a group searches the whole group by default, and a decision that spans the services is recorded on the group itself, so every member sees it.

## Getting Started

There is no release yet; build from source:

```bash
git clone https://github.com/alnovis/munin.git
cd munin
cargo install --locked --path .
claude mcp add --scope user munin -- munin serve
```

Connecting Codex, Cursor and OpenCode, the command line and the search syntax are described in the repository's [install](https://github.com/alnovis/munin/blob/master/docs/INSTALL.md) and [usage](https://github.com/alnovis/munin/blob/master/docs/USAGE.md) guides.
