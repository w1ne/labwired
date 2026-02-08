// LabWired - Firmware Simulation Platform
// Copyright (C) 2026 Andrii Shylenko
//
// This software is released under the MIT License.
// See the LICENSE file in the project root for full license information.

use labwired_dap::server::DapServer;
use std::io;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut server = DapServer::new();
    server.run(stdin.lock(), stdout.lock())?;

    Ok(())
}
