#[macro_use]
extern crate serenity;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate tempdir;
extern crate regex;
extern crate libc;
extern crate env_logger;
extern crate typemap;

mod maths_render;
mod parser;

use serenity::prelude::*;
use serenity::model::{self, MessageId, ChannelId, Message, event};
use serenity::client::{Context};
use std::env;
use std::collections::HashMap;
use maths_render::*;
use parser::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct QualifiedMessageId {
    channel_id: ChannelId,
    message_id: MessageId
}

impl QualifiedMessageId {
    fn delete(self: &Self) -> serenity::Result<()> {
        self.channel_id.delete_message(self.message_id)
    }
}

impl<'a> From<&'a Message> for QualifiedMessageId {
    fn from(message: &'a Message) -> QualifiedMessageId {
        QualifiedMessageId {channel_id:  message.channel_id, message_id:  message.id}
    }
}

impl From<QualifiedMessageId> for serenity::Result<Message> {
    fn from(qual_id: QualifiedMessageId) -> Result<Message, serenity::Error> {
        qual_id.channel_id.message(qual_id.message_id)
    }
}

struct MessageHistory;

impl typemap::Key for MessageHistory {
    type Value = HashMap<QualifiedMessageId, QualifiedMessageId>;
}

fn handle_message(ctx: Context, message: Message, is_update: bool) {
    let content = message.content_safe();

    let mut typemap = ctx.data.lock();
    let history: &mut HashMap<QualifiedMessageId, QualifiedMessageId> = typemap.get_mut::<MessageHistory>().unwrap();

    let msg_to_delete = history.get(&QualifiedMessageId::from(&message)).cloned();

    if is_update && msg_to_delete.is_none() {
        warn!("Message {:?} was updated, but is not in the history map", message.id);
    }

    match parser::classify_message(&content) {
        MessageType::LaTeX => {
            info!("message {:?} from {} containing LaTeX; update? {}",
                  message.id, message.author.tag(), is_update);

            if message.author.bot {
                warn!("LaTeX ignoring {:?} since author is a bot",
                      message.id);
                return;
            }

            match render_maths(&content) {
                Ok(image) => {
                    if let Ok(response) = message.channel_id.send_files(vec![(image.as_slice(), "maths.png")], |m| m) {
                        info!("image sent");

                        history.insert(QualifiedMessageId::from(&message),
                                       QualifiedMessageId::from(&response));

                        if let Some(msg) = msg_to_delete {
                            if msg.delete().is_err() {
                                error!("Failed to delete message {:?} for some reason :thonking:", msg);
                            }
                        }

                        debug!("Reactions: {:?}", message.reactions);
                        for mr in message.reactions {
                            if mr.me {
                                debug!("deleting reaction {} from {:?}", mr.reaction_type, message.id);
                                let _ = message.channel_id.delete_reaction(
                                    message.id,
                                    None,
                                    mr.clone().reaction_type);
                            }
                        }
                    } else {
                        error!("couldn't send message in response to {:?} for some reason", message.id)
                    }

                }
                Err(e) => handle_error(e, message)
            }

        }
        MessageType::Plain => {
            debug!("message {:?} ignored since it is plain; update? {}", message.id, is_update);

            if let Some(msg) = msg_to_delete {
                if msg.delete().is_err() {
                    error!("Failed to delete message {:?} for some reason :thonking:", msg);
                }
            }

            debug!("Reactions: {:?}", message.reactions);
            for mr in message.reactions {
                if mr.me {
                    debug!("deleting reaction {} from {:?}", mr.reaction_type, message.id);
                    let _ = message.channel_id.delete_reaction(
                        message.id,
                        None,
                        mr.clone().reaction_type);
                }
            }

        }
    }


    debug!("{:?}", history);

}

struct Handler;

impl EventHandler for Handler {
    fn on_ready(&self, ctx: Context, _: model::Ready) {
        ctx.set_game_name("LaTeX");
        info!("bot ready");
    }

    fn on_message(&self, ctx: Context, message: Message) {
        handle_message(ctx, message, false);
    }

    fn on_message_update(&self, ctx: Context, update: event::MessageUpdateEvent) {
        if let Ok(message) = update.channel_id.message(update.id) {
            handle_message(ctx, message, true);
        } else {
            warn!("failed to get the message that was updated (id {:?})", update.id);
        }
    }
}

fn main() {
    let _ = env_logger::init();

    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler);

    client.data.lock().insert::<MessageHistory>(HashMap::default());

    // start listening for events by starting a single shard
    let _ = client.start();
    info!("bot started");
}

command!(ping(_context, message) {
    info!("ping from {}", message.author.tag());
    let _ = message.reply("Pong!");
});

fn handle_error(e: MathsRenderError, msg: serenity::model::Message) {
    if let MathsRenderError::LatexError(status, output) = e {
        warn!("LatexError occoured: {:?}, {:?}", status, output);

        msg.react('⚠').unwrap();

        let minimised = minimise_latex_error(&output);
        let content = format!("⚠ **An error occurred rendering your LaTeX** ⚠\n\n```tex\n{}\n```\nContact <@114301691804909569> for help 😉", minimised);
        let _ = msg.author.direct_message(|m| m.content(&content));
    } else {
        error!("Unexpected error: {:?}", e);

        let _ = msg.react('🤖');
        let _ = msg.react('☠');
        let _ = msg.author.direct_message(|m| m.content("🤖☠ **An unexpected error occurred displaying your LaTeX** ☠🤖\n\nThis may not be your fault – Contact <@114301691804909569> for help 🙂"));
    }
}

fn minimise_latex_error(output: &str) -> String {
    match output.find('!') {
        Some(idx) => output.chars().skip(idx).collect::<String>(),
        None => output.into()
    }
}
