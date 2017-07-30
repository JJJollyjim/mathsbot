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


fn main() {
    env_logger::init();

    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"));
    client.with_framework(|f| {
        debug!("configuring bot");
        f.configure(|c| c.prefix("!")) // set the bot's prefix to "!"
            .on("ping", ping)
    });

    client.on_message(|_ctx, message| {
        let content = message.content_safe();
        let fragments = parser::find_maths_fragments(&content);
        if fragments.len() != 0 {
            info!("message {:?} from {}#{} with {} fragments", message.id, message.author.name, message.author.discriminator, fragments.len());

            if message.author.bot {
                info!("ignoring {:?} since author is a bot", message.id);
                return;
            }

            for fragment in fragments {
                let image = render_maths(fragment).unwrap();
                message
                    .channel_id
                    .send_files(vec![(image.as_slice(), "maths.png")], |m| m)
                    .unwrap();

                info!("image sent");
            }
        }
    });

    // start listening for events by starting a single shard
    let _ = client.start();
    info!("bot started");
}

command!(ping(_context, message) {
    info!("ping from {}#{}", message.author.name, message.author.discriminator);
    let _ = message.reply("Pong!");
});
