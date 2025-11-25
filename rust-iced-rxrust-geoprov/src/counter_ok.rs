use iced::widget::{button, column, container, row, text};
use iced::{Center, Element, Fill, Theme};
use tracing::{info, Level};

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Iced counter 🔥");

    iced::application("Iced Counter ❤️", Counter::update, Counter::view)
        .theme(|_| Theme::Dark)
        .run()
}

#[derive(Default)]
struct Counter {
    count: i64,
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
        }
        
        let doubled = self.count * 2;
        info!("Counter → {} | Doubled → {}", self.count, doubled);
    }

    fn view(&self) -> Element<Message> {
        let doubled = self.count * 2;

        let content = column![
            text("Iced Counter").size(32),
            text(format!("Count: {}", self.count)).size(48),
            text(format!("Doubled: {}", doubled)).size(64),
            row![
                button(text("-").size(48).center())
                    .width(100)
                    .height(100)
                    .on_press(Message::Decrement),
                button(text("+").size(48).center())
                    .width(100)
                    .height(100)
                    .on_press(Message::Increment),
            ]
            .spacing(40)
            .align_y(Center),
        ]
        .spacing(30)
        .align_x(Center);

        container(content)
            .width(Fill)
            .height(Fill)
            .center(Fill)
            .into()
    }
}
