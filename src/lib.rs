use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use log::{debug, info};

use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Widget, Wrap},
};

pub mod loggers;
pub mod prompts;
pub mod webscraping;

#[derive(Debug, Clone)]
pub struct TypeRacePrompt {
    prompt: String,
    input: String,
}

impl TypeRacePrompt {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            input: String::new(),
        }
    }
    pub fn apply_key(&mut self, key_event: KeyEvent) {
        let KeyEvent { code, kind, .. } = key_event;
        if let KeyEventKind::Release = kind {
            return;
        } else if let KeyCode::Char(c) = code {
            self.input.push(c);
        } else if let KeyCode::Backspace = code {
            self.input.pop();
        }
        debug!(
            "Received key press: {code:?}. Typed input is now string is: {}",
            self.input
        );
    }
    pub fn is_complete(&self) -> bool {
        self.prompt == self.input
    }

    fn correct_length(&self) -> usize {
        self.input
            .as_bytes()
            .iter()
            .zip(self.prompt.as_bytes())
            .take_while(|(i, p)| i == p)
            .count()
    }

    pub fn correct_input(&self) -> &str {
        self.split_input().0
    }

    pub fn incorrect_input(&self) -> &str {
        self.split_input().1
    }

    pub fn split_input(&self) -> (&str, &str) {
        self.input.split_at(self.correct_length())
    }
}

impl Widget for &TypeRacePrompt {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        //TODO: fix ASCII assumption
        let (correct, mistakes) = self.split_input();
        let remaining = self.prompt.get(self.input.len()..).unwrap_or_default();
        let green = Style::default().fg(Color::Green);
        let red = Style::default().bg(Color::Red).add_modifier(Modifier::BOLD);
        let correct = Span::styled(correct, green);
        let mistakes = Span::styled(mistakes, red);
        let remaining = Span::raw(remaining);
        let spans = Spans(vec![correct, mistakes, remaining]);
        let paragraph = Paragraph::new(spans).wrap(Wrap { trim: false });
        paragraph.render(area, buf);
    }
}
