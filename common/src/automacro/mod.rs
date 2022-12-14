use crate::action::AutomationAction;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AutomationMacro {
    name: String,
    pub actions: Vec<AutomationAction>,
}

impl AutomationMacro {
    pub fn new(name: String, actions: Vec<AutomationAction>) -> AutomationMacro {
        AutomationMacro { name, actions }
    }
}
