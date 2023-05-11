use std::ops::Deref;

use wgpu::{CommandEncoder, CommandEncoderDescriptor, Device, Queue};

/// Combines the WGPU [`Queue`] und [`CommandEncoder`]. Records all commands to
/// its internal [`CommandEncoder`] untill submit is called and a new
/// [`CommandEncoder`]. This is done to reduce the amount of [`CommandEncoder`]
/// submited to a [`Queue`]. This seams to be inefficient.
pub struct CommandQueue<'a> {
    queue: &'a Queue,
    command_encoder: Option<CommandEncoder>,
}

impl<'a> CommandQueue<'a> {
    /// Creates a new instance
    pub fn new(queue: &'a Queue) -> Self {
        Self {
            queue,
            command_encoder: None,
        }
    }

    /// Gets the internal [`Queue`]
    pub fn queue(&self) -> &'a Queue {
        self.queue
    }

    /// Gets a [`CommandEncoder`]
    pub fn command_encoder(&mut self, device: &Device) -> &mut CommandEncoder {
        self.command_encoder.get_or_insert_with(|| {
            device.create_command_encoder(&CommandEncoderDescriptor { label: None })
        })
    }

    /// Submits the internal [`CommandEncoder`]
    pub fn submit(&mut self) {
        if let Some(command_encoder) = self.command_encoder.take() {
            self.queue.submit([command_encoder.finish()]);
        }
    }
}

impl<'a> Deref for CommandQueue<'a> {
    type Target = Queue;

    fn deref(&self) -> &Self::Target {
        self.queue
    }
}

impl<'a> Drop for CommandQueue<'a> {
    fn drop(&mut self) {
        self.submit()
    }
}
