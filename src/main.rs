use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::{channel::Message, guild};
use serenity::prelude::*;
use std::env;
use std::{io, sync::Arc};
use std::{thread, time};

mod discord_client;

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn cache_ready(
        &self,
        ctx: serenity::prelude::Context,
        vec: Vec<serenity::model::id::GuildId>,
    ) {
        thread::spawn(move || {
            discord_client::tui(ctx);
        });
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    };
}
