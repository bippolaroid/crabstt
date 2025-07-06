use std::{error::Error, io};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Host, InputCallbackInfo, Stream, StreamError,
};
use crossbeam::channel::Sender;

pub fn init_input_stream(host: &Host, sender: Sender<Vec<f32>>) -> Result<Stream, Box<dyn Error>> {
    let input_device = host
        .default_input_device()
        .ok_or("Could not find input device.")?;
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
        _ => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Only f32 input supported.",
            )))
        }
    };
    Ok(input_stream)
}
