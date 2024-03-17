// implements a wavetable LFO


use std::f32::consts::PI;
use crate::ring_buffer::RingBuffer;

/// LFO is a struct that contains a wavetable and a phase, frequency, and amplitude.
pub struct LFO {
    sample_rate: f32,
    wavetable: RingBuffer<f32>,
    phase: f32,
    frequency: f32,
    amplitude: f32,

}
/// implements functions for LFO struct
impl LFO {
    /// creates a new LFO with a given sample rate, frequency, and amplitude, only does so for one period
    pub fn new(sample_rate: f32, frequency: f32, amplitude: f32) -> LFO {
        let wavetable_size = (sample_rate / frequency) as usize;
        let mut wavetable = RingBuffer::new(wavetable_size);
        for i in 0..wavetable_size {
            let value = amplitude * (2.0 * PI * i as f32 / wavetable_size as f32).sin();
            wavetable.push(value);
        }
        LFO {
            sample_rate,
            wavetable,
            phase: 0.0,
            frequency,
            amplitude: 1.0,
        }
    }

    /// set a new frequency for the LFO
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        let wavetable_size = (self.sample_rate / frequency) as usize;
        self.wavetable = RingBuffer::new(wavetable_size);
        for i in 0..wavetable_size {
            let value = self.amplitude * (2.0 * PI * i as f32 / wavetable_size as f32).sin();
            self.wavetable.push(value);
        }
    }
    /// set a new amplitude for the LFO
    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
        let wavetable_size = (self.sample_rate / self.frequency) as usize;
        self.wavetable = RingBuffer::new(wavetable_size);
        for i in 0..wavetable_size {
            let value = amplitude * (2.0 * PI * i as f32 / wavetable_size as f32).sin();
            self.wavetable.push(value);
        }
    }
    /// get the next sample from the LFO
    pub fn get_sample(&mut self) -> f32 {
        let value = self.wavetable.peek();
        self.wavetable.pop();
        self.wavetable.push(value);
        value
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_lfo() {
        // test that LFO generates the correct values.
        let sample_rate = 44100.0;
        let frequency = 1.0;
        let amplitude = 1.0;
        let mut lfo = LFO::new(sample_rate, frequency, amplitude);
        let period = (sample_rate / frequency) as usize; // number of samples in one period
        for i in 0..period {
            let expected = amplitude * (2.0 * PI * i as f32 / period as f32).sin();
            let actual = lfo.get_sample();
            assert!((expected - actual).abs() < 1e-6);
        }
    }
    #[test]
    fn test_set_frequency() {
        // test that set_frequency changes the frequency of the LFO.
        let sample_rate = 44100.0;
        let frequency = 1.0;
        let amplitude = 1.0;
        let mut lfo = LFO::new(sample_rate, frequency, amplitude);
        lfo.set_frequency(2.0);
        let period = (sample_rate / 2.0) as usize; 
        for i in 0..period {
            let expected = amplitude * (2.0 * PI * i as f32 / period as f32).sin();
            let actual = lfo.get_sample();
            assert!((expected - actual).abs() < 1e-6);
        }
    }

    #[test]
    fn test_set_amplitude() {
        // test that set_amplitude changes the amplitude of the LFO.
        let sample_rate = 44100.0;
        let frequency = 1.0;
        let amplitude = 1.0;
        let mut lfo = LFO::new(sample_rate, frequency, amplitude);
        lfo.set_amplitude(2.0);
        let period = (sample_rate / frequency) as usize; 
        for i in 0..period {
            let expected = 2.0 * (2.0 * PI * i as f32 / period as f32).sin();
            let actual = lfo.get_sample();
            assert!((expected - actual).abs() < 1e-6);
        }

    }
}