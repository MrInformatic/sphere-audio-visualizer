use crate::utils::TypeMap;

/// The [`Module`] trait is used by different visualizer pipelines as pipline
/// element.
/// A [`Module`] contains settings from which it could be reconstructed.
pub trait Module: Default + Send + Sync {
    /// The Type of the Settings
    type Settings: Default + Clone + Send + Sync;

    /// Creates a new instance from the module settings
    fn from_settings(settings: Self::Settings) -> Self {
        Self::default().with_settings(settings)
    }

    /// Sets the module settings
    fn with_settings(mut self, settings: Self::Settings) -> Self {
        self.set_settings(settings);
        self
    }

    /// Sets the module settings
    fn set_settings(&mut self, settings: Self::Settings) -> &mut Self;

    /// Gets the module settings
    fn settings(&self) -> Self::Settings;
}

/// Stores module settings and modules for recycling.
pub struct ModuleManager<'a> {
    module_bin: TypeMap,
    settings_bin: &'a mut TypeMap,
}

impl<'a> ModuleManager<'a> {
    /// Creates a new instance from a collection of module settings
    pub fn new(settings_bin: &'a mut TypeMap) -> Self {
        Self {
            module_bin: TypeMap::new(),
            settings_bin,
        }
    }

    /// Insterts a module
    pub fn insert<M: Module + 'static>(&mut self, module: M)
    where
        <M as Module>::Settings: 'static,
    {
        self.settings_bin.insert(module.settings());
        self.module_bin.insert(module);
    }

    /// Inserts a object without settings it still gets recycled but the
    /// settings are lost.
    pub fn insert_lossy<M: Send + Sync + 'static>(&mut self, module: M) {
        self.module_bin.insert(module);
    }

    /// Extracts a module. If the module could not be recycled it tries to
    /// recreate the module from settings. If the settings could not be found
    /// it uses the default settings.
    pub fn extract<M: Module + 'static>(&mut self) -> M
    where
        <M as Module>::Settings: 'static,
    {
        let settings = self
            .settings_bin
            .get::<M::Settings>()
            .cloned()
            .unwrap_or_default();

        self.extract_or_default::<M>().with_settings(settings)
    }

    /// Extracts a object. Only returns Some if the object could be recycled.
    pub fn extract_optional<M: Send + Sync + 'static>(&mut self) -> Option<M> {
        self.module_bin.remove::<M>()
    }

    /// Extracts a object. Creates a object with default initializer if it
    /// could not be recycled.
    pub fn extract_or_default<M: Default + Send + Sync + 'static>(&mut self) -> M {
        self.extract_optional::<M>().unwrap_or_default()
    }

    /// Extracts a object. Uses the passed initializer if it could not recycle
    /// the object.
    pub fn extract_or_else<M: Send + Sync + 'static>(&mut self, f: impl FnOnce() -> M) -> M {
        self.extract_optional::<M>().unwrap_or_else(f)
    }
}
