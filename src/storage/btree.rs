/*
  The family of data structures called "B-trees" are effective data structures for the minimizing the number of random disk accesses when operating on data that
  doesn't fit into memory. The fact b-trees can be used to maximise sequential disk access and minimise random access on
  disk which is way slower is why they are such a good fit for databases.

  A btree allows for a notion of locality grouping "nearby" interior nodes into a bigger node which
  maps to a "page" on disk. The page is smallest amount of disk data which is allocatable to memory.
  A disk page is interchangeable with a node in our B-tree so consider them interchangeable terms
  in this context. Think of btrees as a way of organizing disk pages. Puting many keys in a node (determined by page size)
  mean there is one sequential disk seek to get all of these keys (and their values/refs).
  A naive binary search tree on the other hand would be a bad fit as there is no idea of locality,
  every key comparison would equate to a random disk seek.

  See the CLRS book which proves that a B-tree with a height of two and 1001 children is able to store more than one billion keys
  and yet only a disk access of two is required to find any key (Cormen et al., 2009).

  Lets implement a B+tree from scratch.

  The B+Tree differs from a classical B-Tree in that we store all the data in the leaf nodes
  and the non leaf nodes act only as sign posts. Think of the bottom level of leaf nodes
  with sibling pointers as a linked-list where the nodes above it guide us efficiently to
  some node in this linked-list of leaf nodes.

    B+trees are used mainly over B-trees in practice because they are more space efficient.
  This is because the values for keys (PageId for tuple holding the key) are only in leaf nodes,
  which gives us a higher fanout. As for the terminology: Fanout = branching factor = n = degree = order.
  Fanout determines how many keys we can squeeze into a node/page and consequently the number of
  child pointers in an inner node. Higher fanout means smaller tree height and the faster the lookups.

*/
use anyhow::{anyhow, Result};
use std::cmp::Ordering;
use std::vec::{self, Vec};

#[derive(Debug, PartialEq)]
struct Btree<'a, K: Ord, V: Ord> {
    interior_node_count: u64, // The k in "k-ary btree" or number of interior node per node.
    root: RootNode<'a, K, V>,
}

impl<'a, K: Ord, V: Ord> Btree<'a, K, V> {
    pub fn new(interior_node_count: u64, key: K, value: V) -> Self {
        let mut root: RootNode<'a, K, V> = RootNode {
            interior_nodes: vec![],
        };

        let interior_node: InnerNodeInterior<'a, K, V> =
            InnerNodeInterior::new(key, value, Vec::new());

        root.insert_interior_node(interior_node);

        Btree {
            interior_node_count,
            root,
        }
    }
}

#[derive(Debug)]
enum NonRootNode<'a, K: Ord, V: Ord> {
    Inner(InnerNode<'a, K, V>),
    Leaf(InnerNodeInterior<'a, K, V>),
}

impl<'a, K: Ord, V: Ord> InnerNodeInterior<'a, K, V> {
    pub fn new(
        key: K,
        value: V,
        children: Vec<NonRootNode<'a, K, V>>,
    ) -> InnerNodeInterior<'a, K, V> {
        InnerNodeInterior {
            key,
            value,
            children,
        }
    }
}

impl<'a, K: Ord, V: Ord> PartialEq for InnerNodeInterior<'a, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<'a, K: Ord, V: Ord> PartialOrd for InnerNodeInterior<'a, K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, K: Ord + PartialOrd, V: Ord> Ord for InnerNodeInterior<'a, K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.key).cmp(&other.key)
    }
}

impl<'a, K: Ord + PartialOrd, V: Ord> Eq for InnerNodeInterior<'a, K, V> {}

// Generic operations on all nodes.
trait HasInteriorNodes<'a, K: Ord, V: Ord> {
    fn find_interior_node(&self) -> Option<&InnerNodeInterior<'a, K, V>>;

    fn insert_interior_node(&mut self, n: InnerNodeInterior<'a, K, V>) -> ();

    fn delete_interior_node(&mut self, key: K) -> ();
}

#[derive(Debug, PartialEq)]
struct RootNode<'a, K: Ord, V: Ord> {
    interior_nodes: Vec<InnerNodeInterior<'a, K, V>>,
}

