---
title: "When Clean Design Feels Dead: A Recovery Story"
slug: "redesigning-my-blog"
description: "How I transformed a minimalist but dry blog into something that actually feels alive — one CSS tweak at a time."
date: "2025-01-26T12:00:00Z"
tags: ["design", "css", "web", "ux"]
draft: false
cover_image: "https://res.cloudinary.com/ddkzhz9b4/image/upload/nebula/design-recovery-cover.webp"
---

I built this blog with Rust, Axum, and HTMX. It was fast. It was minimal. It was... boring.

Don't get me wrong — I love minimalism. But there's a difference between "clean" and "lifeless." My blog was firmly in the latter category. White text on dark background, no images, no personality. It looked like a terminal that learned HTML.

So I decided to fix it.

## The Spark

It started with a WordPress article about blog design best practices. The usual stuff: use images, add visual hierarchy, make it scannable. Nothing revolutionary.

But one line stuck with me: "Your blog should feel like a place, not a document."

I looked at my blog. It felt like a README file.

## The Approach

I didn't want a complete redesign. That's a rabbit hole. Instead, I committed to small, iterative changes. Each tweak had to be reversible. If something didn't work, I'd revert it immediately.

This turned out to be the key. Instead of spending weeks on a redesign that might not work, I spent hours on tiny improvements that I could evaluate instantly.

## The Changes

### Hero Section: Adding Life

The hero section was just text. Functional, but flat.

I added a subtle gradient — a radial glow emanating from the top. Nothing dramatic, just enough to create depth.

```css
.hero {
    background:
        linear-gradient(to bottom, var(--color-bg), transparent 15%),
        radial-gradient(ellipse 100% 70% at 50% -10%,
            rgba(99, 102, 241, 0.18), transparent 60%);
}
```

The trick was layering two gradients: a radial one for the glow, and a linear one on top to soften the edge. Without that second gradient, there was a harsh line where the glow ended.

### Cover Images: Visual Anchors

Blog posts without images are walls of text. I added support for cover images — a simple `cover_image` field in the frontmatter.

On the blog list, posts with covers display in a two-column layout: image on the left, text on the right. On the post itself, the cover appears as a panoramic banner with a 21:9 aspect ratio.

```css
.post-hero {
    aspect-ratio: 21 / 9;
    overflow: hidden;
}

.post-hero img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}
```

The wide aspect ratio keeps images from dominating the page while still providing that visual anchor.

### Reading Time: Setting Expectations

Nothing fancy here — just showing estimated reading time next to the date. But it matters. Readers want to know what they're committing to.

### Back to Top: Respecting Long Reads

For longer posts, I added a floating button that appears after scrolling down 300 pixels. It's small, unobtrusive, and surprisingly useful.

### Share Buttons: Enabling Distribution

At the bottom of each post, three share buttons: Twitter/X, LinkedIn, and Telegram. No JavaScript widgets, no tracking — just simple links that open share dialogs.

```html
<a href="https://twitter.com/intent/tweet?url={{ url }}&text={{ title }}">
    Share on Twitter
</a>
```

Clean, fast, privacy-respecting.

## What I Learned

### Small iterations beat big redesigns

Every change was immediately visible. Bad ideas got reverted in seconds. Good ideas built on each other. This feedback loop is invaluable.

### Gradients are tricky

Getting a gradient to look natural is harder than it seems. Hard edges appear where you don't expect them. Layering gradients is often the solution.

### Aspect ratios are powerful

`aspect-ratio` in CSS is a game-changer for responsive images. No more padding hacks, no more JavaScript calculations.

### Minimalism isn't about absence

It's about intentionality. Every element should earn its place. But that doesn't mean having nothing — it means having exactly what you need.

## The Result

The blog still feels minimal. But now it has personality. The gradient gives depth. The images provide visual breaks. The share buttons invite engagement.

Most importantly, it feels like a place now. Not just a document.

---

*The best time to improve your design was when you launched. The second best time is now.*
