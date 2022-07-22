use std::{net::SocketAddr};

use anyhow::{anyhow, Result};
use s2n_quic::{Client, client::Connect, Connection, Server};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let socket : SocketAddr = args[2].parse()?;

    match args[1].as_ref() {
        "server" => server(socket).await,
        "client" => client(socket).await,
        _ => Err(anyhow!("First arguments needs to be server or client"))
    }
}

// Client will connect to the `socket` address and send messages of increasing size.
// For each message it waits for the response from the server.
//
// Messages are framed by 4 byte message length.
async fn client(socket: SocketAddr) -> Result<()> {
    let client = Client::builder()
        .with_tls(include_str!("../certs/root.pem"))?
        .with_io("0.0.0.0:0")?
        .start()
        .map_err(|_| anyhow::anyhow!("Error building QUIC client"))?;

    let connect = Connect::new(socket).with_server_name("server");
    let mut connection : Connection = client.connect(connect).await?;
    let stream = connection.open_bidirectional_stream().await?;
    let (mut recv, mut send) = stream.split();
    for i in 0..16 {
        let msg = vec![0; (2usize).pow(i)];
        let size = (msg.len() as u32).to_le_bytes();
        println!("Message {i}: {} bytes", msg.len());
        send.write_all(&size).await?;
        send.write_all(&msg).await?;

        let mut size = [0u8; 4];
        recv.read_exact(&mut size).await?;
        let size = u32::from_le_bytes(size);
        println!("Reading msg with {} bytes", size);
        let mut buffer = vec![0u8; size as usize];
        recv.read_exact(&mut buffer).await?;
        println!("Received msg with {} bytes", buffer.len());
    }
    Ok(())
}

// Server accepts connections to `socket` and spawn a new Tokio task to handle each connection.
// Messages are read and a fixed 4 byte response is sent back.
async fn server(socket: SocketAddr) -> Result<()> {
    let cert = include_str!("../certs/server.pem");
    let key = include_str!("../certs/server.pk.pem");
    let mut server = Server::builder()
        .with_tls((cert, key))?
        .with_io(socket)?
        .start()
        .map_err(|_| anyhow::format_err!("Error building QUIC server"))?;

    while let Some(connection) = server.accept().await {
        tokio::spawn(handle_conn_wrapper(connection));
    }
    Ok(())
}

async fn handle_conn_wrapper(connection: Connection) {
    if let Err(e) = handle_conn(connection).await {
        println!("Error handling connection {e}");
    }
}

async fn handle_conn(mut connection: Connection) -> Result<()> {
    let stream = connection.accept_bidirectional_stream().await?.unwrap();
    let (mut recv, mut send) = stream.split();

    loop {
        let mut size = [0u8; 4];
        recv.read_exact(&mut size).await?;
        let size = u32::from_le_bytes(size);
        println!("Reading msg with {} bytes", size);
        let mut buffer = vec![0u8; size as usize];
        recv.read_exact(&mut buffer).await?;
        println!("Recv msg with {} bytes", buffer.len());

        let msg: Vec<u8>= vec![1,2,3,4];
        let size = (msg.len() as u32).to_le_bytes();
        send.write_all(&size).await?;
        send.write_all(&msg).await?;
    }
}