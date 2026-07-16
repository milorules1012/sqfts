# Uploading to the GitHub Wiki

These markdown files are written as **individual wiki pages**. Upload them manually to your repository’s Wiki (or sync with a wiki git remote).

## Page map

| File in `docs/` | Wiki page title |
|---|---|
| `Home.md` | Home (wiki landing page — required name) |
| `_Sidebar.md` | `_Sidebar` (GitHub renders this as the wiki sidebar) |
| `Getting-Started.md` | Getting-Started |
| `Basic-Concepts.md` | Basic-Concepts |
| … | Same as filename without `.md` |

GitHub wiki links of the form `[text](Page-Name)` resolve to `/wiki/Page-Name`. Filenames use Title-Case-With-Dashes to match those slugs.

## Suggested upload steps

1. Open the repository on GitHub → **Wiki** → Create the first page if needed.
2. For each `docs/*.md` (except this file):
   - Create a page whose title matches the filename without `.md` (e.g. `Getting-Started`).
   - Paste the markdown body.
3. Upload `Home.md` as the wiki **Home** page.
4. Upload `_Sidebar.md` as page `_Sidebar` for left navigation.

## Do not need to upload

| File | Reason |
|---|---|
| `Wiki-Upload.md` (this file) | Maintainer instructions only |

## After upload

Cross-links between handbook pages should work without path prefixes. The normative specification is stored at `docs/design-history/language-specification.md`; on the wiki, either:

- Change specification links to absolute GitHub blob URLs after upload, or
- Upload the specification and design-history index as additional wiki pages.

## Keeping docs in sync

The archived language specification remains normative. When language rules change, update the affected handbook pages and specification together.
