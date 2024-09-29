use anyhow::{anyhow, Result};
use std::cmp::Ordering;
use std::vec::{self, Vec};

/*

  Btrees are effective data structures for the minimizing the number of random disk accesses when operating on data that
  doesn't fit into memory. Thie fact b-trees can be used to maximise sequential disk access and minimise random access on
  disk which is way slower is why they are such a good fit for databases.

  A btree allows for a notion of locality grouping "nearby" interior nodes into a bigger node which
  maps to a "page" on disk. The page is smallest amount of disk data which is allocatable to memory.

  grouping interior nodes together, so interior node comparisons equate to a sequential disk seek.
  A naive binary search tree on the other hand would be a bad fit as there is no idea of locality,
  every node comparison equates to a random disk seek.

  See the CLRS book which
  which proves that a B-tree with a height of two and 1001 children is able to store more than one billion keys
  and yet only a disk access of two is required to find any key (Cormen et al., 2009).

  Lets implement a btree from scratch to see how it works.

*/

/*


 --- Fanout determined by sqlite's page size ----

 btrees have high fanout, which refers to the maximum number of children per node and is
 inversely correlated to height of the tree.

 Example of a N-ary btree.

                 |   100   | <-- root node has one interior node which has two children [88, 103]
                  |      |
 leaf node -> | 88 |   |103;110| <-- Child of root node is a node with 2 "interior" nodes.
                          |
                       |102;105|
                        ^    ^ leaf nodes (no children)
*/

#[derive(Debug)]
struct InteriorNode<'a, K: Ord, V> {
    key: K,   //   Key is used to maintain the order of the tree,
    value: V, // Value is the actual data being stored.
    // Maximum of N + 1 child nodes
    children: Vec<NonRootNode<'a, K, V>>,
}

impl<'a, K: Ord, V> InteriorNode<'a, K, V> {
    pub fn new(key: K, value: V, children: Vec<NonRootNode<'a, K, V>>) -> InteriorNode<'a, K, V> {
        InteriorNode {
            key,
            value,
            children,
        }
    }
}

impl<'a, K: Ord, V> PartialEq for InteriorNode<'a, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<'a, K: Ord, V> PartialOrd for InteriorNode<'a, K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, K: Ord + PartialOrd, V> Ord for InteriorNode<'a, K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.key).cmp(&other.key)
    }
}

impl<'a, K: Ord + PartialOrd, V> Eq for InteriorNode<'a, K, V> {}

trait HasInteriorNodes<'a, K: Ord, V> {
    fn find_interior_node(&self) -> Option<&InteriorNode<'a, K, V>>;

    fn insert_interior_node(&mut self, n: InteriorNode<'a, K, V>) -> ();

    fn delete_interior_node(&mut self, key: K) -> ();
}

#[derive(Debug, PartialEq)]
struct RootNode<'a, K: Ord, V> {
    interior_nodes: Vec<InteriorNode<'a, K, V>>,
}

// A Node maps to a "page" on disk. Think of btrees as a way of organizing disk pages.
// Root node and non root nodes are distinguished in the types so that
// we don't to have an optional parent field which will be populated for all but one of the many
// nodes in our tree.
#[derive(Debug)]
struct NonRootNode<'a, K: Ord, V> {
    parent: &'a NonRootNode<'a, K, V>,
    // Maximum of N interior nodes
    interior_nodes: Vec<InteriorNode<'a, K, V>>, // sorted by K
}

impl<'a, K: Ord, V> NonRootNode<'a, K, V> {
    pub fn new(parent: &'a NonRootNode<'a, K, V>, key: K, value: V) -> NonRootNode<'a, K, V> {
        NonRootNode {
            parent,
            // Maximum of N interior nodes
            interior_nodes: vec![InteriorNode {
                key,
                value,
                children: Vec::new(),
            }],
        }
    }
}

impl<'a, K: Ord, V> HasInteriorNodes<'a, K, V> for RootNode<'a, K, V> {
    // insert in sorted order
    fn insert_interior_node(&mut self, n: InteriorNode<'a, K, V>) -> () {
        match self.interior_nodes.binary_search(&n) {
            Ok(pos) | Err(pos) => self.interior_nodes.insert(pos, n),
        }
    }

    fn find_interior_node(&self) -> Option<&InteriorNode<'a, K, V>> {
        None
    }

    fn delete_interior_node(&mut self, _key: K) -> () {}
}

#[derive(Debug, PartialEq)]
struct Btree<'a, K: Ord, V> {
    interior_node_count: u64, // The k in "k-ary btree" or number of interior node per node.
    root: RootNode<'a, K, V>,
}

impl<'a, K: Ord, V> Btree<'a, K, V> {
    pub fn new(interior_node_count: u64, key: K, value: V) -> Self {
        let mut root: RootNode<'a, K, V> = RootNode {
            interior_nodes: vec![],
        };

        let interior_node: InteriorNode<'a, K, V> = InteriorNode::new(key, value, Vec::new());

        root.insert_interior_node(interior_node);

        Btree {
            interior_node_count,
            root,
        }
    }
}

/* balancing operations */

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
                interior_nodes: vec![InteriorNode {
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
