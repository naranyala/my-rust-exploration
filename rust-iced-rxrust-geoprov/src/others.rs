use iced::widget::{button, column, container, row, text};
use iced::{Center, Element, Fill, Subscription, Theme};
use rxrust::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{info, Level};

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Iced + rxrust reactive counter 🔥");

    iced::application("Iced + rxrust = ❤️", Counter::update, Counter::view)
        .subscription(Counter::subscription)
        .theme(|_| Theme::Dark)
        .run()
}

// Reactive state container
struct ReactiveState {
    subject: subject::LocalSubject<'static, i64, ()>,
    count: Rc<RefCell<i64>>,
    doubled: Rc<RefCell<i64>>,
}

impl ReactiveState {
    fn new() -> Self {
        let subject = subject::LocalSubject::new();
        let count = Rc::new(RefCell::new(0i64));
        let doubled = Rc::new(RefCell::new(0i64));

        // Set up reactive pipeline: count -> map to doubled -> side effects
        let count_ref = count.clone();
        let doubled_ref = doubled.clone();

        subject.clone().subscribe(move |value: i64| {
            let d = value * 2;
            info!("rxrust stream → count: {value}, doubled: {d}");
            *count_ref.borrow_mut() = value;
            *doubled_ref.borrow_mut() = d;
        });

        // Emit initial value
        subject.clone().next(0);

        Self { subject, count, doubled }
    }

    fn emit(&self, value: i64) {
        self.subject.clone().next(value);
    }

    fn count(&self) -> i64 {
        *self.count.borrow()
    }

    fn doubled(&self) -> i64 {
        *self.doubled.borrow()
    }
}

struct Counter {
    reactive: ReactiveState,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            reactive: ReactiveState::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
}

impl Counter {
    fn update(&mut self, message: Message) {
        let current = self.reactive.count();
        match message {
            Message::Increment => self.reactive.emit(current + 1),
            Message::Decrement => self.reactive.emit(current - 1),
        }
    }

    fn view(&self) -> Element<Message> {
        let count = self.reactive.count();
        let doubled = self.reactive.doubled();

        let content = column![
            text("Iced + rxrust Counter").size(32),
            text(format!("Count: {count}")).size(48),
            text(format!("Doubled (reactive): {doubled}")).size(48),
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

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
