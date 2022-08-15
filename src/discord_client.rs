use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use serenity::{model::prelude::GuildChannel, prelude::*};

use std::io;

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

struct App<'a> {
    server_list: StatefulList<ListItem<'a>>,
    channel_list: StatefulList<ListItem<'a>>,
    previous_server_index: u64,
}
impl<'a> App<'a> {
    fn default() -> App<'a> {
        App {
            server_list: StatefulList::with_items(vec![]),
            channel_list: StatefulList::with_items(vec![]),
            //random number
            previous_server_index: 305429,
        }
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

}

pub fn tui(ctx: Context) -> Result<(), io::Error> {
    //terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    //app is passed on to run_app()
    let mut app = App::default();
    let _guilds = ctx.cache.guilds();
    let mut guild_names = vec![];
    let mut guild_vec = vec![];
    let mut _guild_vec = vec![];
    for gid in _guilds.iter() {
        guild_vec.push(gid.0);
        _guild_vec.push((gid.0,ctx.cache.guild_field(gid, |g| g.name.clone())));
        
    }
    for (_, v) in _guild_vec.iter() {
        guild_names.push(ListItem::new(v.as_ref().unwrap().as_str()));
    }

    app.server_list = StatefulList::with_items(guild_names);
    run_app(&mut terminal, app, &guild_vec, ctx)?;

    //this part runs after the program has been quit, basically restoring the terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    guild_vec: &Vec<u64>,
    ctx: Context,
) -> io::Result<()> {
    app.server_list.next();
    loop {
        let index = &app.server_list.state.selected().unwrap_or(0);
        let index = *index as u64;
        if index != app.previous_server_index {
            let mut channel_id = 0;
            for k in guild_vec.into_iter() {
                if channel_id == index {
                    channel_id = *k;
                    break;
                }
                channel_id += 1;
            }

            let mut channel_vec = vec![];
            let channel_list = ctx.cache.guild_channels(channel_id).unwrap();
            for (_, v) in channel_list.into_iter() {
                channel_vec.push(ListItem::new(v.name));
            }
            let channel_list = ctx.cache.guild_channels(channel_id).unwrap();
            app.channel_list = StatefulList::with_items(channel_vec);
            let mut channels = channel_list
                .iter()
                .map(|channel| channel.clone())
                .collect::<Vec<GuildChannel>>();

            channels.sort_by_key(|c| c.position);
            let channels = channels
                .iter()
                .map(|channel| ListItem::new(channel.name.clone()))
                .collect::<Vec<ListItem>>();
            app.channel_list = StatefulList::with_items(channels);
        }
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
                KeyCode::Up => app.server_list.previous(),

                KeyCode::Down => app.server_list.next(),
                KeyCode::Left => app.channel_list.next(),
                KeyCode::Right => app.channel_list.previous(),
                _ => {}
            }
        }

        app.previous_server_index = index;
    }
}
fn ui<B: Backend>(f: &mut Frame<B>, app: &App, _ctx: &Context) {
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

    let block = Block::default().title("Discord").borders(Borders::all());
    f.render_widget(block, right_chunk[0]);

    let chatbox = Block::default().title("chatbox").borders(Borders::all());
    f.render_widget(chatbox, right_chunk[1]);
    let server_list = List::new(app.server_list.items.clone())
        .block(Block::default().title("Servers").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol("> ");
    f.render_stateful_widget(
        server_list,
        left_chunk[0],
        &mut app.server_list.state.clone(),
    );
    let channel_list = List::new(app.channel_list.items.clone())
        .block(Block::default().title("Channels").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol("> ");
    f.render_stateful_widget(
        channel_list,
        left_chunk[1],
        &mut app.channel_list.state.clone(),
    );
}
