use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, self},
};
use std::{error::Error, io, alloc::LayoutErr};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::env;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::thread;
use std::time::Duration;
mod discord_client;

struct Handler;
#[async_trait]
impl EventHandler for Handler{
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
struct App;
impl Default for App {
    fn default() -> App {
        App {
           
        }
    }
}
fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    thread::spawn(||{
        bot();
    });
    let app = App::default();
    run_app(&mut terminal, app)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;
    Ok(())
}
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop{
        terminal.draw(|f|ui(f,&app))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(50),
        ].as_ref())
        .split(f.size());
    let left_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .margin(2)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(100),
            ].as_ref())
            .split(chunks[0]);
    let right_chunk = Layout::default()
    .direction(Direction::Vertical)
    .margin(2)
    .constraints([
        Constraint::Percentage(90),
        Constraint::Percentage(100),
    ].as_ref())
    .split(chunks[1]);

    let server_list = Block::default()
                .borders(Borders::all());
    f.render_widget(server_list,left_chunk[0]);


    let channel_list = Block::default()
                .title("title")
                .borders(Borders::all());
    f.render_widget(channel_list,left_chunk[1]);

    let block = Block::default()
    .title("Discord")
    .borders(Borders::all());
    f.render_widget(block, right_chunk[0]);
    let chatbox = Block::default()
    .title("chatbox")
    .borders(Borders::all());
    f.render_widget(chatbox, right_chunk[1]);
}


async fn bot() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
