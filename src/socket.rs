use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use rocket::{
    serde::{
        json::{self, serde_json, Value},
        Deserialize, Serialize,
    },
    tokio::{
        self,
        net::{TcpListener, TcpStream},
        sync::broadcast::{self, Receiver, Sender},
    },
};
use std::sync::Arc;
use tokio_tungstenite::accept_async;
use tungstenite::{Message, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub dat: Value,
}

impl Event {
    pub fn new(
        id: i32,
        name: impl ToString,
        dat: impl Serialize,
    ) -> Result<Self, serde_json::error::Error> {
        Ok(Self {
            id,
            name: name.to_string(),
            dat: json::to_value(dat)?,
        })
    }

    pub fn from(id: i32, value: &str) -> Result<Self, &str> {
        #[derive(Serialize, Deserialize, Debug)]
        #[serde(crate = "rocket::serde")]
        struct FromEvent {
            pub name: String,
            pub dat: Value,
        }

        let value: FromEvent = json::from_str(value).map_err(|_| "消息格式不正确")?;

        Ok(Self {
            id,
            name: value.name,
            dat: value.dat,
        })
    }
}

pub struct Socket {
    s: Sender<Event>,
}

impl Socket {
    pub async fn new(
        addr: &'static str,
        handler: impl Fn(Event) -> Option<Event> + Send + Sync + Copy + 'static,
    ) -> Self {
        let listener = TcpListener::bind(addr).await.expect("Can't listen");
        info!("WS Listening on: {}", addr);

        let (s, _) = broadcast::channel(20);
        let ps = s.clone();

        let id = Arc::new(Mutex::new(0));
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let mut id = id.lock();
                *id += 1;
                tokio::spawn(handle_connection(
                    stream,
                    ps.clone(),
                    ps.subscribe(),
                    *id,
                    handler,
                ));
            }
        });

        Self { s }
    }

    pub fn send(&self, event: Event) {
        self.s.send(event).unwrap_or_default();
    }
}

async fn handle_connection(
    stream: TcpStream,
    sender: Sender<Event>,
    mut receiver: Receiver<Event>,
    id: i32,
    handler: impl Fn(Event) -> Option<Event>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?.into_text()?;
                        match Event::from(id, &msg) {
                            Ok(event) => {
                                if let Some(event) = handler(event) {
                                    sender.send(event).unwrap();
                                }
                            },
                            Err(error) => {
                                sender.send(Event::new(id, "error", error).unwrap()).unwrap();
                            }
                        }
                    }
                    None => break,
                }
            },
            msg = receiver.recv() => {
                match msg {
                    Ok(msg) => {
                        if msg.id == id || msg.id == 0 {
                            ws_sender.send(Message::Text(json::to_string(&msg).unwrap())).await?;
                        }
                    }
                    Err(error) => {
                        error!("{}", error);
                    },
                }
            }
        }
    }

    Ok(())
}
