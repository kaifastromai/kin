/*!
Simple DSL to specify kingraph relationships
Each statement is a line in the DSL
Each line first specifies a person, followed M|F in parenthesies, then a relationship, then a person
The last statement queries the graph for a relationship between two people
{NAME} {M|F} {RELATIONSHIP} {NAME} {M|F}
The relationships are in all caps, and are:
  PARENT
  CHILD
  RP (reproductive partner)
  SIBLING
The last line is a query, and is of the form:
{NAME} TO {NAME}
*/

use std::collections::HashMap;

use itertools::Itertools;

use crate::{KinGraph, Person};
pub fn parse_kin_dsl(string: &str) -> anyhow::Result<KinGraph> {
    //first trim the string
    let string = string.trim();
    let mut kg = KinGraph::new();
    let mut persons = HashMap::<&str, Person>::new();
    let mut person_counter = 0;
    let lines = string.lines().collect_vec();
    for line in lines[0..lines.len() - 1].iter() {
        let words = line.trim();
        let mut words = line.split_whitespace();
        let name = words.next().unwrap();
        let sex = words.next().unwrap();
        let sex = match sex {
            "M" => crate::Sex::Male,
            "F" => crate::Sex::Female,
            _ => anyhow::bail!("Invalid sex"),
        };
        let p_id = match persons.get(name) {
            Some(id) => *id,
            None => {
                let person = kg.np(sex);
                persons.insert(name, person);
                person
            }
        };
        let rel = words.next().unwrap();
        let rel = match rel {
            "PARENT" => crate::Kind::Parent,
            "CHILD" => crate::Kind::Child,
            "RP" => crate::Kind::RP,
            "SIBLING" => crate::Kind::Sibling,
            _ => anyhow::bail!("Invalid relationship"),
        };
        let name2 = words.next().unwrap();
        let sex2 = words.next().unwrap();
        let sex2 = match sex2 {
            "M" => crate::Sex::Male,
            "F" => crate::Sex::Female,
            _ => anyhow::bail!("Invalid sex"),
        };
        let p_id2 = match persons.get(name2) {
            Some(id) => *id,
            None => {
                let person = kg.np(sex2);
                persons.insert(name2, person);
                person
            }
        };
        //add relationship
        kg.add_relation(p_id, p_id2, rel)?;
    }
    let mut words = lines[lines.len() - 1].split_whitespace();
    let name = words.next().unwrap();
    let name2 = words.nth(1).unwrap();
    let p_id = match persons.get(name) {
        Some(id) => *id,
        None => anyhow::bail!("Invalid person"),
    };
    let p_id2 = match persons.get(name2) {
        Some(id) => *id,
        None => anyhow::bail!("Invalid person"),
    };
    let rels = kg.get_canonical_relationships(p_id, p_id2)?;
    println!("RELATIONS: {:?}", rels);
    Ok(kg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_dsl() {
        let dsl = r#"
        Izy F PARENT Mary F
        Solomon M CHILD Mary F
        Solomon M RP Izy F
        Izy TO Solomon
        "#;
        let kg = parse_kin_dsl(dsl).unwrap();
        //represnt half-sibling relation ship
        let half_sib_dsl = r#"
        Izy F RP John M
        John M RP Mary F
        Mike M CHILD Izy F
        Mike M CHILD John M
        Kalob M CHILD Mary F
        Kalob M CHILD John M
        Kalob TO Mike
        "#;
        let kg = parse_kin_dsl(half_sib_dsl).unwrap();
    }
}
