mod common;
mod engine;
mod voice;
mod utils;

use crate::common::{SynthEvent, Param};
use crate::engine::SynthEngine;
use crate::utils::{msg, LogLevel, midi_to_freq};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use midir::{MidiInput, Ignore};
use std::io::{stdin, stdout, Write};
use std::sync::mpsc; // 使用标准库 MPSC
use eframe::egui;

struct SynthApp {
    event_tx: mpsc::Sender<SynthEvent>,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    mod_range: f32,
    lfo_freq: f32,
}

impl SynthApp {
    fn new(tx: mpsc::Sender<SynthEvent>) -> Self {
        Self {
            event_tx: tx,
            attack: 0.1,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
            mod_range: 20.0,
            lfo_freq: 3.14,
        }
    }
}

impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Noriv Synthesizer");
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label("ADSR setting");
                
                if ui.add(egui::Slider::new(&mut self.attack, 0.01..=2.0).text("Attack (s)")).changed() {
                    let _ = self.event_tx.send(SynthEvent::ParamChange(Param::Attack, self.attack));
                }
                
                if ui.add(egui::Slider::new(&mut self.decay, 0.01..=2.0).text("Decay (s)")).changed() {
                    let _ = self.event_tx.send(SynthEvent::ParamChange(Param::Decay, self.decay));
                }
                
                if ui.add(egui::Slider::new(&mut self.sustain, 0.0..=1.0).text("Sustain Level")).changed() {
                    let _ = self.event_tx.send(SynthEvent::ParamChange(Param::Sustain, self.sustain));
                }
                
                if ui.add(egui::Slider::new(&mut self.release, 0.01..=5.0).text("Release (s)")).changed() {
                    let _ = self.event_tx.send(SynthEvent::ParamChange(Param::Release, self.release));
                }
            });

            ui.group(|ui| {
                ui.label("LFO");
                // LFO 频率滑条
                if ui.add(egui::Slider::new(&mut self.lfo_freq, 0.1..=20.0).text("LFO Freq(Hz)")).changed() {
                    let _ = self.event_tx.send(SynthEvent::ParamChange(Param::LfoFreq, self.lfo_freq));
                }

                // 调制范围滑条
                if ui.add(egui::Slider::new(&mut self.mod_range, 0.0..=200.0).text("FM Range (Hz)")).changed() {
                    let _ = self.event_tx.send(SynthEvent::ParamChange(Param::ModRange, self.mod_range));
                }

                    ui.label(format!("MOD depth: MIDI CC#1"));
                });

            ui.add_space(20.0);
            ui.label("hello synthesizer");
        });
    }
}

fn main() -> anyhow::Result<()> {
    // MIDI 初始化
    let mut midi_in = MidiInput::new("MIDI Input")?;
    midi_in.ignore(Ignore::None);
    let ports = midi_in.ports();
    
    msg(LogLevel::Prompt, "正在扫描 MIDI 设备...");
    for (i, p) in ports.iter().enumerate() {
        msg(LogLevel::Prompt, format!("{}: {}", i, midi_in.port_name(p)?));
    }
    
    msg(LogLevel::Prompt, "请选择设备序号: ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let port = &ports[input.trim().parse().unwrap_or(0)];

    // 使用 MPSC 通道替代 ringbuf
    let (tx, rx) = mpsc::channel::<SynthEvent>();

    // 音频输出初始化
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| {
        msg(LogLevel::Error, "无输出设备");
        anyhow::anyhow!("No audio device")
    })?;
    
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    msg(LogLevel::Info, format!("音频初始化成功: {} Hz", sample_rate));

    let mut engine = SynthEngine::new(sample_rate);

    // 音频流线程
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // 处理所有积压的事件
            while let Ok(event) = rx.try_recv() {
                engine.handle_event(event);
            }
            for sample in data.iter_mut() {
                *sample = engine.next_sample();
            }
        },
        |err| msg(LogLevel::Error, format!("音频流错误: {}", err)),
        None
    )?;
    stream.play()?;

    // MIDI 监听线程 (克隆发送端)
    let midi_tx = tx.clone();
    let _conn_in = midi_in.connect(port, "midir-main", move |_, message, _| {
        if message.len() >= 3 {
            let status = message[0] & 0xF0; // 获取消息类型
            let data1 = message[1];
            let data2 = message[2];
            
            match status {
                144 if data2 > 0 => { // Note On
                    let _ = midi_tx.send(SynthEvent::NoteOn(data1, midi_to_freq(data1)));
                }
                128 | 144 => { // Note Off
                    let _ = midi_tx.send(SynthEvent::NoteOff(data1));
                }
                176 => { // Control Change
                    let _ = midi_tx.send(SynthEvent::ControlChange(data1, data2));
                }
                _ => {}
            }
        }
    }, ())?;

    // 启动 UI 窗口 (传递原始发送端)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Noriv Synthesizer 0.0.1a",
        options,
        Box::new(|_cc| Box::new(SynthApp::new(tx))),
    ).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(())
}