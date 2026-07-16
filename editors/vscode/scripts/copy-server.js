const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "../../..");
const destDir = path.resolve(__dirname, "../server");
fs.mkdirSync(destDir, { recursive: true });

const isWin = process.platform === "win32";
const exe = isWin ? "sqfts-language-server.exe" : "sqfts-language-server";
const src = path.join(root, "target", "release", exe);

if (!fs.existsSync(src)) {
  console.error(`Missing ${src}. Run: cargo build --release -p sqfts-lsp`);
  process.exit(1);
}
fs.copyFileSync(src, path.join(destDir, exe));
console.log(`Copied ${exe} -> editors/vscode/server/`);
