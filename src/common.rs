pub enum SynthEvent {
    NoteOn(u8, f32),  // MIDI 编号, 频率
    NoteOff(u8),      // MIDI 编号
}

pub const MAX_VOICES: usize = 8;