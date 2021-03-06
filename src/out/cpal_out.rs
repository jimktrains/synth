use anyhow;

use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use std::sync::Arc;

#[allow(dead_code)]
pub struct CpalOut {
    stream: Arc<dyn StreamTrait>,
    dummy: i16,
}

impl CpalOut {
    pub fn from_defaults<F>(mut next_sample: F) -> anyhow::Result<CpalOut>
    where
        F: FnMut() -> i16 + Send + 'static,
    {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("failed to find output device");

        let config = device.default_output_config().unwrap();
        let channels = config.channels() as usize;
        println!("Output device: {}", device.name()?);
        println!("Default output config: {:?}", config);

        let write_data = move |output: &mut [f32], _cbi: &cpal::OutputCallbackInfo| {
            for frame in output.chunks_mut(channels) {
                let samp = next_sample();
                let s = (((samp) as f64) / (i16::min_value() as f64)) as f32;
                for sample in frame.iter_mut() {
                    *sample = s;
                }
            }
        };

        println!("{:?}", config.config());
        let stream = device.build_output_stream(&config.into(), write_data, move |e| {
            println!("{}", e);
        })?;
        stream.play()?;

        Ok(CpalOut {
            stream: Arc::new(stream),
            dummy: 0,
        })
    }
}
