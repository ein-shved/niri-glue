use crate::{Runner, Format};

use std::{io::{self, Error}, collections::HashMap};
use clap::Parser;
use niri_ipc::{socket::Socket, Request, Event, KeyboardLayouts, Action, LayoutSwitchTarget};
use regex::Regex;

fn parse_key_val(s: &str) -> Result<(String, String), Error>
{
    if let Some((var, value)) = s.split_once('=') {
        Ok((var.to_owned(), value.to_owned()))
    } else {
        Err(Error::new(io::ErrorKind::InvalidInput,
                format!("invalid KEY=value: no `=` found in `{}`", s)))
    }
}

/// Keyboard layout monitor.
///
/// Produces to stdout messages about keyboard layout actions.
#[derive(Parser, Debug, Clone)]
#[command(about, long_about)]
pub struct Layout {
    /// Aliases for languages
    ///
    /// Each argument must have format `Source=ALIAS`
    #[arg(short, long="alias", value_parser = parse_key_val, number_of_values = 1)]
    aliases: Vec<(String, String)>
}

struct LayoutRunner {
    aliases: HashMap<String, String>,
    layouts: KeyboardLayouts,
    re: Regex,
}

impl Runner for Layout {
    fn run(self, socket: Socket, format: Format) {
        let runner = LayoutRunner::new(self, format);
        runner.run(socket)
    }
}

impl LayoutRunner {
    fn new(config: Layout, format: Format) -> Self {
        assert_eq!(format, Format::Waybar, "Only Waybar format of layout supported");
        let aliases = config.aliases.into_iter().map(|(k,v)| (k.to_lowercase(), v)).collect::<HashMap<String, String>>();
        Self {
            aliases,
            layouts: KeyboardLayouts{ names: Default::default(), current_idx: 0 },
            re: Regex::new(r"^(?<full>\S+)\s*(\((?<alias>\S+)\))?$").unwrap(),
        }
    }

    fn process_event(&mut self, ev:Event)
    {
        match ev {
            Event::KeyboardLayoutsChanged { keyboard_layouts } => self.changed(keyboard_layouts),
            Event::KeyboardLayoutSwitched { idx } => self.switched(idx),
            _ => ()
        }
    }

    fn changed(&mut self, layouts: KeyboardLayouts)
    {
        self.layouts.names = layouts.names.into_iter().map(|name|self.alias_layout(&name)).collect();
        self.switched(layouts.current_idx)
    }

    fn switched(&mut self, idx: u8)
    {
        if let Some(layout) = self.layouts.names.get(usize::from(idx)) {
            println!("{{ \"text\": \"{layout}\", \"class\": \"layout\" }}");
            self.layouts.current_idx = idx;
        }
    }

    fn alias_layout(&self, name: &str) -> String
    {
        if let Some(caps) = self.re.captures(name) {
            let mut alias: Option<&str> = caps.name("alias").map(|m| m.as_str());
            if alias == Some("") {
                alias = None
            }
            let alias_alias = self.alias_for(alias);
            if let Some(alias_alias) = alias_alias {
                alias_alias.into()
            } else {
                let mut full = caps.name("full").map(|m| m.as_str());
                if full == Some("") {
                    full = None
                }
                let full_alias = self.alias_for(full);

                if let Some(full_alias) = full_alias {
                    full_alias.into()
                } else {
                    if let Some(alias) = alias {
                        alias.into()
                    } else {
                        if let Some(full) = full {
                            full.into()
                        } else {
                            name.into()
                        }
                    }
                }
            }
        } else {
            name.into()
        }
    }
    fn alias_for<'a>(&'a self, name: Option<&'a str>) -> Option<&'a str>
    {
        if let Some(name) = name {
            self.aliases.get(&name.to_lowercase()).map(String::as_str)
        } else {
            None
        }
    }
    fn run(mut self, socket: Socket) {
        let (_, mut functor) = socket.send(Request::EventStream).unwrap();
        loop {
            let event = functor().unwrap();
            self.process_event(event);
        }
    }
}

/// Keyboard layout switcher.
///
/// Switches the keyboard layout.
#[derive(Parser, Debug, Clone)]
#[command(about, long_about)]
pub struct SwitchLayout {
}

impl Runner for SwitchLayout {
    fn run(self, socket: Socket, _format: Format) {
        let _ = socket.send(Request::Action(Action::SwitchLayout { layout: LayoutSwitchTarget::Next }));
    }
}
