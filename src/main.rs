#[macro_use]
extern crate serenity;
extern crate tempdir;
extern crate regex;
extern crate libc;
#[macro_use]
extern crate lazy_static;

mod maths_render;
mod parser;

use serenity::prelude::*;
use std::env;
use maths_render::*;



fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"));
    client.with_framework(|f| {
        f.configure(|c| c.prefix("!")) // set the bot's prefix to "!"
            .on("ping", ping)
    });

    client.on_message(|_ctx, message| {
        let content = message.content_safe();
        let fragments = parser::find_maths_fragments(&content);
        for fragment in fragments {
            let image = render_maths(fragment).unwrap();
            message
                .channel_id
                .send_files(vec![(image.as_slice(), "maths.png")], |m| m)
                .unwrap();
        }
    });

    // start listening for events by starting a single shard
    let _ = client.start();
}

command!(ping(_context, message) {
    let _ = message.reply("Pong!");
});
