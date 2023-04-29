use std::str::FromStr;

use primal::is_prime;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

use anyhow::{anyhow, Context, Result};

const METHOD: &str = "isPrime";

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

#[derive(Debug, Clone, Deserialize)]
struct PrimeRequest {
    method: String,
    number: serde_json::Number,
}

#[derive(Debug, Clone, Serialize)]
struct PrimeResponse {
    method: String,
    prime: bool,
}

async fn handle_client(mut tcp_stream: TcpStream) -> Result<()> {
    let (rdr, mut wtr) = tcp_stream.split();
    let mut reader = BufReader::new(rdr);

    let mut data = String::with_capacity(1024);
    loop {
        let bytes_read = reader.read_line(&mut data).await?;
        if bytes_read == 0 {
            break;
        }

        let Ok(request) = data.parse() else {
            wtr.write_all(b"Invalid request").await?;
            break;
        };
        let response = handle_prime_request(&request);
        let json = serde_json::to_string(&response)?;
        wtr.write_all(json.as_bytes()).await?;
        wtr.write_u8(b'\n').await?;
        data.clear();
    }

    Ok(())
}

impl FromStr for PrimeRequest {
    type Err = anyhow::Error;

    /// handle a prime number request
    ///
    /// returns `Ok(PrimeResponse)` if everything is fine.
    ///
    /// # Errors
    /// If there is a network issue then return Err.
    /// If there is a malformed request, then return Ok(None)
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let request: PrimeRequest = serde_json::from_str(s).context("your data is bad")?;
        match request.method.as_str() {
            METHOD => Ok(request),
            _ => Err(anyhow!("Invalid method")),
        }
    }
}

fn handle_prime_request(request: &PrimeRequest) -> PrimeResponse {
    if let Some(n) = request.number.as_u64() {
        PrimeResponse {
            method: METHOD.to_string(),
            prime: is_prime(n),
        }
    } else {
        PrimeResponse {
            method: METHOD.to_string(),
            prime: false,
        }
    }
}
