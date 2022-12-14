use std::path::Path;

use axum::Router;
use home_automation_common::action::AutomationAction;
use tokio::sync::mpsc::UnboundedSender;

use crate::websocket::dto::AutomationServerStatusUpdate;

pub mod philipshue;
pub mod streamdeck;

pub trait AutomationModule {
    fn new(
        application_folder: &Path,
        status_update_sender: UnboundedSender<AutomationServerStatusUpdate>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn get_routes(&self) -> Option<Router>;
    fn handle_action(&mut self, automation_action: &AutomationAction) -> anyhow::Result<bool>;
    fn send_initial_state(&self, client_id: usize) -> anyhow::Result<()>;
}

pub struct CompositeAutomationModule {
    modules: Vec<Box<dyn AutomationModule + Send>>,
}

impl CompositeAutomationModule {
    pub fn add_module(&mut self, module: Box<dyn AutomationModule + Send>) {
        self.modules.push(module);
    }
}

impl AutomationModule for CompositeAutomationModule {
    fn new(_: &Path, _: UnboundedSender<AutomationServerStatusUpdate>) -> anyhow::Result<Self> {
        Ok(CompositeAutomationModule {
            modules: Vec::new(),
        })
    }

    fn get_routes(&self) -> Option<Router> {
        let routers_iter = self.modules.iter().filter_map(|module| module.get_routes());

        let mut merged_child_routers = Router::new();
        for router in routers_iter {
            merged_child_routers = merged_child_routers.merge(router);
        }

        let merged_routes = Router::new().nest("/api", merged_child_routers);
        Some(merged_routes)
    }

    fn handle_action(&mut self, automation_action: &AutomationAction) -> anyhow::Result<bool> {
        for module in &mut self.modules {
            let handled = module.handle_action(automation_action)?;
            if handled {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn send_initial_state(&self, client_id: usize) -> anyhow::Result<()> {
        for module in &self.modules {
            module.send_initial_state(client_id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use home_automation_common::test::TestContext;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use tokio::sync::mpsc::unbounded_channel;

    use super::*;

    struct TestData {
        module: CompositeAutomationModule,
    }

    #[test]
    fn send_initial_state() {
        let mut context = TestContext::<TestData>::just_setup(&setup);
        let module = &mut context.test_data.module;

        let test_module_data_1 = Arc::new(Mutex::new(TestModuleData {
            sent_initial_state: false,
            should_handle: false,
            handled_action: false,
        }));
        module.add_module(Box::new(TestModule::new(test_module_data_1.clone())));
        let test_module_data_2 = Arc::new(Mutex::new(TestModuleData {
            sent_initial_state: false,
            should_handle: false,
            handled_action: false,
        }));
        module.add_module(Box::new(TestModule::new(test_module_data_2.clone())));

        module.send_initial_state(1).unwrap();

        assert!(test_module_data_1.lock().unwrap().sent_initial_state);
        assert!(test_module_data_2.lock().unwrap().sent_initial_state);
    }

    #[test]
    fn handle_first() {
        let mut context = TestContext::<TestData>::just_setup(&setup);
        let module = &mut context.test_data.module;

        let test_module_data_1 = Arc::new(Mutex::new(TestModuleData {
            sent_initial_state: false,
            should_handle: true,
            handled_action: false,
        }));
        module.add_module(Box::new(TestModule::new(test_module_data_1.clone())));
        let test_module_data_2 = Arc::new(Mutex::new(TestModuleData {
            sent_initial_state: false,
            should_handle: true,
            handled_action: false,
        }));
        module.add_module(Box::new(TestModule::new(test_module_data_2.clone())));

        let action = AutomationAction::StreamdeckClientReloadDeviceConfiguration;
        let result = module.handle_action(&action);

        assert!(result.unwrap());
        assert!(test_module_data_1.lock().unwrap().handled_action);
        assert!(!test_module_data_2.lock().unwrap().handled_action);
    }

    #[test]
    fn handle_last() {
        let mut context = TestContext::<TestData>::just_setup(&setup);
        let module = &mut context.test_data.module;

        let test_module_data_1 = Arc::new(Mutex::new(TestModuleData {
            sent_initial_state: false,
            should_handle: false,
            handled_action: false,
        }));
        module.add_module(Box::new(TestModule::new(test_module_data_1.clone())));
        let test_module_data_2 = Arc::new(Mutex::new(TestModuleData {
            sent_initial_state: false,
            should_handle: true,
            handled_action: false,
        }));
        module.add_module(Box::new(TestModule::new(test_module_data_2.clone())));

        let action = AutomationAction::StreamdeckClientReloadDeviceConfiguration;
        let result = module.handle_action(&action);

        assert!(result.unwrap());
        assert!(!test_module_data_1.lock().unwrap().handled_action);
        assert!(test_module_data_2.lock().unwrap().handled_action);
    }

    #[test]
    fn not_handled() {
        let mut context = TestContext::<TestData>::just_setup(&setup);
        let module = &mut context.test_data.module;

        let test_module_data = Arc::new(Mutex::new(TestModuleData {
            sent_initial_state: false,
            should_handle: false,
            handled_action: false,
        }));
        module.add_module(Box::new(TestModule::new(test_module_data.clone())));

        let action = AutomationAction::StreamdeckClientReloadDeviceConfiguration;
        let result = module.handle_action(&action);

        assert!(!result.unwrap());
    }

    fn setup() -> TestData {
        let application_folder = PathBuf::new();
        let (tx, _) = unbounded_channel::<AutomationServerStatusUpdate>();
        let module = CompositeAutomationModule::new(&application_folder, tx).unwrap();

        TestData { module }
    }

    struct TestModuleData {
        should_handle: bool,
        handled_action: bool,
        sent_initial_state: bool,
    }

    struct TestModule {
        data: Arc<Mutex<TestModuleData>>,
    }

    impl TestModule {
        fn new(data: Arc<Mutex<TestModuleData>>) -> TestModule {
            TestModule { data }
        }
    }

    impl AutomationModule for TestModule {
        fn new(_: &Path, _: UnboundedSender<AutomationServerStatusUpdate>) -> anyhow::Result<Self>
        where
            Self: Sized,
        {
            unimplemented!()
        }

        fn get_routes(&self) -> Option<Router> {
            None
        }

        fn handle_action(&mut self, _: &AutomationAction) -> anyhow::Result<bool> {
            let mut data = self.data.lock().unwrap();
            if data.should_handle {
                data.handled_action = true;
                Ok(true)
            } else {
                Ok(false)
            }
        }

        fn send_initial_state(&self, _: usize) -> anyhow::Result<()> {
            let mut data = self.data.lock().unwrap();
            data.sent_initial_state = true;
            Ok(())
        }
    }
}
