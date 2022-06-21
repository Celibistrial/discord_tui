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
    widgets::{Block, Borders, List, ListItem, ListState,StatefulWidget},
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

    let mut app = App::default();
    let mut guild_hm = HashMap::new();
    let guilds = ctx.cache.guilds();
    let mut guild_names = vec![];

    for gid in guilds.iter() {

        guild_hm.insert(gid.0, ctx.cache.guild_field(gid,|g| g.name.clone()));
       
    }
    for (k,v) in guild_hm.iter(){
    
        guild_names.push(ListItem::new( v.as_ref().unwrap().as_str()));
    };
    app.set(guild_names);
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
              
                KeyCode::Left => {
                    app.items.unselect();
                    
                }
                KeyCode::Up => {
                    app.items.previous();
                    
                }
                KeyCode::Down => {
                    app.items.next();
                    
                }
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
fn ui<B: Backend>(f: &mut Frame<B>, app: &App, ctx: &Context) {
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
 

    for gid in guilds.iter() {

        guild_hm.insert(gid.0, ctx.cache.guild_field(gid,|g| g.name.clone()));
       
    }
   
   
   
   
    let items = &app.items.items;

    let items = List::new(items.clone())
        .block(Block::default().title("Servers").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol("> ");
   // f.render_widget(items, left_chunk[0]);
    f.render_stateful_widget(items, left_chunk[0], &mut app.items.state.clone());
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}
impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App<'a>{
    items: StatefulList<ListItem<'a>>
}
impl<'a> App<'a> {
    fn default() -> App<'a> {
        App {
            items: StatefulList::with_items(vec![])
        }
    }
    fn set(&mut self,items: Vec<ListItem<'a>>){
        self.items = StatefulList::with_items(items);
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
