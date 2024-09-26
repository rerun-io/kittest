use crate::filter::By;
use crate::query::hidden::IterType;
use crate::Node;
use accesskit_consumer::{FilterResult, Node as AKNode};
use std::collections::BTreeSet;
use std::iter::{Filter, FusedIterator};
use std::ops::Deref;

fn query_by_impl<'tree>(mut iter: impl Iterator<Item = Node<'tree>>) -> Option<Node<'tree>> {
    let result = iter.next();

    if let Some(second) = iter.next() {
        let first = result?;
        panic!(
            "Found two or more nodes matching the query:\n{:?}\n{:?}",
            first, second,
        );
    }
    result
}

fn get_all_by_impl<'tree>(mut iter: impl IterType<'tree>) -> impl IterType<'tree> {
    let mut iter = iter.peekable();
    if iter.peek().is_none() {
        panic!("No nodes found matching the query");
    }
    iter
}

/// Provides convenience methods for querying nodes in the tree, inspired by https://testing-library.com/.
pub trait Queryable<'tree, 'node> {
    fn node(&'node self) -> crate::Node<'tree>;

    /// Query all nodes in the tree that match the given [`By`] query.
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

    fn get_all(&'node self, by: By<'tree>) -> impl IterType<'tree> + 'tree {
        get_all_by_impl(self.query_all(by))
    }

    fn query(&'node self, by: By<'tree>) -> Option<Node<'tree>> {
        query_by_impl(self.query_all(by))
    }

    fn get(&'node self, by: By<'tree>) -> Node<'tree> {
        self.query(by).expect("No node found matching the query")
    }

    /// Query all nodes in the tree that match the given predicate.
    fn query_all_by(
        &'node self,
        f: impl Fn(&Node<'_>) -> bool + 'tree,
    ) -> impl IterType<'tree> + 'tree {
        self.query_all(By::new().predicate(f))
    }

    /// Get all nodes in the tree that match the given predicate.
    /// Returns at least one node.
    ///
    /// # Panics
    /// Will panic if no nodes are found matching the query.
    fn get_all_by(
        &'node self,
        f: impl Fn(&Node<'_>) -> bool + 'tree,
    ) -> impl IterType<'tree> + 'tree {
        self.get_all(By::new().predicate(f))
    }

    /// Query a single node in the tree that matches the given predicate.
    /// Returns `None` if no nodes are found.
    ///
    /// # Panics
    /// Will panic if more than one node is found matching the query.
    fn query_by(&'node self, f: impl Fn(&Node<'_>) -> bool + 'tree) -> Option<Node<'tree>> {
        self.query(By::new().predicate(f))
    }

    /// Get a single node in the tree that matches the given predicate.
    ///
    /// # Panics
    /// Will panic if no nodes are found matching the query.
    /// Will panic if more than one node is found matching the query.
    fn get_by(&'node self, f: impl Fn(&Node<'_>) -> bool + 'tree) -> Node<'tree> {
        self.get(By::new().predicate(f))
    }

    /// Query all nodes in the tree with the given name. The name must be an exact match.
    /// If the node is labelled by another node, the label node will not be returned.
    fn query_all_by_name(&'node self, name: &'tree str) -> impl IterType<'tree> + 'tree {
        self.query_all(By::new().name(name))
    }

    /// Find all nodes in the tree with the given name. The name must be an exact match.
    /// If the node is labelled by another node, the label node will not be returned.
    /// Returns at least one node.
    ///
    /// # Panics
    /// Will panic if no nodes are found matching the query.
    fn get_all_by_name(&'node self, name: &'tree str) -> impl IterType<'tree> + 'tree {
        self.get_all(By::new().name(name))
    }

    fn query_by_name(&'node self, name: &'tree str) -> Option<Node<'tree>> {
        self.query(By::new().name(name))
    }

    fn get_by_name(&'node self, name: &'tree str) -> Node<'tree> {
        self.get(By::new().name(name))
    }

    fn query_all_by_role(&'node self, role: accesskit::Role) -> impl IterType<'tree> + 'tree {
        self.query_all(By::new().role(role))
    }

    fn get_all_by_role(&'node self, role: accesskit::Role) -> impl IterType<'tree> + 'tree {
        self.get_all(By::new().role(role))
    }

    fn query_by_role(&'node self, role: accesskit::Role) -> Option<Node<'tree>> {
        self.query(By::new().role(role))
    }

    fn get_by_role(&'node self, role: accesskit::Role) -> Node<'tree> {
        self.get(By::new().role(role))
    }
}


mod hidden {
    use super::*;
    pub trait IterType<'tree>:
    DoubleEndedIterator<Item=Node<'tree>> + FusedIterator<Item=Node<'tree>>
    {}

    impl<'tree, T> IterType<'tree> for T
    where
        T: DoubleEndedIterator<Item=Node<'tree>> + FusedIterator<Item=Node<'tree>>,
    {}
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
