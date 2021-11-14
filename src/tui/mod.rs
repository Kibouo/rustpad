mod layout;
mod widgets;

use std::io::{self};

use anyhow::{Context, Result};
use tui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    Terminal,
};

use crate::block::block_size::BlockSize;

use self::{layout::TuiLayout, widgets::Widgets};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    title_style: Style,
    min_width_for_horizontal_layout: u16,
}

impl Tui {
    pub fn new(block_size: &BlockSize) -> Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear().context("Failed to clear terminal view")?;

        let title_style = Style::default().fg(Color::Cyan);

        let tui = Self {
            terminal,
            title_style,
            // enough space to display 2 tables of hex encoded blocks + padding
            min_width_for_horizontal_layout: (*block_size as usize * 8) as u16,
        };

        Ok(tui)
    }

    // pub fn setup_layout(&mut self) -> Result<&mut Self> {
    //     self.terminal.draw(|frame| {
    //         let size = frame.size();
    //         frame.render_widget(main_block, size);
    //     })?;

    //     Ok(self)
    // }

    pub fn setup(&mut self) -> Result<&mut Self> {
        self.terminal.draw(|frame| {
            let layout = TuiLayout::calculate(frame.size(), self.min_width_for_horizontal_layout);
            let widgets = Widgets::build(self.title_style);

            frame.render_widget(widgets.outer_border().clone(), frame.size());

            frame.render_widget(
                widgets.forged_block_view().clone(),
                *layout.forged_block_area(),
            );
            frame.render_widget(
                widgets.intermediate_block_view().clone(),
                *layout.intermediate_block_area(),
            );
            frame.render_widget(widgets.plaintext_view().clone(), *layout.plaintext_area());

            frame.render_widget(
                widgets.status_panel_border().clone(),
                *layout.status_panel_area(),
            );
            frame.render_widget(widgets.progress_bar().clone(), *layout.progress_bar_area());
            frame.render_widget(widgets.logs_view().clone(), *layout.logs_area());
        })?;

        Ok(self)
    }
}
