mod layout;
pub mod ui_event;
mod widgets;

use std::{
    cmp::{max, min},
    io::{self},
    process,
    sync::{
        atomic::{AtomicBool, AtomicI32, AtomicU16, AtomicUsize, Ordering},
        Mutex,
    },
    thread::sleep,
    time::Duration,
};

use anyhow::{Context, Result};
use atty::Stream;
use crossterm::{
    cursor::Show,
    event::{Event, EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetSize,
    },
};
use futures::FutureExt;
use futures_timer::Delay;
use log::error;
use tui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use crate::{
    block::{
        block_size::{BlockSize, BlockSizeTrait},
        Block,
    },
    logging::LOG_TARGET,
};

use self::{
    layout::TuiLayout,
    ui_event::{UiControlEvent, UiDecryptionEvent, UiEncryptionEvent, UiEvent},
    widgets::Widgets,
};

const FRAME_SLEEP_MS: u64 = 20;
const INPUT_POLL_MS: u64 = 50;

pub(super) struct Tui {
    // the usage of a mutex here could be prevented by separating `Terminal` from `Tui`, it's only needed in the draw thread. However, the overhead of handling the mutex should be so small (especially given that only the draw thread accesses it) should be so small that it's unneeded.
    terminal: Mutex<Terminal<CrosstermBackend<io::Stdout>>>,
    min_width_for_horizontal_layout: u16,
    cols: AtomicU16,
    rows: AtomicU16,
    // because we enter a "different terminal" during the application's runtime, nothing is left when the user exits the program. This stores a list of messages to print after leaving the "different terminal", but before quitting the application
    print_after_exit: Mutex<Vec<String>>,
    exit_code: AtomicI32,

    ui_state: UiState,
    app_state: AppState,
}

struct UiState {
    running: AtomicBool,
    slow_redraw: AtomicBool,
    redraw: AtomicBool,

    log_view_state: Mutex<TuiWidgetState>,
    blocks_view_state: Mutex<TableState>,
}

struct AppState {
    // for progress calculation
    bytes_to_finish: AtomicUsize,
    bytes_finished: AtomicUsize,

    cypher_text_blocks: Mutex<Vec<Block>>,
    forged_blocks: Mutex<Vec<Block>>,
    intermediate_blocks: Mutex<Vec<Block>>,
    plain_text_blocks: Mutex<Vec<Block>>,
}

impl Tui {
    pub(super) fn new(block_size: &BlockSize) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear().context("Clearing terminal failed")?;
        let cols = AtomicU16::new(terminal.size()?.width);
        let rows = AtomicU16::new(terminal.size()?.height);

        let tui = Self {
            terminal: Mutex::new(terminal),
            // enough space to display 2 tables of hex encoded blocks + padding
            min_width_for_horizontal_layout: (**block_size as usize * 12) as u16,
            cols,
            rows,
            print_after_exit: Mutex::new(vec![]),
            exit_code: AtomicI32::new(0),

            ui_state: UiState {
                running: AtomicBool::new(true),
                slow_redraw: AtomicBool::new(false),
                redraw: AtomicBool::new(true),

                log_view_state: Mutex::new(TuiWidgetState::new()),
                blocks_view_state: Mutex::new(TableState::default()),
            },

            app_state: AppState {
                bytes_to_finish: AtomicUsize::new(1),
                bytes_finished: AtomicUsize::new(0),

                cypher_text_blocks: Mutex::new(vec![]),
                forged_blocks: Mutex::new(vec![]),
                intermediate_blocks: Mutex::new(vec![]),
                plain_text_blocks: Mutex::new(vec![]),
            },
        };

