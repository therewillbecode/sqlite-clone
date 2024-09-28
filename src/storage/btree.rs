use anyhow::{anyhow, Result};
use std::vec::Vec;
/*

  Btrees are effective data structures for the minimizing the number of random disk accesses when operating on data that
  doesn't fit into memory. Thie fact b-trees can be used to maximise sequential disk access and minimise random access on
  disk which is way slower is why they are such a good fit for databases.

  A btree allows for a notion of locality grouping "nearby" interior nodes into a bigger node which
  maps to a "page" on disk. The page is smallest amount of disk data which is allocatable to memory.
  So each node we find would correspond to a random disk seek but then once we load that page in memory
  things are faster???

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

struct InteriorNode<'a, K: PartialOrd, V> {
    key: K,   //   Key is used to maintain the order of the tree,
    value: V, // Value is the actual data being stored.
    // Maximum of N + 1 child nodes
    children: Vec<Node<'a, K, V>>,
}

struct RootNode<'a, K: PartialOrd, V> {
    interior_nodes: Vec<Node<'a, K, V>>,
}

// A Node maps to a "page" on disk. Think of btrees as a way of organizing disk pages.
struct Node<'a, K: PartialOrd, V> {
    parent: &'a Node<'a, K, V>,
    // Maximum of N interior nodes
    interior_nodes: Vec<InteriorNode<'a, K, V>>, // sorted by K
}

impl<'a, K: PartialOrd> Node<'a, K, V> {
    // insert in sorted order
    pub fn insert_interior(&mut self, value: InteriorNode<'a, K, V>) {
        match self.vec.binary_search(&value) {
            Ok(pos) | Err(pos) => self.vec.insert(pos, value),
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
    fn new_btree_inits_correctly() {
        let init_btree = todo!();
        let expected_btree = todo!();

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
