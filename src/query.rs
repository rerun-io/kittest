use crate::filter::By;
use crate::query::hidden::IterType;
use crate::Node;
use accesskit_consumer::FilterResult;
use std::collections::BTreeSet;
use std::iter::FusedIterator;

macro_rules! impl_helper {
    (
        $match_doc:literal,
        $query_all_name:ident,
        $get_all_name:ident,
        $query_name:ident,
        $get_name:ident,
        ($($args:ident: $arg_ty:ty),*),
        $by_expr:expr,
        $(#[$extra_doc:meta])*
    ) => {
        /// Query all nodes in the tree where
        #[doc = $match_doc]
        $(#[$extra_doc])*
        #[track_caller]
        fn $query_all_name(&'node self, $($args: $arg_ty),*) -> impl IterType<'tree> + 'tree {
            self.query_all($by_expr)
        }

        /// Get all nodes in the tree where
        #[doc = $match_doc]
        /// Returns at least one node.
        $(#[$extra_doc])*
        ///
        /// # Panics
        /// - if no nodes are found matching the query.
        #[track_caller]
        fn $get_all_name(&'node self, $($args: $arg_ty),*) -> impl IterType<'tree> + 'tree {
            self.get_all($by_expr)
        }

        /// Query a single node in the tree where
        #[doc = $match_doc]
        /// Returns `None` if no nodes are found.
        $(#[$extra_doc])*
        #[track_caller]
        fn $query_name(&'node self, $($args: $arg_ty),*) -> Option<Node<'tree>> {
            self.query($by_expr)
        }

        /// Get a single node in the tree where
        #[doc = $match_doc]
        $(#[$extra_doc])*
        ///
        /// # Panics
        /// - if no nodes are found matching the query.
        /// - if more than one node is found matching the query.
        #[track_caller]
        fn $get_name(&'node self, $($args: $arg_ty),*) -> Node<'tree> {
            self.get($by_expr)
        }
    };
}

/// Provides convenience methods for querying nodes in the tree, inspired by <https://testing-library.com/>.
pub trait Queryable<'tree, 'node> {
    fn node(&'node self) -> crate::Node<'tree>;

    impl_helper!(
        "the node name exactly matches given name.",
        query_all_by_name,
        get_all_by_name,
        query_by_name,
        get_by_name,
        (name: &'tree str),
        By::new().name(name),
        #[doc = ""]
        #[doc = "If the node is labelled by another node, the label node will not be included in the results."]
    );

    impl_helper!(
        "the node name contains the given substring.",
        query_all_by_name_contains,
        get_all_by_name_contains,
        query_by_name_contains,
        get_by_name_contains,
        (name: &'tree str),
        By::new().name_contains(name),
        #[doc = ""]
        #[doc = "If the node is labelled by another node, the label node will not be included in the results."]
    );

    impl_helper!(
        "the node role and name exactly match the given role and name.",
        query_all_by_role_and_name,
        get_all_by_role_and_name,
        query_by_role_and_name,
        get_by_role_and_name,
        (role: accesskit::Role, name: &'tree str),
        By::new().role(role).name(name),
        #[doc = ""]
        #[doc = "If the node is labelled by another node, the label node will not be included in the results."]
    );

    impl_helper!(
        "the node role matches the given role.",
        query_all_by_role,
        get_all_by_role,
        query_by_role,
        get_by_role,
        (role: accesskit::Role),
        By::new().role(role),
    );

    impl_helper!(
        "the node value exactly matches the given value.",
        query_all_by_value,
        get_all_by_value,
        query_by_value,
        get_by_value,
        (value: &'tree str),
        By::new().value(value),
    );

    impl_helper!(
        "the node matches the given predicate.",
        query_all_by,
        get_all_by,
        query_by,
        get_by,
        (f: impl Fn(&Node<'_>) -> bool + 'tree),
        By::new().predicate(f),
    );

    /// Query all nodes in the tree that match the given [`By`] query.
    #[track_caller]
    fn query_all(&'node self, by: By<'tree>) -> impl IterType<'tree> + 'tree {
        let should_filter_labels = by.should_filter_labels();

        let queue = self.node().queue();
        let results = self
            .node()
            .filtered_children(move |node| {
                if by.matches(&Node::new(*node, queue)) {
                    FilterResult::Include
                } else {
                    FilterResult::ExcludeNode
                }
            })
            .map(|node| Node::new(node, queue));

        let nodes = results.collect::<Vec<_>>();

        // If the widget label is provided by a different node, both will have the same name.
        // We only want to return the node that is labelled by the other node, not the label node.
        // (This matches the behavior of the testing-library getByLabelText query.)
        let labels: BTreeSet<_> = if should_filter_labels {
            nodes
                .iter()
                // TODO(lucas): It would be nicer if we could just get ids via something like labelled_by_ids
                .flat_map(|node| node.labelled_by())
                .map(|node| node.id())
                .collect()
        } else {
            BTreeSet::new()
        };

        nodes.into_iter().filter(move |node| {
            if should_filter_labels {
                !labels.contains(&node.id())
            } else {
                true
            }
        })
    }

    #[track_caller]
    fn get_all(&'node self, by: By<'tree>) -> impl IterType<'tree> + 'tree {
        let debug_query = by.debug_clone_without_predicate();
        let mut iter = self.query_all(by).peekable();
        assert!(iter.peek().is_some(), "No nodes found matching the query: {debug_query:?}");
        iter
    }

    #[track_caller]
    fn query(&'node self, by: By<'tree>) -> Option<Node<'tree>> {
        let debug_query = by.debug_clone_without_predicate();
        let mut iter = self.query_all(by);
        let result = iter.next();

        if let Some(second) = iter.next() {
            let first = result?;
            panic!(
                "Found two or more nodes matching the query: \n{debug_query:?}\n\nFirst node: {first:?}\nSecond node: {second:?}\
                \n\nIf you were expecting multiple nodes, use query_all instead of query."
            );
        }
        result
    }

    #[track_caller]
    fn get(&'node self, by: By<'tree>) -> Node<'tree> {
        let debug_query = by.debug_clone_without_predicate();
        let option = self.query(by);
        if let Some(node) = option {
            node
        } else {
            panic!("No nodes found matching the query: {debug_query:?}");
        }
    }
}

mod hidden {
    use super::{FusedIterator, Node};
    pub trait IterType<'tree>:
        DoubleEndedIterator<Item = Node<'tree>> + FusedIterator<Item = Node<'tree>>
    {
    }

    impl<'tree, T> IterType<'tree> for T where
        T: DoubleEndedIterator<Item = Node<'tree>> + FusedIterator<Item = Node<'tree>>
    {
    }
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
