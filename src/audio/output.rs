use std::{error::Error, io};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Host, OutputCallbackInfo, Sample, Stream, StreamError,
};
use crossbeam::channel::Receiver;

pub fn init_output_stream(
    host: &Host,
    receiver: Receiver<Vec<f32>>,
) -> Result<Stream, Box<dyn Error>> {
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
        _ => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Only f32 output supported.",
            )))
        }
    };
    Ok(output_stream)
}
