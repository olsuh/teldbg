use std::os::unix::prelude::AsRawFd;

use tokio::codec::Decoder;
use tokio::net::TcpListener;
use tokio::prelude::*;

use telnet_codec::codec::TelnetCodec;
use telnet_codec::event::TelnetEvent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:7000").await?;
    println!("listening on port 7000");

    loop {
        let (socket, a) = listener.accept().await?;
        println!("{socket:?} {a}");

        tokio::spawn(async move {
            let fd = socket.as_raw_fd();
            let peer = ""; //socket.peer_addr().unwrap();
            let codec = TelnetCodec::new(4096_usize);
            let (mut sink, mut input) = codec.framed(socket).split();

            println!(" new thread {fd} {peer}");

            while let Some(Ok(event)) = input.next().await {
                println!("{fd} Event {:?}", event);
                match event {
                    TelnetEvent::Message(value) => {
                        println!("{fd} Echoing message: {}", value);
                        match sink.send(TelnetEvent::Message(value)).await {
                            Ok(_x) => {}
                            Err(e) => println!("An error occured {:?}", e),
                        }
                    }
                    TelnetEvent::Do(_o) => println!("Do {:?}", _o),
                    TelnetEvent::Dont(_o) => println!("Dont {:?}", _o),
                    TelnetEvent::Will(_o) => println!("Will {:?}", _o),
                    TelnetEvent::Wont(_o) => println!("Wont {:?}", _o),
                    TelnetEvent::Subnegotiation(_o) => println!("Subnegotiation {:?}", _o),
                    TelnetEvent::Character(_o) => println!("Character {:?}", _o),
                    TelnetEvent::EraseCharacter => println!("EraseCharacter"),
                    TelnetEvent::EraseLine => println!("EraseLine"),
                    TelnetEvent::Nop => println!("Nop"),
                }
            }
            println!(" exit from thread {fd} {peer}");
        });
    }
}
