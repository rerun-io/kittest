use crate::filter::By;
use crate::{AccessKitNode, NodeT};
use std::collections::BTreeSet;
use std::iter::FusedIterator;

#[allow(clippy::needless_pass_by_value)]
#[track_caller]
fn query_all<'tree, Node: NodeT<'tree> + 'tree>(
    node: Node,
    by: By<'tree>,
) -> impl DoubleEndedIterator<Item = Node> + FusedIterator<Item = Node> + 'tree {
    let should_filter_labels = by.should_filter_labels();

    let results = node
        .children_maybe_recursive(by.recursive)
        .filter(move |node| by.matches(&node.accesskit_node()));

    let nodes = results.collect::<Vec<_>>();

    // If the widget label is provided by a different node, both will have the same label.
    // We only want to return the node that is labelled by the other node, not the label node.
    // (This matches the behavior of the testing-library getByLabelText query.)
    let labels: BTreeSet<_> = if should_filter_labels {
        nodes
            .iter()
            // TODO(lucas): It would be nicer if we could just get ids via something like labelled_by_ids
            .flat_map(|node| node.accesskit_node().labelled_by())
            .map(|node| node.id())
            .collect()
    } else {
        BTreeSet::new()
    };

    nodes.into_iter().filter(move |node| {
        if should_filter_labels {
            !labels.contains(&node.accesskit_node().id())
        } else {
            true
        }
    })
}

#[allow(clippy::needless_pass_by_value)]
#[track_caller]
fn get_all<'tree, Node: NodeT<'tree> + 'tree>(
    node: Node,
    by: By<'tree>,
) -> impl DoubleEndedIterator<Item = Node> + FusedIterator<Item = Node> + 'tree {
    let debug_query = by.debug_clone_without_predicate();
    let mut iter = query_all(node.clone(), by).peekable();
    assert!(
        iter.peek().is_some(),
        "No nodes found matching the query:\n{debug_query:#?}\n\nOn node:\n{node:#?}"
    );
    iter
}

#[allow(clippy::needless_pass_by_value)]
#[track_caller]
fn query<'tree, Node: NodeT<'tree> + 'tree>(node: Node, by: By<'tree>) -> Option<Node> {
    let debug_query = by.debug_clone_without_predicate();
    let mut iter = query_all(node, by);
    let result = iter.next();

    if let Some(second) = iter.next() {
        let first = result?;
        panic!(
            "Found two or more nodes matching the query: \n{debug_query:#?}\n\nFirst node:\n{first:#?}\n\nSecond node: {second:#?}\
                \n\nIf you were expecting multiple nodes, use query_all instead of query."
        );
    }
    result
}

#[allow(clippy::needless_pass_by_value)]
#[track_caller]
fn get<'tree, Node: NodeT<'tree> + 'tree>(node: Node, by: By<'tree>) -> Node {
    let debug_query = by.debug_clone_without_predicate();
    let option = query(node.clone(), by);
    if let Some(node) = option {
        node
    } else {
        panic!("No nodes found matching the query:\n{debug_query:#?}\n\nOn node:\n{node:#?}");
    }
}

