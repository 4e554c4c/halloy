use data::config;
use iced::{
    alignment,
    widget::{button, column, container, text},
    Length,
};

use super::Close;
use crate::{theme, widget::Element};

pub fn view<'a>(error: &config::Error) -> Element<'a, Close> {
    container(
        column![
            text("Error reloading configuration file"),
            text(error.to_string()).style(theme::text::error),
            button(
                container(text("Close"))
                    .align_x(alignment::Horizontal::Center)
                    .width(Length::Fill),
            )
            .padding(5)
            .width(Length::Fixed(250.0))
            .style(theme::button::primary)
            .on_press(Close),
        ]
        .spacing(20)
        .align_items(iced::Alignment::Center),
    )
    .width(Length::Shrink)
    .style(theme::container::error_banner)
    .padding(25)
    .into()
}
