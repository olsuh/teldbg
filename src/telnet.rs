use std::{net::SocketAddr, os::unix::prelude::AsRawFd};

use futures::StreamExt;
use nectar::{event::TelnetEvent, TelnetCodec};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::{command::command, connection::Connection, Result};

pub async fn telnet_connection_loop(stream: TcpStream, addr: SocketAddr) -> Result<()> {
    let frame = Framed::new(stream, TelnetCodec::new(1024));
    
    let tcp = frame.get_ref();
    let s = format!("teldbg: {}, peer: {}, fd: {}, {:?}\r\n>", tcp.local_addr().unwrap(), tcp.peer_addr().unwrap(), tcp.as_raw_fd(), frame.codec());
    
    let mut conn = Connection::new(addr, frame);

    conn.send_message(&s).await?;

    // Display the logo.
    /*  let logo = tokio::fs::read_to_string("logo.txt").await;
    if let Ok(logo) = logo {
        conn.send_message(&logo).await?;
    }*/

    // Connection initialization. Log a player in (or create an account).
    /*let maybe_player = authenticate(&mut conn, pg.clone()).await?;
    if maybe_player.is_none() {
        tracing::info!("Authentication failed.");
        return Ok(());
    }*/

    loop {
        tokio::select! {
            // Handles messages received from the peer (via telnet)
            result = conn.frame_mut().next() => match result {
                Some(Ok(msg)) => {
                    tracing::trace!("Received message: {:?}", msg);

                    match msg {
                        TelnetEvent::Message(mut msg) => {

                            if msg.trim().is_empty() {
                                //tx_broker.send(Event::Client(id, ClientEvent::Ping))?;
                                continue;
                            }
                            //tx_broker.send(Event::Client(id, ClientEvent::Command(Input::from(msg))))?;

                            let _x = command(&mut msg, &mut conn).await?;
                        }
                        _ => continue,
                    }


                },
                Some(Err(e)) => {
                    tracing::error!(%e, "Error reading from connection: {}", addr);
                    break;
                }
                None => {
                    tracing::info!("Telnet connection closed: {}", addr);
                    break;
                }
            }
        }
    }

    conn.send_message("\nGoodbye!\n").await?;

    Ok(())
}
