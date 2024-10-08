use crate::Node;
use accesskit::Role;
use std::fmt::{Debug, Formatter};

/// Create an empty filter.
/// Convenience function for [`By::new`].
pub fn by<'a>() -> By<'a> {
    By::new()
}

/// A filter for nodes.
/// The filters are combined with a logical AND.
pub struct By<'a> {
    name: Option<&'a str>,
    name_contains: bool,
    include_labels: bool,
    #[allow(clippy::type_complexity)]
    predicate: Option<Box<dyn Fn(&Node<'_>) -> bool + 'a>>,
    had_predicate: bool,
    role: Option<Role>,
    value: Option<&'a str>,
    pub(crate) recursive: bool,
}

impl<'a> Debug for By<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let By {
            name,
            name_contains,
            include_labels,
            predicate,
            had_predicate,
            role,
            value,
            recursive,
        } = self;
        let mut s = f.debug_struct("By");
        if let Some(name) = name {
            if *name_contains {
                s.field("name_contains", &name);
            } else {
                s.field("name", &name);
            }
        }
        if *include_labels {
            s.field("include_labels", &true);
        }
        if predicate.is_some() || *had_predicate {
            s.field("predicate", &"<function>");
        }
        if let Some(role) = role {
            s.field("role", &role);
        }
        if let Some(value) = value {
            s.field("value", &value);
        }
        if !*recursive {
            s.field("recursive", recursive);
        }
        s.finish()
    }
}

impl<'a> Default for By<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> By<'a> {
    /// Create an empty filter.
    pub fn new() -> Self {
        Self {
            name: None,
            name_contains: false,
            include_labels: false,
            predicate: None,
            had_predicate: false,
            role: None,
            value: None,
            recursive: true,
        }
    }

    /// Filter by the name of the node with an exact match.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Filter by the name of the node with a substring match.
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

    /// Filter by a custom predicate.
    pub fn predicate(mut self, predicate: impl Fn(&Node<'_>) -> bool + 'a) -> Self {
        self.predicate = Some(Box::new(predicate));
        self.had_predicate = true;
        self
    }

    /// Filter by the role of the node.
    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Filter by the value of the node with an exact match.
    pub fn value(mut self, value: &'a str) -> Self {
        self.value = Some(value);
        self
    }

    /// Should we search recursively?
    /// Default is true.
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Should the labels of labelled nodes be filtered?
    pub(crate) fn should_filter_labels(&self) -> bool {
        !self.include_labels && self.name.is_some()
    }

    /// Since we can't clone the predicate, we can't implement Clone for By.
    /// Since we still need to clone By in some cases to show debug info, and since the predicate
    /// can't be shown in debug info anyway, we just don't clone the predicate and
    /// just remember if we had one.
    pub(crate) fn debug_clone_without_predicate(&self) -> Self {
        Self {
            name: self.name,
            name_contains: self.name_contains,
            include_labels: self.include_labels,
            predicate: None,
            had_predicate: self.had_predicate,
            role: self.role,
            value: self.value,
            recursive: self.recursive,
        }
    }

    /// Returns true if the given node matches this filter.
    /// Note: For correct filtering if [`Self::include_labels`] is false, the tree should be
    /// filtered like in [`crate::Queryable::query_all`].
    /// Note: Remember to check for recursive filtering
    pub(crate) fn matches(&self, node: &Node<'_>) -> bool {
        if let Some(name) = self.name {
            if let Some(node_name) = node.name() {
                if self.name_contains {
                    if !node_name.contains(name) {
                        return false;
                    }
                } else if node_name != name {
                    return false;
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
