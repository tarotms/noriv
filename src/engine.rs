use crate::common::{SynthEvent, MAX_VOICES, Param};
use crate::voice::{Voice, VoiceState};

pub struct SynthEngine {
    voices: Vec<Voice>,
    sample_rate: f32,
}

impl SynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        let mut voices = Vec::with_capacity(MAX_VOICES);
        for _ in 0..MAX_VOICES {
            voices.push(Voice::new());
        }
        Self { voices, sample_rate }
    }

    pub fn handle_event(&mut self, event: SynthEvent) {
        match event {
            
            SynthEvent::NoteOn(n, f) => {
                // 1. 先尝试找一个完全关闭的
                let mut target_voice = self.voices.iter_mut().find(|v| matches!(v.state, VoiceState::Off));

                // 2. 如果没找到，再找一个正在释放的
                if target_voice.is_none() {
                    target_voice = self.voices.iter_mut().find(|v| matches!(v.state, VoiceState::Release));
                }

                // 3. 如果找到了（无论是 Off 还是 Release），就激活它
                if let Some(v) = target_voice {
                    v.note_on(n, f);
                }
            }
            SynthEvent::NoteOff(n) => {
                // 只关闭匹配 Note 编号且不在 Off 状态的通道
                if let Some(v) = self.voices.iter_mut().find(|v| !matches!(v.state, VoiceState::Off) && v.note == n) {
                    v.note_off();
                }
            }
            SynthEvent::ControlChange(1, value) => {
                // CC 1 是标准的 Mod Wheel
                let val_f32 = value as f32 / 127.0;
                for v in self.voices.iter_mut() {
                    v.mod_wheel = val_f32;
                }
            }
            SynthEvent::ParamChange(Param::ModRange, value) => {
                for v in self.voices.iter_mut() {
                    v.mod_range = value;
                }
            }
            SynthEvent::ParamChange(param, val) => {
                for v in self.voices.iter_mut() {
                    match param {
                        // ... 其他参数 ...
                        Param::LfoFreq => v.lfo_freq = val,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let mut mixed = 0.0;
        for v in self.voices.iter_mut() {
            mixed += v.render_next(self.sample_rate);
        }
        mixed * 0.2 // 防止过载
    }
}