        Ok(tui)
    }

    /// Clean up terminal "hi-jacking". Ignores errors to try restore as much as possible
    pub(super) fn exit(&self) {
        let _ = disable_raw_mode();

        let (cols, rows) = {
            (
                self.cols.load(Ordering::Relaxed),
                self.rows.load(Ordering::Relaxed),
            )
        };
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            SetSize(cols, rows),
            Show
        );

        // we could separate `self.print_after_exit` into a stdout and a stderr version, but (for now) it's unneeded for our use case
        let use_stderr = self.exit_code.load(Ordering::Relaxed) != 0;
        for message in self.print_after_exit.lock().unwrap().drain(..) {
            if use_stderr {
                eprintln!("{}", message);
            } else {
                println!("{}", message);
            }
        }

        process::exit(self.exit_code.load(Ordering::Relaxed));
    }

    pub(super) async fn main_loop(&self) -> Result<()> {
        let (_, outputs) = async_scoped::AsyncScope::scope_and_block(|scope| {
            scope.spawn(self.draw_loop());
            scope.spawn(self.input_loop());
        });

        outputs.into_iter().collect()
    }

    async fn draw_loop(&self) -> Result<()> {
        while self.ui_state.running.load(Ordering::Relaxed) {
            if self.need_redraw() {
                self.draw().context("Drawing UI failed")?;
                self.ui_state.redraw.store(false, Ordering::Relaxed);
            }

            if self.ui_state.slow_redraw.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(FRAME_SLEEP_MS * 3));
            } else {
                sleep(Duration::from_millis(FRAME_SLEEP_MS));
            }
        }

        // 1 last draw to ensure errors are displayed
        self.draw().context("Drawing UI failed")?;

        Ok(())
    }

    // need to handle user input async. Scrolling can generate too many events which crashes the app :)
    async fn input_loop(&self) -> Result<()> {
        let mut reader = EventStream::new();

        while self.ui_state.running.load(Ordering::Relaxed) {
            let mut delay = Delay::new(Duration::from_millis(INPUT_POLL_MS)).fuse();
            let mut event = futures::StreamExt::next(&mut reader).fuse();

            futures::select_biased! {
                maybe_event = event => {
                    if let Some(fallible_event) = maybe_event {
                        match fallible_event {
                            Ok(event) => self.handle_user_event(event),
                            Err(e) => error!(target: LOG_TARGET, "{:?}", e),
                        }
                    }
                    // else no event
                },
                _ = delay => { /* no event */ }
            }
        }

        Ok(())
    }

    pub(super) fn handle_application_event(&self, event: UiEvent) {
        match event {
            UiEvent::Decryption(event) => self.handle_decryption_event(event),
            UiEvent::Encryption(event) => self.handle_encryption_event(event),
            UiEvent::Control(event) => self.handle_control_event(event),
        }

        self.ui_state.redraw.store(true, Ordering::Relaxed);
    }

    fn handle_decryption_event(&self, event: UiDecryptionEvent) {
        match event {
            UiDecryptionEvent::InitDecryption(original_cypher_text_blocks) => {
                let block_size = original_cypher_text_blocks[0].block_size();
                let amount_cypher_text_blocks = original_cypher_text_blocks.len();

                *self.app_state.cypher_text_blocks.lock().unwrap() = original_cypher_text_blocks;

                let default_blocks = vec![Block::new(&block_size); amount_cypher_text_blocks];
                *self.app_state.forged_blocks.lock().unwrap() = default_blocks.clone();
                *self.app_state.intermediate_blocks.lock().unwrap() = default_blocks.clone();
                *self.app_state.plain_text_blocks.lock().unwrap() = default_blocks;
            }
            UiDecryptionEvent::BlockSolved(forged_block, cypher_text_block_idx) => {
                let intermediate = forged_block.to_intermediate();

                let plain_text = &intermediate
                    ^ &self.app_state.cypher_text_blocks.lock().unwrap()[cypher_text_block_idx - 1];

                self.app_state.forged_blocks.lock().unwrap()[cypher_text_block_idx - 1] =
                    forged_block;
                self.app_state.intermediate_blocks.lock().unwrap()[cypher_text_block_idx] =
                    intermediate;
                self.app_state.plain_text_blocks.lock().unwrap()[cypher_text_block_idx] =
                    plain_text;
            }
            UiDecryptionEvent::BlockWip(forged_block, cypher_text_block_idx) => {
                let intermediate = forged_block.to_intermediate();

                let plain_text = &intermediate
                    ^ &self.app_state.cypher_text_blocks.lock().unwrap()[cypher_text_block_idx - 1];

                // `try_lock` as updating isn't critical. This is mainly for visuals
                if let Ok(mut blocks) = self.app_state.forged_blocks.try_lock() {
                    blocks[cypher_text_block_idx - 1] = forged_block;
                }
                if let Ok(mut blocks) = self.app_state.intermediate_blocks.try_lock() {
                    blocks[cypher_text_block_idx] = intermediate;
                }
                if let Ok(mut blocks) = self.app_state.plain_text_blocks.try_lock() {
                    blocks[cypher_text_block_idx] = plain_text;
                }
            }
        }
    }

    fn handle_encryption_event(&self, event: UiEncryptionEvent) {
        match event {
            UiEncryptionEvent::InitEncryption(plain_text_blocks, init_cypher_text) => {
                let block_size = plain_text_blocks[0].block_size();
                let amount_plain_text_blocks = plain_text_blocks.len();

                *self.app_state.plain_text_blocks.lock().unwrap() = {
                    let mut blocks = vec![Block::new(&block_size)];
                    blocks.extend(plain_text_blocks);
                    blocks
                };

                // +1 for the IV
                let default_blocks = vec![Block::new(&block_size); amount_plain_text_blocks + 1];
                *self.app_state.intermediate_blocks.lock().unwrap() = default_blocks.clone();
                *self.app_state.forged_blocks.lock().unwrap() = default_blocks;

                // the first solve for an encryption gives the before last cypher text, so the initial cypher text needs to be set here
                *self.app_state.cypher_text_blocks.lock().unwrap() = {
                    let mut blocks = vec![Block::new(&block_size); amount_plain_text_blocks];
                    blocks.push(init_cypher_text);
                    blocks
                };
            }
            UiEncryptionEvent::BlockSolved(forged_block, cypher_text_block_idx) => {
                let intermediate = forged_block.to_intermediate();

                let cypher_text = &intermediate
                    ^ &self.app_state.plain_text_blocks.lock().unwrap()[cypher_text_block_idx];

                self.app_state.intermediate_blocks.lock().unwrap()[cypher_text_block_idx] =
                    intermediate;
                self.app_state.forged_blocks.lock().unwrap()[cypher_text_block_idx - 1] =
                    forged_block;
                self.app_state.cypher_text_blocks.lock().unwrap()[cypher_text_block_idx - 1] =
                    cypher_text;
            }
            UiEncryptionEvent::BlockWip(forged_block, cypher_text_block_idx) => {
                let intermediate = forged_block.to_intermediate();

                let cypher_text = &intermediate
                    ^ &self.app_state.plain_text_blocks.lock().unwrap()[cypher_text_block_idx];

                // `try_lock` as updating isn't critical. This is mainly for visuals
                if let Ok(mut blocks) = self.app_state.intermediate_blocks.try_lock() {
                    blocks[cypher_text_block_idx] = intermediate;
                };
                if let Ok(mut blocks) = self.app_state.forged_blocks.try_lock() {
                    blocks[cypher_text_block_idx - 1] = forged_block;
                };
                if let Ok(mut blocks) = self.app_state.cypher_text_blocks.try_lock() {
                    blocks[cypher_text_block_idx - 1] = cypher_text;
                };
            }
        }
    }

    fn handle_control_event(&self, event: UiControlEvent) {
        match event {
            UiControlEvent::IndicateWork(bytes_to_finish) => {
                self.app_state
                    .bytes_to_finish
                    .store(bytes_to_finish, Ordering::Relaxed);
            }
            // due to concurrency, we can't just send which blocks was finished. So this acts as a "ping" to indicate that a byte was locked
            UiControlEvent::ProgressUpdate(newly_solved_bytes) => {
                self.app_state
                    .bytes_finished
                    .fetch_add(newly_solved_bytes, Ordering::Relaxed);
            }
            UiControlEvent::PrintAfterExit(message) => {
                self.print_after_exit.lock().unwrap().push(message);
            }
            UiControlEvent::ExitCode(code) => {
                self.exit_code.store(code, Ordering::Relaxed);
            }
            UiControlEvent::SlowRedraw => {
                // keeping the UI running/application open without a TTY is useless. The user can't read anything anyway
                if !atty::is(Stream::Stdout) {
                    self.exit();
                }
                self.ui_state.slow_redraw.store(true, Ordering::Relaxed);
            }
        }
    }

    fn need_redraw(&self) -> bool {
        self.ui_state.redraw.load(Ordering::Relaxed)
        // during slow redraw, there's no need to optimise the UI. The timeout per frame is already long enough. Also, slow redraw is done after the decryption is finished, so the UI doesn't have to be as optimised
            || self.ui_state.slow_redraw.load(Ordering::Relaxed)
    }

    fn draw(&self) -> Result<&Self> {
        // only draw UI if in a TTY. This allows users to redirect output to a file
        if atty::is(Stream::Stdout) {
            self.terminal.lock().unwrap().draw(|frame| {
                let layout =
                    TuiLayout::calculate(frame.size(), self.min_width_for_horizontal_layout);
                let widgets = Widgets::build(&self.app_state, &self.ui_state);

                frame.render_widget(widgets.outer_border, frame.size());

                let mut blocks_view_state = self.ui_state.blocks_view_state.lock().unwrap().clone();
                frame.render_stateful_widget(
                    widgets.original_cypher_text_view,
                    *layout.original_cypher_text_area(),
                    &mut blocks_view_state,
                );
                frame.render_stateful_widget(
                    widgets.forged_block_view,
                    *layout.forged_block_area(),
                    &mut blocks_view_state,
                );
                frame.render_stateful_widget(
                    widgets.intermediate_block_view,
                    *layout.intermediate_block_area(),
                    &mut blocks_view_state,
                );
                frame.render_stateful_widget(
                    widgets.plain_text_view,
                    *layout.plain_text_area(),
                    &mut blocks_view_state,
                );

                frame.render_widget(widgets.status_panel_border, *layout.status_panel_area());
                frame.render_widget(widgets.progress_bar, *layout.progress_bar_area());
                // no `render_stateful_widget` as `TuiLoggerWidget` doesn't implement `StatefulWidget`, but handles it custom
                frame.render_widget(widgets.logs_view, *layout.logs_area());
            })?;
        }

        Ok(self)
    }

    fn handle_user_event(&self, event: Event) {
        match event {
            Event::Key(pressed_key) => {
                match pressed_key.code {
                    KeyCode::Char(char_key) => {
                        // re-implement CTRL+C which was disabled by raw-mode
                        if char_key == 'c' && pressed_key.modifiers == KeyModifiers::CONTROL {
                            self.exit();
                        }
                    }
                    KeyCode::PageUp => {
                        self.ui_state
                            .log_view_state
                            .lock()
                            .unwrap()
                            .transition(&TuiWidgetEvent::PrevPageKey);
                    }
                    KeyCode::PageDown => {
                        self.ui_state
                            .log_view_state
                            .lock()
                            .unwrap()
                            .transition(&TuiWidgetEvent::NextPageKey);
                    }
                    KeyCode::Up => {
                        let mut state = self.ui_state.blocks_view_state.lock().unwrap();
                        let new_selection = state
                            .selected()
                            // prevent underflow which would wrap around and become more than 0
                            .map(|idx| if idx == 0 { 0 } else { max(idx - 1, 0) })
                            .unwrap_or_default();
                        state.select(Some(new_selection));
                    }
                    KeyCode::Down => {
                        let mut state = self.ui_state.blocks_view_state.lock().unwrap();
                        let new_selection = state
                            .selected()
                            .map(|idx| {
                                min(
                                    idx + 1,
                                    self.app_state.cypher_text_blocks.lock().unwrap().len() - 1,
                                )
                            })
                            .unwrap_or(1);
                        state.select(Some(new_selection));
                    }
                    _ => {}
                };
            }
            Event::Resize(cols, rows) => {
                self.cols.store(cols, Ordering::Relaxed);
                self.rows.store(rows, Ordering::Relaxed);
                self.ui_state.redraw.store(true, Ordering::Relaxed);
            }
            Event::Mouse(_) => {}
        };
    }
}
