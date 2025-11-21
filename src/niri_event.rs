// logging

// options struct
use crate::options::Options;

// std lib
use std::fmt;

// system cmd
use std::process::Command;

// global hashmap with Mutex
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
lazy_static! {
    // hashmap to store windows and thier layouts
    static ref HASHMAP: Mutex<HashMap<u64, u64>> = Mutex::new(HashMap::new());
    // vec to store layouts (long names)
    pub static ref LAYOUTS: Mutex<Vec<String>> =  Mutex::new(Vec::new());
    // last active window address
    static ref ACTIVE_WINDOW: Mutex<u64> = Mutex::new(0);
    // last active window class
    static ref ACTIVE_APP_ID: Mutex<String> = Mutex::new(String::new());
    // current active layout index
    static ref ACTIVE_LAYOUT: Mutex<u64> = Mutex::new(0);
}

macro_rules! get_u64_or_return {
    ($obj:expr) => {
        match $obj.as_u64() {
            Some(id) => id,
            None => {
                log::warn!("Event doesn't have a window id");
                return;
            }
        }
    };
}

pub fn handle_layouts(json: Value) -> i16 {
    if json.is_null() || json["names"].is_null() || json["current_idx"].is_null() {
        return -1;
    }
    let layouts: Vec<_> = match json["names"].as_array() {
        Some(vec) => vec.iter().map(Value::to_string).collect(),
        None => return -1,
    };

    if layouts.is_empty() {
        return 0;
    }
    let current_layout_index = match json["current_idx"].as_u64() {
        Some(id) => id as usize,
        None => {
            println!("Failed to get current layout from niri");
            return -1;
        }
    };
    let len = layouts.len();
    for layout in layouts {
        fullfill_layouts_list(layout);
    }
    if let Ok(mut active_layout) = ACTIVE_LAYOUT.lock() {
        *active_layout = current_layout_index as u64
    }

    len as i16
}

