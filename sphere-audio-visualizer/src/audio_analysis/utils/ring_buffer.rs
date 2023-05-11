/// Implementation of a constant size ring buffer based on a [`Vec`].
/// The internal buffer is always considered full and therefore always
/// overrides old entries.
///
/// Example:
///
/// ```
/// use sphere_audio_visualizer::audio_analysis::utils::RingBuffer;
///
/// let mut ring_buffer = RingBuffer::new(vec![1, 2, 3, 4]);
///
/// let mut iter = ring_buffer.iter().cloned();
///
/// assert_eq!(iter.next(), Some(1));
/// assert_eq!(iter.next(), Some(2));
/// assert_eq!(iter.next(), Some(3));
/// assert_eq!(iter.next(), Some(4));
/// assert_eq!(iter.next(), None);
/// ```
///
/// ```
/// use sphere_audio_visualizer::audio_analysis::utils::RingBuffer;
///
/// let mut ring_buffer = RingBuffer::new(vec![1, 2, 3, 4]);
///
/// ring_buffer.push(5);
/// ring_buffer.push(6);
///
/// let mut iter = ring_buffer.iter().cloned();
///
/// assert_eq!(iter.next(), Some(3));
/// assert_eq!(iter.next(), Some(4));
/// assert_eq!(iter.next(), Some(5));
/// assert_eq!(iter.next(), Some(6));
/// assert_eq!(iter.next(), None);
/// ```
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    next_index: usize,
}

impl<T> RingBuffer<T> {
    /// Creates a new ring buffer.
    /// The passed vec is used as the internal buffer.
    /// This also means the size of the passed buffer is equal to the capacity
    /// of the ring buffer.
    pub fn new(buffer: Vec<T>) -> Self {
        Self {
            buffer,
            next_index: 0,
        }
    }

    /// Pushes a new element onto the ring buffer. This function overrides
    /// old elements on the buffer.
    pub fn push(&mut self, element: T) {
        self.buffer[self.next_index] = element;
        self.next_index = (self.next_index + 1) % self.buffer.len();
    }

    /// Retunes all items on the ring buffer as iterator.
    /// This function will not return all pushed elements since
    /// [`RingBuffer::push`] will override old elements.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer[self.next_index..self.buffer.len()]
            .iter()
            .chain(self.buffer[0..self.next_index].iter())
    }
}
