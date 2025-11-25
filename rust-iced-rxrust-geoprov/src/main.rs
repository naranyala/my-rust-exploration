use iced::{
    widget::{button, column, container, row, text, Column},
    Alignment, Center, Element, Fill,
};
use iced::{application, Theme};

#[derive(Debug, Clone)]
enum Message {
    Digit(u8),          // 0–9
    Decimal,            // .
    Operator(Operator),
    Equals,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Default)]
struct Calculator {
    display: String,
    previous: Option<f64>,
    current: String,
    operator: Option<Operator>,
    waiting_for_operand: bool,
}

impl Calculator {
    fn update(&mut self, message: Message) {
        match message {
            Message::Digit(d) => {
                if self.waiting_for_operand {
                    self.current = d.to_string();
                    self.waiting_for_operand = false;
                } else {
                    self.current.push(char::from(b'0' + d));
                }
                self.update_display();
            }

            Message::Decimal => {
                if self.waiting_for_operand {
                    self.current = "0.".to_string();
                    self.waiting_for_operand = false;
                } else if !self.current.contains('.') {
                    self.current.push('.');
                }
                self.update_display();
            }

            Message::Operator(op) => {
                if let Some(prev) = self.current.parse::<f64>().ok() {
                    if let Some(previous_val) = self.previous {
                        let result = self.compute(previous_val, prev, self.operator.unwrap());
                        self.previous = Some(result);
                        self.current = result.to_string();
                    } else {
                        self.previous = Some(prev);
                    }
                }
                self.operator = Some(op);
                self.waiting_for_operand = true;
                self.update_display();
            }

            Message::Equals => {
                if let (Some(prev), Some(op)) = (self.previous, self.operator) {
                    if let Ok(current_val) = self.current.parse::<f64>() {
                        let result = self.compute(prev, current_val, op);
                        self.display = result.to_string();
                        self.previous = None;
                        self.current = result.to_string();
                        self.operator = None;
                        self.waiting_for_operand = true;
                    }
                }
            }

            Message::Clear => {
                self.display = "0".to_string();
                self.current = "0".to_string();
                self.previous = None;
                self.operator = None;
                self.waiting_for_operand = false;
            }
        }
    }

    fn compute(&self, a: f64, b: f64, op: Operator) -> f64 {
        match op {
            Operator::Add => a + b,
            Operator::Subtract => a - b,
            Operator::Multiply => a * b,
            Operator::Divide => {
                if b != 0.0 { a / b } else { 0.0 } // simple error handling
            }
        }
    }

    fn update_display(&mut self) {
        let cleaned = self.current.trim_start_matches('-').trim_start_matches('0');
        let display_val = if cleaned.is_empty() || cleaned == "." {
            "0".to_string()
        } else if self.current.starts_with("0.") || self.current.starts_with("-0.") {
            self.current.clone()
        } else if self.current.starts_with('0') && !self.current.contains('.') {
            cleaned.to_string()
        } else {
            self.current.clone()
        };

        // Remove trailing .0 if integer
        self.display = if display_val.ends_with(".0") {
            display_val[..display_val.len() - 2].to_string()
        } else {
            display_val
        };
    }

    fn view(&self) -> Element<Message> {
        let display = text(&self.display)
            .size(64)
            .horizontal_alignment(iced::alignment::Horizontal::Right)
            .width(Fill);

        let button = |label: &str, msg: Message| {
            button(
                text(label)
                    .size(40)
                    .center()
                    .width(Fill)
                    .height(Fill),
            )
            .width(80)
            .height(80)
            .on_press(msg)
        };

        let calc_grid = column![
            // Display
            container(display)
                .width(Fill)
                .padding(20)
                .style(container::rounded_box),

            // Row 1
            row![
                button("C", Message::Clear).style(button::danger),
                button("±", Message::Digit(0)), // placeholder, not implemented
                button("％", Message::Digit(0)), // placeholder
                button("÷", Message::Operator(Operator::Divide)).style(button::primary),
            ]
            .spacing(10),

            // Row 2
            row![
                button("7", Message::Digit(7)),
                button("8", Message::Digit(8)),
                button("9", Message::Digit(9)),
                button("×", Message::Operator(Operator::Multiply)).style(button::primary),
            ]
            .spacing(10),

            // Row 3
            row![
                button("4", Message::Digit(4)),
                button("5", Message::Digit(5)),
                button("6", Message::Digit(6)),
                button("−", Message::Operator(Operator::Subtract)).style(button::primary),
            ]
            .spacing(10),

            // Row 4
            row![
                button("1", Message::Digit(1)),
                button("2", Message::Digit(2)),
                button("3", Message::Digit(3)),
                button("+", Message::Operator(Operator::Add)).style(button::primary),
            ]
            .spacing(10),

            // Row 5
            row![
                button("0", Message::Digit(0)).width(170),
                button(".", Message::Decimal),
                button("=", Message::Equals).style(button::success),
            ]
            .spacing(10),
        ]
        .spacing(10)
        .padding(20)
        .align_x(Center);

        container(Column::with_children(vec![calc_grid.into()]))
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }
}

fn main() -> iced::Result {
    application("Iced Calculator 🧮", Calculator::update, Calculator::view)
        .theme(|_| Theme::Dark)
        .window_size(iced::Size::new(400.0, 600.0))
        .run_with(|| (Calculator::default(), iced::task::nothing()))
}
