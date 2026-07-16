# SQFts VS Code / Cursor extension

Gradually-typed SQF (`.sqfts` / `.d.sqfts`) with live diagnostics, hover, and completion via `sqfts-language-server`.

## Develop

```bash
# From repo root
cargo build --release -p sqfts-lsp
cd editors/vscode
npm install
npm run copy-server:win   # or copy server/ manually on other OS
npm run build
```

Install the extension in Cursor/VS Code:

- **Command Palette** → `Extensions: Install from VSIX…` after `npx vsce package`, or
- Symlink this folder into your extensions directory for development.

Set `sqfts.serverPath` if the binary is not bundled under `server/`.

## Features

- Diagnostics from `sqfts check` (engine command DB + `.d.sqfts`)
- Hover: engine overloads and declared function/global signatures
- Completion: commands, declarations, keywords
- Syntax highlighting: engine commands, `*_fnc_*` functions, locals/globals, types
- Reloads declarations when `*.d.sqfts` or `sqfts.toml` is saved

## Erase to SQF (not automatic)

The extension does **not** emit `.sqf` on save. Use the CLI:

```bash
# From a folder with sqfts.toml
cargo run -p sqfts-cli --release -- build .
# or, if sqfts is on PATH:
sqfts build .
```

That writes plain SQF under `out_dir` from `sqfts.toml` (annotations erased).
