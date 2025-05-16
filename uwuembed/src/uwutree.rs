use std::ops::Index;

use crate::{Embedding, Naive, NodeId, Query, graph::Graph, query::Update, vec::DVec};

#[derive(Debug, Clone)]
pub struct UwuTree<'a, const D: usize> {
    positions: Vec<DVec<D>>,
    graph: &'a Graph,
    spatial_neighbors_lists: Vec<Vec<NodeId>>,
    // clusters: Vec<Cluster<D>>,
}

#[derive(Debug, Clone, Default)]
pub struct Cluster<const D: usize> {
    members: Vec<NodeId>,
    position: DVec<D>,
    max_dist: f64,
    max_weight: f64,
    is_leaf: bool,
    total_children: usize,
}

impl<const D: usize> Cluster<D> {
    fn intersects(&self, other: &Cluster<D>) -> bool {
        let dist = self.position.distance(&other.position) - self.max_dist - other.max_dist;
        dist <= (self.max_weight * other.max_weight)
    }

    fn split_weight_classes(&self, classes: &[f64], tree: &UwuTree<'_, D>) -> Vec<Cluster<D>> {
        let mut output = vec![Self::default(); classes.len()];
        for &node in &self.members {
            let weight = tree.graph.nodes[node].weight;
            let index = classes.iter().position(|&x| x > weight).unwrap_or_default();
            for i in 0..index {
                output[i].members.push(node);
            }
        }
        output.iter_mut().for_each(|c| c.update(tree));
        output
    }

    fn update(&mut self, tree: &UwuTree<'_, D>) {
        let sum: DVec<D> = self.members.iter().map(|id| tree.positions[*id]).sum();
        let center = sum / self.members.len() as f64;

        let max_dist = self
            .members
            .iter()
            .map(|&id| tree.positions[id].distance_squared(&center))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let max_weight = self
            .members
            .iter()
            .map(|&id| tree.graph.nodes[id].weight)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        self.position = center;
        self.max_dist = max_dist.unwrap_or_default();
        self.max_weight = max_weight.unwrap_or_default();
    }
}

impl<const D: usize> Update<D> for UwuTree<'_, D> {
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
                    max_dist: 0.,
                    max_weight: node.weight,
                    is_leaf: true,
                    total_children: 0,
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
            cluster.members.push(i);
            cluster.max_dist = cluster.max_dist.max(best_cluster_dist.sqrt());
            cluster.max_weight = cluster.max_weight.max(node.weight);
        }
    }

    fn compute_weight_threshold(&self, num: usize) -> f64 {
        let mut weights = self.graph.nodes.clone();
        weights.sort_unstable_by_key(|n| -(n.neighbors.len() as isize));
        weights[num.min(weights.len() - 1)].weight
    }

    pub fn evaluate_intersections(&mut self) {
        for i in (19_000..100_000.min(self.positions.len())).step_by(1000) {
            self.spatial_neighbors_lists = vec![Vec::new(); self.graph.nodes.len()];
            println!("comuting weigth threshold");
            let weight_threshold = self.compute_weight_threshold(i);
            println!("weight threshold: {}", weight_threshold);
            let mut clusters = self.generate_clusters(weight_threshold);
            clusters.sort_unstable_by_key(|c| {
                std::cmp::Reverse((c.max_weight * c.max_dist * 1000000.) as usize)
            });
            let clusters = &mut clusters[..i];
            println!("found {} clusters", clusters.len());
            println!("assigning nodes to closest cluster");
            self.assign_nodes_to_clusters(clusters);
            println!("finished assigning nodes to clusters");

            println!(
                "i: {} ({:.2}%)",
                i,
                clusters.len() as f64 / self.positions.len() as f64 * 100.
            );
            let mut tree = UTree::default();
            tree.arena.reserve(clusters.len() * 2);
            println!("strarting insert");

            // TODO: insert clusters by desc weight / score

            for cluster in clusters.iter() {
                tree.insert(cluster.clone());
            }

            // println!("merging {} roots", tree.roots.len());
            // tree.merge_roots();
            println!("finished building tree");
            // dbg!(tree.roots.len(), tree.arena[tree.roots[0]].members.len());
            println!("clusters in tree root: {}", tree.arena[0].members.len());
            // assert!(tree.roots.len() == 1);
            println!("querying all nodes");
            let mut i = 0;
            let mut total_nodes_queried = 0;
            let mut total_intersections = 0;
            let start = std::time::Instant::now();
            for i in 0..self.graph.nodes.len() {
                let node = Cluster {
                    position: self.positions[i],
                    max_weight: self.graph.nodes[i].weight,
                    ..Default::default()
                };
                let mut sum_intersections = 0;

                let mut intersections = Vec::new();
                let mut checks = 0;
                let depth = tree.query(0, &node, &mut intersections, &mut checks, 0);
                for &other_cluster in &intersections {
                    let other_cluster = &tree.arena[other_cluster];
                    for &other_node in &other_cluster.members {
                        if self.positions[i].distance_squared(&self.positions[other_node])
                            < (self.graph.nodes[i].weight * self.graph.nodes[other_node].weight)
                                .powi(2)
                        {
                            self.spatial_neighbors_lists[i].push(other_node);
                            sum_intersections += 1;
                            total_intersections += 1;
                        }
                    }
                }
                if i % (self.graph.nodes.len() / 10) == 0 {
                    println!("depth: {depth}, intersections: {}", sum_intersections);
                }
            }
            assert_eq!(total_intersections, 361766);
            println!(
                "\n\n\ndone querying after {}s queried {} nodes with {} total intersections\n\n\n",
                start.elapsed().as_secs(),
                total_nodes_queried,
                total_intersections
            );
        }
    }
}

