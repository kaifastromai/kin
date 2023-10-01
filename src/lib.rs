//! Represents familial relationships between persons (that reproduce offspring via dimorphic sexual relations).

//Two coprime numbers
pub const PPRIME: usize = 2_000_003;
pub const CPRIME: usize = 2_000_029;
pub const RPRIME: usize = 2_000_039;
mod states;
mod kin_dsl;

mod uuid {

    pub fn gen_64() -> u64 {
        uuid::Uuid::new_v4().as_u128() as u64
    }
}

use itertools::Itertools;
type Nd = NodeIndex<usize>;
use anyhow::*;
use indexmap::IndexSet;
use petgraph::algo::*;
use petgraph::prelude::*;
use states::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::from_fn;
use std::iter::FromFn;
use std::{
    collections::{BTreeMap, VecDeque},
    hash::Hash,
    ops::{Add, Sub},
};
use thiserror::Error;
fn render_to<W: std::io::Write>(output: &mut W, graph: &KinGraph) {
    dot::render(graph, output).unwrap();
}

pub enum ETransitionStates {
    ///Initial parent
    P = 0,
    ///Initial child
    C,
    ///The nth parent of a node
    NP,
    ///The nth child of a node
    NC,
    ///Sibling
    S,
    ///Sibling-in-law
    SinL,
    ///Reproductive partner
    RP,
    ///The nth parent-in-law of a node
    NPinL,
    ///The nth child-in-law of a node
    NCinL,
    ///Niece/Nephew
    NN,
    ///Aunt/uncle
    AU,
    ///The stop state. Indicates that there is no more relationship.
    STOP,
}

#[derive(Error, Debug)]
pub enum KinError {
    #[error("Person {index:?} can not be related themselves!")]
    SelfCycle { index: usize },
    #[error("Person {p1:?} and Person {p2:?} have an invalid Parent/Child or Sibling relation.")]
    InvalidRelation { p1: usize, p2: usize },

    #[error("Person {p1:?} and Person{p2:?} have Reproductive partners of the same sex")]
    SameSexError { p1: usize, p2: usize },
    #[error("Parent not added. Person {p:?} already has two parents")]
    AlreadyTwoParents { p: usize },

    #[error("An unknown error occured")]
    Unknown,
}

///Describes the possible fundamental types of relationships (that is, all others
/// can be represented as a combination of these).
///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Parent = 1,
    //The inverse of a parent
    Child = -1,
    //symmetric
    Sibling = 2,
    //Reproductive partner, symmetric
    RP = 3,
}

impl Kind {
    fn get_value(&self) -> i32 {
        match self {
            Kind::Parent => 1,
            Kind::Child => -1,
            //This should probably be changed later, but for now this is easier...
            Kind::Sibling => 1,
            //Experiment with zero length repat
            Kind::RP => 0,
        }
    }
    fn get_cost(&self) -> u32 {
        match self {
            Kind::Parent => 1,
            Kind::Child => 1,
            Kind::Sibling => 1,
            Kind::RP => 1,
        }
    }
    fn is_blood_kind(&self) -> bool {
        match self {
            Kind::Parent => true,
            Kind::Child => true,
            Kind::Sibling => true,
            Kind::RP => false,
        }
    }
    fn get_prime(&self) -> usize {
        match self {
            Kind::Parent => PPRIME,
            Kind::Child => CPRIME,
            Kind::Sibling => 0,
            Kind::RP => RPRIME,
        }
    }
    fn into_base_state(self, sex: Sex) -> Box<dyn State> {
        match self {
            Kind::Parent => Box::new(NParentState { n: 0, sex }),
            Kind::Child => Box::new(NChildState { n: 0, sex }),
            Kind::Sibling => Box::new(SiblingState {
                is_half: false,
                sex,
            }),
            Kind::RP => Box::new(RPState { sex }),
        }
    }
}
impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Parent => write!(f, "P"),
            Kind::Child => write!(f, "C"),
            Kind::Sibling => write!(f, "S"),
            Kind::RP => write!(f, "R"),
        }
    }
}
impl std::fmt::Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}
//This represents the location of the person in the family tree.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Location {
    ///depth of the person in the tree, indicated by [Kind::Parent]
    d: i32,
    ///the 'width' of the person in the tree, this the 'sideways' movement indicated by [Kind::Sibling] or [Kind::Repat]
    w: i32,
}
impl Location {
    fn dot(&self, other: Location) -> i32 {
        self.d * other.w + self.w * other.d
    }
}
//impl display for Location
impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(d: {},w: {})", self.d, self.w)
    }
}
//impl Add and Sub for Location
impl Add for Location {
    type Output = Location;
    fn add(self, other: Location) -> Location {
        Location {
            d: self.d + other.d,
            w: self.w + other.w,
        }
    }
}
impl Sub for Location {
    type Output = Location;
    fn sub(self, other: Location) -> Location {
        Location {
            d: self.d - other.d,
            w: self.w - other.w,
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, PartialOrd, Ord, Debug)]
pub enum Sex {
    Male,
    Female,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Debug)]
