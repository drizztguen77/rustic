mod popup;
mod prompt;
mod select_table;
mod sized_gauge;
mod sized_paragraph;
mod sized_table;
mod text_input;
mod with_block;

pub use popup::*;
pub use prompt::*;
use ratatui::widgets::block::Title;
pub use select_table::*;
pub use sized_gauge::*;
pub use sized_paragraph::*;
pub use sized_table::*;
pub use text_input::*;
pub use with_block::*;

use crossterm::event::Event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Clear, Gauge, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    TableState,
};

pub trait ProcessEvent {
    type Result;
    fn input(&mut self, event: Event) -> Self::Result;
}

pub trait SizedWidget {
    fn height(&self) -> Option<u16> {
        None
    }
    fn width(&self) -> Option<u16> {
        None
    }
}

pub trait Draw {
    fn draw(&mut self, area: Rect, f: &mut Frame<'_>);
}

// the widgets we are using and convenience builders
pub type PopUpInput = PopUp<WithBlock<TextInput>>;
pub fn popup_input(
    title: impl Into<Title<'static>>,
    text: &str,
    initial: &str,
    lines: u16,
) -> PopUpInput {
    PopUp(WithBlock::new(
        TextInput::new(Some(text), initial, lines, true),
        Block::bordered().title(title),
    ))
}

pub fn popup_scrollable_text(
    title: impl Into<Title<'static>>,
    text: &str,
    lines: u16,
) -> PopUpInput {
    PopUp(WithBlock::new(
        TextInput::new(None, text, lines, false),
        Block::bordered().title(title),
    ))
}

pub type PopUpText = PopUp<WithBlock<SizedParagraph>>;
pub fn popup_text(title: impl Into<Title<'static>>, text: Text<'static>) -> PopUpText {
    PopUp(WithBlock::new(
        SizedParagraph::new(text),
        Block::bordered().title(title),
    ))
}

pub type PopUpTable = PopUp<WithBlock<SizedTable>>;
pub fn popup_table(
    title: impl Into<Title<'static>>,
    content: Vec<Vec<Text<'static>>>,
) -> PopUpTable {
    PopUp(WithBlock::new(
        SizedTable::new(content),
        Block::bordered().title(title),
    ))
}

pub type PopUpPrompt = Prompt<PopUpText>;
pub fn popup_prompt(title: &'static str, text: Text<'static>) -> PopUpPrompt {
    Prompt(popup_text(title, text))
}

pub type PopUpGauge = PopUp<WithBlock<SizedGauge>>;
pub fn popup_gauge(
    title: impl Into<Title<'static>>,
    text: Span<'static>,
    ratio: f64,
) -> PopUpGauge {
    PopUp(WithBlock::new(
        SizedGauge::new(text, ratio),
        Block::bordered().title(title),
    ))
}
