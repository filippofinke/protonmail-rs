//! Label / folder model.

use serde::{Deserialize, Serialize};

/// A label or folder (distinguished by `label_type`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Label {
    /// Label/folder ID.
    #[serde(rename = "ID")]
    pub id: String,
    /// Display name.
    #[serde(default)]
    pub name: String,
    /// Display color (hex).
    #[serde(default)]
    pub color: String,
    /// Full path (parent folders joined), for nested folders.
    #[serde(default)]
    pub path: String,
    /// Label kind (1 = label, 2 = contact group, 3 = folder, 4 = system folder).
    #[serde(rename = "Type", default)]
    pub label_type: i64,
    /// ID of the parent folder, for nested folders.
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Sort order.
    #[serde(default)]
    pub order: i64,
    /// `1` if the folder triggers notifications.
    #[serde(default)]
    pub notify: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_label() {
        let json =
            serde_json::json!({"ID":"42","Name":"Work","Color":"#fff","Type":3,"Path":"Work"});
        let l: Label = serde_json::from_value(json).unwrap();
        assert_eq!(l.id, "42");
        assert_eq!(l.label_type, 3);
        assert_eq!(l.name, "Work");
    }
}