macro_rules! impl_helper {
    (
        $match_doc:literal,
        $query_all_label:ident,
        $get_all_label:ident,
        $query_label:ident,
        $get_label:ident,
        $node:ident,
        ($($args:ident: $arg_ty:ty),*),
        $by_expr:expr,
        $(#[$extra_doc:meta])*
    ) => {
        /// Query all nodes in the tree where
        #[doc = $match_doc]
        $(#[$extra_doc])*
        #[track_caller]
        fn $query_all_label(
            &'node self, $($args: $arg_ty),*
        ) -> impl DoubleEndedIterator<Item = $node> + FusedIterator<Item = $node> + 'tree {
            query_all(self.queryable_node(), $by_expr)
        }

        /// Get all nodes in the tree where
        #[doc = $match_doc]
        /// Returns at least one node.
        $(#[$extra_doc])*
        ///
        /// # Panics
        /// - if no nodes are found matching the query.
        #[track_caller]
        fn $get_all_label(
            &'node self, $($args: $arg_ty),*
        ) -> impl DoubleEndedIterator<Item = $node> + FusedIterator<Item = $node> + 'tree {
            get_all(self.queryable_node(), $by_expr)
        }

        /// Query a single node in the tree where
        #[doc = $match_doc]
        /// Returns `None` if no nodes are found.
        $(#[$extra_doc])*
        #[track_caller]
        fn $query_label(&'node self, $($args: $arg_ty),*) -> Option<$node> {
            query(self.queryable_node(), $by_expr)
        }

        /// Get a single node in the tree where
        #[doc = $match_doc]
        $(#[$extra_doc])*
        ///
        /// # Panics
        /// - if no nodes are found matching the query.
        /// - if more than one node is found matching the query.
        #[track_caller]
        fn $get_label(&'node self, $($args: $arg_ty),*) -> $node {
            get(self.queryable_node(), $by_expr)
        }
    };
}

/// Provides convenience methods for querying nodes in the tree, inspired by <https://testing-library.com/>.
pub trait Queryable<'tree, 'node, Node: NodeT<'tree> + 'tree> {
    fn queryable_node(&'node self) -> Node;

    impl_helper!(
        "the node matches the given [`By`] filter.",
        query_all,
        get_all,
        query,
        get,
        Node,
        (by: By<'tree>),
        by,
    );

    impl_helper!(
        "the node label exactly matches given label.",
        query_all_by_label,
        get_all_by_label,
        query_by_label,
        get_by_label,
        Node,
        (label: &'tree str),
        By::new().label(label),
        #[doc = ""]
        #[doc = "If a node is labelled by another node, the label node will not be included in the results."]
    );

    impl_helper!(
        "the node label contains the given substring.",
        query_all_by_label_contains,
        get_all_by_label_contains,
        query_by_label_contains,
        get_by_label_contains,
        Node,
        (label: &'tree str),
        By::new().label_contains(label),
        #[doc = ""]
        #[doc = "If a node is labelled by another node, the label node will not be included in the results."]
    );

    impl_helper!(
        "the node role and label exactly match the given role and label.",
        query_all_by_role_and_label,
        get_all_by_role_and_label,
        query_by_role_and_label,
        get_by_role_and_label,
        Node,
        (role: accesskit::Role, label: &'tree str),
        By::new().role(role).label(label),
        #[doc = ""]
        #[doc = "If a node is labelled by another node, the label node will not be included in the results."]
    );

    impl_helper!(
        "the node role matches the given role.",
        query_all_by_role,
        get_all_by_role,
        query_by_role,
        get_by_role,
        Node,
        (role: accesskit::Role),
        By::new().role(role),
    );

    impl_helper!(
        "the node value exactly matches the given value.",
        query_all_by_value,
        get_all_by_value,
        query_by_value,
        get_by_value,
        Node,
        (value: &'tree str),
        By::new().value(value),
    );

    impl_helper!(
        "the node matches the given predicate.",
        query_all_by,
        get_all_by,
        query_by,
        get_by,
        Node,
        (f: impl Fn(&AccessKitNode<'_>) -> bool + 'tree),
        By::new().predicate(f),
    );
}

// TODO: query_all could be optimized by returning different iterators based on should_filter_labels
//
// enum QueryAll<'a, 'b, Filter: FnMut(&'b Node<'b>) -> bool, I: IterType<'a>> {
//     FilterLabels(std::iter::Filter<std::vec::IntoIter<Node<'a>>, Filter>),
//     IncludeLabels(I),
// }
//
// impl<'a, 'b, Filter: FnMut(&'b Node<'b>) -> bool, I: IterType<'a>> Iterator for QueryAll<'a, 'b, Filter, I> {
//     type Item = Node<'a>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//             QueryAll::FilterLabels(i) => i.next(),
//             QueryAll::IncludeLabels(i) => i.next(),
//         }
//     }
// }
//
// impl<'a, 'b, Filter: FnMut(&'b Node<'b>) -> bool, I: IterType<'a>> DoubleEndedIterator
//     for QueryAll<'a, 'b, Filter, I>
// {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         match self {
//             QueryAll::FilterLabels(i) => i.next_back(),
//             QueryAll::IncludeLabels(i) => i.next_back(),
//         }
//     }
// }
//
// impl<'a, 'b, Filter: FnMut(&'b Node<'b>) -> bool, I: IterType<'a>> FusedIterator for QueryAll<'a, 'b, Filter, I> {}

// TODO: I would like to add the find_by_* methods but I'm not sure how I would update the
// application from here?
//
// pub trait Findable<'tree, 'node, 's>: Queryable<'tree, 'node> {
//     fn run(&mut self);
//
//     fn find_timeout(&self) -> std::time::Duration {
//         std::time::Duration::from_secs(5)
//     }
//
//     fn find_all_by(
//         &'node mut self,
//         f: impl Fn(&Node<'_>) -> bool + Copy + 'tree,
//     ) -> impl IterType<'tree> + 'tree {
//         let timeout = self.find_timeout();
//         let step = timeout / 10;
//
//         let mut start_time = std::time::Instant::now();
//
//         loop {
//             {
//                 let node = self.node();
//                 let iter = node.query_all_by(f);
//                 let mut peekable = iter.peekable();
//                 if !peekable.peek().is_none() {
//                     return peekable;
//                 }
//
//                 if start_time.elapsed() > timeout {
//                     panic!("Timeout exceeded while waiting for node");
//                 }
//             }
//
//             std::thread::sleep(step);
//         }
//     }
// }
