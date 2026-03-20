---
title: "Do You Still Need Computer Science in the Age of AI"
slug: "do-you-need-cs-in-ai-era"
description: "AI writes code faster than any developer. But who sets the task, reviews the result, and owns the architecture? On why foundational knowledge matters more now, not less."
date: "2026-03-20T12:00:00Z"
tags: ["opinion", "ai", "computer-science", "career"]
draft: false
cover_image: "do-you-need-cs-in-ai-era-cover.webp"
---

## "Maybe I Should Become a Welder"

A colleague recently told me about a conversation with someone at Amazon. The person described how the company is aggressively pushing AI tools for code generation — management pressure to move faster, ship more, spend less. My colleague, twenty years in the industry, went quiet afterward. "Maybe I should learn welding," he joked. It was only half a joke.

I get it. When you've spent twenty years in this profession and keep hearing that any intern with Copilot will soon do the same thing — it hits not your skills, but your identity. The sense that everything you've learned actually mattered.

Amazon, like any large company, has plenty of strong engineers who've given years to the craft and understand exactly what's happening. But popular opinion doesn't make distinctions: why learn algorithms if AI will write them? Why bother with design patterns if Claude or Copilot can generate a working service in an hour? Why understand how a hash table works when the library gives you one out of the box? It seems like you just need to ask — "build me a service" — and it all works out.

I disagree. Not because I'm defending my profession. But because I can see where mindless use of a tool leads.

## A Tool, Not a Replacement

There was a time when we wrote code in terminal editors. Vim, Emacs — rough days. Then came Eclipse, Visual Studio, IntelliJ IDEA. It was a quantum leap in productivity: autocomplete, refactoring, debugger, VCS integration. Nobody said IDEs replaced programmers. IDEs made programmers faster.

But using even an IDE mindlessly comes at a cost. I once came across job postings — from different companies, so this wasn't a one-off — that listed as a requirement: "ability to write and run a Java program outside of IntelliJ IDEA." I laughed for a while, then stopped to think. People had become so dependent on the tool that they couldn't perform a basic task without it. They didn't understand what was happening under the hood.

AI is the next step in that same evolution. A more powerful tool. But still a tool. And depending on it without understanding the fundamentals leads to the same place — only the stakes are higher.

## What AI Actually Automated

Let's be honest: what does AI do well?

It's excellent at generating boilerplate code. CRUD services, data mapping, standard integrations — everything an experienced developer writes on autopilot, AI writes faster. That's real value, and arguing against it is pointless.

But AI doesn't understand your business context. It doesn't know why you chose eventual consistency over strong consistency. It can't see that your current load will grow tenfold in six months. It doesn't account for the fact that this service will be maintained by a team of three, not twenty. AI doesn't make architectural decisions — it generates code within the decisions that you (or nobody) made for it.

And that "or nobody" is key. If you don't make a decision deliberately, AI will make it for you. Quietly, confidently, and unconsciously.

## Two Price Tags

Every piece of software has two costs: the cost of writing it and the cost of maintaining it. The second is almost always several times higher.

AI has radically reduced the first cost. Generating code is fast, cheap, practically free. But it hasn't reduced the cost of maintenance. If anything, it's made it worse: when code is generated ten times faster, it needs to be reviewed ten times more. And if there's no review, or the reviewer doesn't understand what they're looking at — the cost of maintenance grows exponentially.

Code that's easy to write and expensive to maintain isn't a savings. It's technical debt taken out at a high interest rate.

## What Happens Without Oversight

This isn't theory. In March 2026, Amazon faced a series of incidents tied to AI-generated code. On March 2nd — 120,000 lost orders and 1.6 million website errors. On March 5th — a 99% drop in orders across North America: 6.3 million lost orders in a single day. In a separate case, the AI agent Kiro, attempting to fix an issue with a cost calculation system, decided to delete and recreate the environment from scratch. Thirteen hours of downtime.

Amazon's response is telling. Not abandoning AI. Tightening the requirements for reviewer qualifications. Now any AI-assisted code must be approved by a senior engineer before deployment. Amazon essentially confirmed: the problem isn't the tool — it's who's using it.

And this isn't just Amazon. According to CodeRabbit, AI-generated code contains 1.7x more bugs than human-written code. 1.75x more logic errors. 1.57x more security issues. And eight times more unnecessary I/O operations. In 2025, pull requests per developer increased by 20%, while incidents per PR rose by 23.5%. More code — more problems.

A telling case: in February 2026, AI generated code using `Promise.all` that fired 4,200 simultaneous requests for user data synchronization. The connection pool couldn't handle it — twenty-two minutes of downtime. Anyone who understands concurrency and backpressure would have spotted the problem in seconds. But whoever accepted this code either wasn't taught that, or decided it no longer mattered.

## The Language of Task Definition

And this brings us to the main point. Why do you need Computer Science if the machine writes the code?

Not to write a sorting algorithm on a whiteboard during an interview. Not to memorize the pseudocode for depth-first search. That really is routine work, and yes, the machine can reproduce it.

The value of Computer Science is in the conceptual framework. In the ability to think systematically, decompose problems, distinguish between abstractions, understand constraints and trade-offs. In the ability to verify what AI just generated for you.

Without terminology, a person says: "rewrite this better," "something's off here," "make it look nice," "speed it up," "fix the architecture." Fix it how? Speed up in what direction? What does "better" mean?

With terminology, a person says: isolate the domain from infrastructure, wire dependencies through IoC, implement an adapter for the legacy API integration, replace branching with the strategy pattern, reduce coupling, separate component responsibilities, fix the abstraction leak, break circular dependencies, extract object creation into a factory. Use a doubly linked list instead of an array, optimize hash table collisions, reduce unnecessary allocations. Protect the module from race conditions, add backpressure, use a circuit breaker. Don't mix application service with domain service, preserve aggregate invariants, make the operation idempotent.

The first person asks AI to guess what they mean. The second precisely defines the task. And more importantly, the second one can verify the result.

## CS as a Frame of Reference

I'm not saying you need to implement a hash table from scratch with your eyes closed. I'm saying you need to understand how it works — so you can notice when AI suggests the wrong data structure. You don't need to remember Dijkstra's implementation — but you need to understand algorithmic complexity to see when AI produces O(n²) where O(n log n) would do.

Though it's still better to both understand and know how to implement. Or at least strive for it.

Computer Science isn't a set of skills you need to reproduce by hand. It's a frame of reference in which you think about software. IoC, SOLID, GRASP, DDD, Clean Architecture, CQRS, Event Sourcing, CAP theorem, ACID, consistency models, memory model, cache locality, idempotency, fault tolerance — these aren't for exams. They're a working vocabulary that lets you define tasks precisely, review results meaningfully, and tell working architecture from a convincing-looking hallucination.

## Not About Fear

I'm not trying to scare anyone. AI is a breakthrough, and I use it every day. It makes me more productive. But it doesn't make me unnecessary — just as IntelliJ didn't make Java developers unnecessary.

To that colleague who joked about becoming a welder, I'd say: your twenty years of experience haven't lost their value. They've become more valuable. Because now you're needed not to write code, but to understand what's been written. To control quality. To make architectural decisions. To spot design flaws before they become production incidents.

Computer Science matters. Maybe more than ever. Just not the kind they ask about in interviews. The kind that lets you think.
