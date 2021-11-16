mod layout;
pub mod ui_update;
mod widgets;

use std::{
    cmp::{max, min},
    io::{self},
    process,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex,
    },
    thread::sleep,
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use crate::block::{
    block_size::{BlockSize, BlockSizeTrait},
    Block,
};

use self::{layout::TuiLayout, ui_update::UiEvent, widgets::Widgets};

const FRAME_SLEEP_MS: u64 = 20;

pub struct Tui {
    // the usage of a mutex here could be prevented by separating `Terminal` from `Tui`, it's only needed in the draw thread. However, the overhead of handling the mutex should be so small (especially given that only the draw thread accesses it) should be so small that it's unneeded.
    terminal: Mutex<Terminal<CrosstermBackend<io::Stdout>>>,
    min_width_for_horizontal_layout: u16,

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
    original_cypher_text_blocks: Vec<Block>,
    no_iv: bool,

    // for progress calculation
    bytes_to_finish: usize,
    bytes_finished: AtomicUsize,

    forged_blocks: Mutex<Vec<Block>>,
    intermediate_blocks: Mutex<Vec<Block>>,
    plaintext_blocks: Mutex<Vec<Block>>,
}

impl Tui {
    pub fn new(
        block_size: &BlockSize,
        original_cypher_text_blocks: Vec<Block>,
        no_iv: bool,
    ) -> Result<Self> {
        enable_raw_mode()?;

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear().context("Clearing terminal failed")?;

        let amount_original_blocks = original_cypher_text_blocks.len();
        let default_blocks = vec![Block::new(block_size); amount_original_blocks - 1];

        let tui = Self {
            terminal: Mutex::new(terminal),
            // enough space to display 2 tables of hex encoded blocks + padding
            min_width_for_horizontal_layout: (**block_size as usize * 12) as u16,

            ui_state: UiState {
                running: AtomicBool::new(true),
                slow_redraw: AtomicBool::new(false),
                redraw: AtomicBool::new(true),

                log_view_state: Mutex::new(TuiWidgetState::new()),
                blocks_view_state: Mutex::new(TableState::default()),
            },

            app_state: AppState {
                original_cypher_text_blocks,
                no_iv,

                bytes_to_finish: (amount_original_blocks - 1) * (**block_size as usize),
                bytes_finished: AtomicUsize::new(0),

                forged_blocks: Mutex::new({
                    let mut blocks = default_blocks.clone();
                    blocks.push(Block::new(block_size));
                    blocks
                }),
                intermediate_blocks: Mutex::new({
                    let mut blocks = vec![Block::new(block_size)];
                    blocks.extend(default_blocks.clone());
                    blocks
                }),
                plaintext_blocks: Mutex::new({
                    let mut blocks = vec![Block::new(block_size)];
                    blocks.extend(default_blocks);
                    blocks
                }),
            },
        };

        Ok(tui)
    }

    pub fn exit(&self, exit_code: i32) {
        disable_raw_mode().expect("Disabling raw terminal mode failed");
        self.terminal
            .lock()
            .unwrap()
            .show_cursor()
            .expect("Showing cursor failed");
        process::exit(exit_code);
    }

    pub fn main_loop(&self) -> Result<()> {
        while self.ui_state.running.load(Ordering::Relaxed) {
            self.handle_user_event()?;

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
        self.draw().context("Drawing UI failed").map(|_| ())
    }

    pub fn handle_application_event(&self, event: UiEvent) {
        match event {
            UiEvent::ForgedBlockUpdate((forged_block, block_to_decrypt_idx)) => {
                let intermediate =
                    &forged_block ^ &Block::new_incremental_padding(&forged_block.block_size());

                let plaintext = &intermediate
                    ^ &self.app_state.original_cypher_text_blocks[block_to_decrypt_idx - 1];

                self.app_state.forged_blocks.lock().unwrap()[block_to_decrypt_idx - 1] =
                    forged_block;
                self.app_state.intermediate_blocks.lock().unwrap()[block_to_decrypt_idx] =
                    intermediate;
                self.app_state.plaintext_blocks.lock().unwrap()[block_to_decrypt_idx] = plaintext;
            }
            UiEvent::ForgedBlockWipUpdate((forged_block, block_to_decrypt_idx)) => {
                let intermediate =
                    &forged_block ^ &Block::new_incremental_padding(&forged_block.block_size());

                let plaintext = &intermediate
                    ^ &self.app_state.original_cypher_text_blocks[block_to_decrypt_idx - 1];

                // `try_lock` as updating isn't critical. This is mainly for visuals
                if let Ok(mut blocks) = self.app_state.forged_blocks.try_lock() {
                    blocks[block_to_decrypt_idx - 1] = forged_block;
                }
                if let Ok(mut blocks) = self.app_state.intermediate_blocks.try_lock() {
                    blocks[block_to_decrypt_idx] = intermediate;
                }
                if let Ok(mut blocks) = self.app_state.plaintext_blocks.try_lock() {
                    blocks[block_to_decrypt_idx] = plaintext;
                }
            }
            // due to concurrency, we can't just send which blocks was finished. So this acts as a "ping" to indicate that a byte was locked
            UiEvent::ProgressUpdate => {
                self.app_state
                    .bytes_finished
                    .fetch_add(1, Ordering::Relaxed);
            }
            UiEvent::SlowRedraw => {
                self.ui_state.slow_redraw.store(true, Ordering::Relaxed);
            }
        }

        self.ui_state.redraw.store(true, Ordering::Relaxed);
    }

    fn need_redraw(&self) -> bool {
        self.ui_state.redraw.load(Ordering::Relaxed)
        // during slow redraw, there's no need to optimise the UI. The timeout per frame is already long enough. Also, slow redraw is done after the decryption is finished, so the UI doesn't have to be as optimised
            || self.ui_state.slow_redraw.load(Ordering::Relaxed)
    }

    fn draw(&self) -> Result<&Self> {
        self.terminal.lock().unwrap().draw(|frame| {
            let layout = TuiLayout::calculate(frame.size(), self.min_width_for_horizontal_layout);
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
                widgets.plaintext_view,
                *layout.plaintext_area(),
                &mut blocks_view_state,
            );

            frame.render_widget(widgets.status_panel_border, *layout.status_panel_area());
            frame.render_widget(widgets.progress_bar, *layout.progress_bar_area());
            // no `render_stateful_widget` as `TuiLoggerWidget` doesn't implement `StatefulWidget`, but handles it custom
            frame.render_widget(widgets.logs_view, *layout.logs_area());
        })?;

        Ok(self)
    }

    fn handle_user_event(&self) -> Result<()> {
        if event::poll(Duration::from_millis(0))? {
            let event = event::read()?;
            match event {
                Event::Key(pressed_key) => {
                    // re-implement CTRL+C which was disabled by raw-mode
                    if pressed_key.modifiers == KeyModifiers::CONTROL
                        && pressed_key.code == KeyCode::Char('c')
                    {
                        self.exit(0);
                    }

                    match pressed_key.code {
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
                                        self.app_state.original_cypher_text_blocks.len() - 1,
                                    )
                                })
                                .unwrap_or(1);
                            state.select(Some(new_selection));
                        }
                        _ => {}
                    };
                }
                Event::Resize(_, _) => self.ui_state.redraw.store(true, Ordering::Relaxed),
                Event::Mouse(_) => {}
            };
        }

        Ok(())
    }
}
