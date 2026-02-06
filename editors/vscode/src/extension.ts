import * as vscode from 'vscode';
import * as path from 'path';

export function activate(context: vscode.ExtensionContext) {
    const factory = new LabwiredConfigurationProvider();
    context.subscriptions.push(vscode.debug.registerDebugConfigurationProvider('labwired', factory));

    const adapterFactory = new LabwiredDebugAdapterDescriptorFactory();
    context.subscriptions.push(vscode.debug.registerDebugAdapterDescriptorFactory('labwired', adapterFactory));
}

export function deactivate() {
}

class LabwiredConfigurationProvider implements vscode.DebugConfigurationProvider {
    resolveDebugConfiguration(folder: vscode.WorkspaceFolder | undefined, config: vscode.DebugConfiguration, token?: vscode.CancellationToken): vscode.ProviderResult<vscode.DebugConfiguration> {
        if (!config.program) {
            return vscode.window.showInformationMessage("Cannot find a program to debug").then(_ => {
                return undefined;	// abort launch
            });
        }
        return config;
    }
}

class LabwiredDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(session: vscode.DebugSession, executable: vscode.DebugAdapterExecutable | undefined): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        // Point to the labwired-dap binary
        // Ideally this is bundled or configured via settings
        // For development, we assume cargo build was run and it's in target/debug

        // Find workspace root
        let root = session.workspaceFolder?.uri.fsPath || "";
        let command = "cargo";
        let args = ["run", "-p", "labwired-dap", "-q"]; // Use quiet mode!

        // Alternatively, use the built binary directly
        // let command = path.join(root, "target", "debug", "labwired-dap");
        // let args: string[] = [];

        // For robustness in this environment, let's use the binary path if it exists, otherwise cargo run?
        // But cargo run might rebuild.
        // Let's assume the user has built it.
        const dapPath = path.join(root, "target", "debug", "labwired-dap");

        return new vscode.DebugAdapterExecutable(dapPath, []);
    }
}
