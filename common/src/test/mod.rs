pub struct TestContext<'a, Testdata> {
    pub test_data: Testdata,
    teardown_fn: Option<&'a dyn Fn(&mut Testdata)>,
}

impl<'a, Testdata> TestContext<'a, Testdata> {
    pub fn setup(setup_fn: &dyn Fn() -> Testdata, teardown_fn: &'a dyn Fn(&mut Testdata)) -> Self {
        let test_data = setup_fn();

        Self {
            test_data,
            teardown_fn: Some(teardown_fn),
        }
    }

    pub fn just_setup(setup_fn: &dyn Fn() -> Testdata) -> Self {
        let test_data = setup_fn();

        Self {
            test_data,
            teardown_fn: None,
        }
    }
}

impl<'a, Testdata> Drop for TestContext<'a, Testdata> {
    fn drop(&mut self) {
        if let Some(teardown_fn) = self.teardown_fn {
            teardown_fn(&mut self.test_data);
        }
    }
}
