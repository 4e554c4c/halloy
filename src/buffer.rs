pub use data::buffer::Settings;
use data::user::Nick;
use data::{buffer, file_transfer, history, message, Config};
use iced::Task;

pub use self::channel::Channel;
pub use self::file_transfers::FileTransfers;
pub use self::highlights::Highlights;
pub use self::logs::Logs;
pub use self::query::Query;
pub use self::server::Server;
use crate::screen::dashboard::sidebar;
use crate::widget::Element;
use crate::Theme;

pub mod channel;
pub mod empty;
pub mod file_transfers;
pub mod highlights;
mod input_view;
pub mod logs;
pub mod query;
mod scroll_view;
pub mod server;
pub mod user_context;

#[derive(Clone)]
pub enum Buffer {
    Empty,
    Channel(Channel),
    Server(Server),
    Query(Query),
    FileTransfers(FileTransfers),
    Logs(Logs),
    Highlights(Highlights),
}

#[derive(Debug, Clone)]
pub enum Message {
    Channel(channel::Message),
    Server(server::Message),
    Query(query::Message),
    FileTransfers(file_transfers::Message),
    Logs(logs::Message),
    Highlights(highlights::Message),
}

pub enum Event {
    UserContext(user_context::Event),
    OpenChannel(String),
    GoToMessage(data::Server, String, message::Hash),
    History(Task<history::manager::Message>),
}

impl Buffer {
    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn server(&self) -> Option<&data::Server> {
        match self {
            Buffer::Empty | Buffer::FileTransfers(_) => None,
            Buffer::Channel(state) => Some(&state.server),
            Buffer::Server(state) => Some(&state.server),
            Buffer::Query(state) => Some(&state.server),
            Buffer::Logs(_) => Some(&data::server::LOGS),
            Buffer::Highlights(_) => Some(&data::server::HIGHLIGHTS),
        }
    }

