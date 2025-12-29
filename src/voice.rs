pub enum VoiceState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Voice {
    pub note: u8,
    pub freq: f32,
    phase: f32,
    pub state: VoiceState,
    
    // 包络控制
    amplitude: f32,      // 当前实时音量 (0.0 - 1.0)
    attack_rate: f32,    // 每帧增加多少音量
    decay_rate: f32,     // 每帧减少多少音量
    sustain_level: f32,  // 维持阶段的音量 (0.0 - 1.0)
    release_rate: f32,   // 松开后每帧减少多少音量
}

impl Voice {
    pub fn new() -> Self {
        Self {
            note: 0,
            freq: 0.0,
            phase: 0.0,
            state: VoiceState::Off,
            amplitude: 0.0,

            attack_rate: 1.0 / (0.1 * 44100.0),  /* 100ms 触发  */
            decay_rate: 1.0 / (0.1 * 44100.0),   /* 100ms 衰减 */
            sustain_level: 0.7,                  /* 70% 维持   */
            release_rate: 1.0 / (0.3 * 44100.0), /* 300ms 渐出 */
        }
    }

    pub fn note_on(&mut self, note: u8, freq: f32) {
        self.note = note;
        self.freq = freq;
        self.state = VoiceState::Attack;
        // 注意：不重置 amplitude，可以实现“连奏”时的平滑过渡
    }

    pub fn note_off(&mut self) {
        self.state = VoiceState::Release;
    }

    pub fn render_next(&mut self, sample_rate: f32) -> f32 {
        if let VoiceState::Off = self.state {
            return 0.0;
        }

        // 更新包络（必须每帧调用）
        self.update_envelope();

        // 生成波形
        let raw_wave = 2.0 * self.phase - 1.0;
        
        // 应用当前的音量增益（这是 ADSR 生效的地方）
        let out = raw_wave * self.amplitude; 

        self.phase = (self.phase + self.freq / sample_rate) % 1.0;
        out
    }

    fn update_envelope(&mut self) {
        match self.state {
            VoiceState::Attack => {
                self.amplitude += self.attack_rate;
                if self.amplitude >= 1.0 {
                    self.amplitude = 1.0;
                    self.state = VoiceState::Decay;
                }
            }
            VoiceState::Decay => {
                self.amplitude -= self.decay_rate;
                if self.amplitude <= self.sustain_level {
                    self.amplitude = self.sustain_level;
                    self.state = VoiceState::Sustain;
                }
            }
            VoiceState::Sustain => {
                // 音量保持在 sustain_level，直到收到 NoteOff
            }
            VoiceState::Release => {
                self.amplitude -= self.release_rate;
                if self.amplitude <= 0.0 {
                    self.amplitude = 0.0;
                    self.state = VoiceState::Off;
                    self.phase = 0.0; // 真正关闭时重置相位
                }
            }
            VoiceState::Off => {}
        }
    }
}