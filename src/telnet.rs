use std::net::SocketAddr;

use nectar::{event::TelnetEvent, TelnetCodec};
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::{Result, connection::Connection, command::command};

pub async fn telnet_connection_loop(
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<()> {
    let frame = Framed::new(stream, TelnetCodec::new(1024));
    let s = format!("\nHi, {:?}", &frame);
    let mut conn = Connection::new(addr, frame);

    //println!("{conn}");
    
    conn.send_message(&s).await?;
    conn.send_message("\nit's teldbg\n>").await?;

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
