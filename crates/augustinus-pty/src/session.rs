use std::{
    io::{Read, Write},
    sync::mpsc,
    thread,
};

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

pub struct PtySnapshot {
    pub contents: String,
    pub cursor_row: u16,
    pub cursor_col: u16,
}

pub struct PtySession {
    master: Box<dyn portable_pty::MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    parser: vt100::Parser,
    rx: mpsc::Receiver<Vec<u8>>,
    _reader_thread: thread::JoinHandle<()>,
}

impl PtySession {
    pub fn spawn_command(program: &str, args: &[&str], cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("open pty")?;

        let mut cmd = CommandBuilder::new(program);
        cmd.args(args);
        cmd.env("TERM", "xterm-256color");
        let _child = pair
            .slave
            .spawn_command(cmd)
            .with_context(|| format!("spawn command: {program}"))?;

        let mut reader = pair.master.try_clone_reader().context("clone pty reader")?;
        let writer = pair.master.take_writer().context("take pty writer")?;

        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let reader_thread = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            master: pair.master,
            writer,
            parser: vt100::Parser::new(rows, cols, 2000),
            rx,
            _reader_thread: reader_thread,
        })
    }

    pub fn spawn(shell: &str, cols: u16, rows: u16) -> Result<Self> {
        Self::spawn_command(shell, &[], cols, rows).context("spawn shell")
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        let rows = rows.max(1);
        let cols = cols.max(1);
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("resize pty")?;
        self.parser.set_size(rows, cols);
        Ok(())
    }

    pub fn poll(&mut self) {
        while let Ok(chunk) = self.rx.try_recv() {
            self.parser.process(&chunk);
        }
    }

    pub fn snapshot(&self) -> PtySnapshot {
        let screen = self.parser.screen();
        let (row, col) = screen.cursor_position();
        PtySnapshot {
            contents: screen.contents(),
            cursor_row: row,
            cursor_col: col,
        }
    }

    pub fn send_key(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(bytes) = key_to_bytes(key) {
            self.writer.write_all(&bytes).context("write key bytes")?;
            self.writer.flush().ok();
        }
        Ok(())
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes).context("write bytes")?;
        self.writer.flush().ok();
        Ok(())
    }

    pub fn send_paste(&mut self, text: &str) -> Result<()> {
        self.send_bytes(text.as_bytes())
    }
}

fn key_to_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    let mods = key.modifiers;
    match key.code {
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        KeyCode::Char(ch) => {
            if mods.contains(KeyModifiers::CONTROL) {
                Some(vec![ctrl_code(ch)])
            } else {
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                Some(s.as_bytes().to_vec())
            }
        }
        _ => None,
    }
}

fn ctrl_code(ch: char) -> u8 {
    let upper = ch.to_ascii_uppercase() as u8;
    upper & 0x1f
}
