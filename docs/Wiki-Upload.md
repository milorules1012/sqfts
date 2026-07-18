# Uploading to the GitHub Wiki

These markdown files are written as **individual wiki pages**. On each published GitHub Release, [`.github/workflows/wiki-sync.yml`](../.github/workflows/wiki-sync.yml) mirrors the handbook into the repository wiki.

## Automatic sync

1. Publish a GitHub Release (or re-run the workflow manually: **Actions** → **Wiki sync** → **Run workflow**).
2. The workflow copies top-level `docs/*.md` pages (except this file) into `https://github.com/<owner>/<repo>.wiki.git` and pushes.

### Prerequisites

- The repository **Wiki** feature is enabled, and at least one wiki page already exists (that initializes the `.wiki.git` remote).
- Repository secret **`WIKI_TOKEN`**: a fine-grained personal access token with **Contents: Read and write** on this repository (or a classic token with the `repo` scope). The default `GITHUB_TOKEN` cannot reliably push to the wiki git remote.

## Page map

| File in `docs/` | Wiki page title |
|---|---|
| `Home.md` | Home (wiki landing page — required name) |
| `_Sidebar.md` | `_Sidebar` (GitHub renders this as the wiki sidebar) |
| `Getting-Started.md` | Getting-Started |
| `Basic-Concepts.md` | Basic-Concepts |
| … | Same as filename without `.md` |

GitHub wiki links of the form `[text](Page-Name)` resolve to `/wiki/Page-Name`. Filenames use Title-Case-With-Dashes to match those slugs.

## Do not upload

| Path | Reason |
|---|---|
| `Wiki-Upload.md` (this file) | Maintainer instructions only |
| `docs/design-history/` | Stays in the repository; handbook links use absolute GitHub blob URLs |

## Keeping docs in sync

Edit handbook pages under `docs/`, then publish a release (or run **Wiki sync** manually). The archived language specification under `docs/design-history/` remains normative; when language rules change, update the affected handbook pages and specification together.
