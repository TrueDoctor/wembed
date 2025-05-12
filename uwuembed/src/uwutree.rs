use crate::{Embedding, NodeId, graph::Graph, query::Update, vec::DVec};

#[derive(Debug, Clone)]
pub struct UwuTree<'a, const D: usize> {
    positions: Vec<DVec<D>>,
    graph: &'a Graph,
    spatial_neighbors_lists: Vec<Vec<NodeId>>,
    // clusters: Vec<Cluster<D>>,
}

#[derive(Debug, Clone)]
pub struct Cluster<const D: usize> {
    members: Vec<NodeId>,
    position: DVec<D>,
    max_dist_squared: f64,
    max_weight: f64,
    is_leaf: bool,
}

impl<const D: usize> Cluster<D> {
    fn intersects(&self, other: &Cluster<D>) -> bool {
        let dist = self.position.distance_squared(&other.position)
            - self.max_dist_squared
            - other.max_dist_squared;
        dist <= (self.max_weight * other.max_weight).powi(2)
    }
}

impl<'a, const D: usize> Update<D> for UwuTree<'a, D> {
    fn update_positions(&mut self, postions: &[DVec<D>]) {
        self.positions = postions.to_vec();
        let weight_threshold = self.compute_weight_threshold(100);
        let mut clusters = self.generate_clusters(weight_threshold);
        self.assign_nodes_to_clusters(&mut clusters);
    }
}

impl<'a, const D: usize> UwuTree<'a, D> {
    pub fn new(embedding: &'a Embedding<'a, D>) -> Self {
        Self {
            positions: embedding.positions.to_vec(),
            graph: embedding.graph,
            spatial_neighbors_lists: Vec::new(),
        }
    }

    fn generate_clusters(&self, weight_threshold: f64) -> Vec<Cluster<D>> {
        let mut clusters = Vec::new();
        for (node, pos) in self.graph.nodes.iter().zip(self.positions.iter()) {
            if node.weight >= weight_threshold {
                clusters.push(Cluster {
                    members: Vec::new(),
                    position: *pos,
                    max_dist_squared: 0.,
                    max_weight: node.weight,
                    is_leaf: true,
                })
            }
        }

        clusters
    }

    fn assign_nodes_to_clusters(&self, clusters: &mut [Cluster<D>]) {
        for (i, (node, pos)) in self
            .graph
            .nodes
            .iter()
            .zip(self.positions.iter())
            .enumerate()
        {
            let mut best_cluster_idx = 0;
            let mut best_cluster_dist = f64::INFINITY;
            for (j, cluster) in clusters.iter().enumerate() {
                let dist = pos.distance_squared(&cluster.position);
                if dist < best_cluster_dist {
                    best_cluster_dist = dist;
                    best_cluster_idx = j;
                }
            }
            let cluster = &mut clusters[best_cluster_idx];
            // println!("i {}", best_cluster_idx);
            cluster.members.push(i);
            cluster.max_dist_squared = cluster.max_dist_squared.max(best_cluster_dist);
            cluster.max_weight = cluster.max_weight.max(node.weight);
        }
    }

    fn compute_weight_threshold(&self, num: usize) -> f64 {
        let mut weights = self.graph.nodes.clone();
        weights.sort_unstable_by_key(|n| -(n.neighbors.len() as isize));
        weights[num.min(weights.len() - 1)].weight
    }

    fn compute_cluster_intersections(&self, clusters: &[Cluster<D>]) {
        let mut num_intersections = 0;
        for (i, c1) in clusters.iter().enumerate() {
            for (j, c2) in clusters.iter().enumerate() {
                if i != j && c1.intersects(c2) {
                    num_intersections += c2.members.len();
                }
            }
        }
        println!(
            "{:.2} found {num_intersections} intersections of {}",
            num_intersections as f64 / clusters.len() as f64,
            clusters.len() * clusters.len()
        );
    }

