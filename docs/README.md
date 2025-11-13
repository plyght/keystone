# Birch Documentation

Documentation site for Birch, powered by Fumadocs.

## Development

Start the development server:

```bash
bun install
bun run dev
```

Then open http://localhost:3000

## Build

Build for production:

```bash
bun run build
bun run start
```

## Content Structure

Documentation content is located in `content/docs/`:

- **Getting Started**: index.mdx, installation.mdx, quick-start.mdx
- **SDK**: sdk/ folder with getting-started, installation, configuration, api-reference, examples, and frameworks
- **Configuration**: configuration.mdx
- **Usage**: usage.mdx and usage/ folder with app-signals and key-pools guides
- **Operators**: operators/ folder with runbook, incident-checklist, and rotation-checklist
- **Connectors**: connectors/ folder with provider-specific guides
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

## Technology Stack

- Fumadocs - Documentation framework
- Next.js - React framework
- Tailwind CSS - Styling
