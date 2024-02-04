use crossterm::{
    ExecutableCommand,
    QueueableCommand,
    terminal,
    cursor,
    style::{
        self,
        Stylize,
        Color,
        Attribute,
        SetBackgroundColor,
        SetForegroundColor
    },
    cursor::position,
    event::{
        poll,
        read,
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode
    },
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode
    },
    Result,
};
use std::{io::stdout};
use std::io::{self, Write};
use serde_json::Value;

pub fn start_list_interface(stdout: &mut io::Stdout, json: Value) -> Result<()> {
    log::trace!("In list to terminal");

    let size = terminal::size()?;
    let terminal_y = &size.1;
    let terminal_x = &size.0;
    log::debug!("Terminal dimensions: {} x {}", terminal_x, terminal_y);

    let padding_x: u16 = 1;
    let padding_y: u16 = 1; // does tmux status bar take up one row
    log::debug!("padding_x: {}, padding_y: {}", padding_x, padding_y);

    let items = json["items"].as_array() else {
        log::error!("items is not an array");
        return Ok(());
    };

    log::debug!("{:?}", items);

    Ok(())
}
