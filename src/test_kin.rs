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
    kg.add_relation(&p1, &p4, Kind::Repat).unwrap();
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
    assert!(!kg.is_rrb(&kg.px(0), &kg.px(2)));
    assert!(kg.is_rrb(&kg.px(0), &kg.px(1)));
    assert!(kg.is_rrb(&kg.px(0), &kg.px(6)));
}

#[test]
pub fn cousins() {
    let mut kg = setup_cousins().unwrap();
    use std::fs::File;
    let mut f = File::create("cousins.dot").unwrap();
    render_to(&mut f, &kg);
    println!(
        "Kin says: {:?}",
        kg.get_canonical_relationships(&kg.px(7), &kg.px(6))
    );
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
    kg.add_relation(&p1, &p4, Kind::Repat).unwrap();
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

    kg.build_map(&p0);

    println!("{:?}", kg.get_canonical_relationships(&p4, &p0).unwrap());
    use std::fs::File;
    let mut f = File::create("out.dot").unwrap();
    render_to(&mut f, &kg);

    Ok(())
}