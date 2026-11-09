---
title: "Phoenix vs ClickHouse: Why the Argument Keeps Coming Back"
slug: "phoenix-vs-clickhouse"
description: "The standup argument about Phoenix and ClickHouse keeps coming back because both sides are right. Both engines are log-structured, and four knobs decide which workload each one handles well. Measured on a test stand, with Cassandra as a third engine."
date: "2026-11-09T11:00:00Z"
tags: ["hbase", "clickhouse", "cassandra", "databases", "data-engineering"]
draft: true
cover_image: "phoenix-vs-clickhouse-cover-v3.svg"
---

<style>
.rum-fig{margin:2rem 0}
.rum-fig svg{display:block;width:100%;height:auto}
.rum-cap{color:var(--color-text-muted);font-size:.9rem;text-align:center;margin-top:.5rem}
.rum-txt{fill:var(--color-text);font:13px var(--font-sans)}
.rum-lbl{fill:var(--color-text-muted);font:13px var(--font-sans)}
.rum-acc{fill:#818cf8;font:600 13px var(--font-sans)}
.rum-warn{fill:var(--color-warning);font:600 13px var(--font-sans)}
.rum-mono{fill:var(--color-text);font:12px var(--font-mono)}
.rum-box{fill:var(--color-bg-secondary);stroke:var(--color-border);stroke-width:1.5}
.rum-boxa{fill:var(--color-bg-secondary);stroke:var(--color-accent);stroke-width:1.5}
.rum-boxw{fill:var(--color-bg-secondary);stroke:var(--color-warning);stroke-width:1.5}
.rum-grp{fill:none;stroke:var(--color-border);stroke-width:1.5;stroke-dasharray:6 4}
.rum-line{fill:none;stroke:var(--color-text-muted);stroke-width:1.5}
.rum-arr{fill:none;stroke:var(--color-text-muted);stroke-width:1.5;marker-end:url(#rum-arrow)}
.rum-arra{fill:none;stroke:var(--color-accent);stroke-width:1.5;marker-end:url(#rum-arrow-a)}
.rum-arrw{fill:none;stroke:var(--color-warning);stroke-width:1.5;marker-end:url(#rum-arrow-w)}
.rum-arrhead{fill:var(--color-text-muted)}
.rum-arrhead-a{fill:var(--color-accent)}
.rum-arrhead-w{fill:var(--color-warning)}
.rum-bg{fill:var(--color-bg)}
.rum-mcard{fill:var(--color-bg-secondary);stroke:var(--color-border);stroke-width:1}
.rum-mbar{fill:var(--color-accent)}
.rum-m{fill:var(--color-text);font:20px "Times New Roman",Times,serif}
.rum-mf{fill:#818cf8}
.rum-mv{font-style:italic}
.rum-details{margin:0 0 1.5rem}
.rum-details summary{cursor:pointer;color:var(--color-text-muted)}
.rum-stripe{fill:var(--color-text-muted);opacity:.35}
.rum-tick{fill:var(--color-text-muted)}
.rum-blk{fill:var(--color-bg-secondary);stroke:#4a4a4a;stroke-width:1}
.rum-row{fill:var(--color-accent)}
.rum-cell{fill:var(--color-text-muted);opacity:.25}
.rum-cellw{fill:var(--color-warning);opacity:.45}
.rum-cella{fill:var(--color-accent)}
.rum-cellx{fill:var(--color-warning)}
.rum-grid{stroke:var(--color-border);stroke-width:1}
.rum-ref{stroke:var(--color-text-muted);stroke-width:1.5;stroke-dasharray:5 4}
.rum-bar{stroke:var(--color-border);stroke-width:2}
.rum-dot{fill:var(--color-text-muted)}
.rum-dota{fill:var(--color-accent)}
.rum-val{fill:var(--color-text);font:12px var(--font-sans)}
.rum-tri{fill:none;stroke:var(--color-text-muted);stroke-width:1.5}
.rum-corner{fill:var(--color-text);font:600 15px var(--font-sans)}
.rum-seg{stroke:var(--color-accent);stroke-width:2.5}
.rum-arrwd{fill:none;stroke:var(--color-warning);stroke-width:1.5;stroke-dasharray:5 4;marker-end:url(#rum-arrow-w)}
</style>

<svg width="0" height="0" style="position:absolute" aria-hidden="true">
<defs>
<marker id="rum-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="rum-arrhead" d="M0,0 L10,5 L0,10 z"/></marker>
<marker id="rum-arrow-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="rum-arrhead-a" d="M0,0 L10,5 L0,10 z"/></marker>
<marker id="rum-arrow-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="rum-arrhead-w" d="M0,0 L10,5 L0,10 z"/></marker>
<pattern id="rum-rows" width="6" height="10" patternUnits="userSpaceOnUse"><rect class="rum-stripe" x="0" y="0" width="4" height="10"/></pattern>
<pattern id="rum-blocks" x="140" y="220" width="16" height="40" patternUnits="userSpaceOnUse"><rect class="rum-blk" x="0.5" y="0.5" width="11" height="39" rx="2"/></pattern>
<pattern id="rum-ticks" x="140" y="204" width="16" height="10" patternUnits="userSpaceOnUse"><rect class="rum-tick" x="0" y="0" width="1.5" height="10"/></pattern>
<pattern id="rum-hrows" x="50" y="72" width="300" height="20" patternUnits="userSpaceOnUse"><rect class="rum-cellw" x="0" y="0" width="8" height="14"/><rect class="rum-cellw" x="10" y="0" width="8" height="14"/><rect class="rum-cella" x="20" y="0" width="8" height="14"/><rect class="rum-cellw" x="30" y="0" width="8" height="14"/><rect class="rum-cellw" x="40" y="0" width="8" height="14"/><rect class="rum-cellw" x="50" y="0" width="8" height="14"/><rect class="rum-cellw" x="60" y="0" width="8" height="14"/><rect class="rum-cella" x="70" y="0" width="8" height="14"/><rect class="rum-cellw" x="80" y="0" width="8" height="14"/><rect class="rum-cella" x="90" y="0" width="8" height="14"/><rect class="rum-cellw" x="100" y="0" width="8" height="14"/><rect class="rum-cellw" x="110" y="0" width="8" height="14"/><rect class="rum-cellw" x="120" y="0" width="8" height="14"/><rect class="rum-cellw" x="130" y="0" width="8" height="14"/><rect class="rum-cellw" x="140" y="0" width="8" height="14"/><rect class="rum-cellw" x="150" y="0" width="8" height="14"/><rect class="rum-cellw" x="160" y="0" width="8" height="14"/><rect class="rum-cellw" x="170" y="0" width="8" height="14"/><rect class="rum-cellw" x="180" y="0" width="8" height="14"/><rect class="rum-cellw" x="190" y="0" width="8" height="14"/><rect class="rum-cellw" x="200" y="0" width="8" height="14"/><rect class="rum-cellw" x="210" y="0" width="8" height="14"/><rect class="rum-cellw" x="220" y="0" width="8" height="14"/><rect class="rum-cellw" x="230" y="0" width="8" height="14"/><rect class="rum-cellw" x="240" y="0" width="8" height="14"/><rect class="rum-cellw" x="250" y="0" width="8" height="14"/><rect class="rum-cellw" x="260" y="0" width="8" height="14"/><rect class="rum-cellw" x="270" y="0" width="8" height="14"/><rect class="rum-cellw" x="280" y="0" width="8" height="14"/><rect class="rum-cellw" x="290" y="0" width="8" height="14"/></pattern>
</defs></svg>

## The Argument

The system I work on keeps its sales documents in two databases, and every few weeks the same argument comes back to our standup.

One side: "The data is already in Phoenix. Let's build the monthly report there. Why copy anything?" The report is a `GROUP BY` over a few months of documents. It runs for minutes, loads the region servers, and the dashboard times out first.

The other side: "It's one document with the wrong status. Just fix it in ClickHouse." The fix is an `ALTER TABLE ... UPDATE`. It changes one row and rewrites far more than one row, and a day of such fixes leaves a queue of mutations behind it.

I have argued both sides, sometimes in the same week.

The usual way to settle it is a label. Phoenix is OLTP, ClickHouse is OLAP; one stores rows, the other stores columns; one is an LSM tree, the other is not. The labels are not exactly wrong, but none of them says what a given query will cost, so the argument comes back.

This post compares what both engines read and write, using the same ten million documents in each on a test stand. The differences come down to four knobs. Each one comes with a small formula that predicts which workload it makes expensive.

## Phoenix Is a Layer over HBase

Phoenix is a SQL layer over HBase. Its tables and indexes are HBase tables, and it compiles SQL into HBase operations.

- A table becomes an HBase table. The primary key columns are joined into the row key; every other column becomes a cell in a column family, under a short encoded qualifier instead of the column name.
- A `SELECT` by the full primary key becomes a lookup of one row key. A range or a full `SELECT` becomes scans, at least one per region, which the client runs in parallel.
- A `GROUP BY` runs in two steps: region servers compute partial aggregates in coprocessors, next to the data, and the client merges them.
- A secondary index is another HBase table, and every upsert writes to it as well.

<figure class="rum-fig">
<svg viewBox="0 0 800 340" role="img" aria-label="A GROUP BY query goes to the Phoenix client, which plans one scan per region. Four regions on two region servers each scan their rows and compute partial sums. The partial sums converge back to the Phoenix client, which merges them into the result.">
<rect class="rum-box" x="230" y="16" width="340" height="56" rx="6"/>
<text class="rum-mono" x="400" y="39" text-anchor="middle">SELECT ... GROUP BY payment_type</text>
<text class="rum-lbl" x="400" y="60" text-anchor="middle">Phoenix client: one scan per region</text>
<path class="rum-arr" d="M400,72 L400,80 Q400,92 388,92 L128,92 Q116,92 116,104 L116,136"/>
<path class="rum-arr" d="M400,72 L400,80 Q400,92 388,92 L306,92 Q294,92 294,104 L294,136"/>
<path class="rum-arr" d="M400,72 L400,80 Q400,92 412,92 L494,92 Q506,92 506,104 L506,136"/>
<path class="rum-arr" d="M400,72 L400,80 Q400,92 412,92 L672,92 Q684,92 684,104 L684,136"/>
<rect class="rum-grp" x="20" y="112" width="370" height="112" rx="8"/>
<rect class="rum-grp" x="410" y="112" width="370" height="112" rx="8"/>
<rect class="rum-boxa" x="36" y="140" width="160" height="56" rx="6"/>
<text class="rum-txt" x="116" y="163" text-anchor="middle">region 1</text>
<text class="rum-lbl" x="116" y="182" text-anchor="middle">scan, partial sums</text>
<rect class="rum-boxa" x="214" y="140" width="160" height="56" rx="6"/>
<text class="rum-txt" x="294" y="163" text-anchor="middle">region 2</text>
<text class="rum-lbl" x="294" y="182" text-anchor="middle">scan, partial sums</text>
<rect class="rum-boxa" x="426" y="140" width="160" height="56" rx="6"/>
<text class="rum-txt" x="506" y="163" text-anchor="middle">region 3</text>
<text class="rum-lbl" x="506" y="182" text-anchor="middle">scan, partial sums</text>
<rect class="rum-boxa" x="604" y="140" width="160" height="56" rx="6"/>
<text class="rum-txt" x="684" y="163" text-anchor="middle">region 4</text>
<text class="rum-lbl" x="684" y="182" text-anchor="middle">scan, partial sums</text>
<text class="rum-lbl" x="205" y="216" text-anchor="middle">region server 1</text>
<text class="rum-lbl" x="595" y="216" text-anchor="middle">region server 2</text>
<path class="rum-line" d="M116,196 L116,228 Q116,240 128,240 L400,240"/>
<path class="rum-line" d="M294,196 L294,228 Q294,240 306,240 L400,240"/>
<path class="rum-line" d="M506,196 L506,228 Q506,240 494,240 L400,240"/>
<path class="rum-line" d="M684,196 L684,228 Q684,240 672,240 L400,240"/>
<path class="rum-arr" d="M400,240 L400,266"/>
<rect class="rum-box" x="230" y="270" width="340" height="56" rx="6"/>
<text class="rum-txt" x="400" y="293" text-anchor="middle">Phoenix client: merge</text>
<text class="rum-lbl" x="400" y="312" text-anchor="middle">four sets of partial sums into one result</text>
</svg>
<figcaption class="rum-cap">Phoenix decides where the work runs. What each region reads from disk is up to HBase.</figcaption>
</figure>

Phoenix's main optimization is summing next to the data: only partial sums cross the network, not rows. It can also add work: an index write for every upsert. What it cannot change is what a region server has to read from disk to answer a query. That is up to HBase, block by block.

So the argument is about Phoenix, but the storage comparison is HBase against ClickHouse's MergeTree. Counting Phoenix's overhead against HBase, or HBase's storage costs against Phoenix, would distort every comparison. From here on, the measurements use HBase directly, through its native client (Get, Put, Scan), and Phoenix comes back only where it adds or removes a cost.

## Both Are Log-Structured

The textbook frame says that an OLTP store is a B-tree that updates its pages in place, and an OLAP store is something else entirely. Neither engine here updates anything in place. Their write paths have the same shape:

<figure class="rum-fig">
<svg viewBox="0 0 800 250" role="img" aria-label="Three write paths in four columns. HBase: a Put goes to the WAL and the memstore, is flushed to an HFile and merged later by compaction. ClickHouse: an INSERT block is sorted in memory, written as a part and merged later in the background. Cassandra: a write goes to the commit log and the memtable, is flushed to an SSTable and merged later by compaction.">
<text class="rum-lbl" x="205" y="26" text-anchor="middle">a write</text>
<text class="rum-lbl" x="365" y="26" text-anchor="middle">buffered, sorted</text>
<text class="rum-lbl" x="525" y="26" text-anchor="middle">immutable sorted file</text>
<text class="rum-lbl" x="685" y="26" text-anchor="middle">merged later</text>
<text class="rum-txt" x="20" y="74">HBase</text>
<rect class="rum-box" x="140" y="44" width="130" height="56" rx="6"/>
<text class="rum-txt" x="205" y="67" text-anchor="middle">Put</text>
<text class="rum-lbl" x="205" y="86" text-anchor="middle">one row's cells</text>
<path class="rum-arr" d="M270,72 L296,72"/>
<rect class="rum-box" x="300" y="44" width="130" height="56" rx="6"/>
<text class="rum-txt" x="365" y="67" text-anchor="middle">WAL + memstore</text>
<text class="rum-lbl" x="365" y="86" text-anchor="middle">sorted in memory</text>
<path class="rum-arr" d="M430,72 L456,72"/>
<rect class="rum-box" x="460" y="44" width="130" height="56" rx="6"/>
<text class="rum-txt" x="525" y="67" text-anchor="middle">HFile</text>
<text class="rum-lbl" x="525" y="86" text-anchor="middle">flushed at 128 MB</text>
<path class="rum-arra" d="M590,72 L616,72"/>
<rect class="rum-boxa" x="620" y="44" width="130" height="56" rx="6"/>
<text class="rum-txt" x="685" y="67" text-anchor="middle">compaction</text>
<text class="rum-lbl" x="685" y="86" text-anchor="middle">minor and major</text>
<text class="rum-txt" x="20" y="146">ClickHouse</text>
<rect class="rum-box" x="140" y="116" width="130" height="56" rx="6"/>
<text class="rum-txt" x="205" y="139" text-anchor="middle">INSERT</text>
<text class="rum-lbl" x="205" y="158" text-anchor="middle">a block of rows</text>
<path class="rum-arr" d="M270,144 L296,144"/>
<rect class="rum-box" x="300" y="116" width="130" height="56" rx="6"/>
<text class="rum-txt" x="365" y="139" text-anchor="middle">block</text>
<text class="rum-lbl" x="365" y="158" text-anchor="middle">sorted in memory</text>
<path class="rum-arr" d="M430,144 L456,144"/>
<rect class="rum-box" x="460" y="116" width="130" height="56" rx="6"/>
<text class="rum-txt" x="525" y="139" text-anchor="middle">part</text>
<text class="rum-lbl" x="525" y="158" text-anchor="middle">one per insert</text>
<path class="rum-arra" d="M590,144 L616,144"/>
<rect class="rum-boxa" x="620" y="116" width="130" height="56" rx="6"/>
<text class="rum-txt" x="685" y="139" text-anchor="middle">merge</text>
<text class="rum-lbl" x="685" y="158" text-anchor="middle">in the background</text>
<text class="rum-lbl" x="20" y="218">Cassandra</text>
<rect class="rum-box" x="140" y="188" width="130" height="56" rx="6"/>
<text class="rum-lbl" x="205" y="211" text-anchor="middle">write</text>
<text class="rum-lbl" x="205" y="230" text-anchor="middle">one row's cells</text>
<path class="rum-arr" d="M270,216 L296,216"/>
<rect class="rum-box" x="300" y="188" width="130" height="56" rx="6"/>
<text class="rum-lbl" x="365" y="211" text-anchor="middle">commit log</text>
<text class="rum-lbl" x="365" y="230" text-anchor="middle">+ memtable</text>
<path class="rum-arr" d="M430,216 L456,216"/>
<rect class="rum-box" x="460" y="188" width="130" height="56" rx="6"/>
<text class="rum-lbl" x="525" y="211" text-anchor="middle">SSTable</text>
<text class="rum-lbl" x="525" y="230" text-anchor="middle">flushed memtable</text>
<path class="rum-arr" d="M590,216 L616,216"/>
<rect class="rum-box" x="620" y="188" width="130" height="56" rx="6"/>
<text class="rum-lbl" x="685" y="211" text-anchor="middle">compaction</text>
<text class="rum-lbl" x="685" y="230" text-anchor="middle">a strategy per table</text>
</svg>
<figcaption class="rum-cap">All three write paths buffer writes, write an immutable sorted file, and merge the files later.</figcaption>
</figure>

An HBase Put goes to the write-ahead log and into the memstore, a sorted buffer in memory; at 128 MB the memstore is flushed into an HFile, and compactions later merge HFiles into bigger ones. A ClickHouse INSERT block is sorted in memory and written as a new part, a directory of column files; background merges later combine parts into bigger ones.

Both are log-structured merge designs, so "LSM vs B-tree" does not describe this comparison.

They also fail the same way. The first load on the stand wrote ten million rows into HBase in key order, so at any moment every write went to the one region whose range was being filled. Flushes produced files faster than compaction merged them. At 17 files in the region, one over `hbase.hstore.blockingStoreFiles`, HBase began delaying each flush by up to 90 seconds. The memstore kept growing to its blocking size, flush size times the block multiplier, 128 MB x 4 = 512 MB, and then the region refused writes:

```text
RegionTooBusyException: Over memstore limit=512.0 M
```

It kept refusing them until a compaction caught up. ClickHouse has the same failure under another name. When inserts create parts faster than merges combine them, it first slows inserts down (`parts_to_delay_insert`) and then rejects them with "Too many parts" (`parts_to_throw_insert`).

Cassandra, the third engine on the stand, belongs to the same family: commit log and memtable, flushed into SSTables, merged by compaction. It is the write path of Google's Bigtable, which HBase copied as well. Its storage choices sit close to HBase's, so it gets no separate pass through the first three knobs. It differs on the fourth, who owns a key, and returns in that section.

## Four Knobs

Both engines append and merge. What makes one good at the monthly report and the other at fixing one document is four choices that every log-structured engine has to make somewhere:

| knob | HBase | ClickHouse | Cassandra |
|---|---|---|---|
| 1. rows behind one index entry | a block: 64&nbsp;KB of whole rows | a granule: up to 8,192 rows | two levels: every partition, then 64&nbsp;KB inside one |
| 2. what one unit of read holds | whole rows, every column of the family | one column, compressed | like HBase: whole rows |
| 3. what a change rewrites | nothing now: a new cell version, merged later | the changed columns of every part with a matching row | like HBase |
| 4. who owns a key on many nodes | one server per key range, storage shared | the shard the sharding key picks, own disks | a hash ring, any node coordinates |

Every setting makes one cost lower and another higher. These costs have a name, the RUM triangle: the cost of **R**eading, of **U**pdating, and of the **M**emory or space the data takes. The next four sections measure what each knob costs, and the triangle at the end puts them together.

## Knob 1: How Many Rows Stand Behind an Index Entry

Neither engine indexes every row. Both keep a sparse index: the rows are sorted by key and cut into chunks, and the index remembers where each chunk starts. To find one row, the engine finds its chunk and processes the entire chunk.

The size of the chunk is the first knob. Call it *g*, the rows behind one index entry:

<figure class="rum-fig">
<svg viewBox="0 0 800 200" role="img" aria-label="E equals N over g: index entries for N rows with g rows behind each. Rows processed by a point lookup are about g, depending on the plan. g equals N over E: for a fixed N, every entry cut from the index is rows added to every point read.">
<rect class="rum-mcard" x="1" y="1" width="798" height="198" rx="8"/><rect class="rum-mbar" x="1" y="1" width="4" height="198" rx="2"/>
<text class="rum-m" x="36" y="42"><tspan class="rum-mv">E</tspan> = <tspan class="rum-mv">N</tspan> / <tspan class="rum-mv">g</tspan></text>
<text class="rum-lbl" x="36" y="64">index entries for N rows, with g rows behind each entry</text>
<text class="rum-m" x="36" y="104"><tspan class="rum-mf">rows processed</tspan>(point lookup) ≈ <tspan class="rum-mv">g</tspan></text>
<text class="rum-lbl" x="36" y="126">about, for a lookup by the full key; the plan decides (a key on a granule boundary takes two); rows, not bytes</text>
<text class="rum-m" x="36" y="166"><tspan class="rum-mv">g</tspan> = <tspan class="rum-mv">N</tspan> / <tspan class="rum-mv">E</tspan></text>
<text class="rum-lbl" x="36" y="188">for a fixed N, every entry cut from the index is rows added to every point read</text>
</svg>
</figure>

ClickHouse sets *g* = 8,192 rows and calls the chunk a granule. The test table has 10 million rows and a primary index of 1,224 marks: 4.9 KB, always in memory (`marks` and `primary_key_bytes_in_memory` in `system.parts`). At the same *g*, a billion rows would need about 122 thousand marks, which is still negligible. The cost is on the other side of the formula: to return one row, ClickHouse processes its whole granule, 8,192 rows of every column the query asks for.

HBase cuts by bytes, not by rows. A block closes at 64 KB of uncompressed cells, which for these 1.2 KB rows is about 58 rows. Its *g* is 140 times smaller, so its index has 140 times as many entries: 172,472 for the same table, one per data block (the blocks a full scan counts in HBase's server-side scan metrics). HBase does not keep such an index in memory as one array: it stores it in blocks too, read through the block cache, alongside a bloom filter for each file that answers the question: "Is this key in this file at all?"

<figure class="rum-fig">
<svg viewBox="0 0 800 296" role="img" aria-label="Schematic. ClickHouse: six large granules, an index mark at the start of each; to return one row, the whole granule is processed, 8,192 rows of every column asked for. HBase: forty small blocks, an index entry at the start of each; to return one row, one 64 KB block of about 58 rows is read, plus a bloom filter chunk and an index block.">
<text class="rum-txt" x="20" y="44">ClickHouse</text>
<text class="rum-lbl" x="20" y="62">a mark per 8,192 rows</text>
<text class="rum-lbl" x="776" y="54" text-anchor="end">index: 1,224 marks, 4.9 KB, in memory</text>
<rect class="rum-tick" x="140" y="64" width="1.5" height="10"/><rect class="rum-tick" x="246.7" y="64" width="1.5" height="10"/><rect class="rum-tick" x="353.3" y="64" width="1.5" height="10"/><rect class="rum-tick" x="460" y="64" width="1.5" height="10"/><rect class="rum-tick" x="566.7" y="64" width="1.5" height="10"/><rect class="rum-tick" x="673.4" y="64" width="1.5" height="10"/>
<rect class="rum-box" x="140" y="80" width="102.7" height="40" rx="4"/><rect fill="url(#rum-rows)" x="146" y="86" width="90.7" height="28"/>
<rect class="rum-box" x="246.7" y="80" width="102.7" height="40" rx="4"/><rect fill="url(#rum-rows)" x="252.7" y="86" width="90.7" height="28"/>
<rect class="rum-boxw" x="353.3" y="80" width="102.7" height="40" rx="4"/><rect fill="url(#rum-rows)" x="359.3" y="86" width="90.7" height="28"/>
<rect class="rum-box" x="460" y="80" width="102.7" height="40" rx="4"/><rect fill="url(#rum-rows)" x="466" y="86" width="90.7" height="28"/>
<rect class="rum-box" x="566.7" y="80" width="102.7" height="40" rx="4"/><rect fill="url(#rum-rows)" x="572.7" y="86" width="90.7" height="28"/>
<rect class="rum-box" x="673.4" y="80" width="102.7" height="40" rx="4"/><rect fill="url(#rum-rows)" x="679.4" y="86" width="90.7" height="28"/>
<rect class="rum-row" x="401" y="84" width="4" height="32"/>
<text class="rum-acc" x="404.6" y="144" text-anchor="middle">one row wanted, <tspan class="rum-warn">the whole granule processed: 8,192 rows of each column asked for</tspan></text>
<text class="rum-txt" x="20" y="184">HBase</text>
<text class="rum-lbl" x="20" y="202">an entry per 64 KB block</text>
<text class="rum-lbl" x="776" y="194" text-anchor="end">index: 172,472 entries, stored in blocks</text>
<rect fill="url(#rum-ticks)" x="140" y="204" width="640" height="10"/>
<rect fill="url(#rum-blocks)" x="140" y="220" width="640" height="40"/>
<rect class="rum-boxw" x="412.5" y="220.5" width="11" height="39" rx="2"/>
<rect class="rum-row" x="416.5" y="224" width="3" height="32"/>
<text class="rum-acc" x="412" y="284" text-anchor="middle">one row wanted, <tspan class="rum-warn">one 64 KB block accessed (~58 rows), plus a bloom chunk and an index block</tspan></text>
</svg>
<figcaption class="rum-cap">Schematic. On the stand ClickHouse has 1,224 granules and HBase 172,472 blocks for the same 10 million rows.</figcaption>
</figure>

One point read of a whole row, caches warm, median of 2,000 random keys:

| | *g* | index | to return one row | median |
|---|---|---|---|---|
| ClickHouse | 8,192&nbsp;rows | 1,224 marks, 4.9&nbsp;KB | one granule processed, 8,192 rows | 3.47&nbsp;ms |
| HBase | ~58&nbsp;rows | 172,472 entries | one 64&nbsp;KB data block accessed, plus 256&nbsp;KB of bloom and index blocks, all in the block cache | 0.15&nbsp;ms |

"One granule" counts rows, not bytes, and the bytes actually read differ. A separate run with ClickHouse's own counters, per point read:

| ClickHouse, one row | processed | decompressed | compressed bytes read (page cache) |
|---|---|---|---|
| whole row, 30 columns | 1.70&nbsp;MB | 2.82&nbsp;MB in 38 blocks | 525&nbsp;KB |
| 3 columns | 180&nbsp;KB | 328&nbsp;KB in 5 blocks | 90&nbsp;KB |

More is decompressed than processed because a compressed block holds at least 64 KB of uncompressed data. A one-byte column such as `status` takes 8 KB per granule, so one of its blocks covers eight granules, and reading one granule means decompressing all eight. The three-column read is five blocks of exactly 64 KB: the three columns and the two key columns the filter needs. (The volumes come from that separate run, the latency from the timed one.)

A `SELECT 1` through the same HTTP client takes 0.6 ms: the baseline of any query on this stand, parsing, planning and the round trip included. Above that baseline, three columns cost 0.8 ms and all thirty columns cost 2.9 ms. That is the part that grows with the columns asked for; decoding the granule is in it, and so is opening one column file per column. The timing alone does not split the two.

HBase pays for its small *g* in a way the formula does not show. Of the ~320 KB a Get touches, 64 KB is the data block; the rest is one 128 KB chunk of the bloom filter and one leaf block of the index, up to 128 KB. All of it comes from the block cache here, so it costs microseconds, but it is three blocks per Get, not one. The card in the next section counts data blocks only, and for one file per region it says one; the index and the filter are the reads it leaves out.

For a single row ClickHouse is 24 times slower, and not because it is badly built: it chose a *g* that keeps its index tiny, and a point read pays for that choice.

### Cassandra Lets the Schema Choose *g*

With the stock configuration on this stand, Cassandra's index has two levels. The partition index has an entry for every partition; a sample of it, one entry in 128, sits in memory, next to a bloom filter on the partition key. Inside a partition the rows are sorted by the clustering key, and a partition bigger than 64 KB gets an index of its own, an entry per 64 KB.

So in Cassandra *g* is a schema decision, and the stand can vary it. We loaded the same ten million rows twice: once with a partition per register, a thousand documents ordered by number inside it, and once with a partition per document. Totals across all three nodes:

| partition per | partitions | index on disk | bloom filter | space | point read |
|---|---|---|---|---|---|
| register | 10,000 | 1.45&nbsp;MB | 12.6&nbsp;KB | 684&nbsp;MiB | 223-231&nbsp;µs |
| document | 10,045,874 | 257&nbsp;MB | 12.6&nbsp;MB | 1,564&nbsp;MiB | 206-216&nbsp;µs |

The card holds for a third engine too. A thousand times as many entries require a bloom filter a thousand times larger (ten bits per partition in both tables) and 177 times the index on disk. The point read got 15 microseconds faster, and the table more than doubled. Of the extra 880 MiB, 245 MiB is index; the rest is most likely per-partition overhead and the cost of the row order. With a partition per document the rows lie in hash order, so the rows of one register, whose constant columns compress well side by side, are scattered across the file. That part is an interpretation, not a measurement.

What the small partitions did buy is a bloom filter that works:

<figure class="rum-fig">
<svg viewBox="0 0 800 250" role="img" aria-label="Four SSTables on one node. With a partition per register, every SSTable holds a slice of register 42; the bloom filter is keyed by the partition, so all four are opened, and only one holds document 4242. With a partition per document, only one SSTable holds document 4242; the bloom filter skips the other three and one file is opened.">
<text class="rum-txt" x="20" y="62">partition per</text>
<text class="rum-txt" x="20" y="80">register</text>
<rect class="rum-boxw" x="196" y="40" width="130" height="56" rx="6"/>
<text class="rum-txt" x="261" y="63" text-anchor="middle">SSTable 1</text>
<text class="rum-warn" x="261" y="82" text-anchor="middle">a slice of 42</text>
<rect class="rum-boxw" x="342" y="40" width="130" height="56" rx="6"/>
<text class="rum-txt" x="407" y="63" text-anchor="middle">SSTable 2</text>
<text class="rum-warn" x="407" y="82" text-anchor="middle">a slice of 42</text>
<rect class="rum-boxa" x="488" y="40" width="130" height="56" rx="6"/>
<text class="rum-txt" x="553" y="63" text-anchor="middle">SSTable 3</text>
<text class="rum-acc" x="553" y="82" text-anchor="middle">doc 4242</text>
<rect class="rum-boxw" x="634" y="40" width="130" height="56" rx="6"/>
<text class="rum-txt" x="699" y="63" text-anchor="middle">SSTable 4</text>
<text class="rum-warn" x="699" y="82" text-anchor="middle">a slice of 42</text>
<text class="rum-warn" x="480" y="122" text-anchor="middle">the filter is keyed by register: every file has it, every file is opened</text>
<text class="rum-txt" x="20" y="172">partition per</text>
<text class="rum-txt" x="20" y="190">document</text>
<rect class="rum-box" x="196" y="150" width="130" height="56" rx="6"/>
<text class="rum-lbl" x="261" y="173" text-anchor="middle">SSTable 1</text>
<text class="rum-lbl" x="261" y="192" text-anchor="middle">bloom filter: skip</text>
<rect class="rum-box" x="342" y="150" width="130" height="56" rx="6"/>
<text class="rum-lbl" x="407" y="173" text-anchor="middle">SSTable 2</text>
<text class="rum-lbl" x="407" y="192" text-anchor="middle">bloom filter: skip</text>
<rect class="rum-boxa" x="488" y="150" width="130" height="56" rx="6"/>
<text class="rum-txt" x="553" y="173" text-anchor="middle">SSTable 3</text>
<text class="rum-acc" x="553" y="192" text-anchor="middle">doc 4242</text>
<rect class="rum-box" x="634" y="150" width="130" height="56" rx="6"/>
<text class="rum-lbl" x="699" y="173" text-anchor="middle">SSTable 4</text>
<text class="rum-lbl" x="699" y="192" text-anchor="middle">bloom filter: skip</text>
<text class="rum-acc" x="480" y="232" text-anchor="middle">the filter is keyed by document: one file has it, one file is opened</text>
</svg>
<figcaption class="rum-cap">Reading document 4242 of register 42. Measured: a partition per register, 3.3 files opened per read (2-4 SSTables per node); a partition per document, exactly 1 (5-6 per node).</figcaption>
</figure>

With a partition per register, every file holds a slice of every register, and the filter lets every file through. With a partition per document, each document lives in exactly one file, and the read traces show the filter skipping 4.3 files per read. "Exactly one" holds for this table, loaded once and never updated: an update writes a newer piece of the partition into a newer file, and from then on the filter lets two through. That is the *p* (*h* - 1) term of the next section's card: a filter only helps when its key is as fine-grained as the read.

## Knob 2: What One Read Brings Back

The second knob is what sits inside the chunk. An HBase block holds sorted cells: all the cells of about 58 rows, every column of the column family, one row after another. A ClickHouse granule is not one chunk but one per column, each compressed on its own. A query that needs three columns opens three files and never touches the other twenty-seven.

<figure class="rum-fig">
<svg viewBox="0 0 800 290" role="img" aria-label="Left, HBase: a block holds the sorted cells of about 58 rows, 30 cells each; the three columns a query needs are in accent, the other 27 in amber; narrow or wide, the scan goes through blocks that contain all columns, 11.3 GB on the uncompressed table. Right, ClickHouse: separate column streams in wide parts; only the three accent columns are read, 100 MB.">
<text class="rum-txt" x="200" y="28" text-anchor="middle">HBase: a block holds sorted cells</text>
<text class="rum-lbl" x="200" y="47" text-anchor="middle">one 64 KB block, ~58 rows' worth of cells</text>
<rect class="rum-grp" x="38" y="60" width="324" height="178" rx="8"/>
<rect fill="url(#rum-hrows)" x="50" y="72" width="300" height="154"/>
<text class="rum-warn" x="200" y="260" text-anchor="middle">narrow or wide, blocks contain all columns</text>
<text class="rum-lbl" x="200" y="280" text-anchor="middle">11.3 GB scanned, uncompressed table</text>
<text class="rum-txt" x="594" y="28" text-anchor="middle">ClickHouse: separate column streams (wide parts)</text>
<text class="rum-lbl" x="594" y="47" text-anchor="middle">each compressed on its own</text>
<rect class="rum-cell" x="430" y="72" width="8" height="154"/><rect class="rum-cell" x="441" y="72" width="8" height="154"/><rect class="rum-cella" x="452" y="72" width="8" height="154"/><rect class="rum-cell" x="463" y="72" width="8" height="154"/><rect class="rum-cell" x="474" y="72" width="8" height="154"/><rect class="rum-cell" x="485" y="72" width="8" height="154"/><rect class="rum-cell" x="496" y="72" width="8" height="154"/><rect class="rum-cella" x="507" y="72" width="8" height="154"/><rect class="rum-cell" x="518" y="72" width="8" height="154"/><rect class="rum-cella" x="529" y="72" width="8" height="154"/>
<rect class="rum-cell" x="540" y="72" width="8" height="154"/><rect class="rum-cell" x="551" y="72" width="8" height="154"/><rect class="rum-cell" x="562" y="72" width="8" height="154"/><rect class="rum-cell" x="573" y="72" width="8" height="154"/><rect class="rum-cell" x="584" y="72" width="8" height="154"/><rect class="rum-cell" x="595" y="72" width="8" height="154"/><rect class="rum-cell" x="606" y="72" width="8" height="154"/><rect class="rum-cell" x="617" y="72" width="8" height="154"/><rect class="rum-cell" x="628" y="72" width="8" height="154"/><rect class="rum-cell" x="639" y="72" width="8" height="154"/>
<rect class="rum-cell" x="650" y="72" width="8" height="154"/><rect class="rum-cell" x="661" y="72" width="8" height="154"/><rect class="rum-cell" x="672" y="72" width="8" height="154"/><rect class="rum-cell" x="683" y="72" width="8" height="154"/><rect class="rum-cell" x="694" y="72" width="8" height="154"/><rect class="rum-cell" x="705" y="72" width="8" height="154"/><rect class="rum-cell" x="716" y="72" width="8" height="154"/><rect class="rum-cell" x="727" y="72" width="8" height="154"/><rect class="rum-cell" x="738" y="72" width="8" height="154"/><rect class="rum-cell" x="749" y="72" width="8" height="154"/>
<text class="rum-acc" x="594" y="260" text-anchor="middle">3 column streams of 30 are read</text>
<text class="rum-lbl" x="594" y="280" text-anchor="middle">100 MB: status, payment_type, amount</text>
</svg>
<figcaption class="rum-cap">The query is sum(amount) WHERE status = 1 GROUP BY payment_type: three columns of thirty. All thirty HBase columns are in one column family.</figcaption>
</figure>

For a scan the layout alone decides how many bytes are read, and it fits a formula. For a point read, the same layout works the other way round:

<figure class="rum-fig">
<svg viewBox="0 0 800 282" role="img" aria-label="B row equals N times the sum over all columns C of the family of w of c plus k. B col equals N times the sum over the query's columns Q of w of c. Both are uncompressed bytes. A point read in HBase reads, in a simple model, 1 plus p times h minus 1 data blocks. A point read in ClickHouse reads at least Q blocks, one per column read.">
<rect class="rum-mcard" x="1" y="1" width="798" height="280" rx="8"/><rect class="rum-mbar" x="1" y="1" width="4" height="280" rx="2"/>
<text class="rum-m" x="36" y="44"><tspan class="rum-mv">B</tspan><tspan font-size="13" baseline-shift="sub">row</tspan> = <tspan class="rum-mv">N</tspan> <tspan font-size="26">Σ</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="rum-mv">c</tspan> ∈ <tspan class="rum-mv">C</tspan></tspan> ( <tspan class="rum-mf">w</tspan>(<tspan class="rum-mv">c</tspan>) + <tspan class="rum-mv">k</tspan> )</text>
<text class="rum-lbl" x="36" y="76">HBase scan: every column C of the family; w is a value's width, k the key stored with every cell</text>
<text class="rum-m" x="36" y="116"><tspan class="rum-mv">B</tspan><tspan font-size="13" baseline-shift="sub">col</tspan> = <tspan class="rum-mv">N</tspan> <tspan font-size="26">Σ</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="rum-mv">c</tspan> ∈ <tspan class="rum-mv">Q</tspan></tspan> <tspan class="rum-mf">w</tspan>(<tspan class="rum-mv">c</tspan>)</text>
<text class="rum-lbl" x="36" y="148">ClickHouse scan: only the query's columns Q. Both uncompressed; on disk HBase compresses a block, ClickHouse a column</text>
<text class="rum-m" x="36" y="190"><tspan class="rum-mf">data blocks</tspan><tspan font-size="13" baseline-shift="sub">HBase</tspan>(one row) = 1 + <tspan class="rum-mv">p</tspan> (<tspan class="rum-mv">h</tspan> - 1)</text>
<text class="rum-lbl" x="36" y="212">a simple model: the key in one of h files, the others let through by false positives at rate p</text>
<text class="rum-m" x="36" y="250"><tspan class="rum-mf">data blocks</tspan><tspan font-size="13" baseline-shift="sub">ClickHouse</tspan>(one row) ≥ | <tspan class="rum-mv">Q</tspan> |</text>
<text class="rum-lbl" x="36" y="272">one compressed block per column read, the filter's key columns included; a column can need more</text>
</svg>
</figure>

ClickHouse read 100 MB for the narrow scan: status 10 MB, payment_type 10 MB, amount 80 MB, uncompressed by its own counter. That matches *B*<sub>col</sub> exactly. Compressed, it read 52.6 MB, all from the page cache on this stand: the three column files and nothing else. The wide scan read 315 MB compressed, the whole table. HBase read 172,472 blocks for the narrow scan and the same 172,472 for the wide one, 11.3 GB from the file system either way on the uncompressed table. Asking HBase for 3 columns did not change what it read, only what crossed the network.

The *k* term is large. In the uncompressed HBase table about 80% of the bytes are keys: every cell carries its row key, family, qualifier and timestamp, about 36 bytes (inferred from the table's size, not read out of an HFile), next to 8 bytes of value on average. Prefix encoding and LZ4 bring the table from 11,180 MiB down to 792 MiB. ClickHouse keeps the same rows in 299 MiB.

For point reads the result is reversed. Both tables compressed, medians:

<figure class="rum-fig">
<svg viewBox="0 0 800 268" role="img" aria-label="Median time, log scale. Scan of 3 of 30 columns: HBase 24.7 s, ClickHouse 0.03 s. Scan of all 30 columns: HBase 26.9 s, ClickHouse 0.35 s. Point read of a whole row: HBase 170 microseconds, ClickHouse 3.47 ms. Point read of 3 columns: HBase 197 microseconds, ClickHouse 1.44 ms. The ClickHouse floor for any query over HTTP is 0.6 ms.">
<circle class="rum-dot" cx="236" cy="22" r="6"/><text class="rum-lbl" x="248" y="26">HBase</text>
<rect class="rum-dot" x="304" y="16" width="12" height="12"/><text class="rum-lbl" x="322" y="26">ClickHouse</text>
<text class="rum-acc" x="412" y="26">accent: the faster of the two</text>
<line class="rum-ref" x1="606" y1="22" x2="628" y2="22"/><text class="rum-lbl" x="636" y="26">ClickHouse floor</text>
<line class="rum-grid" x1="220" y1="48" x2="220" y2="206"/><line class="rum-grid" x1="310" y1="48" x2="310" y2="206"/><line class="rum-grid" x1="400" y1="48" x2="400" y2="206"/><line class="rum-grid" x1="490" y1="48" x2="490" y2="206"/><line class="rum-grid" x1="580" y1="48" x2="580" y2="206"/><line class="rum-grid" x1="670" y1="48" x2="670" y2="206"/><line class="rum-grid" x1="760" y1="48" x2="760" y2="206"/>
<line class="rum-ref" x1="290.4" y1="132" x2="290.4" y2="206"/>
<text class="rum-txt" x="20" y="74">scan, 3 of 30 columns</text>
<text class="rum-txt" x="20" y="114">scan, all 30 columns</text>
<text class="rum-txt" x="20" y="154">point read, whole row</text>
<text class="rum-txt" x="20" y="194">point read, 3 columns</text>
<line class="rum-bar" x1="442.9" y1="70" x2="705.4" y2="70"/>
<circle class="rum-dot" cx="705.4" cy="70" r="6"><title>scan, 3 of 30 columns: HBase 24.7 s</title></circle><text class="rum-val" x="705.4" y="58" text-anchor="middle">24.7 s</text>
<rect class="rum-dota" x="436.9" y="64" width="12" height="12"><title>scan, 3 of 30 columns: ClickHouse 0.03 s</title></rect><text class="rum-val" x="442.9" y="58" text-anchor="middle">0.03 s</text>
<line class="rum-bar" x1="538.9" y1="110" x2="708.7" y2="110"/>
<circle class="rum-dot" cx="708.7" cy="110" r="6"><title>scan, all 30 columns: HBase 26.9 s</title></circle><text class="rum-val" x="708.7" y="98" text-anchor="middle">26.9 s</text>
<rect class="rum-dota" x="532.9" y="104" width="12" height="12"><title>scan, all 30 columns: ClickHouse 0.35 s</title></rect><text class="rum-val" x="538.9" y="98" text-anchor="middle">0.35 s</text>
<line class="rum-bar" x1="240.7" y1="150" x2="358.6" y2="150"/>
<circle class="rum-dota" cx="240.7" cy="150" r="6"><title>point read, whole row: HBase 170 µs</title></circle><text class="rum-val" x="240.7" y="138" text-anchor="middle">170 µs</text>
<rect class="rum-dot" x="352.6" y="144" width="12" height="12"><title>point read, whole row: ClickHouse 3.47 ms</title></rect><text class="rum-val" x="358.6" y="138" text-anchor="middle">3.47 ms</text>
<line class="rum-bar" x1="246.5" y1="190" x2="324.3" y2="190"/>
<circle class="rum-dota" cx="246.5" cy="190" r="6"><title>point read, 3 columns: HBase 197 µs</title></circle><text class="rum-val" x="246.5" y="178" text-anchor="middle">197 µs</text>
<rect class="rum-dot" x="318.3" y="184" width="12" height="12"><title>point read, 3 columns: ClickHouse 1.44 ms</title></rect><text class="rum-val" x="330.3" y="178" text-anchor="start">1.44 ms</text>
<text class="rum-lbl" x="220" y="226" text-anchor="middle">100 µs</text><text class="rum-lbl" x="310" y="226" text-anchor="middle">1 ms</text><text class="rum-lbl" x="400" y="226" text-anchor="middle">10 ms</text><text class="rum-lbl" x="490" y="226" text-anchor="middle">100 ms</text><text class="rum-lbl" x="580" y="226" text-anchor="middle">1 s</text><text class="rum-lbl" x="670" y="226" text-anchor="middle">10 s</text><text class="rum-lbl" x="760" y="226" text-anchor="middle">100 s</text>
<text class="rum-lbl" x="490" y="256" text-anchor="middle">median time, log scale</text>
</svg>
<figcaption class="rum-cap">The layout that is 77 to 820 times faster on a scan is 7 to 20 times slower on a point read.</figcaption>
</figure>

<details class="rum-details">
<summary>The same numbers as a table</summary>

| | HBase (FAST_DIFF + LZ4) | ClickHouse (LZ4 + delta codecs) |
|---|---|---|
| scan, 3 of 30 columns | 24.7&nbsp;s | 0.03&nbsp;s |
| scan, all 30 columns | 26.9&nbsp;s | 0.35&nbsp;s |
| point read, whole row | 170&nbsp;µs | 3.47&nbsp;ms |
| point read, 3 columns | 197&nbsp;µs | 1.44&nbsp;ms |
| on disk | 792&nbsp;MiB | 299&nbsp;MiB |

</details>

The point reads show one term of the card per engine. ClickHouse's whole row costs 2.4 times its three columns: one block per requested column, the |*Q*| term. HBase's whole row costs about the same as three columns: the block holds the row either way, and fewer columns only save serialization.

Two caveats, both against HBase. Its LZ4 runs in Java on this stand (the native Hadoop libraries are not loaded), and the aggregation runs in the bench client. Neither explains the gap: the wide scan moves nine times more cells to the client than the narrow one and takes only 1.1 times longer. The time goes to reading blocks. Phoenix's coprocessors would save the transfer, not the blocks.

HBase has its own way to separate columns: column families are stored apart, so putting the few hot columns into a family of their own is manual column storage. It works for the columns you knew about in advance.

## Knob 3: What a Change Rewrites

Now the fix from the introduction: one document with the wrong status. Both engines keep their files immutable, so neither of them can change a value in place. What differs is what each of them writes instead, and when.

<figure class="rum-fig">
<svg viewBox="0 0 800 250" role="img" aria-label="An HBase Put writes the size of the new cells; compactions merge files later. A ClickHouse mutation reads the WHERE columns W of every part P it has to check and the columns being set S of every part T with a matching row, and writes the columns S of every part in T whole.">
<rect class="rum-mcard" x="1" y="1" width="798" height="248" rx="8"/><rect class="rum-mbar" x="1" y="1" width="4" height="248" rx="2"/>
<text class="rum-m" x="36" y="44"><tspan class="rum-mf">written</tspan>(Put) = <tspan class="rum-mf">size</tspan>(new cells)</text>
<text class="rum-lbl" x="36" y="66">HBase: a WAL append and the memstore now; compactions merge files later, a major one every file of the store</text>
<text class="rum-m" x="36" y="118"><tspan class="rum-mf">read</tspan>(mutation) = <tspan font-size="26">Σ</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="rum-mv">p</tspan> ∈ <tspan class="rum-mv">P</tspan></tspan> <tspan font-size="26">Σ</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="rum-mv">c</tspan> ∈ <tspan class="rum-mv">W</tspan></tspan> <tspan class="rum-mf">size</tspan>(<tspan class="rum-mv">p</tspan>, <tspan class="rum-mv">c</tspan>) + <tspan font-size="26">Σ</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="rum-mv">p</tspan> ∈ <tspan class="rum-mv">T</tspan></tspan> <tspan font-size="26">Σ</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="rum-mv">c</tspan> ∈ <tspan class="rum-mv">S</tspan></tspan> <tspan class="rum-mf">size</tspan>(<tspan class="rum-mv">p</tspan>, <tspan class="rum-mv">c</tspan>)</text>
<text class="rum-lbl" x="36" y="148">ClickHouse: W, the WHERE columns of the parts P it must check; S, the set columns of the parts T with a match</text>
<text class="rum-m" x="36" y="198"><tspan class="rum-mf">written</tspan>(mutation) = <tspan font-size="26">Σ</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="rum-mv">p</tspan> ∈ <tspan class="rum-mv">T</tspan></tspan> <tspan font-size="26">Σ</tspan><tspan font-size="13" baseline-shift="sub"><tspan class="rum-mv">c</tspan> ∈ <tspan class="rum-mv">S</tspan></tspan> <tspan class="rum-mf">size</tspan>(<tspan class="rum-mv">p</tspan>, <tspan class="rum-mv">c</tspan>)</text>
<text class="rum-lbl" x="36" y="228">every set column of every touched part, rewritten whole; read counts uncompressed bytes, written ones land compressed</text>
</svg>
</figure>

An HBase Put writes one new cell with a newer timestamp and leaves the old one where it is. A read merges the memstore with the files and the newest timestamp wins. The old cell stays on disk until a compaction merges its file with a newer one. A minor compaction merges a selection of files; a major compaction rewrites every file of a column family's store, whatever changed. The test table has one column family, so a major compaction there rewrites every cell of the region.

A ClickHouse `ALTER TABLE ... UPDATE` is a mutation: it finds the rows, then writes a new version of every part that has one.

<figure class="rum-fig">
<svg viewBox="0 0 800 352" role="img" aria-label="HBase: the memstore holds the new cell status 2 at timestamp 2, the HFile still holds status 1 at timestamp 1; a Get merges both and the newest timestamp wins. The old cell stays until a compaction merges its file. ClickHouse: ALTER UPDATE reads the two WHERE columns and the status column of the part and writes a new part in which the status column is rewritten whole, 1.39 MB to change 1 byte, while the other 29 columns of this wide part are hard-linked.">
<text class="rum-txt" x="20" y="98">HBase</text>
<rect class="rum-boxa" x="140" y="30" width="180" height="56" rx="6"/>
<text class="rum-txt" x="230" y="53" text-anchor="middle">memstore</text>
<text class="rum-acc" x="230" y="72" text-anchor="middle">status = 2, ts 2: new</text>
<rect class="rum-box" x="140" y="100" width="180" height="56" rx="6"/>
<text class="rum-txt" x="230" y="123" text-anchor="middle">HFile on disk</text>
<text class="rum-lbl" x="230" y="142" text-anchor="middle">status = 1, ts 1: still there</text>
<path class="rum-line" d="M320,58 L338,58 Q352,58 352,72 L352,79 Q352,93 366,93"/>
<path class="rum-line" d="M320,128 L338,128 Q352,128 352,114 L352,107 Q352,93 366,93"/>
<path class="rum-arr" d="M366,93 L416,93"/>
<rect class="rum-box" x="420" y="65" width="200" height="56" rx="6"/>
<text class="rum-txt" x="520" y="88" text-anchor="middle">Get</text>
<text class="rum-lbl" x="520" y="107" text-anchor="middle">newest timestamp wins: 2</text>
<text class="rum-warn" x="460" y="186" text-anchor="middle">the old cell stays until a compaction merges its file</text>
<text class="rum-txt" x="20" y="266">ClickHouse</text>
<text class="rum-lbl" x="260" y="216" text-anchor="middle">part, 30 column files</text>
<text class="rum-lbl" x="620" y="216" text-anchor="middle">new part, same rows</text>
<rect class="rum-grp" x="138" y="222" width="244" height="80" rx="8"/>
<rect class="rum-cellw" x="146.0" y="230" width="5" height="64"/><rect class="rum-cellw" x="153.6" y="230" width="5" height="64"/><rect class="rum-cell" x="161.2" y="230" width="5" height="64"/><rect class="rum-cell" x="168.8" y="230" width="5" height="64"/><rect class="rum-cellw" x="176.4" y="230" width="5" height="64"/><rect class="rum-cell" x="184.0" y="230" width="5" height="64"/><rect class="rum-cell" x="191.6" y="230" width="5" height="64"/><rect class="rum-cell" x="199.2" y="230" width="5" height="64"/><rect class="rum-cell" x="206.8" y="230" width="5" height="64"/><rect class="rum-cell" x="214.4" y="230" width="5" height="64"/><rect class="rum-cell" x="222.0" y="230" width="5" height="64"/><rect class="rum-cell" x="229.6" y="230" width="5" height="64"/><rect class="rum-cell" x="237.2" y="230" width="5" height="64"/><rect class="rum-cell" x="244.8" y="230" width="5" height="64"/><rect class="rum-cell" x="252.4" y="230" width="5" height="64"/>
<rect class="rum-cell" x="260.0" y="230" width="5" height="64"/><rect class="rum-cell" x="267.6" y="230" width="5" height="64"/><rect class="rum-cell" x="275.2" y="230" width="5" height="64"/><rect class="rum-cell" x="282.8" y="230" width="5" height="64"/><rect class="rum-cell" x="290.4" y="230" width="5" height="64"/><rect class="rum-cell" x="298.0" y="230" width="5" height="64"/><rect class="rum-cell" x="305.6" y="230" width="5" height="64"/><rect class="rum-cell" x="313.2" y="230" width="5" height="64"/><rect class="rum-cell" x="320.8" y="230" width="5" height="64"/><rect class="rum-cell" x="328.4" y="230" width="5" height="64"/><rect class="rum-cell" x="336.0" y="230" width="5" height="64"/><rect class="rum-cell" x="343.6" y="230" width="5" height="64"/><rect class="rum-cell" x="351.2" y="230" width="5" height="64"/><rect class="rum-cell" x="358.8" y="230" width="5" height="64"/><rect class="rum-cell" x="366.4" y="230" width="5" height="64"/>
<rect class="rum-bg" x="392" y="240" width="96" height="18"/>
<text class="rum-mono" x="440" y="253" text-anchor="middle">ALTER UPDATE</text>
<path class="rum-arr" d="M382,266 L494,266"/>
<rect class="rum-grp" x="498" y="222" width="244" height="80" rx="8"/>
<rect class="rum-cell" x="506.0" y="230" width="5" height="64"/><rect class="rum-cell" x="513.6" y="230" width="5" height="64"/><rect class="rum-cell" x="521.2" y="230" width="5" height="64"/><rect class="rum-cell" x="528.8" y="230" width="5" height="64"/><rect class="rum-cellx" x="536.4" y="230" width="5" height="64"/><rect class="rum-cell" x="544.0" y="230" width="5" height="64"/><rect class="rum-cell" x="551.6" y="230" width="5" height="64"/><rect class="rum-cell" x="559.2" y="230" width="5" height="64"/><rect class="rum-cell" x="566.8" y="230" width="5" height="64"/><rect class="rum-cell" x="574.4" y="230" width="5" height="64"/><rect class="rum-cell" x="582.0" y="230" width="5" height="64"/><rect class="rum-cell" x="589.6" y="230" width="5" height="64"/><rect class="rum-cell" x="597.2" y="230" width="5" height="64"/><rect class="rum-cell" x="604.8" y="230" width="5" height="64"/><rect class="rum-cell" x="612.4" y="230" width="5" height="64"/>
<rect class="rum-cell" x="620.0" y="230" width="5" height="64"/><rect class="rum-cell" x="627.6" y="230" width="5" height="64"/><rect class="rum-cell" x="635.2" y="230" width="5" height="64"/><rect class="rum-cell" x="642.8" y="230" width="5" height="64"/><rect class="rum-cell" x="650.4" y="230" width="5" height="64"/><rect class="rum-cell" x="658.0" y="230" width="5" height="64"/><rect class="rum-cell" x="665.6" y="230" width="5" height="64"/><rect class="rum-cell" x="673.2" y="230" width="5" height="64"/><rect class="rum-cell" x="680.8" y="230" width="5" height="64"/><rect class="rum-cell" x="688.4" y="230" width="5" height="64"/><rect class="rum-cell" x="696.0" y="230" width="5" height="64"/><rect class="rum-cell" x="703.6" y="230" width="5" height="64"/><rect class="rum-cell" x="711.2" y="230" width="5" height="64"/><rect class="rum-cell" x="718.8" y="230" width="5" height="64"/><rect class="rum-cell" x="726.4" y="230" width="5" height="64"/>
<text class="rum-warn" x="260" y="322" text-anchor="middle">read: register_id, doc_id, status</text>
<text class="rum-lbl" x="260" y="342" text-anchor="middle">130 MB uncompressed</text>
<text class="rum-warn" x="620" y="322" text-anchor="middle">status rewritten whole: 1.39 MB</text>
<text class="rum-lbl" x="620" y="342" text-anchor="middle">to change 1 byte; 29 hard-linked (wide part)</text>
</svg>
<figcaption class="rum-cap">One status changed. HBase adds a cell and defers the rewrite; ClickHouse rewrites a column of the part now.</figcaption>
</figure>

*P* is the set of parts the mutation has to check; on a real table the partition key and the primary index can rule many of them out. On the stand the table is one part of 10 million rows, so there is nothing to rule out: *P* and *T* are the same single part, *W* is the key (`register_id`, `doc_id`) and *S* is `status`. One update read 130 MB, register_id 40 MB plus doc_id 80 MB plus status 10 MB, uncompressed, exactly what the card predicts. It wrote the status column of the part, 1.39 MB compressed, to change one byte.

The card also says the cost is set by the touched parts and columns, not by the number of rows changed. On this stand, updating more rows in the same part barely changes the cost:

| rows changed at once | 1 | 1,003 | 100,000 |
|---|---|---|---|
| ClickHouse `ALTER UPDATE`, wait for the mutation | 157&nbsp;ms | 139&nbsp;ms | 138&nbsp;ms |
| HBase Puts, one batch | (warm-up) | 310&nbsp;ms | 850&nbsp;ms |

(The single-row HBase batch measured the client's first region lookup, not the Put.)

The gap is widest for a stream of single fixes. A hundred single-row updates, one after another, medians: an HBase Put 472 µs, a ClickHouse mutation 81.7 ms. That is 173 times; at those medians the same hundred statuses take 0.05 s on HBase and 8 s on ClickHouse.

### ClickHouse Updates by Appending Versions

There is a known workaround: do not change the value, append a new version and let reads pick the newest. HBase does that natively. ClickHouse has done it for years with `ReplacingMergeTree` and `argMax`, which is the trick [an earlier post](/blog/aggregate-is-a-function) is built on. And ClickHouse 26.3 does it inside the engine, as a beta: a lightweight `UPDATE` writes a small patch part with only the changed rows, readers apply the patches on the fly, and a merge or `APPLY PATCHES` folds them into the data later.

The same hundred single-row updates as a lightweight `UPDATE`: 3.12 ms each, 26 times faster than the mutation and still 6.6 times slower than HBase. The cost moved to readers and to a later rewrite:

<figure class="rum-fig">
<svg viewBox="0 0 800 262" role="img" aria-label="Who pays for one updated value, and when. HBase Put: 472 microseconds now; readers merge memstore and files on every read, which is the normal path; later a major compaction rewrites the whole store, here the whole region, 52 seconds per region. ClickHouse lightweight UPDATE: 3.1 ms now, a patch part; readers pay 8 bytes more per row read while a patch is pending; later APPLY PATCHES, 103 ms once for all 100 updates. ClickHouse ALTER UPDATE: 81.7 ms now, the status column rewritten; nothing for readers and nothing later.">
<text class="rum-lbl" x="250" y="26" text-anchor="middle">the write</text>
<text class="rum-lbl" x="470" y="26" text-anchor="middle">readers, meanwhile</text>
<text class="rum-lbl" x="690" y="26" text-anchor="middle">later</text>
<text class="rum-txt" x="20" y="72">HBase Put</text>
<rect class="rum-box" x="150" y="40" width="200" height="56" rx="6"/>
<text class="rum-txt" x="250" y="63" text-anchor="middle">472 µs</text>
<text class="rum-lbl" x="250" y="82" text-anchor="middle">a cell into WAL + memstore</text>
<rect class="rum-box" x="370" y="40" width="200" height="56" rx="6"/>
<text class="rum-txt" x="470" y="63" text-anchor="middle">merge on every read</text>
<text class="rum-lbl" x="470" y="82" text-anchor="middle">memstore + files, always</text>
<rect class="rum-boxw" x="590" y="40" width="200" height="56" rx="6"/>
<text class="rum-txt" x="690" y="63" text-anchor="middle">major compaction</text>
<text class="rum-warn" x="690" y="82" text-anchor="middle">the whole store, 52 s a region</text>
<text class="rum-lbl" x="20" y="133">ClickHouse</text>
<text class="rum-txt" x="20" y="151">lightweight</text>
<rect class="rum-box" x="150" y="110" width="200" height="56" rx="6"/>
<text class="rum-txt" x="250" y="133" text-anchor="middle">3.1 ms</text>
<text class="rum-lbl" x="250" y="152" text-anchor="middle">a patch part</text>
<rect class="rum-boxw" x="370" y="110" width="200" height="56" rx="6"/>
<text class="rum-txt" x="470" y="133" text-anchor="middle">+8 B per row read</text>
<text class="rum-warn" x="470" y="152" text-anchor="middle">while a patch is pending</text>
<rect class="rum-boxw" x="590" y="110" width="200" height="56" rx="6"/>
<text class="rum-txt" x="690" y="133" text-anchor="middle">APPLY PATCHES</text>
<text class="rum-warn" x="690" y="152" text-anchor="middle">103 ms, once for all 100</text>
<text class="rum-lbl" x="20" y="203">ClickHouse</text>
<text class="rum-txt" x="20" y="221">ALTER UPDATE</text>
<rect class="rum-boxw" x="150" y="180" width="200" height="56" rx="6"/>
<text class="rum-txt" x="250" y="203" text-anchor="middle">81.7 ms</text>
<text class="rum-warn" x="250" y="222" text-anchor="middle">status column rewritten</text>
<rect class="rum-box" x="370" y="180" width="200" height="56" rx="6"/>
<text class="rum-lbl" x="470" y="213" text-anchor="middle">nothing</text>
<rect class="rum-box" x="590" y="180" width="200" height="56" rx="6"/>
<text class="rum-lbl" x="690" y="213" text-anchor="middle">nothing</text>
<text class="rum-lbl" x="470" y="256" text-anchor="middle">medians of 100 single-row updates; amber: where the cost lands</text>
</svg>
<figcaption class="rum-cap">A cheaper write moves more of the cost to readers and to a later rewrite.</figcaption>
</figure>

The read overhead is large relative to the size of the patch. With one patch part pending, a part of 100 rows and 1,900 bytes, the narrow scan read 180 MB instead of 100 and took twice as long. On this stand the pending patch added roughly 8 bytes per scanned row, though only 100 of the 10 million rows had changed. `APPLY PATCHES` rewrote the status column once, in 103 ms, and the scans went back to 100 MB. (Measured on one part with one patch part, which the background merge had already made out of the hundred; many unmerged patches were not measured.)

"ClickHouse cannot update" is wrong. It can update the way HBase does: write a small delta now, merge it on every read, fold it into the data later. A change can impose costs at three points: when it is written, while it is read, and when the data is rewritten later. Each engine distributes these costs differently, and even different update methods within the same engine make different trade-offs. A plain `ALTER UPDATE` pays the cost up front. Cassandra writes a new cell like HBase, and its later rewrite, the compaction strategy, is a knob of its own, the one measured in the triangle section at the end.

## Knob 4: Who Owns a Key When There Are Many Nodes

Everything so far ran on one server. Big data rarely does, and the fourth knob only exists on many nodes: what the unit of placement is, and who moves it. The three engines give three different answers.

<figure class="rum-fig">
<svg viewBox="0 0 800 348" role="img" aria-label="Three ways to own a key. HBase: key ranges over a shared store; the client caches who owns which range and goes to the owning server in one hop, and a move hands over files, not bytes. ClickHouse: shards chosen by a sharding key, each with its own disk; a query goes to an initiator that plans it, sends it to the shards and merges the answers, and a row's shard is fixed at insert. Cassandra: a hash ring with no leader; a token-aware client goes straight to the replica, any other node forwards the request to it.">
<text class="rum-txt" x="135" y="16" text-anchor="middle">HBase</text>
<text class="rum-lbl" x="135" y="34" text-anchor="middle">key ranges, shared store</text>
<text class="rum-txt" x="400" y="16" text-anchor="middle">ClickHouse</text>
<text class="rum-lbl" x="400" y="34" text-anchor="middle">sharding key, shared nothing</text>
<text class="rum-txt" x="670" y="16" text-anchor="middle">Cassandra</text>
<text class="rum-lbl" x="670" y="34" text-anchor="middle">hash ring, no leader</text>
<line class="rum-grid" x1="262" y1="8" x2="262" y2="340"/><line class="rum-grid" x1="537" y1="8" x2="537" y2="340"/>
<rect class="rum-box" x="45" y="48" width="180" height="50" rx="6"/>
<text class="rum-txt" x="135" y="69" text-anchor="middle">client</text>
<text class="rum-lbl" x="135" y="87" text-anchor="middle">owners cached from meta</text>
<path class="rum-arra" d="M77,98 L77,156"/>
<text class="rum-acc" x="85" y="132">one hop</text>
<rect class="rum-boxa" x="25" y="160" width="105" height="50" rx="6"/>
<text class="rum-txt" x="77" y="181" text-anchor="middle">server 1</text>
<text class="rum-lbl" x="77" y="199" text-anchor="middle">regions A, B</text>
<rect class="rum-box" x="140" y="160" width="105" height="50" rx="6"/>
<text class="rum-txt" x="192" y="181" text-anchor="middle">server 2</text>
<text class="rum-lbl" x="192" y="199" text-anchor="middle">regions C, D</text>
<path class="rum-line" d="M77,210 L77,246"/><path class="rum-line" d="M192,210 L192,246"/>
<rect class="rum-box" x="25" y="246" width="220" height="40" rx="6"/>
<text class="rum-txt" x="135" y="271" text-anchor="middle">shared store (HDFS)</text>
<text class="rum-lbl" x="135" y="332" text-anchor="middle">a move hands over files, not bytes</text>
<rect class="rum-box" x="325" y="48" width="150" height="50" rx="6"/>
<text class="rum-txt" x="400" y="69" text-anchor="middle">client</text>
<text class="rum-lbl" x="400" y="87" text-anchor="middle">any query</text>
<path class="rum-arr" d="M400,98 L400,112"/>
<rect class="rum-boxw" x="325" y="116" width="150" height="50" rx="6"/>
<text class="rum-txt" x="400" y="137" text-anchor="middle">initiator</text>
<text class="rum-warn" x="400" y="155" text-anchor="middle">plans, sends, merges</text>
<path class="rum-arr" d="M400,166 L400,170 Q400,180 390,180 L347,180 Q337,180 337,190 L337,212"/>
<path class="rum-arr" d="M400,166 L400,170 Q400,180 410,180 L452,180 Q462,180 462,190 L462,212"/>
<rect class="rum-boxa" x="285" y="216" width="105" height="50" rx="6"/>
<text class="rum-txt" x="337" y="237" text-anchor="middle">shard 1</text>
<text class="rum-lbl" x="337" y="255" text-anchor="middle">own disk</text>
<rect class="rum-box" x="410" y="216" width="105" height="50" rx="6"/>
<text class="rum-txt" x="462" y="237" text-anchor="middle">shard 2</text>
<text class="rum-lbl" x="462" y="255" text-anchor="middle">own disk</text>
<text class="rum-lbl" x="400" y="332" text-anchor="middle">a row's shard is fixed at insert</text>
<rect class="rum-box" x="570" y="48" width="200" height="50" rx="6"/>
<text class="rum-txt" x="670" y="69" text-anchor="middle">client</text>
<text class="rum-lbl" x="670" y="87" text-anchor="middle">token-aware or not</text>
<path class="rum-arra" d="M590,98 L590,212"/>
<text class="rum-acc" x="596" y="160">token-aware</text>
<path class="rum-arr" d="M750,98 L750,212"/>
<text class="rum-lbl" x="744" y="160" text-anchor="end">any node</text>
<rect class="rum-boxa" x="555" y="216" width="70" height="50" rx="6"/>
<text class="rum-txt" x="590" y="237" text-anchor="middle">node 1</text>
<text class="rum-acc" x="590" y="255" text-anchor="middle">replica</text>
<rect class="rum-box" x="635" y="216" width="70" height="50" rx="6"/>
<text class="rum-txt" x="670" y="237" text-anchor="middle">node 2</text>
<text class="rum-lbl" x="670" y="255" text-anchor="middle">own disk</text>
<rect class="rum-box" x="715" y="216" width="70" height="50" rx="6"/>
<text class="rum-txt" x="750" y="237" text-anchor="middle">node 3</text>
<text class="rum-lbl" x="750" y="255" text-anchor="middle">own disk</text>
<path class="rum-arrw" d="M750,266 L750,280 Q750,290 740,290 L600,290 Q590,290 590,280 L590,270"/>
<text class="rum-warn" x="670" y="310" text-anchor="middle">forwards to the replica</text>
<text class="rum-lbl" x="670" y="332" text-anchor="middle">any node can coordinate</text>
</svg>
<figcaption class="rum-cap">One point read, three answers to "who owns this key". Highlighted: the path that goes straight to the owner.</figcaption>
</figure>

First, a limitation of the stand. All the pods run on one machine with six physical cores, so adding a second node adds another CPU quota, not another physical machine. Each half of the wide scan, alone on its shard, took 176 ms; both halves at once took 340 ms each. Speedup cannot be measured here. What can be measured is where the work goes and how many nodes a request has to touch, and that is what this section is about.

<figure class="rum-fig">
<svg viewBox="0 0 800 356" role="img" aria-label="Nodes a point read asks: 1 when the key decides the owner, S otherwise. Work on the initiator for an aggregation is of the order of S times G, the partial states of G groups from S shards. Share of the data that moves when a cluster grows from S to S plus 1 nodes: S over S plus 1 with a modulo hash, about 1 over S plus 1 with a ring, and nothing at once for a range split, whose bytes compaction rewrites later.">
<rect class="rum-mcard" x="1" y="1" width="798" height="354" rx="8"/><rect class="rum-mbar" x="1" y="1" width="4" height="354" rx="2"/>
<text class="rum-m" x="36" y="44"><tspan class="rum-mf">nodes asked</tspan>(point read) = 1 or <tspan class="rum-mv">S</tspan></text>
<text class="rum-lbl" x="36" y="66">1 when the key names the owner: HBase, Cassandra token-aware, ClickHouse with the sharding key in WHERE</text>
<text class="rum-m" x="36" y="112"><tspan class="rum-mf">work</tspan><tspan font-size="13" baseline-shift="sub">initiator</tspan>(GROUP BY) = <tspan class="rum-mf">O</tspan>(<tspan class="rum-mv">S</tspan> · <tspan class="rum-mv">G</tspan>)</text>
<text class="rum-lbl" x="36" y="134">partial states of G groups from each of S shards; zero when no group crosses shards and the engine knows it</text>
<text class="rum-m" x="36" y="180"><tspan class="rum-mf">moved</tspan><tspan font-size="13" baseline-shift="sub">hash</tspan>(<tspan class="rum-mv">S</tspan> → <tspan class="rum-mv">S</tspan> + 1) = <tspan class="rum-mv">S</tspan> / (<tspan class="rum-mv">S</tspan> + 1)</text>
<text class="rum-lbl" x="36" y="202">share of the data a modulo hash moves when one node is added; ClickHouse does not reshard by itself</text>
<text class="rum-m" x="36" y="248"><tspan class="rum-mf">moved</tspan><tspan font-size="13" baseline-shift="sub">ring</tspan>(<tspan class="rum-mv">S</tspan> → <tspan class="rum-mv">S</tspan> + 1) ≈ 1 / (<tspan class="rum-mv">S</tspan> + 1)</text>
<text class="rum-lbl" x="36" y="270">a hash ring: the new node takes its share and streams it from the others</text>
<text class="rum-m" x="36" y="316"><tspan class="rum-mf">moved</tspan><tspan font-size="13" baseline-shift="sub">range</tspan>(split) = 0</text>
<text class="rum-lbl" x="36" y="338">a split region keeps its bytes as references; compaction rewrites them later</text>
</svg>
</figure>

### One Read, One Node, or a Detour

An HBase read has no coordinator at all. The client reads the region locations once from the `hbase:meta` table, caches them, and sends every Get straight to the server that owns the key. With two region servers, both answered a whole-row Get in about 250 µs, and no other node was involved.

ClickHouse puts an initiator in front of the shards: the server that receives the query plans it again, sends it to each shard over TCP (on this stand its own shard too) and merges what comes back. For one row that detour cost more than the read itself:

<figure class="rum-fig">
<svg viewBox="0 0 800 200" role="img" aria-label="Point read of a whole row, median, log scale. HBase with two region servers, straight to the owner: 250 microseconds, no coordinator. Cassandra with three nodes: 230 microseconds token-aware, 363 microseconds when a coordinator forwards. ClickHouse with two shards: 3.5 ms on one server, 8.3 ms through the Distributed table.">
<circle class="rum-dota" cx="246" cy="22" r="6"/><text class="rum-lbl" x="258" y="26">straight to the data</text>
<circle class="rum-dot" cx="410" cy="22" r="6"/><text class="rum-lbl" x="422" y="26">through a coordinator</text>
<line class="rum-grid" x1="220" y1="44" x2="220" y2="158"/><line class="rum-grid" x1="348.8" y1="44" x2="348.8" y2="158"/><line class="rum-grid" x1="490" y1="44" x2="490" y2="158"/><line class="rum-grid" x1="618.8" y1="44" x2="618.8" y2="158"/><line class="rum-grid" x1="760" y1="44" x2="760" y2="158"/>
<text class="rum-txt" x="20" y="66">HBase, 2 servers</text>
<text class="rum-txt" x="20" y="106">Cassandra, 3 nodes</text>
<text class="rum-txt" x="20" y="146">ClickHouse, 2 shards</text>
<circle class="rum-dota" cx="327.5" cy="62" r="6"><title>HBase, two region servers: 250 µs, no coordinator</title></circle><text class="rum-val" x="327.5" y="50" text-anchor="middle">250 µs</text>
<text class="rum-lbl" x="342" y="66">no coordinator</text>
<line class="rum-bar" x1="317.6" y1="102" x2="371.2" y2="102"/>
<circle class="rum-dota" cx="317.6" cy="102" r="6"><title>Cassandra, token-aware: 230 µs</title></circle><text class="rum-val" x="311" y="90" text-anchor="end">230 µs</text>
<circle class="rum-dot" cx="371.2" cy="102" r="6"><title>Cassandra, a coordinator forwards: 363 µs</title></circle><text class="rum-val" x="378" y="90" text-anchor="start">363 µs</text>
<line class="rum-bar" x1="636.9" y1="142" x2="738.2" y2="142"/>
<circle class="rum-dota" cx="636.9" cy="142" r="6"><title>ClickHouse, one server: 3.5 ms</title></circle><text class="rum-val" x="636.9" y="130" text-anchor="middle">3.5 ms</text>
<circle class="rum-dot" cx="738.2" cy="142" r="6"><title>ClickHouse, through Distributed: 8.3 ms</title></circle><text class="rum-val" x="738.2" y="130" text-anchor="middle">8.3 ms</text>
<text class="rum-lbl" x="220" y="176" text-anchor="middle">100 µs</text><text class="rum-lbl" x="348.8" y="176" text-anchor="middle">300</text><text class="rum-lbl" x="490" y="176" text-anchor="middle">1 ms</text><text class="rum-lbl" x="618.8" y="176" text-anchor="middle">3</text><text class="rum-lbl" x="760" y="176" text-anchor="middle">10 ms</text>
<text class="rum-lbl" x="490" y="196" text-anchor="middle">point read of a whole row, median, log scale</text>
</svg>
<figcaption class="rum-cap">The extra latency is what the coordinator does on the way: a Cassandra node forwards, a ClickHouse initiator plans, sends and merges a query.</figcaption>
</figure>

The owning shard did exactly the work a single server does, a granule in 4 ms. The initiator added 4 to 5 ms of its own on top, a fixed cost per query that is invisible under a 0.3 s scan and doubles a point read. Without the sharding key in the `WHERE`, it also sent the query to the shard that does not have the row, and that shard on average read a granule of its own to find nothing: 17.3 thousand rows processed per lookup instead of 8.2 thousand. With `optimize_skip_unused_shards` the query goes to the owner only. That halves the cluster's work but saves just 0.5 ms of latency, because the two shards were asked in parallel.

In Cassandra the detour is the client's choice. Every node can coordinate. A token-aware driver computes the partition's token from the bound key and sends the request to a replica, like the HBase client. If the client sends the request to another node, that node forwards it, much like a ClickHouse initiator. One table, both paths, measured inside the same run: 245 µs when the coordinator was the replica, 363 µs when it forwarded. On this read at `ONE`, the extra coordination cost was about 120 µs, largely consistent with one more network hop. A coordinator also enforces the consistency level and collects and reconciles the answers; at `ONE` there is little of that to do.

### Where New Rows Go

Ownership by key range has a well-known weakness: a key that only grows. Two million new documents written in `doc_id` order into an HBase table with eight regions on two servers:

<figure class="rum-fig">
<svg viewBox="0 0 800 196" role="img" aria-label="Two million new documents in key order. With the key equal to doc_id, all two million rows went to server 1 and none to server 2, at 39.8 thousand rows per second; a range of 1,000 new documents is one scan, 11.3 ms. With a salt byte in front of doc_id, each server got one million rows, at 59.1 thousand rows per second; the same range is 8 scans in parallel, 4.1 ms on an idle cluster.">
<text class="rum-lbl" x="370" y="22" text-anchor="middle">rows per region server</text>
<text class="rum-lbl" x="590" y="22">writes; 1,000 new docs read</text>
<text class="rum-mono" x="20" y="62">key = doc_id</text>
<text class="rum-lbl" x="20" y="80">monotonic</text>
<text class="rum-lbl" x="242" y="57" text-anchor="end">server 1</text>
<rect class="rum-cellx" x="250" y="46" width="240" height="14" rx="2"/><text class="rum-val" x="496" y="57">2,000,000</text>
<text class="rum-lbl" x="242" y="79" text-anchor="end">server 2</text>
<rect class="rum-cell" x="250" y="68" width="2" height="14"/><text class="rum-val" x="258" y="79">0</text>
<text class="rum-warn" x="590" y="57">39.8K rows/s</text>
<text class="rum-lbl" x="590" y="79">1 scan, 11.3 ms</text>
<text class="rum-mono" x="20" y="132">key = salt + doc_id</text>
<text class="rum-lbl" x="20" y="150">8 salt buckets</text>
<text class="rum-lbl" x="242" y="127" text-anchor="end">server 1</text>
<rect class="rum-cella" x="250" y="116" width="120" height="14" rx="2"/><text class="rum-val" x="376" y="127">1,000,000</text>
<text class="rum-lbl" x="242" y="149" text-anchor="end">server 2</text>
<rect class="rum-cella" x="250" y="138" width="120" height="14" rx="2"/><text class="rum-val" x="376" y="149">1,000,000</text>
<text class="rum-acc" x="590" y="127">59.1K rows/s</text>
<text class="rum-lbl" x="590" y="149">8 scans in parallel, 4.1 ms</text>
<text class="rum-lbl" x="400" y="186" text-anchor="middle">one client, WAL on, balancer off while writing; range reads on an idle cluster</text>
</svg>
<figcaption class="rum-cap">With a key that only grows, every new row lands in the last region. A salt byte spreads the writes and splits every range read eight ways.</figcaption>
</figure>

With the plain key, every new row went to the last region and so to one server, while the other server and seven of the eight regions sat idle. Pre-splitting the existing key range does not help: the new writes still land beyond the last split point. Salting, what Phoenix calls `SALT_BUCKETS`, spread the same rows exactly evenly and wrote 1.5 times faster from the same client. Its price is on the read side: a range of new documents is now eight scans instead of one. On an idle cluster the eight parallel scans were even faster, 4.1 ms against 11.3. On a loaded cluster, they still generate eight times as many requests; we did not measure the impact.

With a hash-based sharding expression, ClickHouse makes the same trade; the user chooses the expression, and it need not hash at all. Cassandra's partitioner makes it by default. A hash spreads monotonic keys by itself, and in exchange a range that is not inside one shard or partition goes to every node. Cassandra goes further: there is no order across partitions at all, only inside one.

### What Moving Costs

When ownership changes, the knob decides whether bytes move. On the stand, in HBase:

| HBase, two region servers | took | readers saw | bytes rewritten |
|---|---|---|---|
| move a region to the other server | 1.0&nbsp;s | one refused Get, retried in 138&nbsp;ms | none: the new owner opens the same files |
| split a 99&nbsp;MiB region | 1.1&nbsp;s | one pause of 757&nbsp;ms | none at once: both daughters start as references to the parent's file, and rewrite their halves in a compaction within a minute |
| merge the daughters back | 0.6&nbsp;s | | none at once, references again |

Ownership in HBase is metadata, because the storage is shared; the bytes move later, during compaction. A ClickHouse shard owns its rows on its own disks, so there is nothing to hand over: moving rows means copying them, and a new shard only changes where new rows go. In Cassandra a new node takes over a slice of the ring and streams its data from the others, about 1 / (*S* + 1) of it; that was not measured here. Its placement needed no pre-split at all: with 16 virtual nodes per server, the three nodes got 35.7, 32.7 and 31.6% of the rows on their own.

### Coordinator Work Grows With the Number of Groups

A scan's partial results have to be merged somewhere, and for a `GROUP BY` that is the initiator. Its merge work grows with the number of groups, not with the number of rows. Ten million rows on two shards:

| 10M groups | | wall | initiator CPU | initiator receives |
|---|---|---|---|---|
| `GROUP BY doc_id` | one server | 261&nbsp;ms | | |
| | two shards, initiator merges | 896&nbsp;ms | 1,360&nbsp;ms | 92&nbsp;MB of partial states |
| | two shards, told no group crosses shards | 229&nbsp;ms | 5&nbsp;ms | almost nothing |
| `GROUP BY register_id, doc_id` | two shards, initiator merges | 1,017&nbsp;ms | 2,041&nbsp;ms | 124&nbsp;MB |
| | two shards, sharding key recognised | 293&nbsp;ms | 6&nbsp;ms | almost nothing |

With ten million groups the initiator received every shard's partial states and spent as much CPU merging them as one server needs for the whole query. The two-shard cluster was 3.4 times slower than one server. When ClickHouse may treat the groups as disjoint, the initiator does nothing. You can tell it so (`distributed_group_by_no_merge = 2`, correct here only because each `doc_id` lives on one shard), or it can work that out itself when the sharding key is in the `GROUP BY`. On 26.3 the second needed both `optimize_distributed_group_by_sharding_key` and `optimize_skip_unused_shards`; the first alone did nothing. For about a million groups the cost was small: 132 ms of initiator CPU, and 113 ms of wall time against 75 on one server. The sharding key, chosen for writes, also determines this read cost.

### Replicas, and What "Acknowledged" Means

Only Cassandra in this post keeps replicas of its own: HBase leaves them to HDFS, and the ClickHouse cluster here had none. With three replicas, every request picks how many of them must answer. A point read, token-aware, warm medians:

| consistency level | replicas that answer | median |
|---|---|---|
| ONE | 1 | 264-280&nbsp;µs |
| QUORUM | 2 | 362-385&nbsp;µs |
| ALL | 3 | 400-439&nbsp;µs |

The coordinator asks the replicas in parallel; what changes with the level is how many answers it waits for. `QUORUM` waits for two replica responses instead of one, and on this stand that added roughly 100 µs, comparable to the forwarding penalty measured above. `ALL` waits for the slower of the two remote replicas as well.

The ten million rows were loaded at the driver's default, `LOCAL_ONE`: a write is acknowledged when one replica has it. Under that load the replicas reported 116 thousand dropped mutations, timing out on six shared cores, and the client saw no error. Hinted handoff handled the catch-up: after the load had ended, the hint dispatchers on all three nodes kept completing tasks, the replicas' local write counts kept rising, by 565 thousand in half an hour, and then no node had a pending hint left. The counters do not match one to one, and the number of hints written was not checked: a hint is stored for a missing answer, not for a missing write, so the replayed writes, 1.8 million over the 30 million expected, include writes that had landed already. After compaction the three copies were exactly the same size, 717,670,958 bytes each; their contents were not compared. "Acknowledged" had meant one replica; the other two caught up in the background, without a repair, at least by size.

## The Triangle

In 2016 Athanassoulis and co-authors put a name to what every section above kept running into: the RUM conjecture. An access method can keep any two of three overheads low, the cost of **R**eading, of **U**pdating and of the **M**emory or space the data takes, but not all three. Each knob is a way to move cost from one corner to another:

- **Knob 1**, a bigger *g*: the index shrinks (M), and every point read processes more rows (R).
- **Knob 2**, a column per unit of read: a scan reads only its columns and they compress well (R for scans, M), while a point read opens a block per column (R for point reads). Row storage makes the opposite trade.
- **Knob 3**, append a version instead of rewriting: cheap now (U), paid by readers who merge (R) and by old versions that take space (M) until the next rewrite.
- **Knob 4** is not on the triangle. It multiplies whatever the other three cost by the number of nodes a request has to touch.

<figure class="rum-fig">
<svg viewBox="0 0 800 390" role="img" aria-label="Schematic RUM triangle with corners R, cheap reads, at the top, U, cheap updates, at the bottom left, and M, little space, at the bottom right. HBase sits on the R-U edge: cheap Puts and Gets, 792 MiB on disk. ClickHouse sits on the R-M edge: cheap scans in 299 MiB, 82 ms per update. A lightweight UPDATE moves ClickHouse toward U, and readers pay for it. Cassandra's compaction strategy is a measured segment parallel to the R-U edge, from UCS T4 near U to L10 near R.">
<path class="rum-tri" d="M400,48 L120,340 L680,340 Z"/>
<text class="rum-corner" x="400" y="30" text-anchor="middle">R</text>
<text class="rum-lbl" x="420" y="30">cheap reads, of the kind you run</text>
<text class="rum-corner" x="120" y="366" text-anchor="middle">U</text>
<text class="rum-lbl" x="138" y="366">cheap updates</text>
<text class="rum-corner" x="680" y="366" text-anchor="middle">M</text>
<text class="rum-lbl" x="662" y="366" text-anchor="end">little space</text>
<circle class="rum-dot" cx="258" cy="214" r="7"/>
<text class="rum-txt" x="222" y="180" text-anchor="end">HBase</text>
<text class="rum-lbl" x="222" y="198" text-anchor="end">cheap Puts and Gets,</text>
<text class="rum-lbl" x="222" y="216" text-anchor="end">792 MiB on disk</text>
<circle class="rum-dot" cx="542" cy="214" r="7"/>
<text class="rum-txt" x="578" y="180">ClickHouse</text>
<text class="rum-lbl" x="578" y="198">cheap scans in 299 MiB,</text>
<text class="rum-lbl" x="578" y="216">82 ms per update</text>
<path class="rum-arrwd" d="M534,222 L452,290"/>
<text class="rum-warn" x="470" y="314" text-anchor="middle">lightweight UPDATE: toward U, readers pay</text>
<line class="rum-seg" x1="262" y1="276" x2="365" y2="165"/>
<circle class="rum-dota" cx="262" cy="276" r="6"/><text class="rum-acc" x="276" y="288">T4</text>
<circle class="rum-dota" cx="365" cy="165" r="6"/><text class="rum-acc" x="379" y="170">L10</text>
<text class="rum-acc" x="330" y="234">Cassandra UCS</text>
</svg>
<figcaption class="rum-cap">Schematic, no common scale: each engine sits where its own kind of read is cheap. Only the Cassandra segment is measured, below.</figcaption>
</figure>

Both engines claim the R corner, each for its own kind of read: HBase for a point read, ClickHouse for a scan. The positions are not coordinates measured on a common scale: each engine optimizes a different kind of read, and the RUM conjecture itself is about access methods, not whole databases. Changing a setting moves cost between corners; it does not make all three low. Phoenix is not a point on it at all: a layer moves work between the client and the servers, not between the corners.

### Cassandra: Compaction Strategy as a Table Option

In HBase and ClickHouse the position is built into the engine. In Cassandra one part of it is a table option: the compaction strategy decides how often data is rewritten (U) against how many files a read has to open (R). Cassandra 5.0 folds the classic strategies into one, the Unified Compaction Strategy, with a single parameter that slides from tiered (`T4`) through `N` to leveled (`L10`).

We wrote the same stream into four tables that differed only in that option: ten million rows, then five million updates of two columns at random rows. Then we waited until compaction had nothing left to do:

<figure class="rum-fig">
<svg viewBox="0 0 800 300" role="img" aria-label="Rewrites per byte against SSTables opened per read, after compaction settled, the same stream into four tables. UCS T4: 2.84 rewrites, 2.92 files per read. UCS L10: 8.49 rewrites, 1.66 files per read. STCS, the table default: 3.75 rewrites, 3.27 files. UCS N: 5.54 rewrites, 3.65 files. T4 and L10 trade one for the other; STCS and N are worse than T4 on both.">
<line class="rum-grid" x1="120" y1="40" x2="120" y2="240"/><line class="rum-grid" x1="248" y1="40" x2="248" y2="240"/><line class="rum-grid" x1="376" y1="40" x2="376" y2="240"/><line class="rum-grid" x1="504" y1="40" x2="504" y2="240"/><line class="rum-grid" x1="632" y1="40" x2="632" y2="240"/><line class="rum-grid" x1="760" y1="40" x2="760" y2="240"/>
<line class="rum-grid" x1="120" y1="40" x2="760" y2="40"/><line class="rum-grid" x1="120" y1="90" x2="760" y2="90"/><line class="rum-grid" x1="120" y1="140" x2="760" y2="140"/><line class="rum-grid" x1="120" y1="190" x2="760" y2="190"/><line class="rum-grid" x1="120" y1="240" x2="760" y2="240"/>
<line class="rum-ref" x1="301.8" y1="94" x2="663.4" y2="157"/>
<circle class="rum-dota" cx="301.8" cy="94" r="7"><title>UCS T4: 2.84 rewrites per byte, 2.92 SSTables per read, space +3.5%</title></circle>
<text class="rum-acc" x="296" y="118" text-anchor="middle">T4</text>
<circle class="rum-dota" cx="663.4" cy="157" r="7"><title>UCS L10: 8.49 rewrites per byte, 1.66 SSTables per read, space +1.3%</title></circle>
<text class="rum-acc" x="663.4" y="181" text-anchor="middle">L10</text>
<circle class="rum-dot" cx="360" cy="76.5" r="7"><title>STCS, the table default: 3.75 rewrites per byte, 3.27 SSTables per read, space +4.9%</title></circle>
<text class="rum-txt" x="374" y="72">STCS, the default</text>
<circle class="rum-dot" cx="474.6" cy="57.5" r="7"><title>UCS N: 5.54 rewrites per byte, 3.65 SSTables per read, space +5.0%</title></circle>
<text class="rum-txt" x="488.6" y="53">N</text>
<text class="rum-lbl" x="140" y="228">fewer rewrites, fewer files: empty</text>
<text class="rum-lbl" x="120" y="258" text-anchor="middle">0</text><text class="rum-lbl" x="248" y="258" text-anchor="middle">2</text><text class="rum-lbl" x="376" y="258" text-anchor="middle">4</text><text class="rum-lbl" x="504" y="258" text-anchor="middle">6</text><text class="rum-lbl" x="632" y="258" text-anchor="middle">8</text><text class="rum-lbl" x="760" y="258" text-anchor="middle">10</text>
<text class="rum-lbl" x="110" y="244" text-anchor="end">0</text><text class="rum-lbl" x="110" y="194" text-anchor="end">1</text><text class="rum-lbl" x="110" y="144" text-anchor="end">2</text><text class="rum-lbl" x="110" y="94" text-anchor="end">3</text><text class="rum-lbl" x="110" y="44" text-anchor="end">4</text>
<text class="rum-lbl" x="440" y="284" text-anchor="middle">bytes written by compaction per byte of data (U)</text>
<text class="rum-lbl" x="20" y="24">SSTables opened per read (R)</text>
</svg>
<figcaption class="rum-cap">T4 and L10 show the expected trade; the default and the middle setting rewrote more than T4 and opened more files per read. Latency is discussed below.</figcaption>
</figure>

<details class="rum-details">
<summary>The same numbers as a table</summary>

| strategy | rewrites per byte | SSTables opened per read | space over the floor | point read, median |
|---|---|---|---|---|
| STCS, the table default | 3.75 | 3.27 | +4.9% | 293-329&nbsp;µs |
| UCS T4 | 2.84 | 2.92 | +3.5% | 355-398&nbsp;µs |
| UCS N | 5.54 | 3.65 | +5.0% | 272-297&nbsp;µs |
| UCS L10 | 8.49 | 1.66 | +1.3% | 237-251&nbsp;µs |
| any, after a major compaction | | 1.00 | 0 | 223-299&nbsp;µs |

</details>

"Rewrites per byte" is what compaction wrote, divided by the size of the same table after a major compaction; the flush writes everything once more on top. SSTables per read are counted from read traces, one event per file a read opens.

From T4 to L10 the rewrites triple and the files a read opens nearly halve. Once settled, L10 reads as fast as a fully compacted table. That matches the textbook trade-off.

The middle setting does not. N rewrote twice as much as T4 and still left reads opening the most files. An interpretation, based on UCS's design rather than its source code: with flushes of a few megabytes and about 260 MB per node, a node holds some 65 flushes of data. That is about three levels at a fanout of 4, six at a fanout of 2 (N), two at a fanout of 10, and each level is a rewrite. On this workload, N was not a midpoint between T4 and L10. The default STCS was also behind T4 on both axes in this run. One stream and one stopping point, though: which levels happen to be full when the stream ends can move the count per read by a whole file.

Latency does not follow the file count. N opened the most files and still answered faster than T4: 272-297 µs against 355-398. L10, with the fewest files, was the fastest, so the count is not noise; but with every file in the page cache and the key cache warm, an extra file cost tens of microseconds, and something else, not traced here, outweighed it between N and T4. So the chart shows the trade between rewriting and files touched per read. The number of files is a useful measure of read cost and a poor predictor of latency on a warm cache; on a cold disk each extra file would cost more.

Space barely moved: it stayed within 5% of the floor, because the updates touched two columns of thirty and the old versions were small. The bloom filter helped none of the strategies: with a partition per register every file holds a slice of every partition. The same rows with a partition per document read exactly one file of five or six without any help from compaction (knob 1 again). In Cassandra the schema moves the R corner as much as the compaction setting does.

## Choosing an Engine

Back to the standup. Both scenes from the beginning, with the knob that decides each of them and what it cost on the stand:

<figure class="rum-fig">
<svg viewBox="0 0 800 236" role="img" aria-label="The two standup scenes revisited. The monthly report on Phoenix is decided by knob 2, whole rows per block: HBase took 24.7 s because every column is read, ClickHouse 0.03 s reading 3 columns of 30. Fixing one status in ClickHouse is decided by knob 3, a whole column rewritten: ClickHouse took 81.7 ms, rewriting 1.39 MB for one byte, HBase 472 microseconds, writing one new cell.">
<text class="rum-lbl" x="20" y="24">the scene</text>
<text class="rum-lbl" x="385" y="24" text-anchor="middle">the knob that decides</text>
<text class="rum-lbl" x="565" y="24" text-anchor="middle">on the engine it was sent to</text>
<text class="rum-lbl" x="720" y="24" text-anchor="middle">on the other one</text>
<rect class="rum-box" x="20" y="40" width="250" height="56" rx="6"/>
<text class="rum-txt" x="145" y="63" text-anchor="middle">the monthly report on Phoenix</text>
<text class="rum-lbl" x="145" y="82" text-anchor="middle">GROUP BY over months of documents</text>
<path class="rum-arr" d="M270,68 L296,68"/>
<rect class="rum-box" x="300" y="40" width="170" height="56" rx="6"/>
<text class="rum-txt" x="385" y="63" text-anchor="middle">knob 2</text>
<text class="rum-lbl" x="385" y="82" text-anchor="middle">whole rows per block</text>
<path class="rum-arr" d="M470,68 L491,68"/>
<rect class="rum-boxw" x="495" y="40" width="140" height="56" rx="6"/>
<text class="rum-txt" x="565" y="63" text-anchor="middle">HBase: 24.7 s</text>
<text class="rum-warn" x="565" y="82" text-anchor="middle">every column read</text>
<rect class="rum-boxa" x="650" y="40" width="140" height="56" rx="6"/>
<text class="rum-txt" x="720" y="63" text-anchor="middle">ClickHouse: 0.03 s</text>
<text class="rum-acc" x="720" y="82" text-anchor="middle">3 columns of 30</text>
<rect class="rum-box" x="20" y="126" width="250" height="56" rx="6"/>
<text class="rum-txt" x="145" y="149" text-anchor="middle">fix one status in ClickHouse</text>
<text class="rum-mono" x="145" y="168" text-anchor="middle">ALTER TABLE ... UPDATE</text>
<path class="rum-arr" d="M270,154 L296,154"/>
<rect class="rum-box" x="300" y="126" width="170" height="56" rx="6"/>
<text class="rum-txt" x="385" y="149" text-anchor="middle">knob 3</text>
<text class="rum-lbl" x="385" y="168" text-anchor="middle">a whole column rewritten</text>
<path class="rum-arr" d="M470,154 L491,154"/>
<rect class="rum-boxw" x="495" y="126" width="140" height="56" rx="6"/>
<text class="rum-txt" x="565" y="149" text-anchor="middle">ClickHouse: 81.7 ms</text>
<text class="rum-warn" x="565" y="168" text-anchor="middle">1.39 MB for 1 byte</text>
<rect class="rum-boxa" x="650" y="126" width="140" height="56" rx="6"/>
<text class="rum-txt" x="720" y="149" text-anchor="middle">HBase: 472 µs</text>
<text class="rum-acc" x="720" y="168" text-anchor="middle">one new cell</text>
<text class="rum-lbl" x="400" y="222" text-anchor="middle">medians on the stand: the narrow scan for the report, one single-row update for the fix</text>
</svg>
<figcaption class="rum-cap">In each scene the workload went to the engine that makes its main operation expensive.</figcaption>
</figure>

Neither engine was inherently the wrong choice. In each scene the workload's dominant cost was one that the other engine handles cheaply. The report is one scan over a few columns, and HBase reads whole rows. The fix is one value in one part, and ClickHouse rewrites the column. Both engines did exactly what their knobs say, and were slower by factors of about 800 on the report's scan and 170 on the fix.

So the choice can come from the knobs rather than from taste:

- **Point reads versus scans, and how many columns a query touches**, decide knobs 1 and 2. A workload of point reads by key wants a small *g* and whole rows. A workload of scans over a few of many columns wants a granule and a file per column.
- **How often rows change, and how scattered the changes are**, decide knob 3. A stream of single-row fixes is a stream of column rewrites for a mutation, Puts of under half a millisecond each for HBase, and something in between for a lightweight `UPDATE` whose cost goes to the readers.
- **The key decides knob 4.** Route by it and a point read touches one node. Group by it and the coordinator has nothing to merge. Make it grow monotonically under a range partitioning, and one server takes every write.
- **The SQL layer is a separate decision.** Phoenix on HBase buys SQL, secondary indexes and partial aggregation on the region servers. It does not change the storage.

Some workloads need both: a hot table of changing documents and a cold history for scans. For them, one answer is two systems connected by change data capture, with its own cost in lag and operations. Or one engine and the append-a-version trick, which is what [the cells in the earlier post](/blog/aggregate-is-a-function) do inside ClickHouse. Either way the cost of the other corner remains; the design only decides where and when it is paid.

The next time someone says "the data is already in Phoenix", the first thing to check is which knob the query's largest cost term depends on. I still argue both sides, but now I know which side I am on before I start.

<details class="rum-details">
<summary>Every number in this post, and the stand they come from</summary>

One k3s node, an AMD Ryzen 5 5600X with 6 physical cores and 12 hardware threads, shared by every pod. ClickHouse 26.3.9.8, 5 CPU and 12 GiB per server. HBase 2.5.16, standalone with 5 CPU, 12 GiB and an 8 GiB heap, or two region servers with 5 CPU, 6 GiB and a 4 GiB heap each over one shared volume. Cassandra 5.0.9, three nodes with 4 CPU, 5 GiB and a 2 GiB heap each. One synthetic table of 10 million documents, 30 columns, the same rows in every engine. Latencies are medians of warm runs.

| | HBase | ClickHouse | Cassandra |
|---|---|---|---|
| on disk, 10M rows | 792&nbsp;MiB (FAST_DIFF + LZ4) | 299&nbsp;MiB (LZ4 + delta codecs) | 684&nbsp;MiB (LZ4, a partition per register) |
| point read, whole row | 170&nbsp;µs; 250&nbsp;µs on two servers | 3.47&nbsp;ms | 216-243&nbsp;µs, token-aware |
| point read, 3 columns | 197&nbsp;µs | 1.44&nbsp;ms | |
| point read through a coordinator | none | 8.3&nbsp;ms (Distributed) | 362-364&nbsp;µs (forwarded) |
| scan, 3 of 30 columns | 24.7&nbsp;s | 0.03&nbsp;s | |
| scan, all 30 columns | 26.9&nbsp;s | 0.35&nbsp;s | |
| one single-row update | 472&nbsp;µs (Put) | 81.7&nbsp;ms (`ALTER UPDATE`), 3.12&nbsp;ms (lightweight) | |
| 100,000 rows changed at once | 850&nbsp;ms | 138&nbsp;ms | |
| `GROUP BY` of 10M groups, one node and two | | 261&nbsp;ms and 896&nbsp;ms | |
| a region moved, split, merged | 1.0&nbsp;s, 1.1&nbsp;s, 0.6&nbsp;s | | |
| a monotonic key, rows per server | 2,000,000 and 0; salted 1,000,000 each | | |
| read at `QUORUM` of 3 replicas | | | 362-385&nbsp;µs |
| compaction rewrites per byte, T4 to L10 | | | 2.84 to 8.49 |

</details>

## Prior Art

None of the four knobs is new, and neither is the triangle. The log-structured merge tree comes from O'Neil, Cheng, Gawlick and O'Neil, "The Log-Structured Merge-Tree (LSM-Tree)", Acta Informatica, 1996. HBase and Cassandra both store data the way Google's "Bigtable: A Distributed Storage System for Structured Data" (Chang et al., OSDI 2006) describes, and Cassandra places it the way Amazon's "Dynamo" (DeCandia et al., SOSP 2007) does. The column store half of the story is in "C-Store: A Column-oriented DBMS" (Stonebraker et al., VLDB 2005) and in Abadi, Madden and Hachem, "Column-Stores vs. Row-Stores: How Different Are They Really?" (SIGMOD 2008). The triangle is "Designing Access Methods: The RUM Conjecture" (Athanassoulis et al., EDBT 2016). Chapter 3 of Martin Kleppmann's *Designing Data-Intensive Applications* tells the same story at book length.

Our own contribution is only the stand: the same ten million documents in three engines, the formula cards checked against the engines' own counters, and a note every time a number did not fit. Several did not: a point read decompresses more than its granule. A pending patch taxes every row of a scan. Replicas left behind by writes acknowledged at `LOCAL_ONE` caught up by themselves, minutes later, at least by size. And the middle setting of a compaction slider was not in the middle.