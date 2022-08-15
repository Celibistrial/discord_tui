use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::env;
use std::thread;

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
        _vec: Vec<serenity::model::id::GuildId>,
    ) {
        thread::spawn(move || {
            let err = discord_client::tui(ctx);
            if err.is_err(){
                println!("Unable to start TUI thread {:?}",err);
            }
        });
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    };
}
