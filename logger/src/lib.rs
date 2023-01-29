use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum Log {
    Str(&'static str),
    String(String),
}

impl From<String> for Log {
    fn from(value: String) -> Self {
        return Log::String(value);
    }
}

impl From<&'static str> for Log {
    fn from(value: &'static str) -> Self {
        return Log::Str(value);
    }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Log::Str(val) => write!(f, "{}", val),
            Log::String(val) => write!(f, "{}", val),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Log, Warn, Unexpected, Error, Info,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
enum LogSource {
    Renderer, Interface, GameLogic, Main, None,
}

impl std::fmt::Display for LogSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogSource::Main => write!(f, "[Main]"),
            LogSource::Interface => write!(f, "[Interface]"),
            LogSource::GameLogic => write!(f, "[Game Logic]"),
            LogSource::Renderer => write!(f, "[Renderer]"),
            LogSource::None => write!(f, ""),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct LogEnveloppe {
    level: LogLevel,
    log: Log,
    source: LogSource,
    time: Instant,
}

impl LogEnveloppe {
    fn new(level: LogLevel, source_int: u32, log: Log) -> LogEnveloppe {
        let source;

        match source_int {
            0 => source = LogSource::Main,
            1 => source = LogSource::Renderer,
            2 => source = LogSource::GameLogic,
            3 => source = LogSource::Interface,
            _ => source = LogSource::None,
        }

        return LogEnveloppe { level, log, source, time: Instant::now() };
    }
}

#[allow(unused)]
/// 0 => Main
/// 1 => Renderer
/// 2 => GameLogic
/// 3 => Interface
/// 4 => Asteroids
pub fn log<L>(source: u32, log: L) where L: Into<Log> {
    let log: Log = log.into();
    LOG.lock().unwrap().new_logs.push(LogEnveloppe::new(LogLevel::Log, source, log));
}

#[allow(unused)]
/// 0 => Main
/// 1 => Renderer
/// 2 => GameLogic
/// 3 => Interface
/// 4 => Asteroids
pub fn warn<L>(source: u32, log: L) where L: Into<Log> {
    let log: Log = log.into();
    LOG.lock().unwrap().new_logs.push(LogEnveloppe::new(LogLevel::Warn, source, log));
}

#[allow(unused)]
/// 0 => Main
/// 1 => Renderer
/// 2 => GameLogic
/// 3 => Interface
/// 4 => Asteroids
pub fn unexpected<L>(source: u32, log: L) where L: Into<Log> {
    let log: Log = log.into();
    LOG.lock().unwrap().new_logs.push(LogEnveloppe::new(LogLevel::Unexpected, source, log));
}

#[allow(unused)]
/// 0 => Main
/// 1 => Renderer
/// 2 => GameLogic
/// 3 => Interface
/// 4 => Asteroids
pub fn error<L>(source: u32, log: L) where L: Into<Log> {
    let log: Log = log.into();
    LOG.lock().unwrap().new_logs.push(LogEnveloppe::new(LogLevel::Error, source, log));
}

#[allow(unused)]
/// 0 => Main
/// 1 => Renderer
/// 2 => GameLogic
/// 3 => Interface
/// 4 => Asteroids
pub fn info<L>(source: u32, log: L) where L: Into<Log> {
    let log: Log = log.into();
    LOG.lock().unwrap().new_logs.push(LogEnveloppe::new(LogLevel::Info, source, log));
}

struct Logger {
    new_logs: Vec<LogEnveloppe>,
    logs: Vec<LogEnveloppe>,
}

static LOG: Mutex<Logger> = Mutex::new(Logger { new_logs: Vec::new(), logs: Vec::new() });

fn read_new_logs() -> Vec<LogEnveloppe> {
    let mut logs = LOG.lock().unwrap();

    let new_logs: Vec<_> = logs.new_logs.drain(..).collect();

    logs.logs.append(&mut new_logs.clone());

    return new_logs;
}

fn read_logs() -> Vec<LogEnveloppe> {
    return LOG.lock().unwrap().logs.clone();
}

pub struct UiLogger {
    info_enabled: bool,
    log_enabled: bool,
    warn_enabled: bool,
    unexpected_enabled: bool,
    error_enabled: bool,
    
    main_enabled: bool,
    renderer_enabled: bool,
    game_logic_enabled: bool,
    interface_enabled: bool,
    other_enabled: bool,   
}

impl UiLogger {
    pub fn new() -> UiLogger {
        return UiLogger { 
            info_enabled: true, 
            log_enabled: true, 
            warn_enabled: true, 
            unexpected_enabled: true, 
            error_enabled: true, 
            main_enabled: true, 
            renderer_enabled: true, 
            game_logic_enabled: true, 
            interface_enabled: true, 
            other_enabled: true, 
        };
    }
    
    pub fn render(&mut self, ctx: &egui::Context) {
            read_new_logs();

        let logs = read_logs();
    
        egui::Window::new("Logs").show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(600.)
                .show(ui, |ui| {
                    for log in logs {
                        if match log.level {
                            LogLevel::Info => self.info_enabled,
                            LogLevel::Log => self.log_enabled,
                            LogLevel::Unexpected => self.unexpected_enabled,
                            LogLevel::Warn => self.warn_enabled,
                            LogLevel::Error => self.warn_enabled,  
                        } {  // The level requirement is met
                            if match log.source {
                                LogSource::Main => self.main_enabled,
                                LogSource::Renderer => self.renderer_enabled,
                                LogSource::GameLogic => self.game_logic_enabled,
                                LogSource::Interface => self.interface_enabled,
                                LogSource::None => self.other_enabled,
                            } {  // The source requirement is met
                                let string = format!("{} {}", log.source, log.log);
    
                                match log.level {
                                    LogLevel::Warn => ui.colored_label(egui::Color32::YELLOW, string),
                                    LogLevel::Error => ui.colored_label(egui::Color32::RED, string),
                                    _ => ui.label(string),
                                };
                            }
                        }
                    }
                });
            
            ui.horizontal(|ui| {
                ui.label("Level Trace: ");
                ui.toggle_value(&mut self.log_enabled, "Log");
                ui.toggle_value(&mut self.info_enabled, "Info");
                ui.toggle_value(&mut self.unexpected_enabled, "Unexpected");
                ui.toggle_value(&mut self.warn_enabled, "Warn");
                ui.toggle_value(&mut self.error_enabled, "Error");
            });
            
            ui.horizontal(|ui| {
                ui.label("Source: ");
                ui.toggle_value(&mut self.main_enabled, "Main");
                ui.toggle_value(&mut self.renderer_enabled, "Renderer");
                ui.toggle_value(&mut self.game_logic_enabled, "Game Logic");
                ui.toggle_value(&mut self.interface_enabled, "Interface");
            });
        });
    }
}