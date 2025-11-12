# Keystone Documentation

This is the Fumadocs-powered documentation site for Keystone.

## Development

```bash
bun install
bun run dev
```

Then open [http://localhost:3000](http://localhost:3000).

## Build

```bash
bun run build
bun run start
```

## Content

Documentation content is located in `content/docs/`:

- **Getting Started**: index.mdx, installation.mdx, quick-start.mdx
- **Configuration**: configuration.mdx
- **Usage**: usage.mdx and usage/ folder
- **Operators**: operators/ folder
- **Connectors**: connectors/ folder
- **CLI Reference**: cli-reference.mdx

## Adding Content

Create new `.mdx` files in `content/docs/` with frontmatter:

```mdx
---
title: Page Title
description: Page description
---

# Page Title

Your content here...
```

Update `meta.json` files to control navigation order.

## Powered By

- [Fumadocs](https://fumadocs.dev) - Documentation framework
- [Next.js](https://nextjs.org) - React framework
- [Tailwind CSS](https://tailwindcss.com) - Styling
