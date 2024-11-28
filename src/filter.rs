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
    label: Option<&'a str>,
    label_contains: bool,
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
            label,
            label_contains,
            include_labels,
            predicate,
            had_predicate,
            role,
            value,
            recursive,
        } = self;
        let mut s = f.debug_struct("By");
        if let Some(label) = label {
            if *label_contains {
                s.field("label_contains", &label);
            } else {
                s.field("label", &label);
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
            label: None,
            label_contains: false,
            include_labels: false,
            predicate: None,
            had_predicate: false,
            role: None,
            value: None,
            recursive: true,
        }
    }

    /// Filter by the label of the node with an exact match.
    ///
    /// Note that [in AccessKit](https://docs.rs/accesskit/latest/accesskit/struct.Node.html#method.label),
    /// a widget with `Role::Label`, stores it's label in `Node::value`.
    /// We check for this and use the value if the role is `Role::Label`.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Filter by the label of the node with a substring match.
    ///
    /// Note that [in AccessKit](https://docs.rs/accesskit/latest/accesskit/struct.Node.html#method.label),
    /// a widget with `Role::Label`, stores it's label in `Node::value`.
    /// We check for this and use the value if the role is `Role::Label`.
    pub fn label_contains(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self.label_contains = true;
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
        !self.include_labels && self.label.is_some()
    }

    /// Since we can't clone the predicate, we can't implement Clone for By.
    /// Since we still need to clone By in some cases to show debug info, and since the predicate
    /// can't be shown in debug info anyway, we just don't clone the predicate and
    /// just remember if we had one.
    pub(crate) fn debug_clone_without_predicate(&self) -> Self {
        Self {
            label: self.label,
            label_contains: self.label_contains,
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
        if let Some(label) = self.label {
            // In AccessKit, a widget with `Role::Label`, stores it's label in `Node::value`.
            let node_label = if node.role() == Role::Label {
                node.value()
            } else {
                node.label()
            };

            if let Some(node_label) = node_label {
                if self.label_contains {
                    if !node_label.contains(label) {
                        return false;
                    }
                } else if node_label != label {
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
