use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{
    client,
    model::{channel::Message, guild},
    FutureExt,
};
use std::collections::HashMap;
use std::env;
use std::sync::mpsc;
use std::sync::Mutex;
use std::{io, sync::Arc};
use std::{thread, time};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};


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
            tui(ctx);
        });
    }
  
  
    async fn ready(&self, ctx: Context, ready: Ready) {
       
        println!("{} is connected!", ready.user.name);
    }
}
fn tui(ctx: Context) -> Result<(), io::Error> {
   
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::default();
    run_app(&mut terminal, app, ctx)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;
    Ok(())
}
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    ctx: Context,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app, &ctx))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('`') => {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('w') => {
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
fn ui<B: Backend>(f: &mut Frame<B>, _app: &App, ctx: &Context) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(50)].as_ref())
        .split(f.size());
    let left_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(100)].as_ref())
        .split(chunks[0]);
    let right_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(100)].as_ref())
        .split(chunks[1]);

    // let server_list = Block::default()
    //             .borders(Borders::all());
    // f.render_widget(server_list,left_chunk[0]);

    let channel_list = Block::default().title("title").borders(Borders::all());
    f.render_widget(channel_list, left_chunk[1]);

    let block = Block::default().title("Discord").borders(Borders::all());
    f.render_widget(block, right_chunk[0]);

    let chatbox = Block::default().title("chatbox").borders(Borders::all());
    f.render_widget(chatbox, right_chunk[1]);

    let mut guild_hm = HashMap::new();
    let guilds = ctx.cache.guilds();
    let mut guild_names = vec![];

    for gid in guilds.iter() {

        guild_hm.insert(gid.0, ctx.cache.guild_field(gid,|g| g.name.clone()));
       
    }
    for (k,v) in guild_hm.iter(){
    
        guild_names.push(ListItem::new( v.as_ref().unwrap().as_str()));
    };
   
    
    let items = List::new(guild_names)
        .block(Block::default().title("Servers").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");
    f.render_widget(items, left_chunk[0]);
}

struct App;
impl Default for App {
    fn default() -> App {
        App {}
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    };
}
