import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

function resolveServerPath(context: vscode.ExtensionContext): string {
  const configured = vscode.workspace
    .getConfiguration("sqfts")
    .get<string>("serverPath");
  if (configured && configured.trim().length > 0 && fs.existsSync(configured)) {
    return configured;
  }

  const exe =
    process.platform === "win32"
      ? "sqfts-language-server.exe"
      : "sqfts-language-server";
  const bundled = path.join(context.extensionPath, "server", exe);
  if (fs.existsSync(bundled)) {
    return bundled;
  }

  // Fall back to PATH
  return exe.replace(/\.exe$/, "");
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const serverPath = resolveServerPath(context);

  const serverOptions: ServerOptions = {
    run: { command: serverPath, transport: TransportKind.stdio },
    debug: { command: serverPath, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "sqfts" }],
    synchronize: {
      fileEvents: [
        vscode.workspace.createFileSystemWatcher("**/*.d.sqfts"),
        vscode.workspace.createFileSystemWatcher("**/sqfts.toml"),
      ],
    },
  };

  client = new LanguageClient(
    "sqfts",
    "SQFts Language Server",
    serverOptions,
    clientOptions
  );

  context.subscriptions.push(client);
  await client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
