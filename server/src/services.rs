use std::sync::Mutex;

use crate::automodule::CompositeAutomationModule;

pub struct ServicesContext {
    pub modules: Box<Mutex<CompositeAutomationModule>>,
}
