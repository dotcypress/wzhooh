use crate::*;
use core::fmt::Write;
use embedded_hal::serial;
use rtic::Mutex;
use ushell::{control::CTRL_R, *};

pub const CMD_MAX_LEN: usize = 16;
pub const AUTOCOMPLETE: Autocomplete =
    autocomplete::StaticAutocomplete(["clear", "history", "help", "reset", "stats", "version"]);

pub type Autocomplete = autocomplete::StaticAutocomplete<6>;
pub type History = history::LRUHistory<{ CMD_MAX_LEN }, 8>;
pub type Shell = UShell<Serial, Autocomplete, History, { CMD_MAX_LEN }>;
pub type Env<'a> = crate::app::usb_irq::SharedResources<'a>;
pub type EnvResult = SpinResult<Serial, UsbError>;

pub struct Serial {
    port: SerialPort<'static, hal::usb::UsbBus>,
    dev: UsbDevice<'static, hal::usb::UsbBus>,
    scratch: [u8; 64],
    needle: usize,
    len: usize,
}

impl Serial {
    pub fn new(
        port: SerialPort<'static, hal::usb::UsbBus>,
        dev: UsbDevice<'static, hal::usb::UsbBus>,
    ) -> Self {
        Self {
            port,
            dev,
            needle: 0,
            len: 0,
            scratch: [0; 64],
        }
    }

    fn poll_usb(&mut self) -> Result<(), nb::Error<UsbError>> {
        self.needle = 0;
        self.len = 0;
        if self.dev.poll(&mut [&mut self.port]) {
            self.len = self.port.read(&mut self.scratch)?;
        }
        if self.len > 0 {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl serial::Write<u8> for Serial {
    type Error = UsbError;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        while self.port.write(&[word])? == 0 {}
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.port.flush().map_err(nb::Error::Other)
    }
}

impl serial::Read<u8> for Serial {
    type Error = UsbError;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.needle == self.len {
            self.poll_usb()?;
        }
        let res = self.scratch[self.needle];
        self.needle += 1;
        Ok(res)
    }
}

impl Environment<Serial, Autocomplete, History, UsbError, { CMD_MAX_LEN }> for Env<'_> {
    fn command(&mut self, shell: &mut Shell, cmd: &str, args: &str) -> EnvResult {
        shell.write_str(CR)?;
        match cmd {
            "clear" => shell.clear()?,
            "help" => shell.write_str(HELP)?,
            "reset" => self.reset_cmd(shell, args)?,
            "history" => self.history_cmd(shell, args)?,
            "stats" => self.stats_cmd(shell, args)?,
            "version" => write!(
                shell,
                "{}: v{}{CR}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )?,
            "" => {}
            _ => write!(shell, "[ERR] unknown command: \"{cmd}\"{CR}")?,
        }
        shell.write_str(SHELL_PROMPT)?;
        Ok(())
    }

    fn control(&mut self, shell: &mut Shell, ctrl: u8) -> EnvResult {
        match ctrl {
            CTRL_R => self.reset_cmd(shell, ""),
            _ => Ok(()),
        }
    }
}

impl Env<'_> {
    fn reset_cmd(&mut self, _shell: &mut Shell, _args: &str) -> EnvResult {
        self.display.lock(|display| display.reset());
        self.counter.lock(|counter| counter.reset());
        Ok(())
    }

    fn stats_cmd(&mut self, shell: &mut Shell, args: &str) -> EnvResult {
        match args {
            "" => {
                for track in 0..TRACKS {
                    self.print_track_stats(shell, track)?;
                }
            }
            track => match track.parse() {
                Ok(track) => self.print_track_stats(shell, track)?,
                Err(_) => write!(shell, "[ERR] invalid track number: \"{track}\"{CR}")?,
            },
        }
        Ok(())
    }

    fn history_cmd(&mut self, shell: &mut Shell, args: &str) -> EnvResult {
        match args {
            "" => {
                for track in 0..TRACKS {
                    self.print_track_history(shell, track)?;
                }
            }
            track => match track.parse() {
                Ok(track) => self.print_track_history(shell, track)?,
                Err(_) => write!(shell, "[ERR] invalid track number: \"{track}\"{CR}")?,
            },
        }
        Ok(())
    }

    fn print_track_stats(&mut self, shell: &mut Shell, track: Track) -> EnvResult {
        if let Some(stats) = self.counter.lock(|counter| counter.stats(track)) {
            let empty = Duration::from_ticks(0);
            let laps = stats.laps();
            let last = stats.last().unwrap_or(&empty);
            let best = stats.best().unwrap_or(&empty);
            write!(
                shell,
                "track #{track}; laps: {laps}; last: {last}; best: {best};{CR}"
            )?;
        }
        Ok(())
    }

    fn print_track_history(&mut self, shell: &mut Shell, track: Track) -> EnvResult {
        if let Some(stats) = self.counter.lock(|counter| counter.stats(track)) {
            write!(shell, "track #{track}; history: ")?;
            for lap in stats.history() {
                write!(shell, "{lap}, ")?;
            }
            write!(shell, "{CR}")?;
        }
        Ok(())
    }
}

const CR: &str = "\r\n";
const SHELL_PROMPT: &str = "\x1b[35m» \x1b[0m";
const HELP: &str = "\
\x1b[34m _       __      __                __\r\n\
| |     / /___  / /_  ____  ____  / /_\r\n\
| | /| / /_  / / __ \\/ __ \\/ __ \\/ __ \\\x1b[33m\r\n\
| |/ |/ / / /_/ / / / /_/ / /_/ / / / /\r\n\
|__/|__/ /___/_/ /_/\\____/\\____/_/ /_/\
\x1b[0m\r\n\r\n\
COMMANDS:\r\n\
\x20 reset              Reset counter\r\n\
\x20 stats [track]      Print track(s) stats (0, 1, 2)\r\n\
\x20 history [track]    Print track(s) history (0, 1, 2)\r\n\
\x20 help               Print help message\r\n\
\x20 version            Print version information\r\n\
\x20 clear              Clear screen\r\n\r\n\
CONTROL KEYS:\r\n\
\x20 Ctrl+R             Reset counter\r\n\r\n\
LINKS:\r\n\
\x20 \x1b[32m Dashboard:  \x1b[34mhttps://wzhooh.vercel.app \x1b[0m\r\n\
\x20 \x1b[32m Repository: \x1b[34mhttps://github.com/dotcypress/wzhooh \x1b[0m\r\n\r\n";
