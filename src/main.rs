mod common;
mod engine;
mod voice;
mod utils;

use crate::common::SynthEvent;
use crate::engine::SynthEngine;
use crate::utils::{msg, LogLevel, midi_to_freq};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use midir::{MidiInput, Ignore};
use ringbuf::HeapRb;
use std::io::{stdin, stdout, Write};

/*
 * ├── Cargo.toml
 * ├── src/
 * │   ├── main.rs          # 程序入口 解析命令行 初始化 MIDI/Audio 设备
 * │   ├── lib.rs           # 库入口 导出核心组件
 * │   ├── audio.rs         # CPAL 相关配置：音频流的启动、线程管理
 * │   ├── midi.rs          # MIDIR 相关配置：设备选择、消息解析
 * │   ├── engine.rs        # 合成器核心：多音管理、声音混合逻辑
 * │   ├── utils.rs         # 调试函数 通知等级
 * │   ├── voice.rs         # 单个发声单元：振荡器(Oscillator) 包络(Envelope)
 * │   └── common.rs        # 通用定义 消息枚举 常量 (如采样率)
 */

fn main() -> anyhow::Result<()> {
    let mut midi_in = MidiInput::new("Rust MIDI Input")?;
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

    /* 环形缓冲区初始化 */
    let rb = HeapRb::<SynthEvent>::new(64);
    let (mut prod, mut cons) = rb.split();
    msg(LogLevel::Info, "环形缓冲区初始化: 完成");

    /* 音频输出初始化 */
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| {
        msg(LogLevel::Error, "无输出设备");
        anyhow::anyhow!("No audio device")
    })?;
    
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    msg(LogLevel::Info, format!("音频初始化成功: {} Hz", sample_rate));

    let mut engine = SynthEngine::new(sample_rate);

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            while let Some(event) = cons.pop() {
                engine.handle_event(event);
            }
            for sample in data.iter_mut() {
                *sample = engine.next_sample();
            }
        },
        |err| msg(LogLevel::Error, format!("音频流错误: {}", err)), // 替换 eprint
        None
    )?;
    stream.play()?;

    /* MIDI 监听线程 */
    let _conn_in = midi_in.connect(port, "midir-main", move |_, message, _| {
        if message.len() >= 3 {
            let status = message[0];
            let note = message[1];
            let vel = message[2];
            
            if status == 144 && vel > 0 {
                let freq = midi_to_freq(note);
                msg(LogLevel::Midi, format!("Note ON  | Key: {} | Freq: {:.2}Hz", note, freq));
                let _ = prod.push(SynthEvent::NoteOn(note, freq));
            } else if status == 128 || (status == 144 && vel == 0) {
                msg(LogLevel::Midi, format!("Note OFF | Key: {}", note));
                let _ = prod.push(SynthEvent::NoteOff(note));
            }
        }
    }, ())?;

    msg(LogLevel::Info, "合成器运行中... 按回车退出");
    stdin().read_line(&mut String::new())?;
    Ok(())
}