pub struct Person {
    id: usize,
    sex: Sex,
    //If this is a shadow person added by the sanitizer
    is_shadow: bool,
}
impl Person {
    pub fn new(sex: Sex) -> Self {
        //assign random (hopefully) unique id.
        let id = uuid::gen_64();
        //on 32bit os's/machines this will be a problem, but we'll think about that later
        Person {
            id: id as usize,
            sex,
            is_shadow: false,
        }
    }
}
impl From<Person> for NodeIndex<usize> {
    fn from(p: Person) -> Self {
        NodeIndex::from(p.id as usize)
    }
}
impl From<&Person> for NodeIndex<usize> {
    fn from(p: &Person) -> Self {
        NodeIndex::from(p.id as usize)
    }
}
///Represents the general directed graph
pub struct KinGraph {
    graph: DiGraph<Person, Kind, usize>,
    //a map between a node index and the id of the person. This is probably
    //not necessary since it appears that DiGraph already increments directly,
    //but I have no absoulte proof of this, so...
    id_indx: HashMap<usize, NodeIndex<usize>>,
    depth_map: Option<BTreeMap<Person, Location>>,
}

impl<'a> dot::Labeller<'a, Nd, petgraph::graph::EdgeReference<'a, Kind, usize>> for KinGraph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("Test").unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n.index())).unwrap()
    }
    fn edge_label(
        &'a self,
        e: &petgraph::graph::EdgeReference<'a, Kind, usize>,
    ) -> dot::LabelText<'a> {
        dot::LabelText::LabelStr(format!("{}", e.weight()).into())
    }
}
impl<'a> dot::GraphWalk<'a, Nd, petgraph::graph::EdgeReference<'a, Kind, usize>> for KinGraph {
    fn nodes(&'a self) -> dot::Nodes<'a, Nd> {
        self.graph.node_indices().collect()
    }

    fn edges(&'a self) -> dot::Edges<'a, petgraph::graph::EdgeReference<'a, Kind, usize>> {
        self.graph.edge_references().collect()
    }

    fn source(&'a self, edge: &petgraph::graph::EdgeReference<'a, Kind, usize>) -> Nd {
        edge.source()
    }

    fn target(&'a self, edge: &petgraph::graph::EdgeReference<'a, Kind, usize>) -> Nd {
        edge.target()
    }
}