// An Inner node is a node that is not a leaf node and not a root Node.
#[derive(Debug)]
struct InnerNode<'a, K: Ord, V: Ord> {
    // -- Our guideposts to get to leaf Nodes which hold the actual data..
    interior_nodes: Vec<InnerNodeInterior<'a, K, V>>, // sorted by K to enable binary search lookup

    // -- Metadata
    // We don't have a pointer to parent because allowing backtracking will open
    // the door to deadlocks when concurrent access to the B+Tree occurs.
    left_sibling: Option<&'a InnerNode<'a, K, V>>,
    right_sibling: Option<&'a InnerNode<'a, K, V>>,
}

#[derive(Debug)]
struct InnerNodeInterior<'a, K: Ord, V: Ord> {
    key: K,   //   Key is used to maintain the order of the tree,
    value: V, // Value is the actual data being stored.
    // Maximum of N + 1 child nodes
    children: Vec<NonRootNode<'a, K, V>>,
}

impl<'a, K: Ord, V: Ord> InnerNode<'a, K, V> {
    pub fn new(
        right_sibling: Option<&'a InnerNode<'a, K, V>>,
        left_sibling: Option<&'a InnerNode<'a, K, V>>,
        key: K,
        value: V,
    ) -> InnerNode<'a, K, V> {
        InnerNode {
            left_sibling,
            right_sibling,
            // Maximum of N interior nodes
            interior_nodes: vec![InnerNodeInterior {
                key,
                value,
                children: Vec::new(),
            }],
        }
    }
}

#[derive(Debug)]
struct LeafNode<'a, K: Ord, V: Ord> {
    // -- Our guideposts --
    // to get to leaf Nodes which hold the actual data..
    interior_nodes: Vec<LeafNodeInterior<K, V>>, // sorted by K to enable binary search lookup

    // -- Metadata --
    // We don't have a pointer to parent because allowing backtracking will open
    // the door to deadlocks when concurrent access to the B+Tree occurs.
    left_sibling: Option<&'a InnerNode<'a, K, V>>,
    right_sibling: Option<&'a InnerNode<'a, K, V>>,
}

#[derive(Debug)]
struct LeafNodeInterior<K: Ord, V> {
    key: K,
    value: V, // Value is the actual data being stored. The PageId in our case for the tuple
              // that is associated with the attribute(s) represented by the key.
}

impl<'a, K: Ord, V: Ord> HasInteriorNodes<'a, K, V> for RootNode<'a, K, V> {
    // insert in sorted order
    fn insert_interior_node(&mut self, n: InnerNodeInterior<'a, K, V>) -> () {
        match self.interior_nodes.binary_search(&n) {
            Ok(pos) | Err(pos) => self.interior_nodes.insert(pos, n),
        }
    }

    fn find_interior_node(&self) -> Option<&InnerNodeInterior<'a, K, V>> {
        None
    }

    fn delete_interior_node(&mut self, _key: K) -> () {}
}

// Public interface

fn find() {}

fn insert() {}

fn delete() {}

/* Private Interface - balancing operations */

fn split() {}

fn merge() {}

#[cfg(test)]
mod tests {
    use super::*;

    /*
        Unit Tests
    */

    #[test]
    fn new_btree_inits_correctly_with_single_key_value() {
        let interior_node_count: u64 = 2;
        let key: u8 = 1;
        let value: u8 = 2;
        let init_btree: Btree<u8, u8> = Btree::new(interior_node_count, key, value);
        let expected_btree = Btree {
            interior_node_count,
            root: RootNode {
                interior_nodes: vec![InnerNodeInterior {
                    key,
                    value,
                    children: Vec::new(),
                }],
            },
        };

        assert_eq!(init_btree, expected_btree);
    }

    /*
         Property Based Tests (PBTs)

         According to Knuth's Definition a B Tree has the following properties.

             1. Every node has at most m children.
             2. Every node, except for the root and the leaves, has at least ⌈m/2⌉ children.
             3. The root node has at least two children unless it is a leaf.
             4. All leaves appear on the same level.
             5. A non-leaf node with k children contains k−1 keys.

          Lets Test these properties with PBT.
    */
}
