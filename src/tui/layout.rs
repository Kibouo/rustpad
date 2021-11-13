use tui::layout::{Constraint, Direction, Layout, Rect};

pub struct TuiLayout {
    // decryption panel
    forged_cypher_text_area: Rect,
    intermediate_area: Rect,
    plaintext_area: Rect,

    // status panel
    status_panel_area: Rect,
    progress_bar_area: Rect,
    logs_area: Rect,
}

impl TuiLayout {
    pub fn calculate(full_frame_size: Rect, min_width_for_horizontal_layout: u16) -> Self {
        let main_vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)].as_ref())
            .split(full_frame_size);

        // main area for fancily showing decryption at work
        let decyption_panel_direction = if full_frame_size.width < min_width_for_horizontal_layout {
            Direction::Vertical
        } else {
            Direction::Horizontal
        };
        let decryption_panel = Layout::default()
            .direction(decyption_panel_direction)
            .constraints(
                [
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(2, 4),
                ]
                .as_ref(),
            )
            .split(main_vertical_layout[0]);

        // lower area for showing the status of the decryption
        let status_panel = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Ratio(1, 5), Constraint::Ratio(4, 5)].as_ref())
            .split(main_vertical_layout[1]);

        Self {
            forged_cypher_text_area: decryption_panel[0],
            intermediate_area: decryption_panel[1],
            plaintext_area: decryption_panel[2],
            status_panel_area: main_vertical_layout[1],
            progress_bar_area: status_panel[0],
            logs_area: status_panel[1],
        }
    }

    pub fn forged_block_area(&self) -> &Rect {
        &self.forged_cypher_text_area
    }

    pub fn intermediate_block_area(&self) -> &Rect {
        &self.intermediate_area
    }

    pub fn plaintext_area(&self) -> &Rect {
        &self.plaintext_area
    }

    pub fn status_panel_area(&self) -> &Rect {
        &self.status_panel_area
    }

    pub fn progress_bar_area(&self) -> &Rect {
        &self.progress_bar_area
    }

    pub fn logs_area(&self) -> &Rect {
        &self.logs_area
    }
}
