use std::{
    borrow::Borrow,
    num::NonZeroU64,
    ops::{Deref, DerefMut},
    ptr::Pointee,
};

use thiserror::Error;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt as DeviceExt2},
    BindGroupEntry, BindingResource, Buffer, BufferAsyncError, BufferBinding, BufferDescriptor,
    BufferSlice, BufferUsages, BufferView, BufferViewMut, CommandEncoder, Device, MapMode, Queue,
};

/// Returns the size of a UnSize value of type T with the provided metadata
pub fn size_of_metadata<T: ?Sized>(metadata: <T as Pointee>::Metadata) -> usize {
    unsafe { std::mem::size_of_val_raw(std::ptr::from_raw_parts::<T>(std::ptr::null(), metadata)) }
}

/// Returns the align of a UnSize value of type T with the provided metadata
pub fn align_of_metadata<T: ?Sized>(metadata: <T as Pointee>::Metadata) -> usize {
    unsafe { std::mem::align_of_val_raw(std::ptr::from_raw_parts::<T>(std::ptr::null(), metadata)) }
}

/// A wrapper for WGPU buffer containing a Value. The size of the value might
/// not be known at compile time!
pub struct TypedBuffer<B: Borrow<Buffer>, T: ?Sized> {
    buffer: B,
    offset: usize,
    metadata: <T as Pointee>::Metadata,
}

impl<B: Borrow<Buffer>, T: ?Sized> TypedBuffer<B, T> {
    /// Crates a new instance from a WGPU Buffer with offset and metadata
    ///
    /// Safety: it should be garanteed by the caller that the passed offset and
    /// metadata are valid.
    pub unsafe fn from_buffer(
        buffer: B,
        offset: usize,
        metadata: <T as Pointee>::Metadata,
    ) -> Self {
        Self {
            buffer,
            offset,
            metadata,
        }
    }

    /// Tries to creates a [`BufferBinding`] for the data inside the
    /// [`TypedBuffer`]
    pub fn buffer_binding(&self) -> Option<BufferBinding> {
        Some(BufferBinding {
            buffer: self.buffer.borrow(),
            offset: self.offset as u64,
            size: Some(NonZeroU64::new(size_of_metadata::<T>(self.metadata) as u64)?),
        })
    }

    /// Tries to creates a [`BindingResource`] for the data inside the
    /// [`TypedBuffer`]
    pub fn binding(&self) -> Option<BindingResource> {
        Some(BindingResource::Buffer(self.buffer_binding()?))
    }

    /// Tries to creates a [`BindGroupEntry`] for the data inside the
    /// [`TypedBuffer`]
    pub fn bind_group_entry(&self, binding: u32) -> Option<BindGroupEntry> {
        Some(BindGroupEntry {
            binding,
            resource: self.binding()?,
        })
    }

    /// Creates a view of the data in the buffer matching the rederence retuned
    /// by the passed function.
    ///
    /// Safety: The memory behind reference passed to the function is not valid
    /// and shoud never under any circumstances be accessed.  
    pub fn view<'a, U: ?Sized, F: FnOnce(&T) -> &U>(
        &'a self,
        mapper: F,
    ) -> TypedBuffer<&'a Buffer, U> {
        let mapped_reference = (mapper)(unsafe {
            &*std::ptr::from_raw_parts::<T>(
                align_of_metadata::<T>(self.metadata) as *const _,
                self.metadata,
            )
        });

        TypedBuffer {
            buffer: &self,
            offset: mapped_reference as *const U as *const () as usize,
            metadata: std::ptr::metadata(mapped_reference),
        }
    }

    /// The offset of the [`TypedBuffer`] related to the underlying WGPU
    /// [`Buffer`]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The metadata of the data inside the Buffer.
    pub fn metadata(&self) -> <T as Pointee>::Metadata {
        self.metadata
    }

    /// Creates a slice of the buffer ready for mapping into main memory
    pub fn slice<'a>(&'a self) -> TypedBufferSlice<'a, T> {
        TypedBufferSlice(self.buffer.borrow().slice(..), self.metadata)
    }

    /// Unmaps the underlying Buffer
    pub fn unmap(&self) {
        self.buffer.borrow().unmap()
    }

    /// Destroys the underlying buffer
    pub fn destroy(&self) {
        self.buffer.borrow().destroy()
    }
}

impl<B: Borrow<Buffer>, T: ?Sized> Deref for TypedBuffer<B, T> {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer.borrow()
    }
}

