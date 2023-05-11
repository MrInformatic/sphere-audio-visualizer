/// A polyfill for the Rust [`Option`]
#[derive(PartialEq, Eq, Debug)]
pub struct OptionPolyfill<T> {
    is_some: bool,
    value: T,
}

impl<T> OptionPolyfill<T> {
    /// Creates a new instance.
    pub fn new(is_some: bool, value: T) -> Self {
        Self { is_some, value }
    }

    /// Creates a None Option
    ///
    /// Example:
    ///
    /// ```
    /// use sphere_audio_visualizer_core::utils::OptionPolyfill;
    ///
    /// let option = OptionPolyfill::<u32>::none();
    ///
    /// assert!(option.is_none());
    /// assert!(!option.is_some());
    /// ```
    pub fn none() -> Self
    where
        T: Uninit,
    {
        Self {
            is_some: false,
            value: Uninit::uninit(),
        }
    }

    /// Creates a Some Option
    ///
    /// Example:
    ///
    /// ```
    /// use sphere_audio_visualizer_core::utils::OptionPolyfill;
    ///
    /// let option = OptionPolyfill::some(16);
    ///
    /// assert!(!option.is_none());
    /// assert!(option.is_some());
    /// assert_eq!(unsafe { option.unwrap() }, 16);
    /// ```
    pub fn some(value: T) -> Self {
        Self {
            is_some: true,
            value,
        }
    }

    /// Get if the Option is some
    pub fn is_some(&self) -> bool {
        self.is_some
    }

    /// Get if the Option is none
    pub fn is_none(&self) -> bool {
        !self.is_some
    }

    /// Gets the internal value
    pub unsafe fn unwrap(self) -> T {
        return self.value;
    }

    /// Applying the function `f` to the contained value.
    ///
    /// Example:
    ///
    /// ```
    /// use sphere_audio_visualizer_core::utils::OptionPolyfill;
    ///
    /// let option = OptionPolyfill::some(16).map(|x| x * 2);
    ///
    /// assert_eq!(option, OptionPolyfill::some(32));
    /// ```
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> OptionPolyfill<U>
    where
        U: Uninit,
    {
        if self.is_some {
            OptionPolyfill::some((f)(self.value))
        } else {
            OptionPolyfill::none()
        }
    }

    /// Applying the function `some` to the contained value if it is some.
    /// Otherwise use the function `none` to generate a value.
    ///
    /// Example:
    ///
    /// ```
    /// use sphere_audio_visualizer_core::utils::OptionPolyfill;
    ///
    /// let some = OptionPolyfill::<u32>::some(16);
    /// let none = OptionPolyfill::<u32>::none();
    ///
    /// assert_eq!(some.map_or_else(|x| x * 2, || 8), 32);
    /// assert_eq!(none.map_or_else(|x| x * 2, || 8), 8);
    /// ```
    pub fn map_or_else<U>(self, some: impl FnOnce(T) -> U, none: impl FnOnce() -> U) -> U {
        if self.is_some {
            (some)(self.value)
        } else {
            (none)()
        }
    }

    /// Combines the values of two options. If both options are some the passed
    /// function is use to merge the two values and generate a new one.
    ///
    /// Example:
    ///
    /// ```
    /// use sphere_audio_visualizer_core::utils::OptionPolyfill;
    ///
    /// assert_eq!(OptionPolyfill::<u32>::some(32).reduce(OptionPolyfill::<u32>::some(16), |x, y| x + y), OptionPolyfill::some(48));
    /// assert_eq!(OptionPolyfill::<u32>::some(32).reduce(OptionPolyfill::<u32>::none(), |x, y| x + y), OptionPolyfill::some(32));
    /// assert_eq!(OptionPolyfill::<u32>::none().reduce(OptionPolyfill::<u32>::some(16), |x, y| x + y), OptionPolyfill::some(16));
    /// assert_eq!(OptionPolyfill::<u32>::none().reduce(OptionPolyfill::<u32>::none(), |x, y| x + y), OptionPolyfill::none());
    /// ```
    pub fn reduce(self, other: Self, f: impl FnOnce(T, T) -> T) -> Self {
        if self.is_some {
            if other.is_some {
                OptionPolyfill::some((f)(self.value, other.value))
            } else {
                self
            }
        } else {
            other
        }
    }
}

/// This trait is used to generate uninitialized values
pub trait Uninit {
    /// generates a uninitialized value
    fn uninit() -> Self;
}

impl<T: Default> Uninit for T {
    fn uninit() -> Self {
        Self::default()
    }
}
