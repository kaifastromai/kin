use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static KINGRAPH_INSTANCE: RefCell<KinGraph> = RefCell::new(KinGraph::new());
}
#[derive(Deserialize, Serialize)]
pub struct KinWasmGraph {
    nodes: Vec<PersonNode>,
}
impl KinWasmGraph {
    pub fn new(nodes: Vec<PersonNode>) -> Self {
        Self { nodes }
    }
}
#[derive(Deserialize, Serialize)]
pub struct PersonNode {
    pub id: u32,
    pub sex: Sex,
    pub name: String,
    //vec of ids
    relations: Vec<Relation>,
}

impl PersonNode {
    pub fn get_sex(&self) -> Sex {
        self.sex
    }
    pub fn get_id(&self) -> u32 {
        self.id
    }
}
impl PersonNode {
    pub fn new(id: u32, sex: Sex, name: String, relations: Vec<Relation>) -> Self {
        Self {
            sex,
            id,
            relations,
            name,
        }
    }
}
#[wasm_bindgen]
#[derive(Deserialize, Serialize, Copy, Clone)]
pub struct Relation {
    pub id: u32,
    pub kind: RelationKind,
}
#[repr(C)]
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RelationKind {
    Parent = 0,
    Child = 1,
    RP = 2,
    Sibling = 3,
}
use crate::{kin_dsl, KinGraph, Kind, Person, Sex};
#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}
#[wasm_bindgen]
///Get a javascript representation of the current graph
pub fn get_graph() -> String {
    KINGRAPH_INSTANCE.with(|kg| {
        let kg = kg.borrow_mut();
        let r = kg.as_wasm_graph();
        let json = json!(r);
        json.to_string()
    })
}
#[wasm_bindgen]
///Add a relationship to the graph
pub fn add_relation(p1: &Person, p2: &Person, kind: RelationKind) -> Result<(), JsValue> {
    let kind = match kind {
        RelationKind::Parent => Kind::Parent,
        RelationKind::Child => Kind::Child,
        RelationKind::RP => Kind::RP,
        RelationKind::Sibling => Kind::Sibling,
    };
    KINGRAPH_INSTANCE.with(|kg| {
        let mut kg = kg.borrow_mut();
        kg.add_relation(p1, p2, kind)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    })
}
#[wasm_bindgen]
pub fn get_relation(p1: &Person, p2: &Person) -> Result<String, JsValue> {
    KINGRAPH_INSTANCE.with(|kg| {
        let kg = kg.borrow();
        let res = kg
            .get_canonical_relationships(p1, p2)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let res_str = res.iter().map(|r| r.to_string()).join(",");
        Ok(res_str)
    })
}
#[wasm_bindgen]
pub struct QueryResult {
    results: Vec<String>,
}
impl QueryResult {
    pub fn new(results: Vec<String>) -> Self {
        Self { results }
    }
}
#[wasm_bindgen]
pub fn append_from_dsl(dsl: &str) -> Result<String, JsValue> {
    KINGRAPH_INSTANCE.with(|kg| {
        let mut kg = kg.borrow_mut();
        let res =
            kin_dsl::query_kin(dsl, &mut kg).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let res_str = res.iter().map(|r| r.to_string()).collect_vec();
        let json = json!({
            "results": res_str
        });
        Ok(json.to_string())
    })
}
