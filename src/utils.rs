pub enum LogLevel {
    Prompt,
    Info,
    Warning,
    Error,
    Midi,
}

pub fn msg(level: LogLevel, message: impl Into<String>) {
    let prefix = match level {
        LogLevel::Prompt   => "",
        LogLevel::Info     => "[INFO]",
        LogLevel::Warning  => "[WARNING]",
        LogLevel::Error    => "[ERROR]",
        LogLevel::Midi     => "[MIDI]",

    };
    
    println!("{} {}", prefix, message.into());
}

pub fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}