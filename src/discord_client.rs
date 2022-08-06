use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use serenity::prelude::*;
use std::collections::HashMap;
use std::{io, sync::Arc};
use std::{thread, time};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
    Frame, Terminal,
};

struct App<'a> {
    //(Server list, cursor pos,channels list state)
    //cursor pos can be 0 or 1 (0 is servers , 1 is channels )
    info: (StatefulList<ListItem<'a>>, u64,ListState),

}
impl<'a> App<'a> {
    fn default() -> App<'a> {
        App {
            info: (StatefulList::with_items(vec![]), 0,ListState::default()),
        }
    }
    fn set(&mut self, info: (StatefulList<ListItem<'a>>,)) {
        self.info = (info.0, 0,ListState::default());
    }
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
        //tttttt
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

pub fn tui(ctx: Context) -> Result<(), io::Error> {
    //terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    let mut guild_hm = HashMap::new();
    let guilds = ctx.cache.guilds();
    // this variable gets sent to app
    let mut guild_names = vec![];

    for gid in guilds.iter() {
        guild_hm.insert(gid.0, ctx.cache.guild_field(gid, |g| g.name.clone()));
    }
    for (k, v) in guild_hm.iter() {
        guild_names.push(ListItem::new(v.as_ref().unwrap().as_str()));
    }

    app.set((StatefulList::with_items(guild_names),));
    run_app(&mut terminal, app, &guild_hm, ctx)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    guild_hm: &HashMap<u64, Option<String>>,
    ctx: Context,
) -> io::Result<()> {
    app.info.0.next();
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
                KeyCode::Up => match app.info.1 {
                    0 => app.info.0.previous(),
                    _ => {}
                },
                KeyCode::Down => match app.info.1 {
                    0 => app.info.0.next(),
                    _ => {}
                },
                KeyCode::Left => {
                    app.info.1 = 0 
                },
                KeyCode::Right  => {
                    app.info.1 = 1 
                },
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

    //    let channel_list = Block::default().title("Channels").borders(Borders::all());
    //    f.render_widget(channel_list, left_chunk[1]);
    //
    let block = Block::default().title("Discord").borders(Borders::all());
    f.render_widget(block, right_chunk[0]);

    let chatbox = Block::default().title("chatbox").borders(Borders::all());
    f.render_widget(chatbox, right_chunk[1]);

    let mut guild_hm = HashMap::new();
    let guilds = ctx.cache.guilds();
    let mut guild_vec = vec![];

    for gid in guilds.iter() {
        guild_hm.insert(gid.0, ctx.cache.guild_field(gid, |g| g.name.clone()));
        guild_vec.push(gid.0);
    }
    let mut guild_id = vec![];

    for (k, v) in guild_hm.iter() {
        guild_id.push(ListItem::new(v.as_ref().unwrap().as_str()));
    }

    let items = &app.info.0.items;
    let items = List::new(items.clone())
        .block(Block::default().title("Servers").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol("> ");
    f.render_stateful_widget(items, left_chunk[0], &mut app.info.0.state.clone());
    // here items can be either the number or the actual id of the guild cuz im lazy
    let items = &app.info.0.state.selected().unwrap_or(0);
    let items = *items as u64;
    let mut index = 0;
    for (k) in guild_vec.into_iter() {
        if (index == items) {
            index = k;
            break;
        }
        index += 1;
    }
    let mut channel_list_state = ListState::default();

    let mut channel_vec = vec![];
    let channel_list = ctx.cache.guild_channels(index).unwrap();
    for (k, v) in channel_list.into_iter() {
        channel_vec.push(ListItem::new(v.name));
    }
    // let channel_list = ctx.cache.guild(index).unwrap().channels;
    // let channel_list = channel_list
    //     .values()
    //     .map(Clone::clone)
    //     .collect::<Vec<GuildChannel>>();
    // channels.sort_by_key(|c| c.position);
    let channel_list = List::new(channel_vec)
        .block(Block::default().title("Channels").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol("> ");
    f.render_stateful_widget(channel_list, left_chunk[1], &mut app.info.2.clone());
}
