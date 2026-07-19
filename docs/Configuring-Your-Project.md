# Configuring Your Project for SQFts

This guide shows how to wire SQFts into an existing Arma 3 mission / addon tree. It uses a typical multi-addon layout (mission + server addon + optional HC) so you can copy the same shape for your own repo.

For language details see [Getting Started](Getting-Started), [Configuration](Configuration), and [Adoption Guide](Adoption-Guide).

## What you end up with

```text
your-mission/
├── sqfts.toml                 # project config (CLI + language server)
├── .vscode/
│   └── settings.json          # points the editor at the language server
├── .sqfts/
│   ├── mission.d.sqfts        # ambient declarations (client / mission)
│   ├── server.d.sqfts         # ambient declarations (server addon)
│   └── …                      # optional scratch / demo .sqfts files
├── MissionName.Map/           # mission sources (may contain .sqfts)
├── addon_server/              # server addon sources
└── addon_hc/                  # optional headless / other addons
```

Typical workflow:

1. Author typed code as `.sqfts`.
2. The language server type-checks against engine commands + your `.d.sqfts` files.
3. On save (optional), annotations are erased to a sibling `.sqf` that HEMTT / your packer already consumes.

## 1. Build the toolchain

From a clone of this repository:

```bash
cargo build --release -p sqfts-cli -p sqfts-lsp
```

Binaries:

| Binary | Path (Windows) |
|---|---|
| CLI | `target/release/sqfts.exe` |
| Language server | `target/release/sqfts-language-server.exe` |

Put `sqfts` on your `PATH`, or invoke it via `cargo run -p sqfts-cli -- …`.

Engine commands load automatically from [arma3-wiki](Engine-Command-Database); no separate extract step is required.

Install the editor extension from `editors/vscode` (VSIX or symlink). See [Editor Support](Editor-Support).

## 2. Add `sqfts.toml` at the mission root

Place `sqfts.toml` next to your mission and addon folders (the directory you open in VS Code / Cursor).

Example for a multi-addon mission tree:

```toml
# Roots walked for .sqfts (and .sqf when check_plain_sqf is true)
sources = ["MissionName.Map", "addon_server", "addon_hc", ".sqfts"]

# Ambient declaration files / directories
declarations = [".sqfts"]

# Optional: extra #include roots (HEMTT LayerType::Include).
# Omit to auto-add ./include when present; use [] to disable.
# include_paths = ["include"]

# Erase .sqfts → .sqf beside the source (same relative path under project root).
# Use this when HEMTT / your packer already ships those .sqf files in-tree.
out_dir = "."

# Language server: on each .sqfts save, erase and write the sibling .sqf
build_on_save = true

[flags]
# Start narrow: only type-check annotated .sqfts files.
# Flip to true later to check plain .sqf against your declarations.
check_plain_sqf = false
```

### Field notes for real missions

| Setting | Why it matters |
|---|---|
| `sources` | List every tree that contains (or will contain) `.sqfts`. Include `.sqfts` if you keep demos / scratch files there. |
| `declarations` | Usually a single `.sqfts/` folder holding `*.d.sqfts`. |
| `include_paths` | Extra directories for `#include "…"`. Relative `#include "..\..\macro.h"` resolves from the file’s path under the project root without this; use `include_paths` for include-layer names with a leading `\`, e.g. `#include "\shared.h"`. |
| `out_dir = "."` | Writes `path/to/fn_foo.sqf` next to `path/to/fn_foo.sqfts`. Prefer a separate folder (e.g. `out/sqf`) only if your pack step copies from that output. |
| `build_on_save` | Editor-only convenience so packaging keeps seeing plain SQF without a manual `sqfts build`. |
| `check_plain_sqf = false` | Quieter first pass while you grow declarations. Turn on when you want call-site checking in existing `.sqf`. |

Missing `sqfts.toml` → defaults (`sources = ["."]`, `out_dir = "out/sqf"`, `build_on_save = false`).

## 3. Generate declaration skeletons

Create `.sqfts/` and run `declgen` against your `CfgFunctions` / `Functions.h` registrations.

