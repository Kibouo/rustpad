use getset::Getters;
use tui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Getters)]
pub(super) struct TuiLayout {
    // logic panel
    #[get = "pub(super)"]
    original_cypher_text_area: Rect,
    #[get = "pub(super)"]
    forged_block_area: Rect,
    #[get = "pub(super)"]
    intermediate_block_area: Rect,
    #[get = "pub(super)"]
    plain_text_area: Rect,

    // status panel
    #[get = "pub(super)"]
    status_panel_area: Rect,
    #[get = "pub(super)"]
    progress_bar_area: Rect,
    #[get = "pub(super)"]
    logs_area: Rect,
}

impl TuiLayout {
    pub(super) fn calculate(full_frame_size: Rect, min_width_for_horizontal_layout: u16) -> Self {
        let main_vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Ratio(3, 5), Constraint::Ratio(2, 5)].as_ref())
            .split(full_frame_size);

        // main area for fancily showing logic at work
        let decyption_panel_direction = if full_frame_size.width < min_width_for_horizontal_layout {
            Direction::Vertical
        } else {
            Direction::Horizontal
        };
        let logic_panel = Layout::default()
            .direction(decyption_panel_direction)
            .constraints(
                [
                    Constraint::Ratio(1, 5),
                    Constraint::Ratio(1, 5),
                    Constraint::Ratio(1, 5),
                    Constraint::Ratio(2, 5),
                ]
                .as_ref(),
            )
            .split(main_vertical_layout[0]);

        // lower area for showing the status
        let status_panel = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Ratio(1, 6), Constraint::Ratio(5, 6)].as_ref())
            .split(main_vertical_layout[1]);

        Self {
            original_cypher_text_area: logic_panel[0],
            forged_block_area: logic_panel[1],
            intermediate_block_area: logic_panel[2],
            plain_text_area: logic_panel[3],
            status_panel_area: main_vertical_layout[1],
            progress_bar_area: status_panel[0],
            logs_area: status_panel[1],
        }
    }
}
