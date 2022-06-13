use super::*;
type Transition = (Kind, Box<dyn State>);
///Represents a possible state, and describes the possible transitions from that state.
pub trait State {
    ///Returns an iterator over all possible states that can be reached from this state.
    fn transitions(&self) -> Vec<Transition>;
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>>;
    fn print_canonical_name(&self) -> String;
}

///An nth level parent (Parent, grandparent, great-grandparent, etc.)
pub struct ParentState {}
impl State for ParentState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(NParentState { n: 1 })),
            (Kind::Child, Box::new(RepatState {})),
            (Kind::Repat, Box::new(NPinLState { n: 1 })),
            (Kind::Sibling, Box::new(ParentState {})),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NParentState { n: 1 }),
            Kind::Child => Box::new(RepatState {}),
            Kind::Repat => Box::new(NPinLState { n: 0 }),
            Kind::Sibling => Box::new(ParentState {}),
            _ => panic!("Invalid transition"),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        "Parent".to_string()
    }
}

pub struct NParentState {
    n: usize,
}
impl State for NParentState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(NParentState { n: self.n + 1 })),
            (Kind::Child, Box::new(NPinLState { n: self.n - 1 })),
            (Kind::Repat, Box::new(NPinLState { n: self.n + 1 })),
            (Kind::Sibling, Box::new(NParentState { n: self.n })),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NParentState { n: self.n + 1 }),
            Kind::Child => Box::new(NPinLState { n: self.n - 1 }),
            Kind::Repat => Box::new(NPinLState { n: self.n + 1 }),
            Kind::Sibling => Box::new(NParentState { n: self.n }),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            1 => "grand".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "grand"
            }
        };
        format!("{}-{}", g, "parent")
    }
}

///Nth parent in law state
pub struct NPinLState {
    n: usize,
}
impl State for NPinLState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(NParentState { n: self.n + 1 })),
            (Kind::Child, Box::new(StopState {})),
            (Kind::Repat, Box::new(NPinLState { n: self.n })),
            (Kind::Repat, Box::new(StopState {})),
            (Kind::Sibling, Box::new(NParentState { n: self.n })),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NParentState { n: self.n + 1 }),
            Kind::Child => Box::new(StopState {}),
            Kind::Repat => Box::new(NPinLState { n: self.n }),
            Kind::Sibling => Box::new(NParentState { n: self.n }),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            0 => "".to_string(),
            1 => "grand".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "grand"
            }
        };
        format!("{}-{}", g, "parent-in-law")
    }
}
pub struct ChildState {}
impl State for ChildState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(SiblingState {})),
            (Kind::Parent, Box::new(StopState {})),
            (Kind::Child, Box::new(NChildState { n: 1 })),
            (Kind::Repat, Box::new(ChildState {})),
            (Kind::Repat, Box::new(StopState {})),
            (Kind::Sibling, Box::new(NNeniState { n: 0 })),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(SiblingState {}),
            Kind::Child => Box::new(NChildState { n: 1 }),
            Kind::Repat => {
                if kg.is_parent(kind.0, kind.2) {
                    Box::new(ChildState {})
                } else {
                    Box::new(StopState {})
                }
            }
            Kind::Sibling => Box::new(NNeniState { n: 0 }),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        "Child".to_string()
    }
}
pub struct NChildState {
    n: usize,
}
impl State for NChildState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(NNeniState { n: self.n - 1 })),
            (Kind::Parent, Box::new(StopState {})),
            (Kind::Child, Box::new(NChildState { n: self.n + 1 })),
            (Kind::Repat, Box::new(NChildState { n: self.n + 1 })),
            (Kind::Sibling, Box::new(NAuncleState { n: self.n })),
            (Kind::Sibling, Box::new(NNeniState { n: self.n })),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NNeniState { n: self.n }),
            Kind::Child => Box::new(NChildState { n: self.n + 1 }),
            Kind::Sibling => Box::new(NAuncleState { n: self.n }),
            Kind::Repat => {
                if kg.is_parent(kind.0, kind.2) {
                    Box::new(NChildState { n: self.n + 1 })
                } else {
                    Box::new(StopState {})
                }
            }
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            0 => "".to_string(),
            1 => "great".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "great"
            }
        };
        format!("{}-{}", g, "child")
    }
}

///Nth child in law
pub struct NCinLState {
    n: usize,
}
impl State for NCinLState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(StopState {})),
            (Kind::Child, Box::new(NCinLState { n: self.n + 1 })),
            (Kind::Repat, Box::new(NCinLState { n: self.n })),
            (Kind::Repat, Box::new(StopState {})),
            (Kind::Sibling, Box::new(NParentState { n: self.n })),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(StopState {}),
            Kind::Child => Box::new(NCinLState { n: self.n + 1 }),
            Kind::Repat => {
                if kg.is_repat(kind.0, kind.2) {
                    Box::new(NCinLState { n: self.n })
                } else {
                    Box::new(StopState {})
                }
            }
            Kind::Sibling => Box::new(StopState {}),
            _ => panic!("Invalid transition"),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            0 => "".to_string(),
            1 => "great".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "great"
            }
        };
        format!("{}-{}", g, "child-in-law")
    }
}

