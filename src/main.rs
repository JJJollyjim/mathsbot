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

mod maths_render;
mod parser;

use serenity::prelude::*;
use std::env;
use maths_render::*;
use parser::*;


fn main() {
    let _ = env_logger::init();

    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"));
    client.with_framework(|f| {
        debug!("configuring bot");
        f.configure(|c| c.prefix("!")) // set the bot's prefix to "!"
            .on("ping", ping)
    });

    client.on_message(|_ctx, message| {
        let content = message.content_safe();

        match parser::classify_message(&content) {
            MessageType::LaTeX => {
                info!("message {:?} from {} containing LaTeX",
                      message.id, message.author.tag());

                if message.author.bot {
                    warn!("ignoring {:?} since author is a bot",
                          message.id);
                    return;
                }

                match render_maths(&content) {
                    Ok(image) => {
                        message.channel_id
                            .send_files(vec![(image.as_slice(), "maths.png")], |m| m)
                            .unwrap();

                        info!("image sent");
                    }
                    Err(e) => handle_error(e, message)
                }

            }
            MessageType::Plain => debug!("message {:?} ignored since it is plain", message.id)
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