    pub fn evaluate_intersections(&self) {
        for i in (0..100_000).step_by(2000) {
            let weight_threshold = self.compute_weight_threshold(i);
            let mut clusters = self.generate_clusters(weight_threshold);
            self.assign_nodes_to_clusters(&mut clusters);
            // dbg!(clusters.len(), self.positions.len());
            for cluster in &clusters {
                // dbg!(cluster.members.len());
            }
            // println!("clusters {:?}", clusters);
            println!(
                "i: {} ({:.2}%)",
                i,
                i as f64 / self.positions.len() as f64 * 100.
            );
            // self.compute_cluster_intersections(&clusters);
            let mut tree = UTree::default();
            for cluster in clusters {
                tree.insert(cluster);
            }
        }
    }
}

type TreeNode<const D: usize> = Cluster<D>;
type TreeNodeId = NodeId;

#[derive(Default)]
struct UTree<const D: usize> {
    arena: Vec<TreeNode<D>>,
    roots: Vec<TreeNodeId>,
}

impl<const D: usize> UTree<D> {
    fn insert(&mut self, node: TreeNode<D>) {
        let Some(&id) = self
            .roots
            .iter()
            .find(|&x| self.arena[*x].intersects(&node))
        else {
            let id = self.alloc_node(node);
            self.roots.push(id);
            return;
        };

        self.insert_into_tree(id, node);

        // TODO: check if root id now intersects with other roots
        let tree = &self.arena[id];
        let Some(intersecting_id) = self
            .roots
            .iter()
            .find(|&&x| x != id && self.arena[x].intersects(tree))
        else {
            return;
        };
        self.arena[*intersecting_id].members.push(id);
        self.roots
            .swap_remove(self.roots.iter().position(|x| x == &id).unwrap());
    }

    fn alloc_node(&mut self, node: TreeNode<D>) -> TreeNodeId {
        self.arena.push(node);
        self.arena.len() - 1
    }

    fn insert_into_tree(&mut self, tree: TreeNodeId, node: TreeNode<D>) {
        let tree_node = self.arena[tree].clone();
        if tree_node.is_leaf {
            self.join_nodes_under_new_parent(tree, node);
            return;
        }

        let Some(mut i) = tree_node
            .members
            .iter()
            .position(|&m| !self.arena[m].intersects(&node))
        else {
            {
                let tree_node = &mut self.arena[tree];
                let dist =
                    tree_node.position.distance_squared(&node.position) + node.max_dist_squared;
                tree_node.max_dist_squared = tree_node.max_dist_squared.max(dist);
                tree_node.max_weight = tree_node.max_weight.max(node.max_weight);
            }
            let id = self.alloc_node(node);
            self.arena[tree].members.push(id);
            return;
        };

        if i == 0 {
            if let Some(new_i) = tree_node
                .members
                .iter()
                .position(|&m| self.arena[m].intersects(&node))
            {
                i = new_i
            } else {
                self.join_nodes_under_new_parent(tree, node);
                return;
            }
        }

        self.insert_into_tree(i, node);

        let new_tree = self.arena[i].clone();
        let tree_node = &mut self.arena[tree];
        tree_node.max_weight = tree_node.max_weight.max(new_tree.max_weight);
        let dist =
            tree_node.position.distance_squared(&new_tree.position) + new_tree.max_dist_squared;
        tree_node.max_dist_squared = tree_node.max_dist_squared.max(dist);
    }

    fn join_nodes_under_new_parent(&mut self, tree: usize, node: Cluster<D>) {
        let tree_node = &self.arena[tree];
        // TODO: sinnvoll mathe machen siehe foto vom 12.05 dennis;
        let new_pos = (tree_node.position + node.position) / 2.;
        let d1 = new_pos.distance_squared(&tree_node.position) + tree_node.max_dist_squared;
        let d2 = new_pos.distance_squared(&node.position) + node.max_dist_squared;
        let new_max_dist = d1.max(d2);
        let new_max_weight = node.max_weight.max(tree_node.max_weight);
        let new_node_id = self.alloc_node(node);
        let mut new_node = TreeNode {
            members: vec![new_node_id],
            position: new_pos,
            max_dist_squared: new_max_dist,
            max_weight: new_max_weight,
            is_leaf: false,
        };
        std::mem::swap(&mut new_node, &mut self.arena[tree]);
        let new_tree_id = self.alloc_node(new_node);
        self.arena[tree].members.push(new_tree_id);
    }
}
