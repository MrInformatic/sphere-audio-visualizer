use crate::audio_analysis::utils::RingBuffer;

/// Implementation of a Infinite Impulse Response(IIR) Filter
pub struct IIRFilter {
    buffer_a: Vec<f32>,
    buffer_b: Vec<f32>,
    ring_buffer_x: RingBuffer<f32>,
    ring_buffer_y: RingBuffer<f32>,
}

impl IIRFilter {
    /// Creates a new IIR filter
    pub fn new(mut buffer_a: Vec<f32>, mut buffer_b: Vec<f32>) -> Self {
        if !buffer_a.is_empty() {
            let buffer_a_0 = buffer_a[0];

            buffer_a.iter_mut().skip(1).for_each(|i| *i /= buffer_a_0);
            buffer_b.iter_mut().for_each(|i| *i /= buffer_a_0);

            buffer_a[0] = 1f32;
        }

        let ring_buffer_x = RingBuffer::new(vec![0f32; buffer_b.len()]);
        let ring_buffer_y = RingBuffer::new(vec![0f32; buffer_a.len() - 1]);
        Self {
            buffer_a,
            buffer_b,
            ring_buffer_x,
            ring_buffer_y,
        }
    }

    /// Create a new IIR filter which can be use as low pass filter
    pub fn low_pass(frequency: f32, q: f32, sample_rate: f32) -> Self {
        let double_pi = 2.0 * std::f32::consts::PI;

        let mut buffer_a = vec![];
        let mut buffer_b = vec![];

        let w0 = double_pi * frequency / sample_rate;
        let alpha = w0.sin() / (2f32 * q);
        let norm = 1f32 + alpha;
        let c = w0.cos();
        buffer_a.push(1f32);
        buffer_a.push(-2f32 * c / norm);
        buffer_a.push((1f32 - alpha) / norm);
        buffer_b.push((1f32 - c) / (2f32 * norm));
        buffer_b.push((1f32 - c) / norm);
        buffer_b.push(buffer_b[0]);

        Self::new(buffer_a, buffer_b)
    }

    /// Creates a new IIR filter which can be used as high pass filter
    pub fn high_pass(frequency: f32, q: f32, sample_rate: f32) -> Self {
        let double_pi = 2f32 * std::f32::consts::PI;

        let mut buffer_a = vec![];
        let mut buffer_b = vec![];

        let w0 = double_pi * frequency / sample_rate;
        let alpha = w0.sin() / (2f32 * q);
        let norm = 1f32 + alpha;
        let c = w0.cos();
        buffer_a.push(1f32);
        buffer_a.push(-2f32 * c / norm);
        buffer_a.push((1f32 - alpha) / norm);
        buffer_b.push((1f32 + c) / (2f32 * norm));
        buffer_b.push((-1f32 - c) / norm);
        buffer_b.push(buffer_b[0]);

        Self::new(buffer_a, buffer_b)
    }

    /// processes one sample outputs the filtered sample
    pub fn tick(&mut self, sample: f32) -> f32 {
        self.ring_buffer_x.push(sample);

        let x = self
            .ring_buffer_x
            .iter()
            .zip(self.buffer_b.iter().rev())
            .map(|(f, s)| *f * *s)
            .reduce(|a, b| a + b)
            .unwrap_or(0f32);

        let y = self
            .ring_buffer_y
            .iter()
            .zip(self.buffer_a.iter().skip(1).rev())
            .map(|(f, s)| *f * *s)
            .reduce(|a, b| a + b)
            .unwrap_or(0f32);

        let sample = x - y;

        self.ring_buffer_y.push(sample);

        sample
    }
}
