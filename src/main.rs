use std::{error::Error, thread};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    InputCallbackInfo, OutputCallbackInfo, Sample, StreamError,
};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};

fn main() -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    let (sender, receiver): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = unbounded();

    let input_device = host.default_input_device().unwrap();
    let input_config = input_device.default_input_config()?;
    let input_format = input_config.sample_format();
    println!("{:?}", input_format);
    let input_stream_config = cpal::StreamConfig {
        channels: input_config.channels(),
        sample_rate: input_config.sample_rate(),
        buffer_size: cpal::BufferSize::Fixed(1024),
    };
    for config_range in input_device.supported_input_configs()? {
    println!(
        "Default: {:?}, SampleFormat: {:?}",
        config_range.buffer_size(),
        config_range.sample_format()
    );
}

    let input_data_fn = move |data: &[f32], _: &InputCallbackInfo| {
        println!("Sent: {:?}", data.len());
        let owned_data = data.to_vec();
        sender.send(owned_data).expect("Could not send data");
    };
    let input_err_fn = |error: StreamError| {
        println!("{:?}", error);
    };
    let input_stream = match input_format {
        cpal::SampleFormat::F32 => input_device.build_input_stream(
            &input_stream_config,
            input_data_fn,
            input_err_fn,
            None,
        )?,
        _ => return Ok(()),
    };

    let output_device = host.default_output_device().unwrap();
    let output_config = output_device.default_output_config()?;
    let output_format = output_config.sample_format();
    println!("{:?}", output_format);
    let output_stream_config = cpal::StreamConfig {
        channels: output_config.channels(),
        sample_rate: output_config.sample_rate(),
        buffer_size: cpal::BufferSize::Fixed(1024),
    };
    for config_range in output_device.supported_output_configs()? {
    println!(
        "Default: {:?}, SampleFormat: {:?}",
        config_range.buffer_size(),
        config_range.sample_format()
    );
}

    let output_data_fn = move |data: &mut [f32], _: &OutputCallbackInfo| {
        println!("Received: {:?}", data.len());

        match receiver.try_recv() {
            Ok(buffer) => {
                let len = buffer.len().min(data.len());
                data[..len].copy_from_slice(&buffer[..len]);
                if len < data.len() {
                    for sample in &mut data[len..] {
                        *sample = Sample::EQUILIBRIUM;
                    }
                }
            }
            Err(_) => {
                for sample in data.iter_mut() {
                    *sample = Sample::EQUILIBRIUM;
                }
            }
        }
    };
    let output_err_fn = |error: StreamError| {
        println!("{:?}", error);
    };
    let output_stream = match output_format {
        cpal::SampleFormat::F32 => output_device.build_output_stream(
            &output_stream_config,
            output_data_fn,
            output_err_fn,
            None,
        )?,
        _ => return Ok(()),
    };

    input_stream.play()?;
    println!("Input stream running...");
    output_stream.play()?;
    println!("Output stream running...");
    thread::park();
    Ok(())
}
