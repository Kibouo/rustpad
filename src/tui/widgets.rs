use std::{cmp::min, sync::atomic::Ordering};

use getset::Getters;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Row, Table},
};
use tui_logger::TuiLoggerWidget;

use super::AppState;

#[derive(Getters)]
pub(super) struct Widgets {
    pub outer_border: Block<'static>,

    // decryption panel
    pub original_cypher_text_view: Table<'static>,
    pub forged_block_view: Table<'static>,
    pub intermediate_block_view: Table<'static>,
    pub plaintext_view: Table<'static>,

    // status panel
    pub status_panel_border: Block<'static>,
    pub progress_bar: Gauge<'static>,
    pub logs_view: TuiLoggerWidget<'static>,
}

impl Widgets {
    pub(super) fn build(app_state: &AppState) -> Widgets {
        let title_style = Style::default().fg(Color::Cyan);

        Widgets {
            outer_border: build_outer_border(title_style),

            original_cypher_text_view: build_original_cypher_text_view(
                title_style,
                app_state
                    .original_cypher_text_blocks
                    .iter()
                    .map(|b| Row::new([b.to_hex()]))
                    .collect(),
            ),
            forged_block_view: build_forged_block_view(
                title_style,
                app_state
                    .forged_blocks
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|b| Row::new([b.to_hex()]))
                    .collect(),
            ),
            intermediate_block_view: build_intermediate_view(
                title_style,
                app_state
                    .intermediate_blocks
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|b| Row::new([b.to_hex()]))
                    .collect(),
            ),
            plaintext_view: build_plaintext_view(
                title_style,
                app_state
                    .plaintext_blocks
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|b| Row::new([b.to_hex(), b.to_ascii()]))
                    .collect(),
            ),

            status_panel_border: build_status_panel_border(title_style),
            progress_bar: build_progress_bar(min(
                ((app_state.bytes_finished.load(Ordering::Relaxed) as f32
                    / app_state.bytes_to_finish as f32)
                    * 100.0) as u8,
                100,
            )),
            logs_view: build_log_view(title_style),
        }
    }
}

fn build_outer_border(title_style: Style) -> Block<'static> {
    let title = {
        let mut title = Span::from("rustpad");
        title.style = title_style;
        title
    };

    Block::default().title(title).borders(Borders::NONE)
}

fn build_original_cypher_text_view(title_style: Style, rows: Vec<Row>) -> Table {
    let title = {
        let mut title = Span::from("Cypher text");
        title.style = title_style;
        title
    };

    Table::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .widths(&[Constraint::Ratio(1, 1)])
}

fn build_forged_block_view(title_style: Style, rows: Vec<Row>) -> Table {
    let title = {
        let mut title = Span::from("Forged block");
        title.style = title_style;
        title
    };

    Table::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .widths(&[Constraint::Ratio(1, 1)])
}

fn build_intermediate_view(title_style: Style, rows: Vec<Row>) -> Table {
    let title = {
        let mut title = Span::from("Intermediate block");
        title.style = title_style;
        title
    };

    Table::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .widths(&[Constraint::Ratio(1, 1)])
}

fn build_plaintext_view(title_style: Style, rows: Vec<Row>) -> Table {
    let title = {
        let mut title = Span::from("Plain text");
        title.style = title_style;
        title
    };

    Table::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .column_spacing(1)
        .widths(&[Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)])
}

fn build_status_panel_border(title_style: Style) -> Block<'static> {
    let title = {
        let mut title = Span::from("Status");
        title.style = title_style;
        title
    };

    Block::default().title(title).borders(Borders::ALL)
}

fn build_progress_bar(progress: u8) -> Gauge<'static> {
    let label = {
        let mut label = Span::from(format!("{}%", progress));
        label.style = Style::default().fg(Color::DarkGray);
        label
    };

    Gauge::default()
        .gauge_style(Style::default().fg(Color::LightCyan))
        .percent(progress as u16)
        .label(label)
        .use_unicode(true)
}

fn build_log_view(title_style: Style) -> TuiLoggerWidget<'static> {
    let title = {
        let mut title = Span::from("Log");
        title.style = title_style;
        title
    };

    TuiLoggerWidget::default()
        .block(Block::default().title(title).borders(Borders::NONE))
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::LightBlue))
        .style_debug(Style::default().fg(Color::LightGreen))
        .style_trace(Style::default().fg(Color::White))
}