impl KinGraph {
    fn new() -> Self {
        KinGraph {
            graph: Graph::default(),
            id_indx: HashMap::new(),
            depth_map: None,
        }
    }
    ///Get's the person with the given index, based upon the order in which it was added to the graph.
    pub fn px(&self, ix: usize) -> Person {
        self.graph[NodeIndex::from(ix)]
    }
    fn add_person(&mut self, p: &Person) {
        let id = p.id;
        let idx = self.graph.add_node(*p);
        self.id_indx.insert(id, idx);
    }
    fn add_persons(&mut self, ps: &[Person]) {
        for p in ps {
            self.add_person(p);
        }
    }
    ///Adds a new person to the graph.
    fn np(&mut self, sex: Sex) -> Person {
        let p = Person::new(sex);
        let id = p.id;
        let idx = self.graph.add_node(p);
        self.id_indx.insert(id, idx);
        p
    }
    ///Adds kind between p1->p2, and kind^-1 (inverse kind) between p2->p1. If such an edge already exists between these nodes,
    /// it exits silently, not adding the edge
    fn add_edges(&mut self, p1: NodeIndex<usize>, p2: NodeIndex<usize>, kind: Kind) {
        let mut e = self.graph.edges_connecting(p1, p2);
        if !e.any(|e| *e.weight() == kind) && p1 != p2 {
            match kind {
                Kind::Parent => {
                    self.graph.add_edge(p1, p2, Kind::Parent);
                    self.graph.add_edge(p2, p1, Kind::Child)
                }
                Kind::Child => {
                    self.graph.add_edge(p1, p2, Kind::Child);
                    self.graph.add_edge(p2, p1, Kind::Parent)
                }
                Kind::Sibling => {
                    self.graph.add_edge(p2, p1, Kind::Sibling);
                    self.graph.add_edge(p1, p2, Kind::Sibling)
                }
                Kind::RP => {
                    self.graph.add_edge(p2, p1, Kind::RP);
                    self.graph.add_edge(p1, p2, Kind::RP)
                }
            };
        }
    }
    ///Adds a relationship.
    pub fn add_relation(&mut self, p1: Person, p2: Person, kind: Kind) -> Result<()> {
        match kind {
            Kind::Parent => self.add_parent(p1, p2)?,
            Kind::Child => self.add_parent(p2, p1)?,
            Kind::RP => self.add_repat(p1, p2)?,
            Kind::Sibling => self.add_sibling(p1, p2)?,
        };
        Ok(())
    }
    ///Make c a child of both p1, and p2.
    pub fn make_child(&mut self, c: Person, p1: Person, p2: Person) -> Result<()> {
        self.add_relation(p1, c, Kind::Parent)?;
        self.add_relation(p2, c, Kind::Parent)?;
        Ok(())
    }

    fn add_parent(&mut self, p: Person, c: Person) -> Result<()> {
        let px = self.idx(p).unwrap();
        let cx = self.idx(c).unwrap();

        //get the number of parents the child has
        let parents = self
            .graph
            .edges_directed(cx, Direction::Outgoing)
            .filter(|e| *e.weight() == Kind::Child)
            .collect::<Vec<_>>();

        let plen = parents.len();
        if plen >= 2 {
            //don't add. Too many parents
            return Err(KinError::AlreadyTwoParents { p: cx.index() }.into());
        } else {
            if plen == 1 {
                //We don't have to check if we already are an RP because add edge takes care of that already
                //make ourselves the RP of the other parent
                let other_p = parents[0].target();
                self.add_edges(px, other_p, Kind::RP);
            }
            //make ourselves the parent
            self.add_edges(px, cx, Kind::Parent);
        }

        Ok(())
    }

    fn add_sibling(&mut self, p1: Person, p2: Person) -> Result<()> {
        self.add_edges(self.idx(p1).unwrap(), self.idx(p2).unwrap(), Kind::Sibling);
        Ok(())
    }
    fn add_repat(&mut self, p1: Person, p2: Person) -> Result<()> {
        self.add_edges(self.idx(p1).unwrap(), self.idx(p2).unwrap(), Kind::RP);
        Ok(())
    }
    ///Calculates relationship between two persons.
    pub fn get_canonical_relationships(
        &mut self,
        p1: Person,
        p2: Person,
    ) -> Result<Vec<Box<dyn State>>> {
        if p1 == p2 {
            return Ok(vec![Box::new(StopState {})]);
        }
        let paths = self.find_all_paths(p1, p2)?;
        let mut names = HashSet::new();
        for p in paths {
            names.insert(self.calculate_cr_single_path(self.idx(p1).unwrap(), &p)?);
        }
        let names = names
            .into_iter()
            .filter(|s| !s.get_any().is::<StopState>())
            .collect();

        Ok(names)
    }

    ///Calculates canonical relationship given a kind path.
    fn calculate_cr_single_path(
        &mut self,
        p1: Nd,
        path: &Vec<(Nd, Kind)>,
    ) -> Result<Box<dyn State>> {
        let mut sm = StateMachine::new();
        let mut cur_idx = p1;

        for (n, k) in path {
            if sm.transition((cur_idx, *k, *n), self).is_some() {
                cur_idx = *n;
            } else {
                return Ok(sm.get_current_state());
            }
        }
        Ok(sm.get_current_state())
    }