type TreeNode<const D: usize> = Cluster<D>;
type TreeNodeId = NodeId;

struct UTree<const D: usize> {
    arena: Vec<TreeNode<D>>,
}

impl<const D: usize> Default for UTree<D> {
    fn default() -> Self {
        Self {
            arena: vec![Cluster::default()],
        }
    }
}

impl<const D: usize> UTree<D> {
    fn query(
        &self,
        tree: NodeId,
        node: &TreeNode<D>,
        intersections: &mut Vec<NodeId>,
        checks: &mut usize,
        depth: usize,
    ) -> usize {
        if self.arena[tree].is_leaf {
            intersections.push(tree);
            return depth;
        }
        let mut new_depth = depth;
        for &member in &self.arena[tree].members {
            *checks += 1;
            if self.arena[member].intersects(node) {
                let subtree_depth = self.query(member, node, intersections, checks, depth + 1);
                new_depth = new_depth.max(subtree_depth);
            }
        }
        new_depth
    }
    fn tree_size(&self, tree: NodeId) -> usize {
        if self.arena[tree].is_leaf {
            return self.arena[tree].members.len();
        }
        let mut sum = 0;
        for &member in &self.arena[tree].members {
            sum += self.tree_size(member);
        }
        sum
    }

    fn insert(&mut self, node: TreeNode<D>) {
        self.insert_into_tree(0, node);
    }

    fn alloc_node(&mut self, node: TreeNode<D>) -> TreeNodeId {
        self.arena.push(node);
        self.arena.len() - 1
    }

    // Build different versions of the tree using weight classes
    //

    fn insert_into_tree(&mut self, tree: TreeNodeId, node: TreeNode<D>) {
        let tree_node = self.arena[tree].clone();
        // If node is leaf, join both nodes under new parent
        if tree_node.is_leaf {
            self.join_nodes_under_new_parent(tree, node);
            return;
        }
        let mut best_cluster = NodeId::MAX;
        let mut best_cluster_score = f64::INFINITY;
        let mut found_non_intersecting = false;
        for cluster in tree_node.members.iter() {
            let dist = self.arena[*cluster]
                .position
                .distance_squared(&node.position);
            let weight = self.arena[*cluster].max_weight.max(node.max_weight);
            // let score = dist.powi(D as i32) * weight * (children as f64 / total_children as f64);
            let score = dist * weight.powi(2);
            // dbg!(score);
            let intersects = self.arena[*cluster].intersects(&node);
            found_non_intersecting |= !intersects;
            if intersects && score < best_cluster_score {
                best_cluster_score = score;
                best_cluster = *cluster;
            }
        }

        if !found_non_intersecting || best_cluster_score > 5. {
            // If node intersects all nodes in parent, add it as member
            {
                let tree_node = &mut self.arena[tree];
                let dist =
                    tree_node.position.distance_squared(&node.position).sqrt() + node.max_dist;
                tree_node.max_dist = tree_node.max_dist.max(dist);
                tree_node.max_weight = tree_node.max_weight.max(node.max_weight);
            }
            let id = self.alloc_node(node);
            self.arena[tree].members.push(id);
            self.arena[tree].total_children += 1;
            return;
        }

        if best_cluster == NodeId::MAX {
            self.join_nodes_under_new_parent(tree, node);
            return;
        }

        self.insert_into_tree(best_cluster, node);

        let new_tree = self.arena[best_cluster].clone();
        let tree_node = &mut self.arena[tree];
        tree_node.max_weight = tree_node.max_weight.max(new_tree.max_weight);
        let dist = tree_node.position.distance(&new_tree.position) + new_tree.max_dist;
        tree_node.max_dist = tree_node.max_dist.max(dist);
        tree_node.total_children += 1;
    }

    fn join_nodes_under_new_parent(&mut self, tree: usize, node: Cluster<D>) {
        let node_id = self.alloc_node(node);
        self.join_nodes_under_new_parent_by_id(tree, node_id);
    }

    fn join_nodes_under_new_parent_by_id(&mut self, tree: NodeId, node_id: NodeId) {
        let tree_node = &self.arena[tree];
        let node = &self.arena[node_id];
        // TODO: sinnvoll mathe machen siehe foto vom 12.05 dennis;
        let new_pos = (tree_node.position + node.position) / 2.;
        let d1 = new_pos.distance(&tree_node.position) + tree_node.max_dist;
        let d2 = new_pos.distance(&node.position) + node.max_dist;
        let new_max_dist = d1.max(d2);
        let new_max_weight = node.max_weight.max(tree_node.max_weight);
        let mut new_node = TreeNode {
            members: vec![node_id],
            position: new_pos,
            max_dist: new_max_dist,
            max_weight: new_max_weight,
            is_leaf: false,
            total_children: 2,
        };
        std::mem::swap(&mut new_node, &mut self.arena[tree]);
        let new_tree_id = self.alloc_node(new_node);
        self.arena[tree].members.push(new_tree_id);
    }
}

// Discussion with nikolai
// only compute queries to lighter nodes
// don't store the heavy nodes in our data structure at all making clusters smaller
