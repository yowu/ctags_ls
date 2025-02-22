mod ctags;
mod goto_handler;
mod initialize_handler;
mod logger;
mod server;
mod document;
mod workspace;

use logger::Logger;
use lsp_server::Connection;
use server::LspServer;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    Logger::setup()?;

    Logger::info("Starting LSP server...");

    let (connection, io_threads) = Connection::stdio();

    let server = LspServer::new(connection);
    server.run()?;

    io_threads.join().unwrap();
    Ok(())
}
