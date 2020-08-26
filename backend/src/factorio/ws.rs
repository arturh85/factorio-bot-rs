use crate::types::FactorioPlayer;
use actix::prelude::*;
use actix_web_actors::ws;
use actix_web_actors::ws::ProtocolError;
use std::time::Duration;

pub struct FactorioWebSocketClient {}

impl Actor for FactorioWebSocketClient {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ProtocolError>> for FactorioWebSocketClient {
    fn handle(&mut self, _result: Result<ws::Message, ProtocolError>, _ctx: &mut Self::Context) {
        // match result {
        //     Ok(msg) => {
        //         println!("WS: {:?}", msg);
        //         match msg {
        //             Message::Ping(msg) => {
        //                 ctx.pong(&msg);
        //             }
        //             Message::Pong(_) => {}
        //             Message::Text(text) => ctx.text(text),
        //             Message::Binary(bin) => ctx.binary(bin),
        //             Message::Close(_) => {
        //                 ctx.stop();
        //             }
        //             Message::Nop => (),
        //             Message::Continuation(_) => {}
        //         }
        //     }
        //     Err(err) => panic!("error"),
        // }
    }
}

impl Handler<ServerEvent> for FactorioWebSocketClient {
    type Result = ();

    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) {
        ctx.text(msg.event);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterWSClient {
    pub addr: Addr<FactorioWebSocketClient>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct ServerEvent {
    event: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerChangedEvent {
    pub player: FactorioPlayer,
}

pub struct FactorioWebSocketServer {
    pub listeners: Vec<Addr<FactorioWebSocketClient>>,
}

impl Actor for FactorioWebSocketServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(60), |act, _| {
            for l in &act.listeners {
                l.do_send(ServerEvent {
                    event: String::from("Heartbeat"),
                });
            }
        });
    }
}

impl Handler<RegisterWSClient> for FactorioWebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: RegisterWSClient, _: &mut Context<Self>) {
        self.listeners.push(msg.addr);
    }
}

impl Handler<PlayerChangedEvent> for FactorioWebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: PlayerChangedEvent, _: &mut Context<Self>) {
        for l in &self.listeners {
            l.do_send(ServerEvent {
                event: serde_json::to_string(&msg.player).unwrap(),
            });
        }
    }
}
