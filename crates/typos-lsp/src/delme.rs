struct BackendState<'s> {
    storage: Storage,
    config: Config<'s>,
}
struct Storage {}
struct Config<'s> {
    engine: Engine<'s>,
}
struct Engine<'s> {
    storage: &'s Storage,
}

impl <s> BackendState<'s> {
    fn update_config(&'s mut self) {
        self.storage = Storage {};
        let engine = Engine {
            storage: &self.storage,
        };
        self.config = Config { engine };
    }
}
