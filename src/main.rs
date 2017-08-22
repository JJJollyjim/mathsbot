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
use serenity::model::{MessageId, ChannelId, Message};
use serenity::client::{Context};
use std::env;
use std::collections::HashMap;
use maths_render::*;
use parser::*;

#[derive(Debug, PartialEq, Eq, Hash)]
struct QualifiedMessageId {
    channel_id: ChannelId,
    message_id: MessageId
}

impl<'a> From<&'a Message> for QualifiedMessageId {
    fn from(message: &'a Message) -> QualifiedMessageId {
        QualifiedMessageId {channel_id:  message.channel_id, message_id:  message.id}
   }
}

impl From<QualifiedMessageId> for Result<Message, serenity::Error> {
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

    let mut typemap = ctx.data.lock().unwrap();
    let mut hashmap = typemap.get_mut::<MessageHistory>().unwrap();

    if is_update {
        if let Some(response) = hashmap.get(&QualifiedMessageId::from(&message)) {
            response.edit
        }
    }

    match parser::classify_message(&content) {
        MessageType::LaTeX => {
            info!("message {:?} from {} containing LaTeX; update? {}",
                  message.id, message.author.tag(), is_update);

            if message.author.bot {
                warn!("ignoring {:?} since author is a bot",
                      message.id);
                return;
            }

            match render_maths(&content) {
                Ok(image) => {
                    let response = message.channel_id
                        .send_files(vec![(image.as_slice(), "maths.png")], |m| m)
                        .unwrap();

                    info!("image sent");


                    hashmap.insert(QualifiedMessageId::from(&message),
                                   QualifiedMessageId::from(&response));
                }
                Err(e) => handle_error(e, message)
            }

        }
        MessageType::Plain => debug!("message {:?} ignored since it is plain; update? {}", message.id, is_update)
    }

    debug!("{:?}", ctx.data.lock().unwrap().get::<MessageHistory>().unwrap());

}

fn main() {
    let _ = env_logger::init();

    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"));

    client.data.lock().unwrap().insert::<MessageHistory>(HashMap::default());

    client.with_framework(|f| {
        debug!("configuring framework");
        f.configure(|c| c.prefix("!")) // set the bot's prefix to "!"
            .on("ping", ping)
    });

    client.on_ready(|ctx, _| {
        ctx.set_game_name("LaTeX");
        info!("bot ready");
    });

    client.on_message(|ctx, message| {
        handle_message(ctx, message, false);
    });

    client.on_message_update(|ctx, update| {
        if let Ok(message) = update.channel_id.message(update.id) {
            handle_message(ctx, message, true);
        } else {
            warn!("failed to get the message that was updated (id {:?})", update.id);
        }
    });

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
