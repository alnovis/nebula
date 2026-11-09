---
title: "An Aggregate Is a Function, Not a Sum"
slug: "aggregate-is-a-function"
description: "Four ways to keep report aggregates over a stream of receipts in ClickHouse, why each of them failed, and the design we kept: cells that are recomputed from their documents instead of added up."
date: "2026-11-04T12:00:00Z"
tags: ["clickhouse", "data-engineering", "distributed-systems", "kafka"]
cover_image: "aggregate-function-cover.svg"
draft: true
---

<style>
.agg-fig{margin:2rem 0}
.agg-fig svg{display:block;width:100%;height:auto}
.agg-cap{color:var(--color-text-muted);font-size:.9rem;text-align:center;margin-top:.5rem}
.agg-lane{stroke:var(--color-border);stroke-width:2}
.agg-lbl{fill:var(--color-text-muted);font:13px var(--font-sans)}
.agg-ok{stroke:var(--color-text-muted);stroke-width:1.5}
.agg-okdot{fill:var(--color-text-muted)}
.agg-dup{stroke:var(--color-accent);stroke-width:2.5}
.agg-dupdot,.agg-need{fill:var(--color-accent)}
.agg-dupt{fill:var(--color-accent);font:600 13px var(--font-sans)}
.agg-late{stroke:var(--color-warning);stroke-width:2.5}
.agg-latedot{fill:var(--color-warning)}
.agg-latet{fill:var(--color-warning);font:600 13px var(--font-sans)}
.agg-run{fill:var(--color-success);opacity:.55}
.agg-down{fill:none;stroke:var(--color-warning);stroke-width:1.5;stroke-dasharray:4 4}
.agg-txt{fill:var(--color-text);font:13px var(--font-sans)}
.agg-box{fill:var(--color-bg-secondary);stroke:var(--color-border);stroke-width:1.5}
.agg-boxw{fill:var(--color-bg-secondary);stroke:var(--color-warning);stroke-width:1.5}
.agg-boxd{fill:var(--color-bg-secondary);stroke:var(--color-accent);stroke-width:1.5}
.agg-arr{fill:none;stroke:var(--color-text-muted);stroke-width:1.5;marker-end:url(#agg-arrow)}
.agg-arrhead{fill:var(--color-text-muted)}
.agg-gran,.agg-gran2{fill:var(--color-accent);fill-opacity:.2;stroke:var(--color-bg);stroke-width:3}
.agg-bound{stroke:var(--color-warning);stroke-width:2;stroke-dasharray:6 4}
.agg-tooth{stroke:var(--color-text-muted);stroke-width:2}
.agg-tip{stroke:var(--color-accent);stroke-width:4;stroke-linecap:round}
.agg-yes{fill:var(--color-accent)}
.agg-yest{fill:var(--color-accent);font:600 13px var(--font-sans)}
.agg-fail{fill:var(--color-warning)}
.agg-no{fill:none;stroke:var(--color-text-muted);stroke-width:1.5}
.agg-outa{fill:none;stroke:var(--color-accent);stroke-width:2}
.agg-outw{fill:none;stroke:var(--color-warning);stroke-width:2}
.agg-arra{fill:none;stroke:var(--color-accent);stroke-width:1.5;marker-end:url(#agg-arrow-a)}
.agg-arrw{fill:none;stroke:var(--color-warning);stroke-width:1.5;marker-end:url(#agg-arrow-w)}
.agg-arrwd{fill:none;stroke:var(--color-warning);stroke-width:1.5;stroke-dasharray:5 4;marker-end:url(#agg-arrow-w)}
.agg-arrhead-a{fill:var(--color-accent)}
.agg-arrhead-w{fill:var(--color-warning)}
.agg-grp{fill:none;stroke:var(--color-border);stroke-width:1.5;stroke-dasharray:6 4}
.agg-bg{fill:var(--color-bg)}
.agg-monow{fill:var(--color-warning);font:600 12px var(--font-mono)}
.agg-monoa{fill:var(--color-accent);font:600 12px var(--font-mono)}
.agg-cells{fill:var(--color-accent);stroke:var(--color-bg);stroke-width:2}
.agg-docs{fill:#c27d0e;stroke:var(--color-bg);stroke-width:2}
.agg-docsx{fill:var(--color-bg);stroke:#c27d0e;stroke-width:2}
.agg-bar{stroke:var(--color-border);stroke-width:2}
.agg-barx{stroke:var(--color-border);stroke-width:2;stroke-dasharray:4 4}
.agg-grid{stroke:var(--color-border);stroke-width:1}
.agg-budget{stroke:var(--color-text-muted);stroke-width:1.5;stroke-dasharray:5 4}
.agg-val{fill:var(--color-text);font:12px var(--font-sans)}
.agg-mcard{fill:var(--color-bg-secondary);stroke:var(--color-border);stroke-width:1}
.agg-mbar{fill:var(--color-accent)}
.agg-m{fill:var(--color-text);font:20px "Times New Roman",Times,serif}
.agg-mf{fill:#818cf8}
.agg-mv{font-style:italic}
.agg-details{margin:0 0 1.5rem}
.agg-details summary{cursor:pointer;color:var(--color-text-muted)}
</style>

<svg width="0" height="0" style="position:absolute" aria-hidden="true">
<defs>
<marker id="agg-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="agg-arrhead" d="M0,0 L10,5 L0,10 z"/></marker>
<marker id="agg-arrow-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="agg-arrhead-a" d="M0,0 L10,5 L0,10 z"/></marker>
<marker id="agg-arrow-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="agg-arrhead-w" d="M0,0 L10,5 L0,10 z"/></marker>
<pattern id="agg-rows" width="10" height="10" patternUnits="userSpaceOnUse"><rect class="agg-okdot" x="0" y="0" width="7" height="10" opacity=".35"/></pattern>
</defs></svg>

## Reports Over a Stream of Receipts

The system I work on stores sales receipts. Receipts from every cash register arrive through Kafka, and ClickHouse stores them as documents.

On top of the raw documents sit the reports people actually look at: totals per cashier shift, totals per hour and per day for a store, revenue grouped by a custom field the merchant puts on the receipt.

It looks like a standard ClickHouse use case: insert rows, then aggregate them. It took us four attempts.

What makes it hard is not the volume but three properties of the stream:

<figure class="agg-fig">
<svg viewBox="0 0 760 300" role="img" aria-label="Each receipt is a line from the moment it was printed at the register to the moment it was delivered through Kafka, on one shared time axis. Most lines are short and almost vertical. One receipt is delivered twice, one is delivered a week late, and aggregation has a gap while it is down.">
<text class="agg-lbl" x="684" y="26" text-anchor="end">time</text>
<path class="agg-arr" d="M690,22 L736,22"/>
<text class="agg-lbl" x="740" y="44" text-anchor="end">printed at the register</text>
<text class="agg-latet" x="100" y="42" text-anchor="middle">printed a week earlier</text>
<text class="agg-dupt" x="250" y="42" text-anchor="middle">printed once</text>
<line class="agg-lane" x1="40" y1="55" x2="740" y2="55"/>
<text class="agg-lbl" x="40" y="165">delivered through Kafka</text>
<line class="agg-lane" x1="40" y1="140" x2="740" y2="140"/>
<text class="agg-lbl" x="40" y="205">aggregation running</text>
<line class="agg-ok" x1="70" y1="55" x2="82" y2="140"/><circle class="agg-okdot" cx="70" cy="55" r="4"/><circle class="agg-okdot" cx="82" cy="140" r="4"/>
<line class="agg-ok" x1="130" y1="55" x2="142" y2="140"/><circle class="agg-okdot" cx="130" cy="55" r="4"/><circle class="agg-okdot" cx="142" cy="140" r="4"/>
<line class="agg-ok" x1="190" y1="55" x2="202" y2="140"/><circle class="agg-okdot" cx="190" cy="55" r="4"/><circle class="agg-okdot" cx="202" cy="140" r="4"/>
<line class="agg-ok" x1="310" y1="55" x2="322" y2="140"/><circle class="agg-okdot" cx="310" cy="55" r="4"/><circle class="agg-okdot" cx="322" cy="140" r="4"/>
<line class="agg-ok" x1="370" y1="55" x2="382" y2="140"/><circle class="agg-okdot" cx="370" cy="55" r="4"/><circle class="agg-okdot" cx="382" cy="140" r="4"/>
<line class="agg-ok" x1="430" y1="55" x2="442" y2="140"/><circle class="agg-okdot" cx="430" cy="55" r="4"/><circle class="agg-okdot" cx="442" cy="140" r="4"/>
<line class="agg-ok" x1="490" y1="55" x2="502" y2="140"/><circle class="agg-okdot" cx="490" cy="55" r="4"/><circle class="agg-okdot" cx="502" cy="140" r="4"/>
<line class="agg-ok" x1="550" y1="55" x2="562" y2="140"/><circle class="agg-okdot" cx="550" cy="55" r="4"/><circle class="agg-okdot" cx="562" cy="140" r="4"/>
<line class="agg-ok" x1="610" y1="55" x2="622" y2="140"/><circle class="agg-okdot" cx="610" cy="55" r="4"/><circle class="agg-okdot" cx="622" cy="140" r="4"/>
<line class="agg-ok" x1="670" y1="55" x2="682" y2="140"/><circle class="agg-okdot" cx="670" cy="55" r="4"/><circle class="agg-okdot" cx="682" cy="140" r="4"/>
<line class="agg-dup" x1="250" y1="55" x2="262" y2="140"/><line class="agg-dup" x1="250" y1="55" x2="345" y2="140"/><circle class="agg-dupdot" cx="250" cy="55" r="5"/><circle class="agg-dupdot" cx="262" cy="140" r="5"/><circle class="agg-dupdot" cx="345" cy="140" r="5"/>
<text class="agg-dupt" x="262" y="165">same receipt, delivered twice</text>
<line class="agg-late" x1="100" y1="55" x2="520" y2="140"/><circle class="agg-latedot" cx="100" cy="55" r="5"/><circle class="agg-latedot" cx="520" cy="140" r="5"/>
<text class="agg-latet" x="520" y="165" text-anchor="middle">a week late</text>
<rect class="agg-run" x="40" y="215" width="380" height="14" rx="3"/>
<rect class="agg-down" x="424" y="215" width="92" height="14" rx="3"/>
<rect class="agg-run" x="520" y="215" width="220" height="14" rx="3"/>
<text class="agg-latet" x="470" y="250" text-anchor="middle">pipeline down</text>
<line class="agg-ok" x1="44" y1="272" x2="52" y2="292"/><circle class="agg-okdot" cx="44" cy="272" r="3"/><circle class="agg-okdot" cx="52" cy="292" r="3"/>
<text class="agg-lbl" x="66" y="287">one line is one receipt: printed at the top, delivered at the bottom; a normal one takes seconds</text>
</svg>
<figcaption class="agg-cap">Three properties of the stream that a running sum does not handle.</figcaption>
</figure>

- **Documents come twice.** Kafka delivery is at-least-once. A batch fails halfway, the consumer retries, the same receipt is inserted again. Any total that is "the sum of what was inserted" is now wrong.
- **Documents come late.** Most receipts arrive within seconds, but a register that was offline for a week sends its backlog when it reconnects. A small but steady share of documents arrive more than a day late, and some a month late. A report for last Tuesday is never final.
- **The pipeline stops.** Whatever computes the aggregates will sometimes be down: a deploy, a node restart, a bug. When it comes back, it must not lose anything.

The reports also have latency budgets: an interactive call should answer in a few seconds, a heavy report over hundreds of registers in under thirty seconds.

## Four Attempts

| # | The idea | What broke |
|---|---|---|
| 1 | Add it up on insert | a duplicate is counted twice, forever |
| 2 | Recompute from the wide table | read 46 million rows to use 508 thousand |
| 3 | Remember every key | the key set did not fit in memory |
| 4 | Don't store anything | 3.5 billion rows for one report, on the test cluster |

### 1. Add It Up on Insert

The first version was the classic one: materialized views on insert, writing partial sums into a `SummingMergeTree`. Each insert adds its contribution, background merges fold the rows together, and reports read pre-summed rows.

<figure class="agg-fig">
<svg viewBox="0 0 800 200" role="img" aria-label="Kafka sends a batch with receipt 1234 for 500, the consumer inserts it and the hour total becomes 500. The batch fails halfway and is retried; the same batch is inserted again and the hour total becomes 1000.">
<rect class="agg-box" x="20" y="20" width="200" height="56" rx="6"/>
<text class="agg-txt" x="120" y="44" text-anchor="middle">Kafka</text>
<text class="agg-lbl" x="120" y="63" text-anchor="middle">batch: receipt 1234, 500</text>
<path class="agg-arr" d="M220,48 L296,48"/>
<rect class="agg-box" x="300" y="20" width="200" height="56" rx="6"/>
<text class="agg-txt" x="400" y="44" text-anchor="middle">consumer</text>
<text class="agg-lbl" x="400" y="63" text-anchor="middle">insert</text>
<path class="agg-arr" d="M500,48 L576,48"/>
<rect class="agg-box" x="580" y="20" width="200" height="56" rx="6"/>
<text class="agg-txt" x="680" y="44" text-anchor="middle">SummingMergeTree</text>
<text class="agg-lbl" x="680" y="63" text-anchor="middle">hour total: 500</text>
<path class="agg-arrwd" d="M680,76 L680,88 Q680,101 667,101 L133,101 Q120,101 120,114 L120,126"/>
<rect class="agg-bg" x="290" y="91" width="220" height="20"/>
<text class="agg-latet" x="400" y="105" text-anchor="middle">batch fails halfway, retry</text>
<rect class="agg-box" x="20" y="130" width="200" height="56" rx="6"/>
<text class="agg-txt" x="120" y="154" text-anchor="middle">Kafka</text>
<text class="agg-lbl" x="120" y="173" text-anchor="middle">the same batch again</text>
<path class="agg-arr" d="M220,158 L296,158"/>
<rect class="agg-box" x="300" y="130" width="200" height="56" rx="6"/>
<text class="agg-txt" x="400" y="154" text-anchor="middle">consumer</text>
<text class="agg-lbl" x="400" y="173" text-anchor="middle">insert again</text>
<path class="agg-arr" d="M500,158 L576,158"/>
<rect class="agg-boxw" x="580" y="130" width="200" height="56" rx="6"/>
<text class="agg-txt" x="680" y="154" text-anchor="middle">SummingMergeTree</text>
<text class="agg-latet" x="680" y="173" text-anchor="middle">hour total: 1000</text>
</svg>
<figcaption class="agg-cap">The second insert adds to the total instead of replacing it. Nothing in the total says which 500 came twice.</figcaption>
</figure>

It is fast, but a re-delivered receipt adds its sum a second time, and once two numbers have been added there is no way to subtract the right one later. The aggregate does not record which documents contributed to it.

### 2. Recompute From the Wide Table

The second version fixed the double counting with recomputation: a refreshable materialized view periodically rebuilt the totals of recently touched shifts and hours from the documents themselves, with duplicates collapsed by document key.

This version was correct. But it read the main documents table, which is wide (payload, statuses, dozens of columns) and sorted by register and document key, not by time.

The recompute filtered by date, and in ClickHouse a date filter on a table not sorted by date cuts away whole partitions, but not granules (more on granules in a moment). One pass read 46 million rows to use 508 thousand:

<figure class="agg-fig">
<svg viewBox="0 0 800 225" role="img" aria-label="An older month partition is skipped whole. In the current month the table is sorted by register, so each granule holds weeks of one register's rows, oldest to newest. The date filter falls inside every granule, just before its newest rows, so all six granules are read whole for a few rows each.">
<rect class="agg-grp" x="20" y="24" width="130" height="152" rx="8"/>
<text class="agg-lbl" x="85" y="42" text-anchor="middle">older month</text>
<text class="agg-txt" x="85" y="104" text-anchor="middle">skipped</text>
<text class="agg-txt" x="85" y="122" text-anchor="middle">whole</text>
<rect class="agg-grp" x="166" y="24" width="614" height="152" rx="8"/>
<text class="agg-lbl" x="176" y="42">current month, sorted by register, then by document number</text>
<text class="agg-lbl" x="219" y="64" text-anchor="middle">register 1</text><text class="agg-lbl" x="319" y="64" text-anchor="middle">register 2</text>
<text class="agg-lbl" x="419" y="64" text-anchor="middle">register 3</text><text class="agg-lbl" x="519" y="64" text-anchor="middle">register 4</text>
<text class="agg-lbl" x="619" y="64" text-anchor="middle">register 5</text><text class="agg-lbl" x="719" y="64" text-anchor="middle">register 6</text>
<rect class="agg-boxd" x="176" y="72" width="86" height="60" rx="4"/><rect fill="url(#agg-rows)" x="180" y="80" width="58" height="44"/>
<rect class="agg-boxd" x="276" y="72" width="86" height="60" rx="4"/><rect fill="url(#agg-rows)" x="280" y="80" width="58" height="44"/>
<rect class="agg-boxd" x="376" y="72" width="86" height="60" rx="4"/><rect fill="url(#agg-rows)" x="380" y="80" width="58" height="44"/>
<rect class="agg-boxd" x="476" y="72" width="86" height="60" rx="4"/><rect fill="url(#agg-rows)" x="480" y="80" width="58" height="44"/>
<rect class="agg-boxd" x="576" y="72" width="86" height="60" rx="4"/><rect fill="url(#agg-rows)" x="580" y="80" width="58" height="44"/>
<rect class="agg-boxd" x="676" y="72" width="86" height="60" rx="4"/><rect fill="url(#agg-rows)" x="680" y="80" width="58" height="44"/>
<rect class="agg-yes" x="240" y="80" width="7" height="44"/><rect class="agg-yes" x="250" y="80" width="7" height="44"/>
<rect class="agg-yes" x="340" y="80" width="7" height="44"/><rect class="agg-yes" x="350" y="80" width="7" height="44"/>
<rect class="agg-yes" x="440" y="80" width="7" height="44"/><rect class="agg-yes" x="450" y="80" width="7" height="44"/>
<rect class="agg-yes" x="540" y="80" width="7" height="44"/><rect class="agg-yes" x="550" y="80" width="7" height="44"/>
<rect class="agg-yes" x="640" y="80" width="7" height="44"/><rect class="agg-yes" x="650" y="80" width="7" height="44"/>
<rect class="agg-yes" x="740" y="80" width="7" height="44"/><rect class="agg-yes" x="750" y="80" width="7" height="44"/>
<line class="agg-bound" x1="238.5" y1="74" x2="238.5" y2="130"/><line class="agg-bound" x1="338.5" y1="74" x2="338.5" y2="146"/>
<line class="agg-bound" x1="438.5" y1="74" x2="438.5" y2="130"/><line class="agg-bound" x1="538.5" y1="74" x2="538.5" y2="130"/>
<line class="agg-bound" x1="638.5" y1="74" x2="638.5" y2="130"/><line class="agg-bound" x1="738.5" y1="74" x2="738.5" y2="130"/>
<path class="agg-arr" d="M182,146 L254,146"/>
<text class="agg-lbl" x="219" y="164" text-anchor="middle">older to newer</text>
<text class="agg-latet" x="338" y="164" text-anchor="middle">date filter</text>
<text class="agg-lbl" x="85" y="206" text-anchor="middle">0 granules read</text>
<text class="agg-latet" x="176" y="206">6 of 6 granules read whole, for the few newest rows in each</text>
</svg>
<figcaption class="agg-cap">Schematic. The date cuts between months, but inside a month it cuts through every granule.</figcaption>
</figure>

In production the passes started hitting the per-query memory limit and failed repeatedly.

### 3. Remember Every Key

The third version went back to adding up on insert and tried to make duplicates impossible: before adding a receipt, check whether its key was seen before. The key lookup was implemented as a ClickHouse dictionary over a table of recent keys.

A dictionary lives in memory, and a key in it is not small: 161 bytes on our test cluster. A month of keys for every register did not fit in production. The dictionary failed to load. After that, every insert failed:

<figure class="agg-fig">
<svg viewBox="0 0 800 205" role="img" aria-label="A month of keys does not fit, the dictionary fails to load, inserts write their rows and then fail, the consumer retries, and every retry stores another copy of the same receipt.">
<rect class="agg-box" x="20" y="20" width="200" height="56" rx="6"/>
<text class="agg-txt" x="120" y="44" text-anchor="middle">a month of keys</text>
<text class="agg-lbl" x="120" y="63" text-anchor="middle">too big for memory</text>
<path class="agg-arr" d="M120,76 L120,124"/>
<rect class="agg-boxw" x="20" y="130" width="200" height="56" rx="6"/>
<text class="agg-txt" x="120" y="154" text-anchor="middle">dictionary</text>
<text class="agg-txt" x="120" y="173" text-anchor="middle">fails to load</text>
<path class="agg-arr" d="M220,158 C262,158 258,48 294,48"/>
<rect class="agg-boxw" x="300" y="20" width="220" height="56" rx="6"/>
<text class="agg-txt" x="410" y="44" text-anchor="middle">insert writes its rows,</text>
<text class="agg-txt" x="410" y="63" text-anchor="middle">then fails</text>
<path class="agg-arr" d="M480,76 L480,124"/>
<rect class="agg-box" x="300" y="130" width="220" height="56" rx="6"/>
<text class="agg-txt" x="410" y="154" text-anchor="middle">consumer retries</text>
<text class="agg-txt" x="410" y="173" text-anchor="middle">the same batch</text>
<path class="agg-arr" d="M340,130 L340,82"/>
<text class="agg-dupt" x="410" y="108" text-anchor="middle">again and again</text>
<path class="agg-arr" d="M520,48 C560,48 570,55 602,55"/>
<text class="agg-lbl" x="680" y="30" text-anchor="middle">receipt 1234 in storage</text>
<rect class="agg-box" x="610" y="42" width="140" height="26" rx="4"/><text class="agg-txt" x="680" y="60" text-anchor="middle">copy 1</text>
<rect class="agg-boxd" x="610" y="76" width="140" height="26" rx="4"/><text class="agg-txt" x="680" y="94" text-anchor="middle">copy 2</text>
<rect class="agg-boxd" x="610" y="110" width="140" height="26" rx="4"/><text class="agg-txt" x="680" y="128" text-anchor="middle">copy 3</text>
<rect class="agg-boxd" x="610" y="144" width="140" height="26" rx="4"/><text class="agg-txt" x="680" y="162" text-anchor="middle">copy 4</text>
<text class="agg-dupt" x="680" y="194" text-anchor="middle">and counting</text>
</svg>
<figcaption class="agg-cap">Every retry wrote the rows before failing, so every retry stored one more copy.</figcaption>
</figure>

Documents started landing several times each, so the deduplication itself was producing duplicates.

### 4. Don't Store Anything

The fourth version gave up on stored aggregates altogether: compute every report from the documents at read time, collapsing duplicates with a `GROUP BY` on the document key. There is no state to get out of sync, and re-deliveries and late documents are handled for free.

It was correct, and it was slow in a way no tuning could fix. A report over 600 shifts read about 3.5 billion rows. On our test cluster, at the 99th percentile:

| Report | 1 client | 4 clients |
|---|---|---|
| 600 shifts by key | 23.4 s | 119 s |
| hourly retail, 150 registers, a month | 17.0 s | failed on memory and time limits |
| daily retail, 150 registers, a month | 14.3 s | timed out on every call |

### What the Failures Had in Common

None of the three production failures was caused by storing an aggregate:

<figure class="agg-fig">
<svg viewBox="0 0 800 280" role="img" aria-label="A grid of the four attempts against four properties. Attempts 1 to 3 store an aggregate; each fails in a different column: a sum that cannot be corrected, reading too much, remembering every key. Attempt 4 stores nothing and fails on reading too much. The goal row stores an aggregate and fails on none.">
<text class="agg-yest" x="360" y="28" text-anchor="middle">stores an</text><text class="agg-yest" x="360" y="44" text-anchor="middle">aggregate</text>
<text class="agg-latet" x="480" y="28" text-anchor="middle">a sum can't</text><text class="agg-latet" x="480" y="44" text-anchor="middle">be corrected</text>
<text class="agg-latet" x="600" y="28" text-anchor="middle">reads</text><text class="agg-latet" x="600" y="44" text-anchor="middle">too much</text>
<text class="agg-latet" x="720" y="28" text-anchor="middle">remembers</text><text class="agg-latet" x="720" y="44" text-anchor="middle">every key</text>
<line class="agg-lane" x1="20" y1="58" x2="780" y2="58"/>
<text class="agg-txt" x="20" y="89">1. add up on insert</text>
<circle class="agg-yes" cx="360" cy="84" r="8"/><circle class="agg-fail" cx="480" cy="84" r="8"/><circle class="agg-no" cx="600" cy="84" r="5"/><circle class="agg-no" cx="720" cy="84" r="5"/>
<text class="agg-txt" x="20" y="125">2. recompute from the wide table</text>
<circle class="agg-yes" cx="360" cy="120" r="8"/><circle class="agg-no" cx="480" cy="120" r="5"/><circle class="agg-fail" cx="600" cy="120" r="8"/><circle class="agg-no" cx="720" cy="120" r="5"/>
<text class="agg-txt" x="20" y="161">3. remember every key</text>
<circle class="agg-yes" cx="360" cy="156" r="8"/><circle class="agg-no" cx="480" cy="156" r="5"/><circle class="agg-no" cx="600" cy="156" r="5"/><circle class="agg-fail" cx="720" cy="156" r="8"/>
<text class="agg-txt" x="20" y="197">4. store nothing</text>
<circle class="agg-no" cx="360" cy="192" r="5"/><circle class="agg-no" cx="480" cy="192" r="5"/><circle class="agg-fail" cx="600" cy="192" r="8"/><circle class="agg-no" cx="720" cy="192" r="5"/>
<line class="agg-lane" x1="20" y1="216" x2="780" y2="216"/>
<text class="agg-yest" x="20" y="249">what we wanted</text>
<circle class="agg-yes" cx="360" cy="244" r="8"/><circle class="agg-no" cx="480" cy="244" r="5"/><circle class="agg-no" cx="600" cy="244" r="5"/><circle class="agg-no" cx="720" cy="244" r="5"/>
</svg>
<figcaption class="agg-cap">Each production failure sits in its own column, and none of them in the first. The last row is the design we were looking for.</figcaption>
</figure>

Attempt 4 removed storage and paid for it on every read, which led to the same failure as attempt 2.

So the question became: can an aggregate be stored, survive duplicates without remembering keys, and be recomputed without reading the whole documents table?

## A Short Detour: Granules

To see why the read volume matters so much, it helps to know how ClickHouse reads.

A MergeTree table is split into parts, and each part is split into granules of 8,192 rows by default. The primary index is sparse: it stores the first key of every granule, not of every row.

<figure class="agg-fig">
<svg viewBox="0 0 800 225" role="img" aria-label="A part of five granules of 8192 rows. The sparse primary index above them holds only the first key of each granule. A filter on register B can only rule out granules 1 and 5, so granules 2 to 4 are read whole.">
<text class="agg-lbl" x="40" y="18">sparse primary index: the first key of every granule</text>
<rect class="agg-box" x="40" y="28" width="132" height="34" rx="4"/><text class="agg-txt" x="106" y="50" text-anchor="middle">A, 0001</text>
<rect class="agg-box" x="188" y="28" width="132" height="34" rx="4"/><text class="agg-txt" x="254" y="50" text-anchor="middle">A, 5230</text>
<rect class="agg-box" x="336" y="28" width="132" height="34" rx="4"/><text class="agg-txt" x="402" y="50" text-anchor="middle">B, 0410</text>
<rect class="agg-box" x="484" y="28" width="132" height="34" rx="4"/><text class="agg-txt" x="550" y="50" text-anchor="middle">B, 8812</text>
<rect class="agg-box" x="632" y="28" width="132" height="34" rx="4"/><text class="agg-txt" x="698" y="50" text-anchor="middle">C, 0007</text>
<path class="agg-arr" d="M106,62 L106,100"/><path class="agg-arr" d="M254,62 L254,100"/><path class="agg-arr" d="M402,62 L402,100"/><path class="agg-arr" d="M550,62 L550,100"/><path class="agg-arr" d="M698,62 L698,100"/>
<rect class="agg-grp" x="28" y="92" width="748" height="80" rx="8"/>
<text class="agg-lbl" x="772" y="188" text-anchor="end">one part</text>
<rect class="agg-box" x="40" y="104" width="132" height="56" rx="6"/><text class="agg-txt" x="106" y="128" text-anchor="middle">granule 1</text><text class="agg-lbl" x="106" y="147" text-anchor="middle">8192 rows</text>
<rect class="agg-boxd" x="188" y="104" width="132" height="56" rx="6"/><text class="agg-txt" x="254" y="128" text-anchor="middle">granule 2</text><text class="agg-yest" x="254" y="147" text-anchor="middle">read whole</text>
<rect class="agg-boxd" x="336" y="104" width="132" height="56" rx="6"/><text class="agg-txt" x="402" y="128" text-anchor="middle">granule 3</text><text class="agg-yest" x="402" y="147" text-anchor="middle">read whole</text>
<rect class="agg-boxd" x="484" y="104" width="132" height="56" rx="6"/><text class="agg-txt" x="550" y="128" text-anchor="middle">granule 4</text><text class="agg-yest" x="550" y="147" text-anchor="middle">read whole</text>
<rect class="agg-box" x="632" y="104" width="132" height="56" rx="6"/><text class="agg-txt" x="698" y="128" text-anchor="middle">granule 5</text><text class="agg-lbl" x="698" y="147" text-anchor="middle">8192 rows</text>
<text class="agg-yest" x="40" y="210">WHERE register = 'B': granule 2 may end with B, so 2, 3 and 4 are read</text>
</svg>
<figcaption class="agg-cap">The index knows where each granule starts, not what is inside it.</figcaption>
</figure>

When a query filters by the primary key, ClickHouse finds the granules that may contain matching rows and reads them whole. Skip indexes (for example `minmax` over a date column) can throw away more granules, but only if the values inside a granule are close to each other.

So what a query reads depends far more on how rows are laid out than on how many rows it needs. If the rows you want are scattered across the table, you read the granules around each of them:

<figure class="agg-fig">
<svg viewBox="0 0 800 240" role="img" aria-label="Two layouts of the same table, eight granules each. Sorted by hour, the rows a query needs are spread over all eight granules, and all eight are read. Sorted by register, the same rows sit together in two granules, and only those two are read.">
<text class="agg-txt" x="36" y="22">sorted by hour first: the rows we need are everywhere</text>
<rect class="agg-boxd" x="36" y="32" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="40" y="40" width="78" height="44"/>
<rect class="agg-boxd" x="126" y="32" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="130" y="40" width="78" height="44"/>
<rect class="agg-boxd" x="216" y="32" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="220" y="40" width="78" height="44"/>
<rect class="agg-boxd" x="306" y="32" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="310" y="40" width="78" height="44"/>
<rect class="agg-boxd" x="396" y="32" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="400" y="40" width="78" height="44"/>
<rect class="agg-boxd" x="486" y="32" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="490" y="40" width="78" height="44"/>
<rect class="agg-boxd" x="576" y="32" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="580" y="40" width="78" height="44"/>
<rect class="agg-boxd" x="666" y="32" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="670" y="40" width="78" height="44"/>
<rect class="agg-yes" x="60" y="40" width="7" height="44"/>
<rect class="agg-yes" x="180" y="40" width="7" height="44"/>
<rect class="agg-yes" x="230" y="40" width="7" height="44"/><rect class="agg-yes" x="280" y="40" width="7" height="44"/>
<rect class="agg-yes" x="340" y="40" width="7" height="44"/>
<rect class="agg-yes" x="400" y="40" width="7" height="44"/><rect class="agg-yes" x="470" y="40" width="7" height="44"/>
<rect class="agg-yes" x="530" y="40" width="7" height="44"/>
<rect class="agg-yes" x="600" y="40" width="7" height="44"/><rect class="agg-yes" x="630" y="40" width="7" height="44"/>
<rect class="agg-yes" x="730" y="40" width="7" height="44"/>
<rect class="agg-yes" x="690" y="40" width="7" height="44"/>
<text class="agg-latet" x="36" y="112">8 of 8 granules read</text>
<text class="agg-txt" x="36" y="140">sorted by register first: the same rows sit together</text>
<rect class="agg-box" x="36" y="150" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="40" y="158" width="78" height="44"/>
<rect class="agg-box" x="126" y="150" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="130" y="158" width="78" height="44"/>
<rect class="agg-boxd" x="216" y="150" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="220" y="158" width="78" height="44"/>
<rect class="agg-boxd" x="306" y="150" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="310" y="158" width="78" height="44"/>
<rect class="agg-box" x="396" y="150" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="400" y="158" width="78" height="44"/>
<rect class="agg-box" x="486" y="150" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="490" y="158" width="78" height="44"/>
<rect class="agg-box" x="576" y="150" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="580" y="158" width="78" height="44"/>
<rect class="agg-box" x="666" y="150" width="84" height="60" rx="4"/><rect fill="url(#agg-rows)" x="670" y="158" width="78" height="44"/>
<rect class="agg-yes" x="260" y="158" width="7" height="44"/><rect class="agg-yes" x="270" y="158" width="7" height="44"/>
<rect class="agg-yes" x="280" y="158" width="7" height="44"/><rect class="agg-yes" x="290" y="158" width="7" height="44"/>
<rect class="agg-yes" x="310" y="158" width="7" height="44"/><rect class="agg-yes" x="320" y="158" width="7" height="44"/>
<rect class="agg-yes" x="330" y="158" width="7" height="44"/><rect class="agg-yes" x="340" y="158" width="7" height="44"/>
<rect class="agg-yes" x="350" y="158" width="7" height="44"/><rect class="agg-yes" x="360" y="158" width="7" height="44"/>
<rect class="agg-yes" x="370" y="158" width="7" height="44"/><rect class="agg-yes" x="380" y="158" width="7" height="44"/>
<text class="agg-yest" x="36" y="230">2 of 8 granules read</text>
</svg>
<figcaption class="agg-cap">The same twelve rows in both layouts. Only the sort order differs.</figcaption>
</figure>

In our compute-at-read attempt, the narrow retail table was sorted by hour first. Within an hour, receipts of thousands of registers were interleaved, so a report for 150 registers touched nearly every granule of the month.

We even had to rewrite our own acceptance criteria: a limit on "rows read" was impossible to meet with any query shape, because the boundary granules always contain someone else's rows.

The reverse also holds: if the rows of one unit of work sit next to each other, reading them costs a handful of granules. The final design relies on this.

## The Idea: An Aggregate Is a Function

In the design we kept, an aggregate is not a running sum of what was inserted but a function of its documents.

Take the smallest useful unit of aggregation, a cell:

```scala
case class CellKey(register: RegisterId, shift: Int, hour: Instant) // hour in UTC
```

Every document belongs to exactly one cell, and the link is not stored anywhere. It is computed from three columns of the document:

<figure class="agg-fig">
<svg viewBox="0 0 800 244" role="img" aria-label="key of d is register, shift and hour of d. D of k is the set of documents d with key d equal to k. L of k is the latest version of each document in D of k. V of k sums the measures of the documents in L of k.">
<rect class="agg-mcard" x="1" y="1" width="798" height="242" rx="8"/><rect class="agg-mbar" x="1" y="1" width="4" height="242" rx="2"/>
<text class="agg-m" x="36" y="40"><tspan class="agg-mf">key</tspan>(<tspan class="agg-mv">d</tspan>) = ( <tspan class="agg-mf">register</tspan>(<tspan class="agg-mv">d</tspan>), <tspan class="agg-mf">shift</tspan>(<tspan class="agg-mv">d</tspan>), <tspan class="agg-mf">hour</tspan>(<tspan class="agg-mv">t</tspan>(<tspan class="agg-mv">d</tspan>)) )</text>
<text class="agg-lbl" x="36" y="60">the cell a document belongs to; t(d) is the document's own time, hour() rounds it down to the hour</text>
<text class="agg-m" x="36" y="96"><tspan class="agg-mf">D</tspan>(<tspan class="agg-mv">k</tspan>) = { <tspan class="agg-mv">d</tspan> : <tspan class="agg-mf">key</tspan>(<tspan class="agg-mv">d</tspan>) = <tspan class="agg-mv">k</tspan> }</text>
<text class="agg-lbl" x="36" y="116">all documents of cell k, found by the rule, not by a stored link</text>
<text class="agg-m" x="36" y="152"><tspan class="agg-mf">L</tspan>(<tspan class="agg-mv">k</tspan>) = { <tspan class="agg-mf">argmax</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="agg-mv">v</tspan> ∈ <tspan class="agg-mf">D</tspan>(<tspan class="agg-mv">k</tspan>), <tspan class="agg-mf">id</tspan>(<tspan class="agg-mv">v</tspan>) = <tspan class="agg-mf">id</tspan>(<tspan class="agg-mv">d</tspan>)</tspan> <tspan class="agg-mv">w</tspan>(<tspan class="agg-mv">v</tspan>) : <tspan class="agg-mv">d</tspan> ∈ <tspan class="agg-mf">D</tspan>(<tspan class="agg-mv">k</tspan>) }</text>
<text class="agg-lbl" x="36" y="182">for each document, its version with the latest write time w; a re-delivered document collapses here</text>
<text class="agg-m" x="36" y="222"><tspan class="agg-mf">V</tspan>(<tspan class="agg-mv">k</tspan>) = <tspan font-size="26">Σ</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="agg-mv">d</tspan> ∈ <tspan class="agg-mf">L</tspan>(<tspan class="agg-mv">k</tspan>)</tspan> <tspan class="agg-mv">m</tspan>(<tspan class="agg-mv">d</tspan>)</text>
<text class="agg-lbl" x="300" y="222">per measure; first and last numbers and dates take min and max</text>
</svg>
</figure>

The same in code:

```scala
def key(d: Document): CellKey =
  CellKey(d.register, d.shift, d.date.truncatedTo(HOURS))

def docs(k: CellKey): Seq[Document] =
  allDocuments.filter(d => key(d) == k)

def value(k: CellKey): Measures =
  docs(k)
    .groupBy(_.id)
    .values
    .map(_.maxBy(_.writtenAt))        // a re-delivered document collapses here
    .foldLeft(Measures.empty)(_ + _)  // sums, counts, min, max
```

The value of a cell can be computed again at any moment, because the documents are still there and the rule is the same. A new computation is written as a new row, versioned by the time of recomputation, and supersedes the old one in a `ReplacingMergeTree`.

This gives three properties:

- **A re-delivered document changes nothing.** During recomputation the rows of one document are collapsed by document key (`argMax` by write time). Two deliveries become one row; the cell value stays the same. No dictionary, no key table, no memory of what was seen.
- **Recomputing twice is harmless.** Recomputing a cell with no new documents produces the same value. On one node, two recomputations running at once both write a version, and the one that started later has seen at least as much, so it wins. Replicas are a different story (see What It Costs).
- **A late document is not lost.** A receipt that is a month late marks its month-old cell as dirty now, and that cell is recomputed in the next pass.

The second half of the idea is that measures merge across cells. Every measure in a cell is a sum, a count, a `min` or a `max`. Every document is in exactly one cell. So any report over any set of cells is just the combination of their values, without double counting:

<figure class="agg-fig">
<svg viewBox="0 0 800 255" role="img" aria-label="A grid of cells, two registers by four hours, with a revenue number in each. The 14:00 column adds up to the store's hour, 130 plus 150 is 280. Register A's row is its shift 75, 120 plus 80 plus 200 plus 150 is 550. Hours add up to a day, days to a month.">
<text class="agg-lbl" x="137" y="60" text-anchor="middle">11:00</text><text class="agg-lbl" x="199" y="60" text-anchor="middle">12:00</text>
<text class="agg-lbl" x="261" y="60" text-anchor="middle">13:00</text><text class="agg-lbl" x="323" y="60" text-anchor="middle">14:00</text>
<text class="agg-txt" x="20" y="91">register B</text><text class="agg-lbl" x="20" y="107">shift 12</text>
<rect class="agg-box" x="110" y="73" width="54" height="40" rx="4"/><text class="agg-txt" x="137" y="98" text-anchor="middle">70</text>
<rect class="agg-box" x="172" y="73" width="54" height="40" rx="4"/><text class="agg-txt" x="199" y="98" text-anchor="middle">110</text>
<rect class="agg-box" x="234" y="73" width="54" height="40" rx="4"/><text class="agg-txt" x="261" y="98" text-anchor="middle">90</text>
<rect class="agg-box" x="296" y="73" width="54" height="40" rx="4"/><text class="agg-txt" x="323" y="98" text-anchor="middle">130</text>
<text class="agg-txt" x="20" y="141">register A</text><text class="agg-lbl" x="20" y="157">shift 75</text>
<rect class="agg-box" x="110" y="123" width="54" height="40" rx="4"/><text class="agg-txt" x="137" y="148" text-anchor="middle">120</text>
<rect class="agg-box" x="172" y="123" width="54" height="40" rx="4"/><text class="agg-txt" x="199" y="148" text-anchor="middle">80</text>
<rect class="agg-box" x="234" y="123" width="54" height="40" rx="4"/><text class="agg-txt" x="261" y="148" text-anchor="middle">200</text>
<rect class="agg-box" x="296" y="123" width="54" height="40" rx="4"/><text class="agg-txt" x="323" y="148" text-anchor="middle">150</text>
<rect class="agg-outa" x="290" y="67" width="66" height="102" rx="6"/>
<rect class="agg-outw" x="104" y="119" width="252" height="48" rx="6"/>
<path class="agg-arra" d="M356,93 C398,93 394,58 436,58"/>
<rect class="agg-boxd" x="440" y="30" width="190" height="56" rx="6"/>
<text class="agg-txt" x="535" y="52" text-anchor="middle">hour 14:00 of the store</text>
<text class="agg-yest" x="535" y="71" text-anchor="middle">130 + 150 = 280</text>
<path class="agg-arrw" d="M356,143 C398,143 394,178 436,178"/>
<rect class="agg-boxw" x="440" y="150" width="190" height="56" rx="6"/>
<text class="agg-txt" x="535" y="172" text-anchor="middle">shift 75 of register A</text>
<text class="agg-latet" x="535" y="191" text-anchor="middle">120 + 80 + 200 + 150 = 550</text>
<path class="agg-arr" d="M630,58 L656,58"/>
<rect class="agg-box" x="660" y="30" width="130" height="56" rx="6"/>
<text class="agg-txt" x="725" y="52" text-anchor="middle">day</text>
<text class="agg-lbl" x="725" y="71" text-anchor="middle">its 24 hours</text>
<path class="agg-arr" d="M725,86 L725,146"/>
<rect class="agg-box" x="660" y="150" width="130" height="56" rx="6"/>
<text class="agg-txt" x="725" y="172" text-anchor="middle">month</text>
<text class="agg-lbl" x="725" y="191" text-anchor="middle">its days</text>
<text class="agg-lbl" x="400" y="242" text-anchor="middle">sums and counts add up; min and max keep the first and last document numbers and dates</text>
</svg>
<figcaption class="agg-cap">The same cells are summed along different axes. A week, a quarter or any set of stores would be the same addition.</figcaption>
</figure>

None of these needs a table of its own: a new report is a new query over the same cells. Even time zones should work, as long as the offset from UTC is a fixed whole number of hours, since a local day boundary is then still an hour boundary.

We checked this the boring way: hours, days and shifts assembled from cells matched the same reports computed directly from documents on all 209 measures.

### Not Only Receipts

The pattern is not specific to registers and receipts:

| In this article | In general |
|---|---|
| a register | a source of events: a device, an account, a sensor |
| a receipt, stored as a document | an event with a stable key of its own |
| a shift, an hour | natural windows the events fall into |
| sums, counts, `min`, `max` | measures that merge |

It needs three things: every event carries everything needed to compute its cell key, that key does not change when the event is delivered again, and the measures add up, or at least merge. Where those hold, the aggregate can be a function of its events instead of a running sum.

## Making It Work in ClickHouse

Most of the work was in the ClickHouse mechanics.

<figure class="agg-fig">
<svg viewBox="0 0 800 590" role="img" aria-label="Kafka feeds the consumer, which inserts a batch into the documents table on the register's shard. In the same insert, views write a narrow copy of the documents and a dirty mark for each touched cell. Every 30 seconds a refreshable view recomputes the marked cells from the narrow table; another assembles shifts from cells. Reports read cells and shifts.">
<rect class="agg-box" x="315" y="20" width="170" height="56" rx="6"/>
<text class="agg-txt" x="400" y="44" text-anchor="middle">Kafka</text>
<text class="agg-lbl" x="400" y="63" text-anchor="middle">at least once</text>
<path class="agg-arr" d="M400,76 L400,106"/>
<rect class="agg-box" x="315" y="110" width="170" height="56" rx="6"/>
<text class="agg-txt" x="400" y="134" text-anchor="middle">consumer</text>
<text class="agg-lbl" x="400" y="153" text-anchor="middle">one insert per batch</text>
<path class="agg-arr" d="M400,166 L400,196"/>
<g transform="translate(-192,192)">
<rect class="agg-grp" x="405" y="8" width="390" height="204" rx="8"/>
<rect class="agg-box" x="420" y="20" width="170" height="56" rx="6"/>
<text class="agg-txt" x="505" y="44" text-anchor="middle">documents</text>
<text class="agg-lbl" x="505" y="63" text-anchor="middle">on the register's shard</text>
<path class="agg-arr" d="M590,48 L616,48"/>
<rect class="agg-box" x="620" y="20" width="170" height="56" rx="6"/>
<text class="agg-txt" x="705" y="44" text-anchor="middle">narrow documents</text>
<text class="agg-lbl" x="705" y="63" text-anchor="middle">by register, number</text>
<path class="agg-arr" d="M505,76 L505,126"/>
<rect class="agg-boxd" x="420" y="130" width="170" height="56" rx="6"/>
<text class="agg-txt" x="505" y="154" text-anchor="middle">dirty-cell journal</text>
<text class="agg-lbl" x="505" y="173" text-anchor="middle">which cells, since when</text>
<text class="agg-lbl" x="597" y="206" text-anchor="end">written in one insert</text>
<path class="agg-arra" d="M705,76 L705,236"/>
<path class="agg-arra" d="M590,158 C635,158 660,195 662,236"/>
<text class="agg-yest" x="697" y="112" text-anchor="end">recompute,</text>
<text class="agg-yest" x="697" y="128" text-anchor="end">every 30 s</text>
<rect class="agg-boxd" x="620" y="240" width="170" height="56" rx="6"/>
<text class="agg-txt" x="705" y="264" text-anchor="middle">cells</text>
<text class="agg-lbl" x="705" y="283" text-anchor="middle">register, shift, hour</text>
<path class="agg-arra" d="M620,272 L544,272"/>
<text class="agg-yest" x="582" y="262" text-anchor="middle">every 30 s</text>
<rect class="agg-box" x="390" y="240" width="150" height="56" rx="6"/>
<text class="agg-txt" x="465" y="264" text-anchor="middle">shifts</text>
<text class="agg-lbl" x="465" y="283" text-anchor="middle">from cells</text>
<path class="agg-arr" d="M465,296 L465,326"/>
<path class="agg-arr" d="M705,296 L705,326"/>
<rect class="agg-box" x="440" y="330" width="290" height="56" rx="6"/>
<text class="agg-txt" x="585" y="354" text-anchor="middle">reports</text>
<text class="agg-lbl" x="585" y="373" text-anchor="middle">read cells and shifts</text>
</g>
</svg>
<figcaption class="agg-cap">The document, its narrow copy and its dirty mark are written by one insert. Everything after that is recomputation.</figcaption>
</figure>

### A Narrow Table With the Right Order

Recomputation never reads the wide documents table. A materialized view copies every document row into a narrow table with only the columns the measures need, sorted by `(register, document number)`.

There is no time column in that key, and it does not need one. A register numbers its documents in the order it issues them, so the documents of one register in one hour sit next to each other in every part. Finding the documents of a cell is a short range scan. The densest cell in our test history, a little over 5,000 documents, reads 20 granules and finishes in 7 ms. Compared to reading the same minute of traffic from the wide table, the narrow one reads 30 to 37 times fewer rows.

### A Journal Written in the Same Insert

A second materialized view on the same insert writes a dirty mark for every touched cell: the cell key and the time of the mark. No sums, just "this cell needs recomputing".

Both views run as part of the same insert as the document itself, so our code has no separate step between "document stored" and "cell marked". That is not a transaction, though. ClickHouse writes the source table and its views one after another, and if a view fails after the document is stored, the insert reports an error with the document already in place. Our consumer then retries the batch, which normally writes the mark again, and a repeated document is harmless here. We have not checked how that retry meets insert deduplication on replicated tables: if a repeated block is dropped before its views run, the mark is lost and the cell is not recomputed.

Status updates for documents arrive as separate rows without a date and shift, and the views simply skip them, so a status never triggers a recompute. No measure in a cell depends on a status.

The journal is a `ReplacingMergeTree` keyed by the cell, so repeated marks of one cell collapse into one row with the latest time. Small inserts produced about 14 marks per cell per hour; after collapsing it was 1.16, and the main recompute pass became five times cheaper.

### "Recomputed" Lives in the Data

A refreshable materialized view runs every 30 seconds and recomputes the cells that need it. The obvious way to track progress is a cursor, "processed up to time T". We did not keep one.

Instead, each pass compares two times taken from the data itself:

<figure class="agg-fig">
<svg viewBox="0 0 800 228" role="img" aria-label="M of k is the latest mark time of cell k in the journal. R of k is the latest recomputation time of cell k, or zero if it was never computed. dirty is the set of cells with M greater than R minus 60 seconds. A pass writes a new version for the 50 000 oldest dirty cells.">
<rect class="agg-mcard" x="1" y="1" width="798" height="226" rx="8"/><rect class="agg-mbar" x="1" y="1" width="4" height="226" rx="2"/>
<text class="agg-m" x="36" y="40"><tspan class="agg-mf">M</tspan>(<tspan class="agg-mv">k</tspan>) = <tspan class="agg-mf">max</tspan> { <tspan class="agg-mv">t</tspan> : (<tspan class="agg-mv">k</tspan>, <tspan class="agg-mv">t</tspan>) &#x2208; journal }</text>
<text class="agg-lbl" x="36" y="60">the latest mark of cell k</text>
<text class="agg-m" x="36" y="96"><tspan class="agg-mf">R</tspan>(<tspan class="agg-mv">k</tspan>) = <tspan class="agg-mf">max</tspan> { <tspan class="agg-mv">t</tspan> : (<tspan class="agg-mv">k</tspan>, <tspan class="agg-mv">t</tspan>) &#x2208; cells }</text>
<text class="agg-lbl" x="36" y="116">the latest recomputation of cell k, or 0 if it was never computed</text>
<text class="agg-m" x="36" y="152"><tspan class="agg-mf">dirty</tspan> = { <tspan class="agg-mv">k</tspan> : <tspan class="agg-mf">M</tspan>(<tspan class="agg-mv">k</tspan>) &gt; <tspan class="agg-mf">R</tspan>(<tspan class="agg-mv">k</tspan>) &#x2212; 60 s }</text>
<text class="agg-lbl" x="36" y="172">marked after its last recomputation, with a minute of overlap</text>
<text class="agg-m" x="36" y="208"><tspan class="agg-mf">pass</tspan> = { (<tspan class="agg-mv">k</tspan>, <tspan class="agg-mf">V</tspan>(<tspan class="agg-mv">k</tspan>), now) : <tspan class="agg-mv">k</tspan> &#x2208; <tspan class="agg-mf">oldest</tspan>(<tspan class="agg-mf">dirty</tspan>, 50 000) }</text>
<text class="agg-lbl" x="580" y="208">a new version per cell</text>
</svg>
</figure>

The same in code:

```scala
def lastMark(k: CellKey): Instant =
  journal.marks(k).max

def lastRecompute(k: CellKey): Instant =
  cells.versions(k).maxOption.getOrElse(Instant.EPOCH) // never computed yet

def dirty(k: CellKey): Boolean =
  lastMark(k).isAfter(lastRecompute(k).minusSeconds(60))

def pass(): Seq[Cell] =
  journal.keys
    .filter(dirty)
    .sortBy(lastMark)
    .take(50_000)
    .map(k => Cell(k, value(k), recomputedAt = Instant.now()))
```

A cell that was never computed has no version yet, so it counts as computed at the epoch and turns dirty with its first mark.

A cursor has to be moved atomically together with the write, and it breaks under retries, parallel runs and restarts. The comparison is not affected by any of these. After an outage of up to a day, the next pass finds every cell that was marked after its last recompute. The 60-second overlap covers the fact that parts of one insert become visible at slightly different moments, so a mark can appear just before its document does. The price is an occasional extra recompute.

A pass reads only the documents of the cells it recomputes.

### Two Windows That Must Not Overlap

The main pass looks at marks from the last two hours. A second, catch-up view runs every ten minutes and looks at marks between two hours and one day old. It exists for outages longer than two hours.

The windows must not overlap. In an early version they did, and after an outage both views grabbed the same cells at the same time: 26 seconds and 2 x 2.8 GB of memory on one node for work that should have been done once.

### Version by Snapshot Time

The version of a cell row is the time its recompute query started, not the latest write time of its documents. We tried the latter first. With many documents written in the same second, and recomputations running close together, the order of versions became undefined. A later snapshot always sees at least as many documents as an earlier one, so snapshot time is a version that is always correct.

That "at least as many" holds for snapshots taken on the same node. Across replicas it does not, and that is the weakest point of the whole design.

### Shifts From Cells, Not From Documents

A shift is assembled from its cells by a second refreshable view: take the latest row of every cell of the shift, sum the measures, take `min` and `max` of numbers and dates. A busy register can have 142,000 documents in a shift, and some shifts stay open for months. Rebuilding a shift from documents on every new receipt would reread it all; from cells it reads one row per hour of the shift.

### Reading Without FINAL

Readers take the latest row of each key with `LIMIT 1 BY` over the key, ordered by recomputation time, and run that subquery on every shard over its local tables. Everything belonging to one register (documents, marks, cells, shifts) lives on the same shard, so "the latest row" is decided correctly locally and nobody waits for background merges.

## Things That Bit Us

### Backfill Floods the Shift View

When we filled cells for 77 days of history in one go, every filled cell looked "recently recomputed" to the shift view. It then spent two hours regrouping millions of cells and writing nothing new.

<figure class="agg-fig">
<svg viewBox="0 0 800 210" role="img" aria-label="Two versions of the same backfill. In the first, filled cells get recomputed_at = now; the shift view sees them as recomputed in the last two hours and regroups every one, 88 million rows and half a core per pass. In the fixed one, they get recomputed_at = now minus 2 days; the shift view skips them, 4 million rows per pass and almost no CPU.">
<text class="agg-latet" x="20" y="20">first backfill</text>
<rect class="agg-box" x="20" y="30" width="220" height="56" rx="6"/>
<text class="agg-txt" x="130" y="54" text-anchor="middle">backfill writes 77 days of cells</text>
<text class="agg-monow" x="130" y="73" text-anchor="middle">recomputed_at = now</text>
<path class="agg-arrw" d="M240,58 L296,58"/>
<rect class="agg-box" x="300" y="30" width="220" height="56" rx="6"/>
<text class="agg-txt" x="410" y="54" text-anchor="middle">shift view sees them</text>
<text class="agg-latet" x="410" y="73" text-anchor="middle">as just recomputed</text>
<path class="agg-arrw" d="M520,58 L586,58"/>
<rect class="agg-boxw" x="590" y="30" width="190" height="56" rx="6"/>
<text class="agg-txt" x="685" y="54" text-anchor="middle">regroups every cell</text>
<text class="agg-latet" x="685" y="73" text-anchor="middle">88M rows, 0.5 core a pass</text>
<text class="agg-yest" x="20" y="130">backfill, fixed</text>
<rect class="agg-box" x="20" y="140" width="220" height="56" rx="6"/>
<text class="agg-txt" x="130" y="164" text-anchor="middle">backfill writes 77 days of cells</text>
<text class="agg-monoa" x="130" y="183" text-anchor="middle">recomputed_at = now - 2 days</text>
<path class="agg-arra" d="M240,168 L296,168"/>
<rect class="agg-box" x="300" y="140" width="220" height="56" rx="6"/>
<text class="agg-txt" x="410" y="164" text-anchor="middle">shift view sees them</text>
<text class="agg-yest" x="410" y="183" text-anchor="middle">as two days old</text>
<path class="agg-arra" d="M520,168 L586,168"/>
<rect class="agg-boxd" x="590" y="140" width="190" height="56" rx="6"/>
<text class="agg-txt" x="685" y="164" text-anchor="middle">skips them</text>
<text class="agg-yest" x="685" y="183" text-anchor="middle">4M rows, almost no CPU</text>
</svg>
<figcaption class="agg-cap">The shift view only looks at cells recomputed in the last 2 hours, its catch-up at the last day. Two days old is outside both; a new mark still gets a backfilled cell recomputed.</figcaption>
</figure>

Backfilled rows now get a recomputation time two days in the past, outside both windows. The shift view went from 88 million rows and half a core per pass to 4 million rows and almost nothing.

### Filter in a Subquery Inside Insert Views

On ClickHouse 26.3 we saw an insert view write column values shifted. Three things came together: output aliases that matched source column names, the filter in the same query, and blocks that mixed documents with status rows.

Moving the filter into a subquery fixed it, and we have a test for it:

```sql
-- values came out shifted
CREATE MATERIALIZED VIEW docs_mv TO docs AS
SELECT register_id, doc_number, ifNull(total, 0) AS total, ...
FROM transactions
WHERE type IS NOT NULL;

-- fixed
CREATE MATERIALIZED VIEW docs_mv TO docs AS
SELECT register_id, doc_number, ifNull(total, 0) AS total, ...
FROM (SELECT * FROM transactions WHERE type IS NOT NULL);
```

### Granule Size Matters for Wide Rows

A cell has about 250 columns, and the retail report reads 189 of them. With the default granule of 8,192 rows, a query for 150 registers decompressed about 1.1 GB to use 22,000 cells.

Smaller granules and smaller compressed blocks on the cell table:

```sql
SETTINGS index_granularity = 1024,
         min_compress_block_size = 8192,
         max_compress_block_size = 65536
```

The same query decompressed about 0.3 GB, and CPU dropped by about 30% for 9% more disk.

### One Aggregate State per Document, Not per Column

Collapsing a document with a separate `argMax` per column builds dozens of aggregate states per document. One `argMax` over a tuple of the columns builds one:

```sql
-- one state per column
SELECT doc_key,
       argMax(total, updated_at),
       argMax(cash, updated_at),
       argMax(card, updated_at),
       ...
FROM docs GROUP BY doc_key;

-- one state per document
SELECT doc_key,
       argMax((total, cash, card, ...), updated_at) AS doc
FROM docs GROUP BY doc_key;
```

## The Numbers

The test cluster is small: two shards on one machine with six physical cores, a five-CPU limit per ClickHouse node, tens of thousands of registers, 77 days of history. Report latency, 99th percentile:

<figure class="agg-fig">
<svg viewBox="0 0 800 292" role="img" aria-label="p99 response time of four reports, computed from documents versus read from cells, at 1 and 4 clients, log scale. 1 client: 600 shifts 23.4 s vs 2.1 s; hourly retail 17.0 vs 2.5; daily retail 14.3 vs 1.3; revenue by field 7.9 vs 0.6. 4 clients: 600 shifts 119 s vs 2.7 s; hourly retail failed vs 6.1; daily retail timed out vs 3.7; revenue by field 100 vs 1.6. The report budget is 30 s.">
<circle class="agg-cells" cx="246" cy="22" r="6"/><text class="agg-lbl" x="258" y="26">cells</text>
<circle class="agg-docs" cx="316" cy="22" r="6"/><text class="agg-lbl" x="328" y="26">computed from documents at read time</text>
<line class="agg-budget" x1="584" y1="22" x2="606" y2="22"/><text class="agg-lbl" x="614" y="26">30 s report budget</text>
<text class="agg-txt" x="365" y="58" text-anchor="middle">1 client</text>
<text class="agg-txt" x="655" y="58" text-anchor="middle">4 clients</text>
<line class="agg-grid" x1="268.9" y1="72" x2="268.9" y2="236"/><line class="agg-grid" x1="314.8" y1="72" x2="314.8" y2="236"/><line class="agg-grid" x1="365" y1="72" x2="365" y2="236"/><line class="agg-grid" x1="461.1" y1="72" x2="461.1" y2="236"/>
<line class="agg-grid" x1="558.9" y1="72" x2="558.9" y2="236"/><line class="agg-grid" x1="604.8" y1="72" x2="604.8" y2="236"/><line class="agg-grid" x1="655" y1="72" x2="655" y2="236"/><line class="agg-grid" x1="751.1" y1="72" x2="751.1" y2="236"/>
<line class="agg-budget" x1="410.8" y1="72" x2="410.8" y2="236"/><line class="agg-budget" x1="700.8" y1="72" x2="700.8" y2="236"/>
<text class="agg-txt" x="20" y="104">600 shifts by key</text>
<text class="agg-txt" x="20" y="144">hourly retail, 150 registers</text>
<text class="agg-txt" x="20" y="184">daily retail, 150 registers</text>
<text class="agg-txt" x="20" y="224">revenue by field, 50 registers</text>
<line class="agg-bar" x1="299.9" y1="100" x2="400.5" y2="100"/>
<circle class="agg-cells" cx="299.9" cy="100" r="6"><title>600 shifts by key, 1 client: cells 2.1 s</title></circle><text class="agg-val" x="299.9" y="90" text-anchor="middle">2.1</text>
<circle class="agg-docs" cx="400.5" cy="100" r="6"><title>600 shifts by key, 1 client: from documents 23.4 s</title></circle><text class="agg-val" x="400.5" y="90" text-anchor="middle">23.4</text>
<line class="agg-bar" x1="307.2" y1="140" x2="387.1" y2="140"/>
<circle class="agg-cells" cx="307.2" cy="140" r="6"><title>hourly retail, 1 client: cells 2.5 s</title></circle><text class="agg-val" x="307.2" y="130" text-anchor="middle">2.5</text>
<circle class="agg-docs" cx="387.1" cy="140" r="6"><title>hourly retail, 1 client: from documents 17.0 s</title></circle><text class="agg-val" x="387.1" y="130" text-anchor="middle">17.0</text>
<line class="agg-bar" x1="279.9" y1="180" x2="379.9" y2="180"/>
<circle class="agg-cells" cx="279.9" cy="180" r="6"><title>daily retail, 1 client: cells 1.3 s</title></circle><text class="agg-val" x="279.9" y="170" text-anchor="middle">1.3</text>
<circle class="agg-docs" cx="379.9" cy="180" r="6"><title>daily retail, 1 client: from documents 14.3 s</title></circle><text class="agg-val" x="379.9" y="170" text-anchor="middle">14.3</text>
<line class="agg-bar" x1="247.6" y1="220" x2="355.2" y2="220"/>
<circle class="agg-cells" cx="247.6" cy="220" r="6"><title>revenue by field, 1 client: cells 0.6 s</title></circle><text class="agg-val" x="247.6" y="210" text-anchor="middle">0.6</text>
<circle class="agg-docs" cx="355.2" cy="220" r="6"><title>revenue by field, 1 client: from documents 7.9 s</title></circle><text class="agg-val" x="355.2" y="210" text-anchor="middle">7.9</text>
<line class="agg-bar" x1="600.4" y1="100" x2="758.3" y2="100"/>
<circle class="agg-cells" cx="600.4" cy="100" r="6"><title>600 shifts by key, 4 clients: cells 2.7 s</title></circle><text class="agg-val" x="600.4" y="90" text-anchor="middle">2.7</text>
<circle class="agg-docs" cx="758.3" cy="100" r="6"><title>600 shifts by key, 4 clients: from documents 119 s</title></circle><text class="agg-val" x="758.3" y="90" text-anchor="middle">119</text>
<line class="agg-barx" x1="634.4" y1="140" x2="775.6" y2="140"/>
<circle class="agg-cells" cx="634.4" cy="140" r="6"><title>hourly retail, 4 clients: cells 6.1 s</title></circle><text class="agg-val" x="634.4" y="130" text-anchor="middle">6.1</text>
<circle class="agg-docsx" cx="775.6" cy="140" r="6"><title>hourly retail, 4 clients: from documents failed on memory and time limits</title></circle><text class="agg-val" x="775.6" y="130" text-anchor="end">failed</text>
<line class="agg-barx" x1="613.5" y1="180" x2="775.6" y2="180"/>
<circle class="agg-cells" cx="613.5" cy="180" r="6"><title>daily retail, 4 clients: cells 3.7 s</title></circle><text class="agg-val" x="613.5" y="170" text-anchor="middle">3.7</text>
<circle class="agg-docsx" cx="775.6" cy="180" r="6"><title>daily retail, 4 clients: from documents, every call timed out at 180 s</title></circle><text class="agg-val" x="775.6" y="170" text-anchor="end">timed out</text>
<line class="agg-bar" x1="578.5" y1="220" x2="751.1" y2="220"/>
<circle class="agg-cells" cx="578.5" cy="220" r="6"><title>revenue by field, 4 clients: cells 1.6 s</title></circle><text class="agg-val" x="578.5" y="210" text-anchor="middle">1.6</text>
<circle class="agg-docs" cx="751.1" cy="220" r="6"><title>revenue by field, 4 clients: from documents 100 s</title></circle><text class="agg-val" x="751.1" y="210" text-anchor="middle">100</text>
<text class="agg-lbl" x="268.9" y="254" text-anchor="middle">1 s</text><text class="agg-lbl" x="314.8" y="254" text-anchor="middle">3</text><text class="agg-lbl" x="365" y="254" text-anchor="middle">10</text><text class="agg-lbl" x="410.8" y="254" text-anchor="middle">30</text><text class="agg-lbl" x="461.1" y="254" text-anchor="middle">100 s</text>
<text class="agg-lbl" x="558.9" y="254" text-anchor="middle">1 s</text><text class="agg-lbl" x="604.8" y="254" text-anchor="middle">3</text><text class="agg-lbl" x="655" y="254" text-anchor="middle">10</text><text class="agg-lbl" x="700.8" y="254" text-anchor="middle">30</text><text class="agg-lbl" x="751.1" y="254" text-anchor="middle">100 s</text>
<text class="agg-lbl" x="510" y="282" text-anchor="middle">p99 response time, log scale; hollow marks: no answer at all</text>
</svg>
<figcaption class="agg-cap">Every report from cells sits left of the budget line, under load as well. From documents, four clients were enough to push every report past it.</figcaption>
</figure>

<details class="agg-details">
<summary>The same numbers as a table</summary>

| Report | From documents, 1 client | Cells, 1 client | From documents, 4 clients | Cells, 4 clients |
|---|---|---|---|---|
| 600 shifts by key | 23.4 s | 2.1 s | 119 s | 2.7 s |
| hourly retail, 150 registers, a month | 17.0 s | 2.5 s | failed | 6.1 s |
| daily retail, 150 registers, a month | 14.3 s | 1.3 s | timed out | 3.7 s |
| revenue by custom field, 50 registers | 7.9 s | 0.6 s | 100 s | 1.6 s |

</details>

The 600-shift report reads up to 360 thousand rows instead of 3.5 billion. Every report call on the cells stays within twice the theoretical granule bound for the rows it needs.

Correctness was checked against an independent reference: an expected value for every day, shift, hour and month of 1,000 registers, computed by the load generator without the service.

There were zero mismatches across 31 checks, after three hours of live traffic in steps of increasing load, the last at three times the first.

## What It Costs

These are the trade-offs of the design and the gaps we know about.

<figure class="agg-fig">
<svg viewBox="0 0 800 160" role="img" aria-label="What happens to a document by how long recomputation has been down. With no outage a report sees it in 75 to 115 seconds. Down under 2 hours, the main view catches up; between 2 hours and a day, the catch-up view does. Between 1 and 7 days a manual pass is needed while the marks are still kept. Past 7 days the marks have expired.">
<path class="agg-outa" d="M20,38 L20,30 L480,30 L480,38" fill="none"/>
<text class="agg-yest" x="250" y="22" text-anchor="middle">handled on its own</text>
<path class="agg-outw" d="M485,38 L485,30 L790,30 L790,38" fill="none"/>
<text class="agg-latet" x="637" y="22" text-anchor="middle">needs a hand</text>
<rect class="agg-boxd" x="20" y="46" width="150" height="56" rx="6"/>
<text class="agg-txt" x="95" y="70" text-anchor="middle">no outage</text>
<text class="agg-lbl" x="95" y="89" text-anchor="middle">seen in 75-115 s</text>
<rect class="agg-boxd" x="175" y="46" width="150" height="56" rx="6"/>
<text class="agg-txt" x="250" y="70" text-anchor="middle">down under 2 h</text>
<text class="agg-lbl" x="250" y="89" text-anchor="middle">main view catches up</text>
<rect class="agg-boxd" x="330" y="46" width="150" height="56" rx="6"/>
<text class="agg-txt" x="405" y="70" text-anchor="middle">2 h to 1 day</text>
<text class="agg-lbl" x="405" y="89" text-anchor="middle">catch-up view</text>
<rect class="agg-boxw" x="485" y="46" width="150" height="56" rx="6"/>
<text class="agg-txt" x="560" y="70" text-anchor="middle">1 to 7 days</text>
<text class="agg-lbl" x="560" y="89" text-anchor="middle">manual pass</text>
<rect class="agg-boxw" x="640" y="46" width="150" height="56" rx="6"/>
<text class="agg-txt" x="715" y="70" text-anchor="middle">over 7 days</text>
<text class="agg-lbl" x="715" y="89" text-anchor="middle">marks have expired</text>
<text class="agg-lbl" x="327.5" y="122" text-anchor="middle">2 h</text>
<text class="agg-lbl" x="482.5" y="122" text-anchor="middle">1 day</text>
<text class="agg-lbl" x="637.5" y="122" text-anchor="middle">7 days</text>
<text class="agg-lbl" x="405" y="150" text-anchor="middle">how long recomputation has been down</text>
</svg>
<figcaption class="agg-cap">The windows bound how long an outage can be, not how late a document can be.</figcaption>
</figure>

- **Lag.** A report sees a new document one or two passes later, usually 75 to 115 seconds after insert. We decided not to compute the pending tail at read time.
- **Long outages need a hand.** Marks older than a day are not picked up automatically. The journal keeps marks for seven days, so there is time to run a manual pass.
- **Replicas recompute on their own, and that is a hole.** Refreshable views run on each replica of a shard, and each replica sees its own copy of the documents and of the journal, which replicate separately. A replica can see a mark before the document behind it, recompute the cell without that document, and stamp the result with a later time than the correct version from the other replica. The later stamp wins. If the gap outlasts the 60-second overlap, nothing marks the cell again until its next document, and for a late document that may be never. Our test cluster had one replica per shard, so it never showed up there; we found this one on paper, while writing this post. The fixes we see: let one replica per shard recompute, derive the version from the documents rather than the clock, or sweep yesterday's cells once a day regardless of their stamps.
- **A retried insert may lose its mark.** If a view fails after its document is stored, the batch is retried, and on replicated tables insert deduplication can drop the repeated block before its views run. Then nothing marks the cell until its next document. We have not checked this yet.
- **Only mergeable measures.** Sums, counts, `min` and `max` merge across cells. Exact distinct counts and exact medians do not add up as plain numbers. ClickHouse can keep a mergeable aggregate state in a cell instead of a number: `uniqState` and `quantileState` combine across cells with `uniqMerge` and `quantileMerge`, approximately, with relatively little storage. The exact variants merge too, but carry every value. Our reports need none of this today; if one does, a cell can hold such states.
- **Backfill is a procedure.** Cells for existing history are built by a separate script, about an hour and a half for our 77 test days.

## Closing

Each attempt fixed the previous failure and ran into the next problem: duplicates, scattered reads, memory for deduplication, and finally the cost of reading everything on every request.

None of it is new, and we could have known that sooner. Databases call it incremental view maintenance: record which parts of a materialized result a change touched, recompute only those parts, replace the old values. Oracle's [partition change tracking](https://docs.oracle.com/en/database/oracle/oracle-database/21/dwhsg/refreshing-materialized-views.html) does exactly that for materialized views, and a [survey from 1995](http://sites.computer.org/debull/95JUN-CD.pdf) had already catalogued the techniques. Our own part is only the choice of the cell, and keeping the mark, the version and the "recomputed" flag inside ClickHouse tables themselves.

What finally worked was not a new ClickHouse feature but a different meaning of an aggregate. As long as an aggregate is a sum of inserts, every duplicate and every late document is a correction you have to remember. When it is a function of its documents, a duplicate or a late document needs no correction: the cell is computed again.