/// The Typed version of the WGPU [`BufferSlice`]
pub struct TypedBufferSlice<'a, T: ?Sized>(BufferSlice<'a>, <T as Pointee>::Metadata);

impl<'a, T: ?Sized> TypedBufferSlice<'a, T> {
    /// Map the buffer. Buffer is ready to map once the callback is called.
    ///
    /// For the callback to complete, either `queue.submit(..)`, `instance.poll_all(..)`, or `device.poll(..)`
    /// must be called elsewhere in the runtime, possibly integrated into an event loop or run on a separate thread.
    ///
    /// The callback will be called on the thread that first calls the above functions after the gpu work
    /// has completed. There are no restrictions on the code you can run in the callback, however on native the
    /// call to the function will not complete until the callback returns, so prefer keeping callbacks short
    /// and used to set flags, send messages, etc.
    pub async fn map_async(&self, mode: MapMode) -> Result<(), BufferAsyncError> {
        self.0.map_async(mode).await
    }

    /// Synchronously and immediately map a buffer for reading. If the buffer is not immediately mappable
    /// through [`BufferDescriptor::mapped_at_creation`] or [`BufferSlice::map_async`], will panic.
    pub fn as_mapped_range(&self) -> TypedBufferRef<'a, T> {
        TypedBufferRef(self.0.get_mapped_range(), self.1)
    }

    /// Synchronously and immediately map a buffer for writing. If the buffer is not immediately mappable
    /// through [`BufferDescriptor::mapped_at_creation`] or [`BufferSlice::map_async`], will panic.
    pub fn as_mapped_range_mut(&self) -> TypedBufferRefMut<'a, T> {
        TypedBufferRefMut(self.0.get_mapped_range_mut(), self.1)
    }

    /// Gets the metadata of the underlying data
    pub fn metadata(&self) -> <T as Pointee>::Metadata {
        self.1
    }
}

/// Typed version of WGPU [`BufferView`]
pub struct TypedBufferRef<'a, T: ?Sized>(BufferView<'a>, <T as Pointee>::Metadata);

impl<'a, T: ?Sized> Deref for TypedBufferRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*std::ptr::from_raw_parts(self.0.deref().as_ptr() as *const _, self.1) }
    }
}

/// Typed version of WGPU [`BufferViewMut`]
pub struct TypedBufferRefMut<'a, T: ?Sized>(BufferViewMut<'a>, <T as Pointee>::Metadata);

impl<'a, T: ?Sized> Deref for TypedBufferRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*std::ptr::from_raw_parts(self.0.deref().as_ptr() as *const _, self.1) }
    }
}

impl<'a, T: ?Sized> DerefMut for TypedBufferRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *std::ptr::from_raw_parts_mut(self.0.deref_mut().as_mut_ptr() as *mut _, self.1)
        }
    }
}

/// Typed version of WGPU [`BufferDescriptor`]
pub struct TypedBufferDescriptor<'a, T: ?Sized> {
    /// Debug label of a buffer. This will show up in graphics debuggers for easy identification.
    pub label: Option<&'a str>,
    /// Usages of a buffer. If the buffer is used in any way that isn't specified here, the operation
    /// will panic.
    pub usage: BufferUsages,
    /// Allows a buffer to be mapped immediately after they are made. It does not have to be [`BufferUsages::MAP_READ`] or
    /// [`BufferUsages::MAP_WRITE`], all buffers are allowed to be mapped at creation.
    pub mapped_at_creation: bool,
    /// The metadata of the stored data.
    pub metadata: <T as Pointee>::Metadata,
}

/// Typed version of WGPU [`BufferInitDescriptor`]
pub struct TypedBufferInitDescriptor<'a, T: ?Sized> {
    /// Debug label of a buffer. This will show up in graphics debuggers for easy identification.
    pub label: Option<&'a str>,
    /// Usages of a buffer. If the buffer is used in any way that isn't specified here, the operation
    /// will panic.
    pub usage: BufferUsages,
    /// Contents of a buffer on creation.
    pub value: &'a T,
}

/// Extension trait for WGPU [`Device`] to create [`TypedBuffer`]
pub trait TypedBufferDeviceExt {
    /// Creates a [`TypedBuffer`] without initial data
    fn create_typed_buffer<'a, T: ?Sized>(
        &self,
        descriptor: &TypedBufferDescriptor<'a, T>,
    ) -> TypedBuffer<Buffer, T>;

    /// Creates a [`TypedBuffer`] with initial data
    fn create_typed_buffer_init<'a, T: ?Sized>(
        &self,
        descriptor: &TypedBufferInitDescriptor<'a, T>,
    ) -> TypedBuffer<Buffer, T>;
}

