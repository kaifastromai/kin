use ::uuid::serde::compact::deserialize;
use kin_dsl::query_kin;

use self::kin_dsl::parse_relations_from_dsl;

use super::*;

fn setup_cousins() -> Result<KinGraph> {
    let mut kg = KinGraph::new();
    let p0 = kg.np(Sex::Male);
    let p1 = kg.np(Sex::Female);
    let p2 = kg.np(Sex::Male);
    let p3 = kg.np(Sex::Female);
    let p4 = kg.np(Sex::Male);
    let p5 = kg.np(Sex::Female);
    let p6 = kg.np(Sex::Male);
    let p7 = kg.np(Sex::Female);
    kg.make_child(&p2, &p0, &p1)?;
    kg.make_child(&p3, &p0, &p1)?;
    kg.make_child(&p6, &p2, &p5)?;
    kg.make_child(&p7, &p3, &p4)?;
    Ok(kg)

    //p1, p2 parents of p3,p4
}
fn setup_half_siblings() -> Result<KinGraph> {
    let mut kg = KinGraph::new();
    let p0 = kg.np(Sex::Male);
    let p1 = kg.np(Sex::Female);
    let p2 = kg.np(Sex::Male);
    let p3 = kg.np(Sex::Female);
    let p4 = kg.np(Sex::Male);
    //p3 is the child of p0 and p1
    kg.make_child(&p3, &p0, &p1)?;
    //p4 is the child of p1 and p2
    kg.make_child(&p4, &p1, &p2)?;
    //they should be cousins
    Ok(kg)
}
//niece/nephew aunt/uncle setup
fn setup_nn_au() -> Result<KinGraph> {
    let mut kg = KinGraph::new();
    let p0 = Person::new(Sex::Female);
    let p1 = Person::new(Sex::Female);
    let p2 = Person::new(Sex::Male);
    let p3 = Person::new(Sex::Male);
    let p4 = Person::new(Sex::Female);
    let _p5 = Person::new(Sex::Female);
    let _p6 = Person::new(Sex::Male);
    let _p7 = Person::new(Sex::Male);
    let _p8 = Person::new(Sex::Female);
    let _p9 = Person::new(Sex::Female);
    kg.add_persons(&[&p0, &p1, &p2, &p3, &p4]);
    kg.add_sibling(&p0, &p1)?;
    kg.add_parent(&p1, &p2)?;
    kg.add_parent(&p2, &p3)?;

    Ok(kg)
}
fn setup_basic_kg() -> Result<KinGraph> {
    let mut kg = KinGraph::new();
    //make some persons, the sexes aren't important
    let p0 = Person::new(Sex::Female);
    let p1 = Person::new(Sex::Female);
    let p2 = Person::new(Sex::Male);
    let p3 = Person::new(Sex::Male);
    let p4 = Person::new(Sex::Female);
    let p5 = Person::new(Sex::Female);
    let p6 = Person::new(Sex::Male);
    let p7 = Person::new(Sex::Male);
    let p8 = Person::new(Sex::Female);
    let p9 = Person::new(Sex::Female);
    //add them as nodes
    kg.add_person(&p0);
    kg.add_person(&p1);
    kg.add_person(&p2);
    kg.add_person(&p3);
    kg.add_person(&p4);
    kg.add_person(&p5);
    kg.add_person(&p6);
    kg.add_person(&p7);
    kg.add_person(&p8);
    kg.add_person(&p9);

    //make relationships
    kg.add_relation(&p0, &p1, Kind::Parent)?;
    kg.add_relation(&p2, &p1, Kind::Parent)?;
    //from here, we have enough information to deduce that p1 and p3 are Repat
    //In real life, we could run some preprocessor over the raw graph to make this and perhaps
    //other observations more explicit, and also to verify that the graph is well formed
    //(i.e. a node cannot have more than 2 parents, and those two cannot be of the same sex,
    // a node can't parent itself, and a node cannot be connected to another node more than once in incompatible ways )
    //give p2 a child
    kg.add_relation(&p1, &p3, Kind::Parent).unwrap();
    //and a spouse
    kg.add_relation(&p1, &p4, Kind::RP).unwrap();
    //p3 is parent of p7
    kg.add_relation(&p3, &p7, Kind::Parent).unwrap();
    //p7 is parent of p8
    kg.add_relation(&p7, &p8, Kind::Parent).unwrap();

    //p5 is parent of p4
    kg.add_relation(&p4, &p3, Kind::Parent).unwrap();
    //p6 is sibling of p4
    kg.add_relation(&p5, &p3, Kind::Sibling).unwrap();
    //and p6 is parent of p7
    kg.add_relation(&p5, &p6, Kind::Parent).unwrap();

    Ok(kg)
}
#[test]
fn test_rrb() {
    let kg = setup_basic_kg().unwrap();
    assert!(!kg.is_rbb(kg.px(0), kg.px(2)));
    assert!(kg.is_rbb(kg.px(0), kg.px(1)));
    assert!(kg.is_rbb(kg.px(0), kg.px(6)));
}

