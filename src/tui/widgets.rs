use getset::Getters;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Row, Table},
};

#[derive(Getters)]
pub struct Widgets {
    #[getset(get = "pub")]
    outer_border: Block<'static>,

    // decryption panel
    #[getset(get = "pub")]
    forged_block_view: Table<'static>,
    #[getset(get = "pub")]
    intermediate_block_view: Table<'static>,
    #[getset(get = "pub")]
    plaintext_view: Table<'static>,

    // status panel
    #[getset(get = "pub")]
    status_panel_border: Block<'static>,
    #[getset(get = "pub")]
    progress_bar: Gauge<'static>,
    #[getset(get = "pub")]
    logs_view: Block<'static>,
}

impl Widgets {
    pub fn build(title_style: Style) -> Widgets {
        Widgets {
            outer_border: build_outer_border(title_style),

            forged_block_view: build_forged_block_view(title_style, vec![]),
            intermediate_block_view: build_intermediate_view(
                title_style,
                vec![Row::new(vec!["hi proper data here"])],
            ),
            plaintext_view: build_plaintext_view(
                title_style,
                vec![Row::new(vec![
                    "baaaaaaaaaaaaaaaaaaaaaaaaaaaaaab",
                    "baaaaaaaaaaaaaab",
                ])],
            ),

            status_panel_border: build_status_panel_border(title_style),
            progress_bar: build_progress_bar(),
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

fn build_progress_bar() -> Gauge<'static> {
    let label = {
        let mut label = Span::from("TODO: 72");
        label.style = Style::default().fg(Color::DarkGray);
        label
    };

    Gauge::default()
        .gauge_style(Style::default().fg(Color::LightCyan))
        .percent(72)
        .label(label)
        .use_unicode(true)
}

fn build_log_view(title_style: Style) -> Block<'static> {
    let title = {
        let mut title = Span::from("Log");
        title.style = title_style;
        title
    };

    Block::default().title(title).borders(Borders::NONE)
}
