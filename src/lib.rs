use obs_wrapper::{
    // Macro for registering modules
    obs_register_module,
    // Macro for creating strings
    obs_string,
    // Everything required for modules
    prelude::*,
    // Everything required for creating a source
    source::*,
};

// The module that will handle creating the source.
struct TestModule {
    context: ModuleContext,
}

// The source that will be shown inside OBS.
struct TestSource;

// The state of the source that is managed by OBS and used in each trait method.
struct SourceData;

// Implement the Sourceable trait for TestSource, this is required for each source.
// It allows you to specify the source ID and type.
impl Sourceable for TestSource {
    fn get_id() -> ObsString {
        obs_string!("test_source")
    }

    fn get_type() -> SourceType {
        SourceType::FILTER
    }
}

// Allow OBS to show a name for the source
impl GetNameSource<SourceData> for TestSource {
    fn get_name() -> ObsString {
        obs_string!("Test Source")
    }
}

// Implement the Module trait for TestModule. This will handle the creation of the source and
// has some methods for telling OBS a bit about itself.
impl Module for TestModule {
    fn new(context: ModuleContext) -> Self {
        Self { context }
    }

    fn get_ctx(&self) -> &ModuleContext {
        &self.context
    }

    // Load the module - create all sources, returning true if all went well.
    fn load(&mut self, load_context: &mut LoadContext) -> bool {
        // Create the source
        let source = load_context
            .create_source_builder::<TestSource, SourceData>()
            // Since GetNameSource is implemented, this method needs to be called to
            // enable it.
            .enable_get_name()
            .build();

        // Tell OBS about the source so that it will show it.
        load_context.register_source(source);

        // Nothing could have gone wrong, so return true.
        true
    }

    fn description() -> ObsString {
        obs_string!("A great test module.")
    }

    fn name() -> ObsString {
        obs_string!("Test Module")
    }

    fn author() -> ObsString {
        obs_string!("TODO")
    }
}
