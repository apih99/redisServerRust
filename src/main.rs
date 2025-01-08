use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use bytes::BytesMut;
use anyhow::Result;

mod resp;
mod command;
mod store;

use resp::RespType;
use command::Command;
use store::Store;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Redis server listening on port 6379");

    let store = Store::new();

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Client connected from: {}", addr);
        let store = store.clone();
        tokio::spawn(async move {
            if let Err(e) = process_connection(socket, store).await {
                eprintln!("Error processing connection: {}", e);
            }
        });
    }
}

async fn process_connection(socket: TcpStream, store: Store) -> Result<()> {
    let (reader, writer) = socket.into_split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        // Read data into the buffer
        match reader.read_buf(&mut buffer).await {
            Ok(0) => {
                // Connection closed normally
                println!("Client disconnected");
                break;
            }
            Ok(_) => {
                // Process all complete commands in the buffer
                while let Some(resp) = RespType::parse(&mut buffer)? {
                    if let Some(cmd) = Command::from_resp(resp) {
                        let response = cmd.execute(&store);
                        writer.write_all(&response.serialize()).await?;
                        writer.flush().await?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from socket: {}", e);
                break;
            }
        }
    }

    Ok(())
} 