    pub fn data(&self) -> Option<&data::Buffer> {
        match self {
            Buffer::Empty => None,
            Buffer::Channel(state) => Some(&state.buffer),
            Buffer::Server(state) => Some(&state.buffer),
            Buffer::Query(state) => Some(&state.buffer),
            Buffer::FileTransfers(_) => None,
            Buffer::Logs(_) => None,
            Buffer::Highlights(_) => None,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        clients: &mut data::client::Map,
        history: &mut history::Manager,
        file_transfers: &mut file_transfer::Manager,
        config: &Config,
    ) -> (Task<Message>, Option<Event>) {
        match (self, message) {
            (Buffer::Channel(state), Message::Channel(message)) => {
                let (command, event) = state.update(message, clients, history, config);

                let event = event.map(|event| match event {
                    channel::Event::UserContext(event) => Event::UserContext(event),
                    channel::Event::OpenChannel(channel) => Event::OpenChannel(channel),
                    channel::Event::History(task) => Event::History(task),
                });

                (command.map(Message::Channel), event)
            }
            (Buffer::Server(state), Message::Server(message)) => {
                let (command, event) = state.update(message, clients, history, config);

                let event = event.map(|event| match event {
                    server::Event::UserContext(event) => Event::UserContext(event),
                    server::Event::OpenChannel(channel) => Event::OpenChannel(channel),
                    server::Event::History(task) => Event::History(task),
                });

                (command.map(Message::Server), event)
            }
            (Buffer::Query(state), Message::Query(message)) => {
                let (command, event) = state.update(message, clients, history, config);

                let event = event.map(|event| match event {
                    query::Event::UserContext(event) => Event::UserContext(event),
                    query::Event::OpenChannel(channel) => Event::OpenChannel(channel),
                    query::Event::History(task) => Event::History(task),
                });

                (command.map(Message::Query), event)
            }
            (Buffer::FileTransfers(state), Message::FileTransfers(message)) => {
                let command = state.update(message, file_transfers, config);

                (command.map(Message::FileTransfers), None)
            }
            (Buffer::Logs(state), Message::Logs(message)) => {
                let (command, event) = state.update(message);

                let event = event.map(|event| match event {
                    logs::Event::UserContext(event) => Event::UserContext(event),
                    logs::Event::OpenChannel(channel) => Event::OpenChannel(channel),
                    logs::Event::History(task) => Event::History(task),
                });

                (command.map(Message::Logs), event)
            }
            (Buffer::Highlights(state), Message::Highlights(message)) => {
                let (command, event) = state.update(message);

                let event = event.map(|event| match event {
                    highlights::Event::UserContext(event) => Event::UserContext(event),
                    highlights::Event::OpenChannel(channel) => Event::OpenChannel(channel),
                    highlights::Event::GoToMessage(server, channel, message) => {
                        Event::GoToMessage(server, channel, message)
                    }
                    highlights::Event::History(task) => Event::History(task),
                });

                (command.map(Message::Highlights), event)
            }
            _ => (Task::none(), None),
        }
    }

    pub fn view<'a>(
        &'a self,
        clients: &'a data::client::Map,
        file_transfers: &'a file_transfer::Manager,
        history: &'a history::Manager,
        settings: &'a buffer::Settings,
        config: &'a Config,
        theme: &'a Theme,
        is_focused: bool,
        sidebar: &'a sidebar::Sidebar,
    ) -> Element<'a, Message> {
        match self {
            Buffer::Empty => empty::view(config, sidebar),
            Buffer::Channel(state) => channel::view(
                state,
                clients,
                history,
                &settings.channel,
                config,
                theme,
                is_focused,
            )
            .map(Message::Channel),
            Buffer::Server(state) => {
                server::view(state, clients, history, config, theme, is_focused)
                    .map(Message::Server)
            }
            Buffer::Query(state) => {
                query::view(state, clients, history, config, theme, is_focused).map(Message::Query)
            }
            Buffer::FileTransfers(state) => {
                file_transfers::view(state, file_transfers).map(Message::FileTransfers)
            }
            Buffer::Logs(state) => logs::view(state, history, config, theme).map(Message::Logs),
            Buffer::Highlights(state) => {
                highlights::view(state, clients, history, config, theme).map(Message::Highlights)
            }
        }
    }

    // TODO: Placeholder in case we need
    #[allow(unused)]
    pub fn get_server(&self, server: &data::Server) -> Option<&Server> {
        if let Buffer::Server(state) = self {
            (&state.server == server).then_some(state)
        } else {
            None
        }
    }

    // TODO: Placeholder in case we need
    #[allow(unused)]
    pub fn get_channel(&self, server: &data::Server, channel: &str) -> Option<&Channel> {
        if let Buffer::Channel(state) = self {
            (&state.server == server && state.channel.as_str() == channel).then_some(state)
        } else {
            None
        }
    }

    pub fn focus(&self) -> Task<Message> {
        match self {
            Buffer::Empty | Buffer::FileTransfers(_) | Buffer::Logs(_) | Buffer::Highlights(_) => {
                Task::none()
            }
            Buffer::Channel(channel) => channel.focus().map(Message::Channel),
            Buffer::Server(server) => server.focus().map(Message::Server),
            Buffer::Query(query) => query.focus().map(Message::Query),
        }
    }

    pub fn reset(&mut self) {
        match self {
            Buffer::Empty | Buffer::FileTransfers(_) | Buffer::Logs(_) | Buffer::Highlights(_) => {}
            Buffer::Channel(channel) => channel.reset(),
            Buffer::Server(server) => server.reset(),
            Buffer::Query(query) => query.reset(),
        }
    }

    pub fn insert_user_to_input(
        &mut self,
        nick: Nick,
        history: &mut history::Manager,
    ) -> Task<Message> {
        if let Some(buffer) = self.data().cloned() {
            match self {
                Buffer::Empty
                | Buffer::Server(_)
                | Buffer::FileTransfers(_)
                | Buffer::Logs(_)
                | Buffer::Highlights(_) => Task::none(),
                Buffer::Channel(channel) => channel
                    .input_view
                    .insert_user(nick, buffer, history)
                    .map(|message| Message::Channel(channel::Message::InputView(message))),
                Buffer::Query(query) => query
                    .input_view
                    .insert_user(nick, buffer, history)
                    .map(|message| Message::Query(query::Message::InputView(message))),
            }
        } else {
            Task::none()
        }
    }

    pub fn scroll_to_start(&mut self) -> Task<Message> {
        match self {
            Buffer::Empty | Buffer::FileTransfers(_) => Task::none(),
            Buffer::Channel(channel) => channel
                .scroll_view
                .scroll_to_start()
                .map(|message| Message::Channel(channel::Message::ScrollView(message))),
            Buffer::Server(server) => server
                .scroll_view
                .scroll_to_start()
                .map(|message| Message::Server(server::Message::ScrollView(message))),
            Buffer::Query(query) => query
                .scroll_view
                .scroll_to_start()
                .map(|message| Message::Query(query::Message::ScrollView(message))),
            Buffer::Logs(log) => log
                .scroll_view
                .scroll_to_start()
                .map(|message| Message::Logs(logs::Message::ScrollView(message))),
            Buffer::Highlights(highlights) => highlights
                .scroll_view
                .scroll_to_start()
                .map(|message| Message::Highlights(highlights::Message::ScrollView(message))),
        }
    }

    pub fn scroll_to_end(&mut self) -> Task<Message> {
        match self {
            Buffer::Empty | Buffer::FileTransfers(_) => Task::none(),
            Buffer::Channel(channel) => channel
                .scroll_view
                .scroll_to_end()
                .map(|message| Message::Channel(channel::Message::ScrollView(message))),
            Buffer::Server(server) => server
                .scroll_view
                .scroll_to_end()
                .map(|message| Message::Server(server::Message::ScrollView(message))),
            Buffer::Query(query) => query
                .scroll_view
                .scroll_to_end()
                .map(|message| Message::Query(query::Message::ScrollView(message))),
            Buffer::Logs(log) => log
                .scroll_view
                .scroll_to_end()
                .map(|message| Message::Logs(logs::Message::ScrollView(message))),
            Buffer::Highlights(highlights) => highlights
                .scroll_view
                .scroll_to_end()
                .map(|message| Message::Highlights(highlights::Message::ScrollView(message))),
        }
    }

    pub fn scroll_to_message(
        &mut self,
        message: message::Hash,
        history: &history::Manager,
        config: &Config,
    ) -> Task<Message> {
        match self {
            Buffer::Empty | Buffer::FileTransfers(_) => Task::none(),
            Buffer::Channel(state) => state
                .scroll_view
                .scroll_to_message(
                    message,
                    scroll_view::Kind::Channel(&state.server, &state.channel),
                    history,
                    config,
                )
                .map(|message| Message::Channel(channel::Message::ScrollView(message))),
            Buffer::Server(state) => state
                .scroll_view
                .scroll_to_message(
                    message,
                    scroll_view::Kind::Server(&state.server),
                    history,
                    config,
                )
                .map(|message| Message::Server(server::Message::ScrollView(message))),
            Buffer::Query(state) => state
                .scroll_view
                .scroll_to_message(
                    message,
                    scroll_view::Kind::Query(&state.server, &state.nick),
                    history,
                    config,
                )
                .map(|message| Message::Query(query::Message::ScrollView(message))),
            Buffer::Logs(state) => state
                .scroll_view
                .scroll_to_message(message, scroll_view::Kind::Logs, history, config)
                .map(|message| Message::Logs(logs::Message::ScrollView(message))),
            Buffer::Highlights(state) => state
                .scroll_view
                .scroll_to_message(message, scroll_view::Kind::Highlights, history, config)
                .map(|message| Message::Highlights(highlights::Message::ScrollView(message))),
        }
    }

    pub fn scroll_to_backlog(
        &mut self,
        history: &history::Manager,
        config: &Config,
    ) -> Task<Message> {
        match self {
            Buffer::Empty | Buffer::FileTransfers(_) => Task::none(),
            Buffer::Channel(state) => state
                .scroll_view
                .scroll_to_backlog(
                    scroll_view::Kind::Channel(&state.server, &state.channel),
                    history,
                    config,
                )
                .map(|message| Message::Channel(channel::Message::ScrollView(message))),
            Buffer::Server(state) => state
                .scroll_view
                .scroll_to_backlog(scroll_view::Kind::Server(&state.server), history, config)
                .map(|message| Message::Server(server::Message::ScrollView(message))),
            Buffer::Query(state) => state
                .scroll_view
                .scroll_to_backlog(
                    scroll_view::Kind::Query(&state.server, &state.nick),
                    history,
                    config,
                )
                .map(|message| Message::Query(query::Message::ScrollView(message))),
            Buffer::Logs(state) => state
                .scroll_view
                .scroll_to_backlog(scroll_view::Kind::Logs, history, config)
                .map(|message| Message::Logs(logs::Message::ScrollView(message))),
            Buffer::Highlights(state) => state
                .scroll_view
                .scroll_to_backlog(scroll_view::Kind::Highlights, history, config)
                .map(|message| Message::Highlights(highlights::Message::ScrollView(message))),
        }
    }

    pub fn is_scrolled_to_bottom(&self) -> Option<bool> {
        match self {
            Buffer::Empty | Buffer::FileTransfers(_) => None,
            Buffer::Channel(channel) => Some(channel.scroll_view.is_scrolled_to_bottom()),
            Buffer::Server(server) => Some(server.scroll_view.is_scrolled_to_bottom()),
            Buffer::Query(query) => Some(query.scroll_view.is_scrolled_to_bottom()),
            Buffer::Logs(log) => Some(log.scroll_view.is_scrolled_to_bottom()),
            Buffer::Highlights(highlights) => Some(highlights.scroll_view.is_scrolled_to_bottom()),
        }
    }
}

impl From<data::Buffer> for Buffer {
    fn from(buffer: data::Buffer) -> Self {
        match buffer {
            data::Buffer::Server(server) => Self::Server(Server::new(server)),
            data::Buffer::Channel(server, channel) => Self::Channel(Channel::new(server, channel)),
            data::Buffer::Query(server, user) => Self::Query(Query::new(server, user)),
        }
    }
}