    ///This builds the depth map. It must be done after all the relations are added.
    /// Starts at the given root person, instead of some global.
    ///  This allows us to store multiple disconnected family trees.
    pub fn build_map(&mut self, root: Person) {
        let mut depth_map = BTreeMap::<Person, Location>::new();
        //first index
        let mut cidx = self.idx(root).unwrap();
        let nbs = self.graph.neighbors_directed(cidx, Direction::Outgoing);
        //find using depth-first search
        let mut visited_stack = VecDeque::<NodeIndex<usize>>::new();
        //used to back track
        let mut v2 = VecDeque::<NodeIndex<usize>>::new();
        //We use this to keep track of the depth when we reset to a previous node in the depth first search
        let mut depth_set = BTreeMap::<usize, i32>::new();

        v2.push_back(cidx);

        //our current depth
        let mut cur_loc = Location { d: 0, w: 0 };

        while !v2.is_empty() {
            visited_stack.push_back(cidx);
            //add this to the map
            depth_map.insert(self.graph[cidx], cur_loc);
            let mut next_i = 0;
            let nit = self
                .graph
                .neighbors_directed(cidx, Direction::Outgoing)
                .collect::<Vec<NodeIndex<usize>>>();

            //This is known to be 1 (an invariant) (only 1 outgoing edge between two nodes), if broken, our graph is illformed
            //We only want to follow the paths that are even or climb the ancestor tree (the links that make us the child)
            let e = |n: usize| self.graph.edges_connecting(cidx, nit[n]).next().unwrap();

            while next_i < nit.len() && (visited_stack.contains(&nit[next_i])) {
                next_i += 1;
            }

            //We've searched all neighbors, and already visited them
            if next_i == nit.len() {
                //go back, we're done
                cidx = v2.pop_back().unwrap();
                cur_loc = *depth_map.get(&self.graph[cidx]).unwrap();
                continue;
            }

            //if the chosen path is Child, then our depth decreases
            //else if the chosen path is a sibling or a repat, our sideways drift increases
            match e(next_i).weight() {
                //Depth
                Kind::Child => {
                    cur_loc.d += 1;
                }
                Kind::Parent => {
                    cur_loc.d -= 1;
                }
                //Sideways
                k => cur_loc.w += k.get_value(),
            }

            v2.push_back(cidx);
            cidx = nit[next_i];
        }
        //print all the nodes in depth map
        for (k, v) in depth_map.iter() {
            println!("Node {:} -> {:}", self.idx(*k).unwrap().index(), v);
        }
        self.depth_map = Some(depth_map);
    }
    ///Finds whether a person is related by blood to another
    fn is_rrb(&self, p1: Person, p2: Person) -> bool {
        //they are related by blood iff there is a path that of only child/parent edges between them
        let sps = all_simple_paths(
            &self.graph,
            self.idx(p1).unwrap(),
            self.idx(p2).unwrap(),
            0,
            None,
        )
        .collect::<Vec<Vec<_>>>();
        let res = sps.iter().any(|p| {
            let sum = p
                .iter()
                .tuple_windows()
                .map(|w: (_, _)| self.graph.edges_connecting(*w.0, *w.1))
                .fold(0, |acc, mut e| {
                    let edge1 = e.find(|e| *e.weight() != Kind::RP);
                    if edge1.is_none() {
                        return acc + RPRIME;
                    };
                    let edge1 = edge1.unwrap();
                    return acc + edge1.weight().get_prime();
                });

            //Return whether the sum is divisible by one of our primes, indicating that the path is just a sum of either PARENT or CHILD
            sum % PPRIME == 0 || sum % CPRIME == 0
        });
        println!("Result {:?}", res);
        res
    }
    fn is_repat(&self, p1: Nd, p2: Nd) -> bool {
        self.graph
            .edges_connecting(p1, p2)
            .any(|e| *e.weight() == Kind::RP)
    }
    ///Finds all paths between two people, with an internal maximum of the order of the graph
    pub fn find_all_paths(
        &self,
        p1: Person,
        p2: Person,
    ) -> Result<Vec<Vec<(NodeIndex<usize>, Kind)>>> {
        let p1x = self.idx(p1).unwrap();
        let goalx = self.idx(p2).unwrap();
        let mut paths = Vec::<Vec<(NodeIndex<usize>, Kind)>>::new();

        // how many nodes are allowed in simple path up to target node
        // it is min/max allowed path length minus one, because it is more appropriate when implementing lookahead
        // than constantly add 1 to length of current path
        //-----Taken and modified from original authors of petgraph, Cephas Jun 12, 2022
        let max_length = self.graph.node_count() - 1;
        let min_length = 1;

        // list of visited nodes
        let mut visited: IndexSet<NodeIndex<usize>> = [p1x].into();
        // list of children of currently exploring path nodes,
        // last elem is list of childs of last visited node
        let mut stack = vec![self.graph.neighbors_directed(p1x, Outgoing)];
        let it: FromFn<_> = from_fn(move || {
            while let Some(children) = stack.last_mut() {
                if let Some(child) = children.next() {
                    if visited.len() < max_length {
                        if child == goalx {
                            if visited.len() >= min_length {
                                let path = visited
                                    .iter()
                                    .cloned()
                                    .chain(std::iter::once(goalx))
                                    .collect::<Vec<_>>();
                                return Some(path);
                            }
                        } else if !visited.contains(&child) {
                            visited.insert(child);
                            stack.push(self.graph.neighbors_directed(child, Outgoing));
                        }
                    } else {
                        if (child == goalx || children.any(|v| v == goalx))
                            && visited.len() >= min_length
                        {
                            let path = visited
                                .iter()
                                .cloned()
                                .chain(std::iter::once(goalx))
                                .collect::<_>();
                            return Some(path);
                        }
                        stack.pop();
                        visited.pop();
                    }
                } else {
                    stack.pop();
                    visited.pop();
                }
            }
            None
        });

        //-------End of taken code----
        // let sps = simple_paths::all_simple_paths(&self.graph, p1x, goalx, 0, None)
        //     .collect::<Vec<Vec<_>>>();
        let sps = it.collect::<Vec<_>>();

        //reconstruct all the path kinds from the sps
        let mut rc_paths = |path: &Vec<NodeIndex<usize>>| {
            let mut p = Vec::<(NodeIndex<usize>, Kind)>::new();
            for i in 0..path.len() - 1 {
                let e = self
                    .graph
                    .edges_connecting(path[i], path[i + 1])
                    .collect::<Vec<_>>();
                //simple case, only one outgoing edge between two nodes
                if e.len() == 1 {
                    p.push((path[i], *e[0].weight()));
                }
            }
            paths.push(p);
        };
        for p in sps {
            rc_paths(&p);
        }

        Ok(paths)
    }
    ///Checks if p1 is a child of p2
    pub fn is_parent(&self, p1: Nd, p2: Nd) -> bool {
        let p = p1;
        let c = p2;
        let mut res = false;
        for e in self.graph.edges_directed(p, Outgoing) {
            if *e.weight() == Kind::Child && e.source() == c {
                res = true;
            }
        }
        res
    }
    ///Checks whether two person share the same set of parents
    pub fn b_share_parents(&self, p1: Nd, p2: Nd) -> bool {
        let p1x = p1;
        let p2x = p2;
        let p1_parents = self
            .graph
            .edges_directed(p1x, Outgoing)
            .filter(|e| *e.weight() == Kind::Parent)
            .collect::<Vec<_>>();
        let p2_parents = self
            .graph
            .edges_directed(p2x, Outgoing)
            .filter(|e| *e.weight() == Kind::Parent)
            .collect::<Vec<_>>();
        let mut res = false;
        //we know that their sizes must be two, or else the graph is ill-formed
        if p1_parents.len() == 2 && p2_parents.len() == 2 {
            let p1_p = p1_parents[0].source();
            let p1_c = p1_parents[1].source();
            let p2_p = p2_parents[0].source();
            let p2_c = p2_parents[1].source();
            if p1_p == p2_p && p1_c == p2_c {
                res = true;
            }
        }
        res
    }

    ///Get NodeIndex from person
    fn idx(&self, p: Person) -> Option<NodeIndex<usize>> {
        self.id_indx.get(&p.id).cloned()
    }
}
#[cfg(test)]
mod test_kin;
