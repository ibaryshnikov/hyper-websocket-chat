use anyhow::Result;
use hyper::upgrade::Upgraded;
use hyper::{Body, Request, Response, StatusCode};
use tokio::io::AsyncRead;
use uuid::Uuid;

use crate::shared::types::*;
use crate::ws::event::*;
use crate::ws::handshake::generate_key_from;
use crate::ws::opcode::Opcode;
use crate::ws::*;
use crate::ClientsRc;

async fn read_messages<T: AsyncRead + Unpin>(
    mut reader: T,
    sender: Sender,
    clients: ClientsRc,
    id: u128,
    short_id: String,
) -> Result<()> {
    loop {
        let frame = read_frame(&mut reader).await?;
        match frame.opcode {
            Opcode::Text => match String::from_utf8(frame.payload) {
                Ok(msg) => {
                    println!("got message: {}", msg);
                    let reply = format!("{}: {}", short_id, msg);
                    sender.send(text_event(reply, EventAddress::All))?;
                }
                Err(e) => println!("error parsing a string: {}", e),
            },
            Opcode::Close => {
                println!("Got CLOSE opcode, closing connection");
                if let Some(writer) = clients.borrow_mut().get_mut(&id) {
                    send_directly(writer, id, EventKind::Close, &frame.payload).await?;
                    println!("CLOSE reply sent");
                } else {
                    return Err(anyhow!(
                        "Can't find own connection to send CLOSE frame for {}",
                        id
                    ));
                }

                let msg = format!("{} leaving the server", short_id);
                sender.send(text_event(msg, EventAddress::All))?;
                println!("Users in chat notified");
                return Ok(());
            }
            code => {
                println!("Unsupported opcode: {:?}", code);
                return Err(anyhow!("Unsupported opcode {:?}", code));
            }
        }
    }
}

fn text_event(data: String, to: EventAddress) -> Event {
    Event {
        kind: EventKind::Text,
        address: to,
        payload: data.into_bytes(),
    }
}

async fn handle_upgraded_connection(
    upgraded: Upgraded,
    sender: Sender,
    clients: ClientsRc,
    id: u128,
) -> Result<()> {
    let (reader, mut writer) = tokio::io::split(upgraded);

    send_directly(&mut writer, id, EventKind::Text, b"Welcome to chat server!").await?;

    clients.borrow_mut().insert(id, writer);

    let short_id = format!("{:#x}", id)[2..10].to_owned();

    let msg = format!("{} joined the server", short_id);
    sender.send(text_event(msg, EventAddress::All))?;

    read_messages(reader, sender, clients, id, short_id).await?;

    Ok(())
}

pub fn handle_ws(mut req: Request<Body>, sender: Sender, clients: ClientsRc) -> Response<Body> {
    println!("ws incoming connection");
    let sec_key = req.headers().get("sec-websocket-key").unwrap();

    let sec_accept = generate_key_from(sec_key.as_bytes());

    tokio::task::spawn_local(async move {
        match hyper::upgrade::on(&mut req).await {
            Ok(upgraded) => {
                println!("upgraded");
                let id = Uuid::new_v4().to_u128_le();
                if let Err(e) =
                    handle_upgraded_connection(upgraded, sender, clients.clone(), id).await
                {
                    println!("error handling upgraded connection: {}", e);
                } else {
                    println!("closing upgraded connection")
                }
                clients.borrow_mut().remove(&id);
            }
            Err(e) => println!("upgrade error: {}", e),
        }
    });

    Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header("access-control-allow-origin", "*")
        .header("upgrade", "websocket")
        .header("connection", "upgrade")
        .header("sec-websocket-accept", sec_accept)
        .body(Body::empty())
        .unwrap()
}
