use crate::Node;
use accesskit::Role;

pub fn by<'a>() -> By<'a> {
    By::new()
}

/// A filter for nodes.
/// The filters are combined with a logical AND.
pub struct By<'a> {
    name: Option<&'a str>,
    name_contains: bool,
    include_labels: bool,
    predicate: Option<Box<dyn Fn(&Node<'_>) -> bool + 'a>>,
    role: Option<Role>,
    value: Option<&'a str>,
}

impl<'a> By<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            name_contains: false,
            include_labels: false,
            predicate: None,
            role: None,
            value: None,
        }
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn name_contains(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self.name_contains = true;
        self
    }

    /// If a node is labelled by another node, should the label node be included in the results?
    /// Default is false.
    pub fn include_labels(mut self) -> Self {
        self.include_labels = true;
        self
    }

    pub fn predicate(mut self, predicate: impl Fn(&Node<'_>) -> bool + 'a) -> Self {
        self.predicate = Some(Box::new(predicate));
        self
    }

    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    pub fn value(mut self, value: &'a str) -> Self {
        self.value = Some(value);
        self
    }


    pub(crate) fn should_filter_labels(&self) -> bool {
        !self.include_labels && self.name.is_some()
    }

    pub(crate) fn matches(&self, node: &Node<'_>) -> bool {
        if let Some(name) = self.name {
            if let Some(node_name) = node.name() {
                if self.name_contains {
                    if !node_name.contains(name) {
                        return false;
                    }
                } else {
                    if node_name != name {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }

        if let Some(predicate) = &self.predicate {
            if !predicate(node) {
                return false;
            }
        }

        if let Some(role) = self.role {
            if node.role() != role {
                return false;
            }
        }

        if let Some(value) = self.value {
            if let Some(node_value) = node.value() {
                if node_value != value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}
