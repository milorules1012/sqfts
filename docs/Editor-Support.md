# Editor Support

SQFts ships a Language Server Protocol implementation and a VS Code / Cursor extension.

## Features

- **Diagnostics** — live `sqfts check` results (engine DB + `.d.sqfts`)
- **Hover** — engine overloads and declared function / global signatures
- **Completion** — commands, declarations, keywords
- **Syntax highlighting** — engine commands, `*_fnc_*` functions, locals / globals, type keywords
- **Reload** — rechecks when `*.d.sqfts` or `sqfts.toml` is saved

## Install (development)

From the repository root:

```bash
cargo build --release -p sqfts-lsp
cd editors/vscode
npm install
npm run copy-server:win   # or copy server/ manually on other OS
npm run build
```

Then either:

- Package a VSIX (`npx vsce package`) and **Extensions: Install from VSIX…**, or
- Symlink `editors/vscode` into your extensions directory for live development

Set `sqfts.serverPath` in settings if the binary is not bundled under `server/`.

## Erase is not automatic

The extension does **not** emit `.sqf` on save. Use the CLI:

```bash
sqfts build .
```

That writes plain SQF under `out_dir` from `sqfts.toml`.

## Language server binary

```bash
cargo run -p sqfts-lsp --release
```

The LSP and CLI share `sqfts-project::Project` for config, discovery, and checking.

## Supported file types

| Extension | Treated as |
|---|---|
| `.sqfts` | Annotated source |
| `.d.sqfts` | Declaration file |
| `.sqf` | Plain SQF (when opened / when `check_plain_sqf` applies) |