// work with messages from niri event stream
pub fn event(name: &str, data: &Value, options: &Options) {
    log::debug!("E:'{}' D:'{}'", name, data);
    let object = match data.as_object() {
        Some(object) => object,
        None => {
            log::debug!("No active window (empty workspace), maintaining current layout");
            return;
        }
    };

    if name == "WindowFocusChanged" {
        let id = get_u64_or_return!(object["id"]);
        if let Ok(mut active_window) = ACTIVE_WINDOW.lock() {
            *active_window = id;
        }
        let map = match HASHMAP.lock() {
            Ok(map) => map,
            Err(_) => return,
        };
        match map.get(&id) {
            Some(index) => {
                log::debug!("{}: {}", id, index);
                // only change layout if it's different from current
                let current_layout = match ACTIVE_LAYOUT.lock() {
                    Ok(layout) => *layout,
                    Err(_) => return,
                };
                if current_layout != *index {
                    change_layout(*index);
                } else {
                    log::debug!("Layout {} already active, skipping change", index);
                }
            }
            None => {
                drop(map);
                log::debug!("added id: {}", id);
                // check if we have default layout for this window
                let default_layouts = &options.default_layouts;

                for (index, app_ids) in default_layouts.iter() {
                    for app_id in app_ids.iter() {
                        if let Ok(active_app_id) = ACTIVE_APP_ID.lock() {
                            for app_active_id in active_app_id.split(",") {
                                if app_active_id.eq(app_id) {
                                    log::debug!(
                                        "Found default layout {} for window {}",
                                        index,
                                        app_active_id
                                    );
                                    // Drop active_class before acquiring new mutex
                                    std::mem::drop(active_app_id);
                                    if let Ok(mut map) = HASHMAP.lock() {
                                        map.insert(id, *index);
                                        // map will be dropped automatically
                                    }
                                    // only change layout if it's different from current
                                    let current_layout = match ACTIVE_LAYOUT.lock() {
                                        Ok(layout) => *layout,
                                        Err(_) => return,
                                    };
                                    if current_layout != *index {
                                        change_layout(*index);
                                    } else {
                                        log::debug!(
                                            "Layout {} already active, skipping change",
                                            index
                                        );
                                    }
                                    return;
                                }
                            }
                        }
                    }
                }
                // set layout to default one (index 0)
                if let Ok(mut map) = HASHMAP.lock() {
                    map.insert(id, 0);
                    // map will be dropped automatically
                }
                // only change layout if it's different from current
                let current_layout = match ACTIVE_LAYOUT.lock() {
                    Ok(layout) => *layout,
                    Err(_) => return,
                };
                if current_layout != 0 {
                    change_layout(0);
                } else {
                    log::debug!("Layout 0 already active, skipping change");
                }
            }
        }
        return;
    }

    if name == "WindowClosed" {
        let id = get_u64_or_return!(object["id"]);
        if let Ok(mut map) = HASHMAP.lock() {
            map.remove(&id);
        }
        return;
    }

    if name == "KeyboardLayoutSwitched" {
        // params ex: keychron-keychron-k2,English (US)
        // params ex with variant: at-translated-set-2-keyboard,English (US, intl., with dead keys)
        let id = get_u64_or_return!(object["idx"]);
        let layout_vec = match LAYOUTS.lock() {
            Ok(vec) => vec,
            Err(_) => return,
        };
        let layout = &layout_vec[id as usize];

        let active_layout = match ACTIVE_LAYOUT.lock() {
            Ok(layout) => *layout,
            Err(_) => return,
        };
        if active_layout == id {
            log::debug!("Layout {} is current", layout);
            return;
        }
        if let Ok(mut active_layout_ref) = ACTIVE_LAYOUT.lock() {
            *active_layout_ref = id;
        }
        let addr = match ACTIVE_WINDOW.lock() {
            Ok(window) => window.clone(),
            Err(_) => return,
        };

        if let Ok(mut map) = HASHMAP.lock() {
            map.insert(addr.clone(), id);
            log::debug!("Saved layout {} with index {} on addr {}", layout, id, addr);
        }

        return;
    }
}
#[derive(Debug)]
pub struct CommandFailed {}
impl fmt::Display for CommandFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Command returned error")
    }
}

// run cli command `niri msg` with given args
pub fn niri_msg_raw(argv: &[&str]) -> Command {
    let mut command = Command::new("niri");
    command.arg("msg").args(argv);
    command
}
pub fn niri_msg(argv: &[&str]) -> Result<String, CommandFailed> {
    let output = niri_msg_raw(argv)
        .output()
        .expect("failed to execute process");
    match output.status.code() {
        Some(code) => {
            log::debug!("Status code is {}", code);
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        None => Err(CommandFailed {}),
    }
}

// updates layout on all active keyboards
// Note: you need to manualy change layout on keyboard to add it into this list
fn change_layout(index: u64) {
    log::debug!("layout change {}", index);
    if let Ok(mut active_layout) = ACTIVE_LAYOUT.lock() {
        *active_layout = index;
    }

    let new_index = &index.to_string();
    let e = niri_msg(&["action", "switch-layout", new_index]);
    match e {
        Ok(code) => {
            log::debug!("Layout changed index:{} exit_code:{}", new_index, code);
        }
        Err(_e) => {
            log::warn!("Failed to switch layout");
        }
    }
}

// we have to fill this layouts list on go
pub fn fullfill_layouts_list(long_name: String) {
    // add kb long name to LAYOUTS if not there
    let mut layout_vec = match LAYOUTS.lock() {
        Ok(vec) => vec,
        Err(_) => return,
    };

    // skip blacklisted layouts
    let blacklisted_layouts = ["wvkbd"];
    for layout in blacklisted_layouts.iter() {
        if layout.eq(&long_name) {
            log::debug!("Layout blacklisted: {}", long_name);
            return;
        }
    }

    if !layout_vec.contains(&long_name) {
        layout_vec.push(long_name.clone());
        log::debug!("Layout stored: {}", long_name);
    }
}