impl TypedBufferDeviceExt for Device {
    fn create_typed_buffer<'a, T: ?Sized>(
        &self,
        descriptor: &TypedBufferDescriptor<'a, T>,
    ) -> TypedBuffer<Buffer, T> {
        unsafe {
            TypedBuffer::from_buffer(
                self.create_buffer(&BufferDescriptor {
                    label: descriptor.label,
                    size: size_of_metadata::<T>(descriptor.metadata) as u64,
                    usage: descriptor.usage,
                    mapped_at_creation: descriptor.mapped_at_creation,
                }),
                0,
                descriptor.metadata,
            )
        }
    }

    fn create_typed_buffer_init<'a, T: ?Sized>(
        &self,
        descriptor: &TypedBufferInitDescriptor<'a, T>,
    ) -> TypedBuffer<Buffer, T> {
        unsafe {
            TypedBuffer::from_buffer(
                self.create_buffer_init(&BufferInitDescriptor {
                    label: descriptor.label,
                    contents: std::slice::from_raw_parts(
                        descriptor.value as *const T as *const u8,
                        std::mem::size_of_val(descriptor.value),
                    ),
                    usage: descriptor.usage,
                }),
                0,
                std::ptr::metadata(descriptor.value as *const T),
            )
        }
    }
}

/// Represents the errors which could happen when copying from one
/// [`TypedBuffer`] to another.
#[derive(Error, Debug)]
pub enum CopyTypedBufferError {
    /// This error happens when the size of the two passed buffers is not equal.
    #[error("size of the two passed buffer must be equal")]
    SizeNotEqual,
}

/// Extension trait for the WGPU [`CommandEncoder`] to enqueue copy commands
/// for the [`TypedBuffer`]
pub trait TypedBufferCommandEncoderExt {
    /// enqueues a copy command to copy the data from one [`TypedBuffer`] to
    /// another.
    fn copy_typed_buffer<T: ?Sized, S: Borrow<Buffer>, D: Borrow<Buffer>>(
        &mut self,
        src: &TypedBuffer<S, T>,
        dst: &TypedBuffer<D, T>,
    ) -> Result<(), CopyTypedBufferError>;
}

impl TypedBufferCommandEncoderExt for CommandEncoder {
    fn copy_typed_buffer<T: ?Sized, S: Borrow<Buffer>, D: Borrow<Buffer>>(
        &mut self,
        src: &TypedBuffer<S, T>,
        dst: &TypedBuffer<D, T>,
    ) -> Result<(), CopyTypedBufferError> {
        let src_size = size_of_metadata::<T>(src.metadata());
        let dst_size = size_of_metadata::<T>(dst.metadata());

        if src_size == dst_size {
            self.copy_buffer_to_buffer(
                src.deref(),
                src.offset() as u64,
                dst.deref(),
                dst.offset() as u64,
                src_size as u64,
            );

            Ok(())
        } else {
            Err(CopyTypedBufferError::SizeNotEqual)
        }
    }
}

///Extension trait for the WGPU [`Queue`] to write data to a [`TypedBuffer`]
pub trait TypedBufferQueueExt {
    /// Writes data to a [`TypedBuffer`]
    fn write_typed_buffer<T: ?Sized, B: Borrow<Buffer>>(&self, dst: &TypedBuffer<B, T>, value: &T);
}

impl TypedBufferQueueExt for Queue {
    fn write_typed_buffer<T: ?Sized, B: Borrow<Buffer>>(&self, dst: &TypedBuffer<B, T>, value: &T) {
        self.write_buffer(&dst, dst.offset() as u64, unsafe {
            std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of_val(value))
        })
    }
}

/// Extension tarit for the WGPU [`Buffer`] to create [`BindGroupEntry`] for one
/// specified binding.
pub trait BufferExt {
    /// Creates a BindGroupEntry [`BindGroupEntry`] for one specified binding.
    fn as_entire_bind_group_entry(&self, binding: u32) -> BindGroupEntry;
}

impl BufferExt for Buffer {
    fn as_entire_bind_group_entry(&self, binding: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: self.as_entire_binding(),
        }
    }
}
