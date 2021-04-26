use anyhow;

use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use std::ops::{Index, IndexMut};
use std::sync::atomic::AtomicI8;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;

use crate::util::Component;

pub struct CpalOut {
    stream: Arc<dyn StreamTrait>,
    //barrier: Arc<Barrier>,
    setval: Arc<AtomicI8>,
    //sender: Sender<u16>,
    dummy: i8,
    cv_in: i8,
}

impl CpalOut {
    pub fn from_defaults() -> anyhow::Result<CpalOut> {
        //let (tx, rx) = channel();
        let val = Arc::new(AtomicI8::new(0));
        let setval = Arc::clone(&val);
        //let barrier = Arc::new(Barrier::new(2));
        //let b2 = barrier.clone();

        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("failed to find output device");

        let config = device.default_output_config().unwrap();
        let channels = config.channels() as usize;
        println!("Output device: {}", device.name()?);
        println!("Default output config: {:?}", config);

        let mut next_value = move || {
            //b2.wait();
            let s = val.load(Ordering::Relaxed);
            -1. * (s as f32) / (i8::min_value() as f32)
        };

        println!("{:?}", config.config());
        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            move |e| {
                println!("{}", e);
            },
        )?;
        stream.play()?;

        Ok(CpalOut {
            stream: Arc::new(stream),
            //barrier: barrier.clone(),
            setval: setval,
            //sender: tx,
            dummy: 0,
            cv_in: 0,
        })
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> T)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
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
        // This is 100% broken. I need to fix it once my internals
        // are moved to fixed point.
        // self.sender.send(self.cv_in as u16).unwrap();
        self.setval.store(self.cv_in, Ordering::Relaxed);
    }
    fn inputs(&self) -> Vec<&'a str> {
        vec!["cv_in"]
    }

    fn outputs(&self) -> Vec<&'a str> {
        vec![]
    }
}
