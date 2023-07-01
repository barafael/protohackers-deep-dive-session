use std::{collections::BTreeMap, ops::Bound::Included};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    while let Ok((mut stream, _client)) = listener.accept().await {
        tokio::spawn(async move {
            let mut buf = [0u8; 9];
            let mut map = BTreeMap::<i32, i32>::new();
            while let Ok(_n) = stream.read_exact(&mut buf).await {
                if buf[0] == b'I' {
                    let time = i32::from_be_bytes(buf[1..=4].try_into().unwrap());
                    let price = i32::from_be_bytes(buf[5..=8].try_into().unwrap());
                    map.insert(time, price);
                } else if buf[0] == b'Q' {
                    let min_time = i32::from_be_bytes(buf[1..=4].try_into().unwrap());
                    let max_time = i32::from_be_bytes(buf[5..=8].try_into().unwrap());

                    if min_time > max_time {
                        stream.write_i32(0).await.unwrap();
                        continue;
                    }

                    let range_values = map
                        .range((Included(min_time), Included(max_time)))
                        .map(|(_time, price)| *price)
                        .collect::<Vec<i32>>();
                    let mean = if range_values.is_empty() {
                        0
                    } else {
                        range_values.iter().map(|n| *n as i64).sum::<i64>()
                            / range_values.len() as i64
                    };
                    stream.write_i32(mean as i32).await.unwrap();
                }
            }
        });
    }
    Ok(())
}
