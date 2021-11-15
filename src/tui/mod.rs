mod layout;
pub mod ui_update;
mod widgets;

use std::{
    io::{self},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex,
    },
    thread::sleep,
    time::Duration,
};

use anyhow::{Context, Result};
use tui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::block::{
    block_size::{BlockSize, BlockSizeTrait},
    Block,
};

use self::{layout::TuiLayout, ui_update::UiUpdate, widgets::Widgets};

const FRAME_SLEEP_MS: u64 = 20;

pub struct Tui {
    // the usage of a mutex here could be prevented by separating `Terminal` from `Tui`, it's only needed in the draw thread. However, the overhead of handling the mutex should be so small (especially given that only the draw thread accesses it) should be so small that it's unneeded.
    terminal: Mutex<Terminal<CrosstermBackend<io::Stdout>>>,
    min_width_for_horizontal_layout: u16,

    ui_state: UiState,
    app_state: AppState,
}

struct UiState {
    pub running: AtomicBool,
    pub slow_redraw: AtomicBool,
    pub redraw: AtomicBool,
    pub previous_terminal_size: Mutex<Rect>,
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
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear().context("Clearing terminal failed")?;

        let terminal_size = terminal.size().context("Getting terminal size failed")?;

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
                previous_terminal_size: Mutex::new(terminal_size),
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

    pub fn main_loop(&self) -> Result<()> {
        while self.ui_state.running.load(Ordering::Relaxed) {
            let terminal_size = self
                .terminal
                .lock()
                .unwrap()
                .size()
                .context("Getting terminal size failed")?;

            if self.need_redraw(&terminal_size) {
                self.draw().context("Drawing UI failed")?;
                self.ui_state.redraw.store(false, Ordering::Relaxed);
                *self.ui_state.previous_terminal_size.lock().unwrap() = terminal_size;
            }

            sleep(Duration::from_millis(FRAME_SLEEP_MS));
        }

        // done, but keep window open. Redraw to prevent user input overwriting TUI
        while self.ui_state.slow_redraw.load(Ordering::Relaxed) {
            let terminal_size = self
                .terminal
                .lock()
                .unwrap()
                .size()
                .context("Getting terminal size failed")?;

            self.draw().context("Drawing UI failed")?;
            *self.ui_state.previous_terminal_size.lock().unwrap() = terminal_size;

            sleep(Duration::from_millis(3 * FRAME_SLEEP_MS));
        }

        // 1 last draw to ensure errors are displayed
        self.draw().context("Drawing UI failed").map(|_| ())
    }

    pub fn update(&self, update: UiUpdate) {
        match update {
            UiUpdate::ForgedBlock((forged_block, block_to_decrypt_idx)) => {
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
            UiUpdate::ForgedBlockWip((forged_block, block_to_decrypt_idx)) => {
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
            UiUpdate::ProgressUpdate => {
                self.app_state
                    .bytes_finished
                    .fetch_add(1, Ordering::Relaxed);
            }
            UiUpdate::SlowRedraw => {
                self.ui_state.slow_redraw.store(true, Ordering::Relaxed);
                self.ui_state.running.store(false, Ordering::Relaxed);
            }
        }

        self.ui_state.redraw.store(true, Ordering::Relaxed);
    }

    fn need_redraw(&self, terminal_size: &Rect) -> bool {
        self.ui_state.redraw.load(Ordering::Relaxed)
            || *terminal_size != *self.ui_state.previous_terminal_size.lock().unwrap()
    }

    fn draw(&self) -> Result<&Self> {
        self.terminal.lock().unwrap().draw(|frame| {
            let layout = TuiLayout::calculate(frame.size(), self.min_width_for_horizontal_layout);
            let widgets = Widgets::build(&self.app_state);

            frame.render_widget(widgets.outer_border, frame.size());

            frame.render_widget(
                widgets.original_cypher_text_view,
                *layout.original_cypher_text_area(),
            );
            frame.render_widget(widgets.forged_block_view, *layout.forged_block_area());
            frame.render_widget(
                widgets.intermediate_block_view,
                *layout.intermediate_block_area(),
            );
            frame.render_widget(widgets.plaintext_view, *layout.plaintext_area());

            frame.render_widget(widgets.status_panel_border, *layout.status_panel_area());
            frame.render_widget(widgets.progress_bar, *layout.progress_bar_area());
            frame.render_widget(widgets.logs_view, *layout.logs_area());
        })?;

        Ok(self)
    }
}