///Nth cousin kth times removed
pub struct NCsnKState {
    n: usize,
    k: i32,
}
impl State for NCsnKState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (
                Kind::Parent,
                Box::new(NCsnKState {
                    n: self.n,
                    k: self.k + 1,
                }),
            ),
            (
                Kind::Child,
                Box::new(NCsnKState {
                    n: self.n + 1,
                    k: self.k,
                }),
            ),
            (
                Kind::Repat,
                Box::new(NCsnKState {
                    n: self.n,
                    k: self.k + 1,
                }),
            ),
            (
                Kind::Sibling,
                Box::new(NCsnKState {
                    n: self.n,
                    k: self.k,
                }),
            ),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NCsnKState {
                n: self.n,
                k: self.k + 1,
            }),
            Kind::Child => {
                if self.n == self.k as usize {
                    Box::new(NNeniState { n: self.n })
                } else {
                    Box::new(NCsnKState {
                        n: self.n,
                        k: self.k - 1,
                    })
                }
            }
            Kind::Repat => Box::new(StopState {}),
            Kind::Sibling => Box::new(NCsnKState {
                n: self.n,
                k: self.k,
            }),
        };

        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        let rmv = (self.n as i32 - self.k.abs()).abs();
        let rmv_str = match rmv {
            0 => "".to_string(),
            1 => " once removed".to_string(),
            2 => " twice removed".to_string(),
            _ => format!(" {}-times removed", rmv),
        };
        let n_string = match self.n {
            1 => "1st".to_string(),
            2 => "2nd".to_string(),
            3 => "3rd".to_string(),
            _ => format!("{}th", self.n),
        };
        format!("{} cousins{}", n_string, rmv_str,)
    }
}

pub struct HalfSiblingState {}

pub struct SiblingState {}

impl State for SiblingState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(NNeniState { n: 0 })),
            (Kind::Child, Box::new(ChildState {})),
            (Kind::Repat, Box::new(SinLState {})),
            (Kind::Sibling, Box::new(SiblingState {})),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NNeniState { n: 0 }),
            Kind::Child => Box::new(ChildState {}),
            Kind::Repat => Box::new(SinLState {}),
            Kind::Sibling => Box::new(SiblingState {}),
            _ => panic!("Invalid transition"),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        "Sibling".to_string()
    }
}

pub struct RepatState {}
impl State for RepatState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(StopState {})),
            (Kind::Child, Box::new(NCinLState { n: 0 })),
            (Kind::Repat, Box::new(StopState {})),
            (Kind::Sibling, Box::new(SinLState {})),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(StopState {}),
            Kind::Child => Box::new(NCinLState { n: 0 }),
            Kind::Repat => Box::new(StopState {}),
            Kind::Sibling => Box::new(SinLState {}),
            _ => panic!("Invalid transition"),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        "Reproductive Partner/Spouse".to_string()
    }
}

pub struct SinLState {}
impl State for SinLState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(NParentState { n: 1 })),
            (Kind::Child, Box::new(RepatState {})),
            (Kind::Repat, Box::new(StopState {})),
            (Kind::Sibling, Box::new(SinLState {})),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NParentState { n: 1 }),
            Kind::Child => Box::new(RepatState {}),
            Kind::Repat => Box::new(StopState {}),
            Kind::Sibling => Box::new(SinLState {}),
            _ => panic!("Invalid transition"),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        "Sibling in Law".to_string()
    }
}
///Nephew/Niece state
pub struct NNeniState {
    n: usize,
}
impl State for NNeniState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(NNeniState { n: self.n })),
            (Kind::Child, Box::new(NChildState { n: self.n + 1 })),
            (Kind::Repat, Box::new(StopState {})),
            (Kind::Sibling, Box::new(NNeniState { n: self.n })),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NCsnKState {
                n: self.n,
                k: self.n as i32,
            }),
            Kind::Child => Box::new(StopState {}),
            Kind::Repat => Box::new(StopState {}),
            Kind::Sibling => Box::new(NNeniState { n: self.n }),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            0 => "".to_string(),
            1 => "great".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "great"
            }
        };
        format!("{}-{}", g, "nephew/niece")
    }
}
///Aunt/uncle state
pub struct NAuncleState {
    n: usize,
}
impl State for NAuncleState {
    fn transitions(&self) -> Vec<Transition> {
        vec![
            (Kind::Parent, Box::new(NAuncleState { n: self.n })),
            (Kind::Child, Box::new(NAuncleState { n: self.n })),
            (Kind::Repat, Box::new(StopState {})),
            (Kind::Sibling, Box::new(NAuncleState { n: self.n })),
        ]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        todo!("Implement aunt/uncle transitions")
        // match kind.1 {
        //     Kind::Parent => Box::new(NAuncleState { n: self.n }),
        //     Kind::Child => Box::new(NAuncleState { n: self.n }),
        //     Kind::Repat => Box::new(StopState {}),
        //     Kind::Sibling => Box::new(NAuncleState { n: self.n }),
        //     _ => panic!("Invalid transition"),
        // }
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            0 => "".to_string(),
            1 => "great".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "great"
            }
        };
        format!("{}-{}", g, "aunt/uncle")
    }
}
pub struct StopState {}
impl State for StopState {
    fn transitions(&self) -> Vec<Transition> {
        vec![]
    }
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        None
    }
    fn print_canonical_name(&self) -> String {
        "Stop".to_string()
    }
}
pub struct StateMachine {
    current_state: Option<Box<dyn State>>,
}
impl StateMachine {
    pub fn new() -> Self {
        StateMachine {
            current_state: None,
        }
    }
    ///Change the state according to the kind input
    pub fn transition(&mut self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<()> {
        if self.current_state.is_none() {
            self.current_state = Some(kind.1.into_base_state());
            Some(())
        } else {
            let mut new_state = self.current_state.as_mut().unwrap().transition(kind, kg);
            if new_state.is_none() {
                None
            } else {
                self.current_state = new_state;
                Some(())
            }
        }
    }
    pub fn print_state_name(&self) -> String {
        self.current_state.as_ref().unwrap().print_canonical_name()
    }
}
