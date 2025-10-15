use anyhow::Result;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode, Clear, ClearType}, cursor, style::Print};
use std::env;
use std::fs;
use std::io::{self, Write};

struct Buffer {
    filename: Option<String>,
    lines: Vec<String>,
    modified: bool,
    readonly: bool,
}

impl Buffer {
    fn new(filename: Option<String>) -> Self {
        let mut lines = vec![String::new()];
        let mut readonly = false;
        if let Some(ref f) = filename {
            if let Ok(text) = fs::read_to_string(f) {
                lines = text.lines().map(|s| s.to_string()).collect();
                if lines.is_empty() { lines.push(String::new()); }
                if fs::metadata(f).map(|m| { m.permissions().readonly() }).unwrap_or(false) {
                    readonly = true;
                }
            }
        }
        Buffer { filename, lines, modified: false, readonly }
    }

    fn save(&mut self) -> Result<String> {
        if self.filename.is_none() {
            return Ok(String::from("No filename"));
        }
        if self.readonly {
            return Ok(String::from("File is read-only"));
        }
        let fname = self.filename.as_ref().unwrap();
        let mut f = fs::File::create(fname)?;
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        self.modified = false;
        Ok(String::from("Saved"))
    }
}

fn draw_screen(buf: &Buffer, cursor_y: usize, cursor_x: usize, msg: &str) -> Result<()> {
    execute!(io::stdout(), Clear(ClearType::All))?;
    for (i, line) in buf.lines.iter().enumerate() {
        println!("{}", line);
    }
    // status
    let name = buf.filename.as_ref().map(|s| s.clone()).unwrap_or(String::from("[No Name]"));
    let status = format!(" {} - {} lines {}", name, buf.lines.len(), if buf.modified {"(modified)"} else {""});
    execute!(io::stdout(), cursor::MoveTo(0, (buf.lines.len()+1) as u16), Print(status))?;
    if !msg.is_empty() {
        execute!(io::stdout(), cursor::MoveTo(0, (buf.lines.len()+2) as u16), Print(msg))?;
    }
    // move cursor
    execute!(io::stdout(), cursor::MoveTo(cursor_x as u16, cursor_y as u16))?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 { Some(args[1].clone()) } else { None };
    let mut buf = Buffer::new(filename);
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut cursor_y = 0usize;
    let mut cursor_x = 0usize;
    let mut msg = String::new();
    draw_screen(&buf, cursor_y, cursor_x, &msg)?;
    loop {
        match read()? {
            Event::Key(KeyEvent { code, modifiers, .. }) => {
                match code {
                    KeyCode::Char('q') => {
                        if buf.modified {
                            msg = String::from("Modified. Press q again to quit without saving or s to save.");
                        } else { break; }
                    }
                    KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                        msg = buf.save()?;
                    }
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        // exit
                        break;
                    }
                    KeyCode::Char(ch) => {
                        let line = &mut buf.lines[cursor_y];
                        line.insert(cursor_x, ch);
                        cursor_x += 1;
                        buf.modified = true;
                    }
                    KeyCode::Enter => {
                        let line = buf.lines[cursor_y].clone();
                        let (left, right) = line.split_at(cursor_x);
                        buf.lines[cursor_y] = left.to_string();
                        buf.lines.insert(cursor_y+1, right.to_string());
                        cursor_y +=1; cursor_x = 0; buf.modified = true;
                    }
                    KeyCode::Backspace => {
                        if cursor_x > 0 {
                            let line = &mut buf.lines[cursor_y];
                            line.remove(cursor_x-1);
                            cursor_x -=1;
                            buf.modified = true;
                        } else if cursor_y > 0 {
                            let prev_len = buf.lines[cursor_y-1].len();
                            let cur = buf.lines.remove(cursor_y);
                            cursor_y -=1; cursor_x = prev_len;
                            let prev = &mut buf.lines[cursor_y];
                            prev.push_str(&cur);
                            buf.modified = true;
                        }
                    }
                    KeyCode::Left => { if cursor_x>0 { cursor_x -=1 } else if cursor_y>0 { cursor_y -=1; cursor_x = buf.lines[cursor_y].len(); } }
                    KeyCode::Right => { if cursor_x < buf.lines[cursor_y].len() { cursor_x +=1 } else if cursor_y < buf.lines.len()-1 { cursor_y +=1; cursor_x = 0 } }
                    KeyCode::Up => { if cursor_y>0 { cursor_y -=1; cursor_x = cursor_x.min(buf.lines[cursor_y].len()); } }
                    KeyCode::Down => { if cursor_y+1 < buf.lines.len() { cursor_y +=1; cursor_x = cursor_x.min(buf.lines[cursor_y].len()); } }
                    _ => {}
                }
            }
            _ => {}
        }
        draw_screen(&buf, cursor_y, cursor_x, &msg)?;
    }
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
