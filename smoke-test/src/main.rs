use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use anyhow::Result;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;

    loop {
        // Accept a new TCP connection
        let (tcp_stream, address) = listener.accept().await?;

        // Spawn a task (multi-threaded) to handle the connection
        // we do not care at this point what happens to the task.
        tokio::spawn(async move {
            println!("Handling client connection from: {address}");
            handle_client(tcp_stream).await.ok();
        });
    }
}

async fn handle_client(mut tcp_stream: TcpStream) -> Result<()> {
    let mut buffer = [0_u8; 1024];
    loop {
        let result = tcp_stream.read(&mut buffer).await?;
        if result == 0 {
            break;
        }
        println!("{:?}", &buffer[..result]);
        tcp_stream.write_all(&buffer[..result]).await?;
    }

    Ok(())
}
