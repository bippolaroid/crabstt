use std::{error::Error, thread};

use cpal::traits::StreamTrait;
use crossbeam::channel::{unbounded, Receiver, Sender};

use crate::audio::{input::init_input_stream, output::init_output_stream};

mod audio;

fn main() -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    let (sender, receiver): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = unbounded();

    let input_stream = init_input_stream(&host, sender)?;
    let output_stream = init_output_stream(&host, receiver)?;

    input_stream.play()?;
    println!("Input stream running...");
    output_stream.play()?;
    println!("Output stream running...");
    thread::park();
    Ok(())
}