**Mission `Functions.h`:**

```bash
sqfts declgen MissionName.Map/Functions.h \
  --root MissionName.Map \
  --tag-default CLI \
  --out .sqfts/mission.d.sqfts
```

**Server addon `config.cpp`:**

```bash
sqfts declgen addon_server/config.cpp \
  --project . \
  --root addon_server \
  --tag-default SRV \
  --cfg-functions \
  --out .sqfts/server.d.sqfts
```

Use your real tags (`CLI`, `SRV`, `HC`, …). Repeat for HC or other addons as needed.

If `file = "addon_name\…"` paths include a PBO prefix that is not on disk under `--root`, add:

```toml
[declgen]
strip_prefixes = ["addon_name/", "addon_name/"]
```

Then re-run with `--project .` so declgen loads that section. Details: [Declgen](Declgen).

Tighten high-traffic declarations by hand over time:

```sqfts
declare function DEMO_fnc_refuel(_vehicle: object, _liters: number): boolean;
```

## 4. Configure the VS Code / Cursor workspace

Open the **mission repository folder** as the workspace root (so `sqfts.toml` is discovered).

Create `.vscode/settings.json`:

```json
{
  "sqfts.serverPath": "C:\\Users\\YOU\\Documents\\GitHub\\sqfts\\target\\release\\sqfts-language-server.exe"
}
```

Use your actual path to the release language server. Alternatives:

- Leave `sqfts.serverPath` empty if the extension bundles the binary under `editors/vscode/server/`, or
- After rebuilding the extension, run `npm run copy-server:win` so the bundled copy is current.

Whenever you rebuild the Rust LSP (`cargo build --release -p sqfts-lsp`), reload the editor window (**Developer: Reload Window**) so it picks up the new binary.

Optional: keep local SQFts pilot files out of git until the team adopts them:

```gitignore
# SQFts local pilot
.sqfts/
sqfts.toml
.vscode/
```

Commit them when the project is ready to share the same config.

## 5. Author typed SQF

Rename or add files as `.sqfts` under a `sources` root. Example:

```sqfts
params [
    "_vehicle": object,
    "_liters": number = 50
];

private _ok: boolean = [_vehicle, _liters] call DEMO_fnc_refuel;
hint str _ok;
```

With `build_on_save = true` and `out_dir = "."`, saving writes the erased sibling `.sqf` (annotations stripped) that your existing build already packs.

CLI equivalents:

```bash
sqfts check .          # type-check the project
sqfts build .          # erase all sources into out_dir
```

## 6. Suggested adoption order

1. **Declarations only** — generate `.d.sqfts`, keep `check_plain_sqf = false`, optionally add a small demo under `.sqfts/`.
2. **Pilot files** — convert a few hot-path scripts to `.sqfts` with `build_on_save` so packaging stays unchanged.
3. **Widen checking** — set `check_plain_sqf = true` and tighten declarations until call-site noise is useful.
4. **Raise strictness** — see [Strictness Flags](Strictness-Flags).

## Checklist

- [ ] Built `sqfts` CLI and `sqfts-language-server`
- [ ] Toolchain builds successfully (`cargo build -p sqfts-cli` — pulls arma3-wiki)
- [ ] VS Code / Cursor extension installed
- [ ] `sqfts.toml` at mission root with your source roots
- [ ] `.sqfts/*.d.sqfts` from `declgen` (mission + server ± HC)
- [ ] `.vscode/settings.json` → `sqfts.serverPath`
- [ ] Opened the mission folder (not only a nested addon) so config discovery works
- [ ] Saved a `.sqfts` file and confirmed the sibling `.sqf` (if `build_on_save`) and editor diagnostics

## Related docs

- [Configuration](Configuration) — full `sqfts.toml` field reference
- [Editor Support](Editor-Support) — extension features and install
- [Adoption Guide](Adoption-Guide) — incremental typing strategy
- [Declgen](Declgen) — declaration generation options
- [CLI Reference](CLI-Reference) — `check` / `build` / `declgen`
