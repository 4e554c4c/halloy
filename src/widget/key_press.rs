use iced::advanced::{widget, Clipboard, Layout, Shell};
pub use iced::keyboard::{key::Named, Key, Modifiers};
use iced::{event, keyboard, mouse, Event, Rectangle};

use super::{wrap, Element, Renderer};

pub fn key_press<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    key: Key,
    modifiers: Modifiers,
    on_press: Message,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    wrap(base)
        .on_event(
            move |inner: &mut Element<'a, Message>,
                  tree: &mut widget::Tree,
                  event: Event,
                  layout: Layout<'_>,
                  cursor: mouse::Cursor,
                  renderer: &Renderer,
                  clipboard: &mut dyn Clipboard,
                  shell: &mut Shell<'_, Message>,
                  viewport: &Rectangle| {
                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key: k,
                    modifiers: m,
                    ..
                }) = &event
                {
                    if key == *k && modifiers == *m {
                        shell.publish(on_press.clone());
                        return event::Status::Captured;
                    }
                }

                inner.as_widget_mut().on_event(
                    tree, event, layout, cursor, renderer, clipboard, shell, viewport,
                )
            },
        )
        .into()
}
