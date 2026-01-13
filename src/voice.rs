pub enum VoiceState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Voice {
    pub note: u8,
    pub base_freq: f32,      // 改名为 base_freq
    phase: f32,
    lfo_phase: f32,          // 新增：LFO 相位
    pub state: VoiceState,
    
    amplitude: f32,
    attack_rate: f32,
    decay_rate: f32,
    pub sustain_level: f32,
    release_rate: f32,

    pub mod_wheel: f32,      // MOD 轮当前值 (0.0 - 1.0)
    pub mod_range: f32,      // 滑条控制的最大调制范围 (例如 0.0 - 50.0 Hz)

    pub lfo_freq: f32,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            note: 0,
            base_freq: 0.0,
            phase: 0.0,
            lfo_phase: 0.0,
            state: VoiceState::Off,
            amplitude: 0.0,
            attack_rate: 1.0 / (0.1 * 44100.0),
            decay_rate: 1.0 / (0.1 * 44100.0),
            sustain_level: 0.7,
            release_rate: 1.0 / (0.3 * 44100.0),
            mod_wheel: 0.0,
            mod_range: 20.0, 
            lfo_freq: 3.14,
        }
    }

    // 新增：动态修改 ADSR 参数的方法
    pub fn set_attack(&mut self, seconds: f32, sample_rate: f32) {
        self.attack_rate = 1.0 / (seconds.max(0.001) * sample_rate);
    }

    pub fn set_decay(&mut self, seconds: f32, sample_rate: f32) {
        self.decay_rate = 1.0 / (seconds.max(0.001) * sample_rate);
    }

    pub fn set_release(&mut self, seconds: f32, sample_rate: f32) {
        self.release_rate = 1.0 / (seconds.max(0.001) * sample_rate);
    }

    // ... 保持原有的 note_on, note_off, render_next, update_envelope 不变 ...
    pub fn note_on(&mut self, note: u8, freq: f32) {
        self.note = note;
        self.base_freq = freq;
        self.state = VoiceState::Attack;
    }

    pub fn note_off(&mut self) {
        self.state = VoiceState::Release;
    }

    pub fn render_next(&mut self, sample_rate: f32) -> f32 {
        if let VoiceState::Off = self.state { return 0.0; }
        
        self.update_envelope();

        // 1. 计算 LFO (使用动态频率 lfo_freq)
        let lfo_val = (self.lfo_phase * 2.0 * std::f32::consts::PI).sin();
        self.lfo_phase = (self.lfo_phase + self.lfo_freq / sample_rate) % 1.0;

        // 2. 计算频率调制深度
        let modulation = lfo_val * self.mod_wheel * self.mod_range;
        let current_freq = (self.base_freq + modulation).max(0.0);

        // 3. 生成波形
        let raw_wave = 2.0 * self.phase - 1.0;
        let out = raw_wave * self.amplitude;
        self.phase = (self.phase + current_freq / sample_rate) % 1.0;

        out
    }

    fn update_envelope(&mut self) {
        match self.state {
            VoiceState::Attack => {
                self.amplitude += self.attack_rate;
                if self.amplitude >= 1.0 { self.amplitude = 1.0; self.state = VoiceState::Decay; }
            }
            VoiceState::Decay => {
                self.amplitude -= self.decay_rate;
                if self.amplitude <= self.sustain_level { self.amplitude = self.sustain_level; self.state = VoiceState::Sustain; }
            }
            VoiceState::Sustain => {}
            VoiceState::Release => {
                self.amplitude -= self.release_rate;
                if self.amplitude <= 0.0 { self.amplitude = 0.0; self.state = VoiceState::Off; self.phase = 0.0; }
            }
            VoiceState::Off => {}
        }
    }
}