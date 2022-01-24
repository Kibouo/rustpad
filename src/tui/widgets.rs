use std::{cmp::min, sync::atomic::Ordering};

use getset::Getters;
use tui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Row, Table},
};
use tui_logger::TuiLoggerWidget;

use super::{AppState, UiState};

#[derive(Getters)]
pub(super) struct Widgets {
    pub(super) outer_border: Block<'static>,

    // logic panel
    pub(super) original_cypher_text_view: Table<'static>,
    pub(super) forged_block_view: Table<'static>,
    pub(super) intermediate_block_view: Table<'static>,
    pub(super) plain_text_view: Table<'static>,

    // status panel
    pub(super) status_panel_border: Block<'static>,
    pub(super) progress_bar: Gauge<'static>,
    pub(super) logs_view: TuiLoggerWidget<'static>,
}

impl Widgets {
    pub(super) fn build(app_state: &AppState, ui_state: &UiState) -> Widgets {
        let title_style = Style::default().fg(Color::Cyan);

        Widgets {
            outer_border: build_outer_border(title_style),

            original_cypher_text_view: build_original_cypher_text_view(
                title_style,
                app_state
                    .cypher_text_blocks
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|block| Row::new([block.to_hex()]))
                    .collect(),
            ),
            forged_block_view: build_forged_block_view(
                title_style,
                app_state
                    .forged_blocks
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|block| Row::new([block.to_hex()]))
                    .collect(),
            ),
            intermediate_block_view: build_intermediate_view(
                title_style,
                app_state
                    .intermediate_blocks
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|block| Row::new([block.to_hex()]))
                    .collect(),
            ),
            plain_text_view: build_plain_text_view(
                title_style,
                app_state
                    .plain_text_blocks
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|block| Row::new([block.to_hex(), block.to_ascii()]))
                    .collect(),
            ),

            status_panel_border: build_status_panel_border(title_style),
            progress_bar: build_progress_bar(min(
                ((app_state.bytes_finished.load(Ordering::Relaxed) as f32
                    / app_state.bytes_to_finish.load(Ordering::Relaxed) as f32)
                    * 100.0) as u8,
                100,
            )),
            logs_view: {
                let mut log_view = build_log_view(title_style);
                log_view.state(&ui_state.log_view_state.lock().unwrap());
                log_view
            },
        }
    }
}

fn build_outer_border(title_style: Style) -> Block<'static> {
    Block::default()
        .title(Span::styled("rustpad", title_style))
        .borders(Borders::NONE)
}

fn build_original_cypher_text_view(title_style: Style, rows: Vec<Row>) -> Table {
    let title = Span::styled("Cypher text ", title_style);
    let key_indicator = Span::styled("[🠕/🠗]", Style::default().add_modifier(Modifier::DIM));

    Table::new(rows)
        .block(
            Block::default()
                .title(vec![title, key_indicator])
                .borders(Borders::ALL),
        )
        .widths(&[Constraint::Ratio(1, 1)])
}

fn build_forged_block_view(title_style: Style, rows: Vec<Row>) -> Table {
    Table::new(rows)
        .block(
            Block::default()
                .title(Span::styled("Forged block", title_style))
                .borders(Borders::ALL),
        )
        .widths(&[Constraint::Ratio(1, 1)])
}

fn build_intermediate_view(title_style: Style, rows: Vec<Row>) -> Table {
    Table::new(rows)
        .block(
            Block::default()
                .title(Span::styled("Intermediate block", title_style))
                .borders(Borders::ALL),
        )
        .widths(&[Constraint::Ratio(1, 1)])
}

fn build_plain_text_view(title_style: Style, rows: Vec<Row>) -> Table {
    Table::new(rows)
        .block(
            Block::default()
                .title(Span::styled("Plain text", title_style))
                .borders(Borders::ALL),
        )
        .column_spacing(1)
        .widths(&[Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)])
}

fn build_status_panel_border(title_style: Style) -> Block<'static> {
    Block::default()
        .title(Span::styled("Status", title_style))
        .borders(Borders::ALL)
}

fn build_progress_bar(progress: u8) -> Gauge<'static> {
    Gauge::default()
        .gauge_style(Style::default().fg(Color::LightCyan))
        .percent(progress as u16)
        .label(Span::styled(
            format!("{}%", progress),
            Style::default().fg(Color::DarkGray),
        ))
        .use_unicode(true)
}

fn build_log_view(title_style: Style) -> TuiLoggerWidget<'static> {
    let title = Span::styled("Log ", title_style);
    let key_indicator = Span::styled("[PgUp/PgDwn]", Style::default().add_modifier(Modifier::DIM));

    TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(vec![title, key_indicator])
                .borders(Borders::NONE),
        )
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::LightBlue))
        .style_debug(Style::default().fg(Color::LightGreen))
        .style_trace(Style::default().fg(Color::White))
}
