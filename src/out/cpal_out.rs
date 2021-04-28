use anyhow;

use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use std::ops::{Index, IndexMut};
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;

use crate::util::Component;

pub struct CpalOut {
    stream: Arc<dyn StreamTrait>,
    dummy: i8,
    cv_in: i8,
    buffer: Sender<i8>,
}

impl CpalOut {
    pub fn from_defaults() -> anyhow::Result<CpalOut> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("failed to find output device");

        let config = device.default_output_config().unwrap();
        let channels = config.channels() as usize;
        // println!("Output device: {}", device.name()?);
        // println!("Default output config: {:?}", config);

        let (tx, rx) = channel();
        let mut last_s = 0;
        let write_data = move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in output.chunks_mut(channels) {
                let s = rx.try_recv();
                for sample in frame.iter_mut() {
                    let s = match s {
                        Ok(s) => {
                            last_s = s;
                            s
                        }
                        Err(TryRecvError::Empty) => last_s,
                        Err(TryRecvError::Disconnected) => panic!("Rx disconnected"),
                    };
                    *sample = (s as f32) / (i8::min_value() as f32)
                }
            }
        };

        // println!("{:?}", config.config());
        let stream = device.build_output_stream(&config.into(), write_data, move |e| {
            println!("{}", e);
        })?;
        stream.play()?;

        Ok(CpalOut {
            stream: Arc::new(stream),
            dummy: 0,
            cv_in: 0,
            buffer: tx,
        })
    }
}

impl Index<&str> for CpalOut {
    type Output = i8;

    fn index(&self, i: &str) -> &Self::Output {
        match i {
            _ => &0,
        }
    }
}

impl IndexMut<&str> for CpalOut {
    fn index_mut(&mut self, i: &str) -> &mut Self::Output {
        match i {
            "cv_in" => &mut self.cv_in,
            _ => &mut self.dummy,
        }
    }
}

impl<'a> Component<'a> for CpalOut {
    fn tick(&mut self) {}
    fn step(&mut self) {
        match self.buffer.send(self.cv_in) {
            Err(_) => panic!("Tx closed"),
            _ => (),
        }
    }
    fn inputs(&self) -> Vec<&'a str> {
        vec!["cv_in"]
    }

    fn outputs(&self) -> Vec<&'a str> {
        vec![]
    }
}
