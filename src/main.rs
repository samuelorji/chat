use futures::{SinkExt, StreamExt};
use std::{error::Error, ops::Index};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpSocket, TcpStream},
    sync::broadcast::{self, Sender},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

type Rez = Result<(), Box<dyn Error>>;
type RezSend = Result<(), Box<dyn Error + Send>>;

const HELP_MSG: &str = include_str!("help.txt");

#[tokio::main]
async fn main() -> Rez {
    let server = TcpListener::bind("127.0.0.1:3000").await?;

    // create broadcast channel
    let (tx, _) = broadcast::channel::<String>(32);

    loop {
        let (mut connection, _) = server.accept().await?;
        tokio::spawn(process_connection(connection, tx.clone()));
    }

    Ok(())
}

async fn process_connection(
    mut conn: TcpStream,
    broadcast_sender: Sender<String>,
) -> anyhow::Result<()> {
    let (read, write) = conn.split();

    let mut reader = FramedRead::new(read, LinesCodec::new());
    let mut writer = FramedWrite::new(write, LinesCodec::new());
    let mut broadcast_receiver = broadcast_sender.subscribe();

    writer.send(HELP_MSG).await?;
    loop {
        // keep polling between reading user input snd checking the receiver
        tokio::select! {
            user_msg = reader.next() => {

            if let Some(Ok(mut msg)) = user_msg {
             match msg.as_str() {
                r"\help" => {
                    writer.send(HELP_MSG).await?;
                }
                r"\quit" => {
                    writer.send("Goodbye!!").await?;
                    break;
                }
                _ => {
                    msg.push_str(" ❤️!");
                    broadcast_sender.send(msg)?;

                }
            }
        }
        }
            peer_msg = broadcast_receiver.recv() => {
                if let Ok(msg) = peer_msg {
                    writer.send(msg).await?
                }
            }
        }
    }

    Ok(())
}
