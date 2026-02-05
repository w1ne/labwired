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
