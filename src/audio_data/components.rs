use crate::audio_data::*;
use std::sync::{Arc, Mutex};
use cpal::{self, Stream};
use ringbuffer::AllocRingBuffer;

pub struct AudioData {
    pub latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    pub stream: Option<Stream>,
    pub latest_average_frequency_value: f32
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            latest_audio_data: Arc::new(Mutex::new(AllocRingBuffer::new(SPECTRUM_DATA_LENGTH))),
            stream: None,
            latest_average_frequency_value: 0.0
        }
    }
}