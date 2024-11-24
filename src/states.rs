use std::{any::Any, hash::Hasher};

use super::*;
type Transition = (Kind, Box<dyn State>);
///Represents a possible state, and describes the possible transitions from that state.
pub trait State: Any {
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>>;
    fn print_canonical_name(&self) -> String;
    fn clone_box(&self) -> Box<dyn State>;
    fn get_any(&self) -> &dyn std::any::Any;
    //get a unique hash for this state.
    fn get_hash(&self) -> u64;
}
impl Clone for Box<dyn State> {
    fn clone(&self) -> Box<dyn State> {
        self.clone_box()
    }
}
impl std::fmt::Debug for dyn State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.print_canonical_name())
    }
}
impl std::fmt::Display for dyn State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.print_canonical_name())
    }
}
impl std::hash::Hash for dyn State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_hash().hash(state);
    }
}
impl std::cmp::PartialEq for dyn State {
    fn eq(&self, other: &dyn State) -> bool {
        self.get_hash() == other.get_hash()
    }
}
impl std::cmp::Eq for dyn State {}
///An nth level parent (Parent, grandparent, great-grandparent, etc.)
#[derive(Hash, Eq, PartialEq, Debug)]
pub struct NParentState {
    pub n: usize,
    pub sex: super::Sex,
}
impl State for NParentState {
    fn transition(&self, kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        if self.n == 0 {
            let res: Box<dyn State> = match kind.1 {
                Kind::Parent => Box::new(NParentState {
                    n: 1,
                    sex: self.sex,
                }),
                Kind::Child => Box::new(RPState { sex: self.sex }),
                Kind::RP => Box::new(NPinLState {
                    n: 0,
                    sex: self.sex,
                }),
                Kind::Sibling => Box::new(NParentState {
                    n: 0,
                    sex: self.sex,
                }),
            };
            Some(res)
        } else {
            let res: Box<dyn State> = match kind.1 {
                Kind::Parent => Box::new(NParentState {
                    n: self.n + 1,
                    sex: self.sex,
                }),
                Kind::Child => Box::new(NPinLState {
                    n: self.n - 1,
                    sex: self.sex,
                }),
                Kind::RP => Box::new(NPinLState {
                    n: self.n + 1,
                    sex: self.sex,
                }),
                Kind::Sibling => Box::new(NParentState {
                    n: self.n,
                    sex: self.sex,
                }),
            };

            Some(res)
        }
    }
    fn print_canonical_name(&self) -> String {
        if self.n == 0 {
            let s = match self.sex {
                Sex::Female => "mother",
                Sex::Male => "father",
            };
            s.to_string()
        } else {
            let g = match self.n {
                1 => "grand".to_string(),
                _ => {
                    let greats_string = "great-".repeat(self.n - 1);
                    greats_string + "grand"
                }
            };
            format!(
                "{}-{}",
                g,
                if self.sex == Sex::Male {
                    "father"
                } else {
                    "mother"
                },
            )
        }
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(NParentState {
            n: self.n,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "NParentState".hash(&mut hasher);
        self.n.hash(&mut hasher);
        hasher.finish()
    }
}

///Nth parent in law state
pub struct NPinLState {
    pub n: usize,
    pub sex: super::Sex,
}
impl State for NPinLState {
    fn transition(&self, kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NParentState {
                n: self.n + 1,
                sex: self.sex,
            }),
            Kind::Child => Box::new(StopState {}),
            Kind::RP => Box::new(NPinLState {
                n: self.n,
                sex: self.sex,
            }),
            Kind::Sibling => Box::new(NParentState {
                n: self.n,
                sex: self.sex,
            }),
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
        format!(
            "{}-{} in law",
            g,
            if self.sex == Sex::Male {
                "father"
            } else {
                "mother"
            },
        )
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(NPinLState {
            n: self.n,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "NPinLState".hash(&mut hasher);
        self.n.hash(&mut hasher);
        hasher.finish()
    }
}

pub struct NChildState {
    pub n: usize,
    pub sex: super::Sex,
}
impl State for NChildState {
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        if self.n != 0 {
            let res: Box<dyn State> = match kind.1 {
                Kind::Parent => {
                    if kg.b_share_parents(kind.0, kind.2) {
                        Box::new(SiblingState {
                            is_half: false,
                            sex: self.sex,
                        })
                    } else {
                        Box::new(NNeniState {
                            n: self.n,
                            is_half: false,
                            sex: self.sex,
                        })
                    }
                }
                Kind::Child => Box::new(NChildState {
                    n: self.n + 1,
                    sex: self.sex,
                }),
                Kind::Sibling => Box::new(NAUState {
                    n: self.n,
                    is_half: false,
                    sex: kg.px(kind.0.index()).sex,
                }),
                Kind::RP => {
                    if kg.is_parent(kind.0, kind.2) {
                        Box::new(NChildState {
                            n: self.n + 1,
                            sex: self.sex,
                        })
                    } else {
                        Box::new(StopState {})
                    }
                }
            };
            Some(res)
        } else {
            let res: Box<dyn State> = match kind.1 {
                Kind::Parent => {
                    if kg.b_share_parents(kind.0, kind.2) {
                        Box::new(SiblingState {
                            is_half: false,
                            sex: self.sex,
                        })
                    } else {
                        Box::new(SiblingState {
                            is_half: true,
                            sex: self.sex,
                        })
                    }
                }
                Kind::Child => Box::new(NChildState {
                    n: 1,
                    sex: self.sex,
                }),
                Kind::RP => {
                    if kg.is_parent(kind.0, kind.2) {
                        Box::new(NChildState {
                            n: 1,
                            sex: self.sex,
                        })
                    } else {
                        Box::new(StopState {})
                    }
                }
                Kind::Sibling => Box::new(NNeniState {
                    n: 0,
                    is_half: false,
                    sex: self.sex,
                }),
            };
            Some(res)
        }
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            0 => "".to_string(),
            1 => "grand-".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "grand-"
            }
        };
        format!(
            "{}{}",
            g,
            if self.sex == Sex::Male {
                "son"
            } else {
                "daughter"
            },
        )
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(NChildState {
            n: self.n,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "NChildState".hash(&mut hasher);
        self.n.hash(&mut hasher);
        hasher.finish()
    }
}

///Nth child in law
pub struct NCinLState {
    pub n: usize,
    pub sex: super::Sex,
}
impl State for NCinLState {
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(StopState {}),
            Kind::Child => Box::new(NCinLState {
                n: self.n + 1,
                sex: self.sex,
            }),
            Kind::RP => {
                if kg.is_repart(kind.0, kind.2) {
                    Box::new(NCinLState {
                        n: self.n,
                        sex: self.sex,
                    })
                } else {
                    Box::new(StopState {})
                }
            }
            Kind::Sibling => Box::new(NNNinLState {
                n: self.n,
                is_half: false,
                sex: self.sex,
            }),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            0 => "".to_string(),
            1 => "great".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "great-"
            }
        };
        format!(
            "{}{} in law",
            g,
            if self.sex == Sex::Male {
                "son"
            } else {
                "daughter"
            },
        )
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(NCinLState {
            n: self.n,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "NCinLState".hash(&mut hasher);
        self.n.hash(&mut hasher);
        hasher.finish()
    }
}

///Nth cousin kth times removed
pub struct NCsnKState {
    pub n: usize,
    pub k: i32,
    pub is_half: bool,
    pub sex: super::Sex,
}
impl State for NCsnKState {
    fn transition(&self, kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NCsnKState {
                n: self.n,
                k: self.k + 1,
                is_half: false,
                sex: self.sex,
            }),
            Kind::Child => {
                if self.n == self.k as usize {
                    Box::new(NNeniState {
                        n: self.n,
                        is_half: self.is_half,
                        sex: self.sex,
                    })
                } else {
                    Box::new(NCsnKState {
                        n: self.n,
                        k: self.k - 1,
                        is_half: false,
                        sex: self.sex,
                    })
                }
            }
            Kind::RP => Box::new(StopState {}),
            Kind::Sibling => Box::new(NCsnKState {
                n: self.n,
                k: self.k,
                is_half: self.is_half,
                sex: self.sex,
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
        let half = if self.is_half {
            "half-".to_string()
        } else {
            "not half".to_string()
        };
        format!("{} {}cousins{}", n_string, half, rmv_str,)
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(NCsnKState {
            n: self.n,
            k: self.k,
            is_half: self.is_half,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "NCsnKState".hash(&mut hasher);
        self.n.hash(&mut hasher);
        self.k.hash(&mut hasher);
        (if self.is_half { "half" } else { "nothalf" }).hash(&mut hasher);
        hasher.finish()
    }
}

pub struct SiblingState {
    pub is_half: bool,
    pub sex: super::Sex,
}

impl State for SiblingState {
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NAUState {
                n: 0,
                is_half: self.is_half,
                sex: kg.px(kind.0.index()).sex,
            }),
            Kind::Child => Box::new(NChildState {
                n: 0,
                sex: self.sex,
            }),
            Kind::RP => Box::new(SinLState {
                is_half: self.is_half,
                sex: self.sex,
            }),
            Kind::Sibling => Box::new(SiblingState {
                is_half: self.is_half,
                sex: self.sex,
            }),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        format!(
            "{}{}",
            if self.is_half { "half-" } else { "" },
            if self.sex == Sex::Male {
                "brother"
            } else {
                "sister"
            },
        )
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(SiblingState {
            is_half: self.is_half,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "SiblingState".hash(&mut hasher);
        self.is_half.hash(&mut hasher);
        hasher.finish()
    }
}

///Reproductive partner
pub struct RPState {
    pub sex: super::Sex,
}
impl State for RPState {
    fn transition(&self, kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(StopState {}),
            Kind::Child => Box::new(NCinLState {
                n: 0,
                sex: self.sex,
            }),
            Kind::RP => Box::new(StopState {}),
            Kind::Sibling => Box::new(SinLState {
                is_half: false,
                sex: self.sex,
            }),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        match self.sex {
            Sex::Female => "female reproductive partner",
            Sex::Male => "male reproductive partner",
        }
        .to_string()
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(RPState { sex: self.sex })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "RPState".hash(&mut hasher);
        hasher.finish()
    }
}

pub struct SinLState {
    pub is_half: bool,
    pub sex: super::Sex,
}
impl State for SinLState {
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NAUinLState {
                n: 0,
                is_half: self.is_half,
                sex: kg.px(kind.0.index()).sex,
            }),
            Kind::Child => Box::new(RPState { sex: self.sex }),
            Kind::RP => Box::new(StopState {}),
            Kind::Sibling => Box::new(SinLState {
                is_half: self.is_half,
                sex: self.sex,
            }),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        format!(
            "{}{} in law",
            if self.sex == Sex::Male {
                "brother"
            } else {
                "sister"
            },
            if self.is_half { "half-" } else { "" }
        )
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(SinLState {
            is_half: self.is_half,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "SinLState".hash(&mut hasher);
        self.is_half.hash(&mut hasher);
        hasher.finish()
    }
}
///Nephew/Niece state
pub struct NNeniState {
    pub n: usize,
    pub is_half: bool,
    pub sex: super::Sex,
}
impl State for NNeniState {
    fn transition(&self, kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NCsnKState {
                n: self.n,
                k: self.n as i32,
                is_half: self.is_half,
                sex: self.sex,
            }),
            Kind::Child => Box::new(StopState {}),
            Kind::RP => Box::new(StopState {}),
            Kind::Sibling => Box::new(NNeniState {
                n: self.n,
                is_half: self.is_half,
                sex: self.sex,
            }),
        };
        Some(res)
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            0 => "".to_string(),
            1 => "grand".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "great-"
            }
        };
        format!(
            "{}{}",
            g,
            if self.sex == Sex::Male {
                "nephew"
            } else {
                "niece"
            }
        )
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(NNeniState {
            n: self.n,
            is_half: self.is_half,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "NNeniState".hash(&mut hasher);
        self.n.hash(&mut hasher);
        self.is_half.hash(&mut hasher);
        hasher.finish()
    }
}

///Nephew/Niece in law state
pub struct NNNinLState {
    pub n: usize,
    pub is_half: bool,
    pub sex: super::Sex,
}

impl State for NNNinLState {
    fn transition(&self, kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => Box::new(NCsnKState {
                n: self.n,
                k: self.n as i32,
                is_half: self.is_half,
                sex: self.sex,
            }),
            Kind::Child => Box::new(StopState {}),
            Kind::RP => Box::new(StopState {}),
            Kind::Sibling => Box::new(NNeniState {
                n: self.n,
                is_half: self.is_half,
                sex: self.sex,
            }),
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
        format!(
            "{}-{} in law",
            g,
            if self.sex == Sex::Male {
                "nephew"
            } else {
                "niece"
            }
        )
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(NNNinLState {
            n: self.n,
            is_half: self.is_half,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "NNNinLState".hash(&mut hasher);
        self.n.hash(&mut hasher);
        self.is_half.hash(&mut hasher);
        hasher.finish()
    }
}

///Aunt/uncle state
pub struct NAUState {
    pub n: usize,
    pub is_half: bool,
    pub sex: super::Sex,
}
impl State for NAUState {
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        if self.n == 0 {
            let res: Box<dyn State> = match kind.1 {
                Kind::Parent => Box::new(NAUState {
                    n: self.n,
                    is_half: false,
                    sex: self.sex,
                }),
                Kind::Child => Box::new(SinLState {
                    is_half: self.is_half,
                    sex: self.sex,
                }),
                Kind::RP => Box::new(NAUinLState {
                    n: self.n,
                    is_half: self.is_half,
                    sex: self.sex,
                }),
                Kind::Sibling => Box::new(NAUState {
                    n: self.n,
                    is_half: self.is_half,
                    sex: self.sex,
                }),
            };
            Some(res)
        } else {
            let res: Box<dyn State> = match kind.1 {
                Kind::Parent => Box::new(NAUState {
                    n: self.n + 1,
                    is_half: false,
                    sex: kg.px(kind.0.index()).sex,
                }),
                Kind::Child => Box::new(NAUinLState {
                    n: self.n - 1,
                    is_half: self.is_half,
                    sex: kg.px(kind.0.index()).sex,
                }),
                Kind::RP => Box::new(NAUinLState {
                    n: self.n,
                    is_half: self.is_half,
                    sex: self.sex,
                }),
                Kind::Sibling => Box::new(NAUState {
                    n: self.n,
                    is_half: self.is_half,
                    sex: self.sex,
                }),
            };
            Some(res)
        }
    }
    fn print_canonical_name(&self) -> String {
        let g = match self.n {
            0 => "".to_string(),
            1 => "grand".to_string(),
            _ => {
                let greats_string = "great-".repeat(self.n - 1);
                greats_string + "great-"
            }
        };
        format!(
            "{}{}",
            g,
            if self.sex == Sex::Male {
                "uncle"
            } else {
                "aunt"
            }
        )
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(NAUState {
            n: self.n,
            is_half: self.is_half,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "NAUState".hash(&mut hasher);
        self.n.hash(&mut hasher);
        self.is_half.hash(&mut hasher);
        hasher.finish()
    }
}
pub struct NAUinLState {
    pub n: usize,
    pub is_half: bool,
    pub sex: super::Sex,
}

impl State for NAUinLState {
    fn transition(&self, kind: (Nd, Kind, Nd), kg: &KinGraph) -> Option<Box<dyn State>> {
        let res: Box<dyn State> = match kind.1 {
            Kind::Parent => {
                if !kg.is_rbb(kg.px(kind.0.index()), kg.px(kind.0.index())) {
                    Box::new(StopState {})
                } else {
                    Box::new(NAUState {
                        n: self.n + 1,
                        is_half: self.is_half,
                        sex: self.sex,
                    })
                }
            }
            Kind::Child => Box::new(StopState {}),
            Kind::RP => Box::new(StopState {}),
            Kind::Sibling => Box::new(NAUinLState {
                n: self.n,
                is_half: self.is_half,
                sex: kg.px(kind.0.index()).sex,
            }),
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
        format!(
            "{}-{} in law",
            g,
            if self.sex == Sex::Male {
                "uncle"
            } else {
                "aunt"
            }
        )
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(NAUinLState {
            n: self.n,
            is_half: self.is_half,
            sex: self.sex,
        })
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "NAUinLState".hash(&mut hasher);
        self.n.hash(&mut hasher);
        self.is_half.hash(&mut hasher);
        hasher.finish()
    }
}
pub struct StopState {}
impl State for StopState {
    fn transition(&self, _kind: (Nd, Kind, Nd), _kg: &KinGraph) -> Option<Box<dyn State>> {
        None
    }
    fn print_canonical_name(&self) -> String {
        "Stop".to_string()
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(StopState {})
    }
    fn get_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_hash(&self) -> u64 {
        //hash based on name and variables
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "StopState".hash(&mut hasher);
        hasher.finish()
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
            self.current_state = Some(kind.1.into_base_state(kg.px(kind.0.index()).sex));
            Some(())
        } else {
            let new_state = self.current_state.as_mut().unwrap().transition(kind, kg);
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
    pub fn get_current_state(&self) -> Box<dyn State> {
        self.current_state.as_ref().unwrap().clone_box()
    }
}
