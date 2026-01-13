pub enum Param {
    Attack,
    Decay,
    Sustain,
    Release,
    ModRange,
    LfoFreq, // 新增：LFO 频率参数
}

pub enum SynthEvent {
    NoteOn(u8, f32),
    NoteOff(u8),
    ParamChange(Param, f32),
    ControlChange(u8, u8), // 新增：处理 MIDI 控制信息（CC）
}

pub const MAX_VOICES: usize = 8;