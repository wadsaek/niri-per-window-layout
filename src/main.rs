use std::io;
use std::io::BufRead; // read unix socket
use std::io::BufReader;
use std::process::Stdio;

mod niri_event; // work with message from socket
use niri_event::{event, fullfill_layouts_list, niri_msg};

mod options;
// read options.toml
use options::read_options;

mod single; // a struct representing one running instance
use single::SingleInstance;

use serde_json::Value;

use crate::niri_event::handle_layouts;
use crate::niri_event::niri_msg_raw;

// listen on Niri event stream
fn listen() -> io::Result<()> {
    let child = match niri_msg_raw(&["-j","event-stream"])
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            println!("Couldn't connect: {e:?}");
            return Err(e);
        }
    };
    let output = match child.stdout {
        Some(stdout) => stdout,
        None => {
            println!("Couldn't get child's stdout");
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "no stdout"));
        }
    };
    let mut reader = BufReader::new(output);
    let opt = read_options();
    'main_loop :loop {
        // read message from socket
        let mut buf: Vec<u8> = vec![];
        let readed = match reader.read_until(b'\n', &mut buf) {
            Ok(size) => size,
            Err(e) => {
                log::warn!("Error reading event: {}", e);
                break Err(e);
            }
        };
        if readed == 0 {
            break Ok(());
        }
        let data = String::from_utf8_lossy(&buf);
        if data.trim().is_empty() { continue 'main_loop}
        let data: Value = match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(e) => {
                println!("Failed to parse an event, {e}");
                return Err(e.into());
            }
        };
        let (name, data) = data
            .as_object()
            .ok_or(io::Error::new(io::ErrorKind::InvalidData, "invalid data"))?
            .iter()
            .next()
            .ok_or(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "event contains no fields",
            ))?;
         event(name, data, &opt)
    }
}

// get layouts listed in niri conf file and push the default one to CURRENT_LAYOUT
// return -1 if failed
fn get_kb_layouts_count() -> i16 {
    // get layouts list from hyprctl cli call
    match niri_msg(&["-j", "keyboard-layouts"]) {
        Ok(output) => {
            log::debug!("keyboard-layouts: {}", output);
            // parse the string from stdin into serde_json::Value
            let json: Value = match serde_json::from_str(&output) {
                Ok(json) => json,
                Err(e) => {
                    log::warn!("Failed to parse JSON: {}", e);
                    return -1;
                }
            };
            handle_layouts(json)
        }
        Err(_e) => {
            println!("Failed to get layouts from niri");
            0
        }
    }
}

// try to get kb layouts count 5 times with 1 sec delay
fn get_kb_layouts_count_retry() -> i16 {
    let mut count = 0;
    loop {
        let layouts_found = get_kb_layouts_count();
        if layouts_found > -1 {
            return layouts_found;
        }
        count += 1;
        if count > 5 {
            return -1;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn kb_file_isset() -> bool {
    // TODO: find a way to get the kb file

    false

    // match hyprctl(["getoption", "input:kb_file", "-j"].to_vec()) {
    //     Ok(output) => {
    //         log::debug!("input:kb_file: {}", output);
    //         // parse the string from stdin into serde_json::Value
    //         let json: Value = match serde_json::from_str(&output) {
    //             Ok(json) => json,
    //             Err(e) => {
    //                 log::warn!("Failed to parse JSON: {}", e);
    //                 return false;
    //             }
    //         };
    //         if json["str"].is_null() {
    //             return false;
    //         }
    //         let value = str::replace(json["str"].to_string().trim(), "\"", "");
    //         value != "[[EMPTY]]"
    //     }
    //     Err(_e) => {
    //         println!("Failed to get option from hyprctl");
    //         false
    //     }
    // }
}

// get default layout from cli command "hyprctl devices -j"

// read env variables and listen Hyprland unix socket
fn main() {
    // to see logs in output: add env RUST_LOG='debug'
    env_logger::init();
    let instance_sock = SingleInstance::new("niri-per-window-layout").unwrap();

    if !instance_sock.is_single() {
        println!("Another instance is running.");
        std::process::exit(1);
    }
    // this program make sense if you have 2+ layouts
    let layouts_found = get_kb_layouts_count_retry();

    if layouts_found < 2 && !kb_file_isset() {
        println!("Fatal error: You need to configure layouts on Niri");
        println!("Add kb_layout option to input group in your config.kdl");
        println!("You don't need this program if you have only 1 keyboard layout");
        std::process::exit(1);
    }

    match listen() {
        Ok(()) => {}
        Err(e) => log::warn!("Error {e}"),
    }
    std::process::exit(1);
}
