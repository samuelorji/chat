use futures::{SinkExt, StreamExt, stream::Any};
use std::{
    collections::HashSet, error::Error, fmt::format, net::SocketAddr, ops::Index, sync::Arc,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpSocket, TcpStream, tcp::WriteHalf},
    sync::{
        Mutex,
        broadcast::{self, Sender},
    },
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};

const HELP_MSG: &str = include_str!("help.txt");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:3000").await?;

    let mut names = Arc::new(Mutex::new(HashSet::<String>::new()));

    // create broadcast channel
    let (tx, _) = broadcast::channel::<String>(32);

    loop {
        let (connection, a) = server.accept().await?;
        let namez = names.clone();
        tokio::spawn(process_connection(connection, a, tx.clone(), namez));
    }

    Ok(())
}

async fn process_connection(
    mut conn: TcpStream,
    socket_address: SocketAddr,
    broadcast_sender: Sender<String>,
    names: Arc<Mutex<HashSet<String>>>,
) -> anyhow::Result<()> {
    let (read, write) = conn.split();

    let mut reader = FramedRead::new(read, LinesCodec::new());
    let mut writer: FramedWrite<WriteHalf<'_>, LinesCodec> =
        FramedWrite::new(write, LinesCodec::new());
    let mut broadcast_receiver = broadcast_sender.subscribe();

    let mut name = format!(
        "{}_{}",
        socket_address.ip().to_string(),
        socket_address.port()
    );

    async fn write_to_connection_with_name(
        msg: &str,
        name: &str,
        writer: &mut FramedWrite<WriteHalf<'_>, LinesCodec>,
    ) -> anyhow::Result<(), LinesCodecError> {
        writer.send(format!("{}: {}", &name, msg)).await
    }

    writer
        .send(format!("You are {}\n{}", &name, HELP_MSG))
        .await?;

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

                    if msg.starts_with("/name ") {
                        if let Some(user_name) =  msg.strip_prefix("/name ") {
                            {
                                let mut names_set = names.lock().await;
                                let not_exists = names_set.insert(user_name.to_string());
                                drop(names_set);
                                if not_exists {
                                    let new_user_name = user_name.to_string();
                                   let message = format!("{} is now {}",&name, &new_user_name);
                                    broadcast_sender.send(message)?;
                                    name = new_user_name

                                } else {
                                    writer.send(format!("name {} already exists", user_name)).await?;
                                }

                             }

                            }

                    } else {
                        msg.push_str(" ❤️!");
                        broadcast_sender.send(format!("{}: {}", &name,msg))?;
                    }

                }
            }
                                        }
        }
            peer_msg = broadcast_receiver.recv() => {
                if let Ok(msg) = peer_msg {
                     writer.send(msg).await?
                      //write_to_connection_with_name(&msg, &name, &mut writer).await?;
                }
            }
        }
    }

    Ok(())
}