#[test]
pub fn cousins() {
    tracing_subscriber::fmt().init();
    let cousin_dsl = r#"
    Me M CHILD Sean M
    John M SIBLING Sean M
    Quinn M CHILD John M
    :QUERY
    Me TO Quinn
    "#;
    let mut kg = KinGraph::new();
    let states = query_kin(&cousin_dsl, &mut kg).unwrap();
    tracing::warn!(relationships=?states);
    use std::fs::File;
    let mut f = File::create("cousins.dot").unwrap();
    render_to(&mut f, &kg);

    let res = states.iter().any(|s| {
        s.get_hash()
            == NCsnKState {
                n: 0,
                k: 0,
                is_half: false,
                sex: Sex::Male,
            }
            .get_hash()
    });

    assert!(res);
}
#[test]
pub fn half_siblings() {
    tracing_subscriber::fmt().init();
    let kg = setup_half_siblings().unwrap();
    use std::fs::File;
    let mut f = File::create("half_siblings.dot").unwrap();
    render_to(&mut f, &kg);
    let states = kg.get_canonical_relationships(kg.px(3), kg.px(4)).unwrap();
    tracing::info!(relations=?states);
    let res = states.iter().any(|s| {
        s.get_hash()
            == SiblingState {
                is_half: true,
                sex: Sex::Female,
            }
            .get_hash()
    });
    assert!(res);
}
#[test]
pub fn incest() {
    let kg = KinGraph::new();
    use std::fs::File;
    let mut f = File::create("incest.dot").unwrap();
    render_to(&mut f, &kg);
}
///Test nephew/niece and aunt/uncle relationship
#[test]
pub fn nn_au() {
    let kg = setup_nn_au().unwrap();
    use std::fs::File;
    let mut f = File::create("nn_au.dot").unwrap();
    render_to(&mut f, &kg);
    let nn_states = kg.get_canonical_relationships(kg.px(2), kg.px(0)).unwrap();
    println!("NN_AU TEST: {:?}", nn_states);
    let res = nn_states.iter().any(|s| {
        s.get_hash()
            == NNeniState {
                n: 0,
                is_half: false,
                sex: kg.px(2).sex,
            }
            .get_hash()
    });
    //let state = states[0].get_any().downcast_ref::<NAUState>().unwrap();
    assert!(res);

    let au_states = kg.get_canonical_relationships(kg.px(0), kg.px(2)).unwrap();
    println!("AU_NN TEST: {:?}", au_states);
    let res = au_states.iter().any(|s| {
        s.get_hash()
            == NAUState {
                n: 0,
                is_half: false,
                sex: kg.px(0).sex,
            }
            .get_hash()
    });
    assert!(res);
}
#[test]
fn test_main() -> Result<()> {
    let mut kg = KinGraph::new();
    //make some persons, the sexes aren't important
    let p0 = Person::new(Sex::Female);
    let p1 = Person::new(Sex::Female);
    let p2 = Person::new(Sex::Male);
    let p3 = Person::new(Sex::Male);
    let p4 = Person::new(Sex::Female);
    let p5 = Person::new(Sex::Female);
    let p6 = Person::new(Sex::Male);
    let p7 = Person::new(Sex::Male);
    let p8 = Person::new(Sex::Female);
    let p9 = Person::new(Sex::Female);
    //add them as nodes
    kg.add_person(&p0);
    kg.add_person(&p1);
    kg.add_person(&p2);
    kg.add_person(&p3);
    kg.add_person(&p4);
    kg.add_person(&p5);
    kg.add_person(&p6);
    kg.add_person(&p7);
    kg.add_person(&p8);
    kg.add_person(&p9);

    //make relationship
    kg.add_relation(&p0, &p1, Kind::Parent)?;
    kg.add_relation(&p2, &p1, Kind::Parent)?;
    //from here, we have enough information to deduce that p1 and p3 are Repat
    //In real life, we could run some preprocessor over the raw graph to make this and perhaps
    //other observations more explicit, and also to verify that the graph is well formed
    //(i.e. a node cannot have more than 2 parents, and those two cannot be of the same sex,
    // a node can't parent itself, and a node cannot be connected to another node more than once in incompatible ways )
    //give p2 a child
    kg.add_relation(&p1, &p3, Kind::Parent).unwrap();
    //and a spouse
    kg.add_relation(&p1, &p4, Kind::RP).unwrap();
    //p3 is parent of p7
    kg.add_relation(&p3, &p7, Kind::Parent).unwrap();
    //p7 is parent of p8
    kg.add_relation(&p7, &p8, Kind::Parent).unwrap();

    //p5 is parent of p4
    kg.add_relation(&p4, &p3, Kind::Parent).unwrap();
    //p6 is sibling of p4
    kg.add_relation(&p5, &p3, Kind::Sibling).unwrap();
    //and p6 is parent of p7
    kg.add_relation(&p5, &p6, Kind::Parent).unwrap();

    println!("{:?}", kg.get_canonical_relationships(&p4, &p0).unwrap());
    use std::fs::File;
    let mut f = File::create("out.dot").unwrap();
    render_to(&mut f, &kg);

    Ok(())
}
#[test]
fn test_child_parent_of_parent() -> Result<()> {
    let dsl = r#"
    Izy F PARENT Mary F
    Mary F PARENT Izy F
    "#;
    let mut kg = KinGraph::new();
    let res = parse_relations_from_dsl(&dsl, &mut kg);
    println!("{:?}", res);
    //assert error
    assert!(res.is_err());
    Ok(())
}
#[test]
fn test_incest() -> Result<()> {
    
    let dsl = r#"
    Izy F PARENT John M
    Izy F RP John M
    "#;
    let mut kg = KinGraph::new();
    let res = parse_relations_from_dsl(&dsl, &mut kg);
    println!("{:?}", res);
    //get rel between Izy and John
    let rel = kg.get_canonical_relationships(kg.px(0), kg.px(1));
    //render the graph to a file
    use std::fs::File;
    let mut f = File::create("incest.dot").unwrap();
    render_to(&mut f, &kg);

    println!("Relationships are {:?}", rel);
    Ok(())
}
#[test]
fn test_is_parent() -> Result<()> {
    let mut kg = KinGraph::new();
    let p0 = Person::new(Sex::Female);
    let p1 = Person::new(Sex::Female);
    kg.add_person(&p0);
    kg.add_person(&p1);
    kg.add_relation(&p0, &p1, Kind::Parent)?;
    let res = kg.is_parent(kg.idx(&p0).unwrap(), kg.idx(&p1).unwrap());
    assert!(res);
    let is_child = kg.is_child(kg.idx(&p1).unwrap(), kg.idx(&p0).unwrap());
    assert!(is_child);
    Ok(())
}
#[test]
//test where there are multiple paths between two nodes
fn find_multiple_paths() -> Result<()> {
    let mut kg = KinGraph::new();
    let dsl = r#"
    Izy F PARENT Mary F
    Izy F RP Mary F"#;
    let res = parse_relations_from_dsl(&dsl, &mut kg);
    println!("{:?}", res);
    let rels = kg.find_all_paths(kg.px(0), kg.px(1));
    println!("{:?}", rels);
    Ok(())
}
