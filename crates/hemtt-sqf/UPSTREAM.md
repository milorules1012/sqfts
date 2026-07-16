# Vendored `hemtt-sqf`

| Field | Value |
|---|---|
| Upstream | https://github.com/BrettMayson/HEMTT |
| Path | `libs/sqf` |
| Commit | `90b58f381a025e9b44fdfa57a610f217e35d332f` |
| Date | 2026-07 (main) |
| License | GPL-2.0 |

## Local changes vs upstream

- `Cargo.toml` rewritten for this workspace: explicit crate versions instead of
  `workspace = true`, default features reduced to `parser` only (no SQFC /
  `hemtt-lzo` unless the `compiler` feature is enabled).
- Source under `src/` is otherwise identical to the pinned commit.

## Refresh procedure

```bash
git clone --depth 1 https://github.com/BrettMayson/HEMTT.git /tmp/HEMTT
# record new rev, copy libs/sqf/src into crates/hemtt-sqf/src
# update rev pins in this file and crates/hemtt-sqf/Cargo.toml
```

Sibling crates (`hemtt-common`, `hemtt-workspace`, `hemtt-preprocessor`) are
consumed as git dependencies at the same revision rather than vendored.
