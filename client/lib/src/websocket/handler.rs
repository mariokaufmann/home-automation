use home_automation_common::action::AutomationStatusUpdate;

pub trait AutomationStatusUpdateHandler: Send {
    fn on_status_update(&mut self, status_update: AutomationStatusUpdate);
}
