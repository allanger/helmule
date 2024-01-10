use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct RegexpPatch {
    pub(crate) name: String,
    pub(crate) targets: Vec<String>,
    pub(crate) before: Option<String>,
    pub(crate) after: Option<String>,
}
