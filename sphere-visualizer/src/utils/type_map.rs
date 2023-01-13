use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap},
    marker::PhantomData,
};

use egui::util::id_type_map::TypeId;

/// Implementation of a type map base on a [`HashMap`]
///
/// Example:
///
/// ```
/// use sphere_visualizer::utils::TypeMap;
///
/// let mut type_map = TypeMap::new();
///
/// type_map.insert(8u8);
/// type_map.insert(16u16);
/// type_map.insert(32u32);
/// type_map.insert(64u64);
/// type_map.insert(128u128);
///
/// assert_eq!(type_map.get::<u8>().cloned(), Some(8));
/// assert_eq!(type_map.get::<u16>().cloned(), Some(16));
/// assert_eq!(type_map.get::<u32>().cloned(), Some(32));
/// assert_eq!(type_map.get::<u64>().cloned(), Some(64));
/// assert_eq!(type_map.get::<u128>().cloned(), Some(128));
/// ```
pub struct TypeMap(HashMap<TypeId, Box<dyn Any + Send + Sync>>);

impl TypeMap {
    /// Creates a new instance
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts a value
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
        self.0
            .insert(TypeId::of::<T>(), Box::new(value))
            .map(|value| unsafe { Box::<T>::into_inner(value.downcast_unchecked::<T>()) })
    }

    /// Retrieves a value
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .map(|value| unsafe { value.downcast_ref_unchecked() })
    }

    /// Retrieves a value
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.0
            .get_mut(&TypeId::of::<T>())
            .map(|value| unsafe { value.downcast_mut_unchecked() })
    }

    /// Retrieves a entry
    pub fn entry<T: Send + Sync + 'static>(&mut self) -> TypeMapEntry<T> {
        TypeMapEntry(self.0.entry(TypeId::of::<T>()), PhantomData)
    }

    /// Removes a value
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.0
            .remove(&TypeId::of::<T>())
            .map(|value| unsafe { Box::<T>::into_inner(value.downcast_unchecked::<T>()) })
    }
}

/// The TypeMap version of a HashMap [`Entry`]
pub struct TypeMapEntry<'a, T: Send + Sync + 'static>(
    Entry<'a, TypeId, Box<dyn Any + Send + Sync>>,
    PhantomData<T>,
);

impl<'a, T: Send + Sync + 'static> TypeMapEntry<'a, T> {
    /// Gets the value or inserts the passed value if it does not exist.
    ///
    /// Example:
    ///
    /// ```
    /// use sphere_visualizer::utils::TypeMap;
    ///
    /// let mut type_map = TypeMap::new();
    ///
    /// type_map.insert(8u8);
    /// type_map.insert(16u16);
    ///
    /// assert_eq!(type_map.get::<u8>().cloned(), Some(8));
    /// assert_eq!(type_map.get::<u16>().cloned(), Some(16));
    /// assert_eq!(*type_map.entry::<u16>().or_insert(32), 16);
    /// assert_eq!(*type_map.entry::<u32>().or_insert(32), 32);
    /// ```
    pub fn or_insert(self, value: T) -> &'a mut T {
        unsafe { self.0.or_insert(Box::new(value)).downcast_mut_unchecked() }
    }

    /// Gets the value or uses the passed fuction to generate a value to insert if it does not exist.
    ///
    /// Example:
    ///
    /// ```
    /// use sphere_visualizer::utils::TypeMap;
    ///
    /// let mut type_map = TypeMap::new();
    ///
    /// type_map.insert(8u8);
    /// type_map.insert(16u16);
    ///
    /// assert_eq!(type_map.get::<u8>().cloned(), Some(8));
    /// assert_eq!(type_map.get::<u16>().cloned(), Some(16));
    /// assert_eq!(*type_map.entry::<u16>().or_insert_with(|| 32), 16);
    /// assert_eq!(*type_map.entry::<u32>().or_insert_with(|| 32), 32);
    /// ```
    pub fn or_insert_with(self, f: impl FnOnce() -> T) -> &'a mut T {
        unsafe {
            self.0
                .or_insert_with(|| Box::new(f()))
                .downcast_mut_unchecked()
        }
    }

    /// Gets the value or inserts the default value if it does not exist.
    ///
    /// Example:
    ///
    /// ```
    /// use sphere_visualizer::utils::TypeMap;
    ///
    /// let mut type_map = TypeMap::new();
    ///
    /// type_map.insert(8u8);
    /// type_map.insert(16u16);
    ///
    /// assert_eq!(type_map.get::<u8>().cloned(), Some(8));
    /// assert_eq!(type_map.get::<u16>().cloned(), Some(16));
    /// assert_eq!(*type_map.entry::<u16>().or_default(), 16);
    /// assert_eq!(*type_map.entry::<u32>().or_default(), 0);
    /// ```
    pub fn or_default(self) -> &'a mut T
    where
        T: Default,
    {
        self.or_insert_with(T::default)
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// Example:
    ///
    /// ```
    /// use sphere_visualizer::utils::TypeMap;
    ///
    /// let mut type_map = TypeMap::new();
    ///
    /// type_map.insert(8u8);
    /// type_map.insert(16u16);
    ///
    /// assert_eq!(type_map.get::<u8>().cloned(), Some(8));
    /// assert_eq!(type_map.get::<u16>().cloned(), Some(16));
    /// assert_eq!(*type_map.entry::<u16>().and_modify(|x| *x *= 2).or_insert(8), 32);
    /// assert_eq!(*type_map.entry::<u32>().and_modify(|x| *x *= 2).or_insert(8), 8);
    /// ```
    pub fn and_modify(self, f: impl FnOnce(&mut T)) -> Self {
        Self(
            self.0
                .and_modify(|value| f(unsafe { value.downcast_mut_unchecked() })),
            self.1,
        )
    }
}
