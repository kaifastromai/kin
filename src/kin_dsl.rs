/*!
Simple DSL to specify kingraph relationships
Each statement is a line in the DSL
Each line first specifies a person, followed M|F in parenthesies, then a relationship, then a person
{NAME} {M|F} {RELATIONSHIP} {NAME} {M|F}
The relationships are in all caps, and are:
  PARENT
  CHILD
  RP (reproductive partner)
  SIBLING
It is then followed by the label ":QUERY". All subsequent lines are queries
of the form:
{NAME} TO {NAME}
*/

use std::collections::HashMap;

use itertools::Itertools;

use crate::{states::State, KinGraph, Person};
///Takes a string representing kin dsl and processes it. If there are any statements, adds the relationships to the graph, and if there are any queries, returns the results of the queries
pub fn query_kin(string: &str, kg: &mut KinGraph) -> anyhow::Result<Vec<Box<dyn State>>> {
    //first trim the string
    let string = string.trim();
    let mut persons = HashMap::<&str, Person>::new();
    let mut statements = Vec::new();
    let mut queries = Vec::new();
    let mut found_query = false;
    for line in string.lines() {
        let line = line.trim();
        if line.starts_with(":QUERY") {
            if found_query {
                anyhow::bail!("Invalid DSL");
            }
            found_query = true;
            continue;
        }
        if found_query {
            queries.push(line);
        } else {
            statements.push(line);
        }
    }
    for line in statements {
        let line = line.trim();
        let mut words = line.split_whitespace();
        let name = words.next().unwrap();
        let sex = words.next().unwrap();
        let sex = match sex {
            "M" => crate::Sex::Male,
            "F" => crate::Sex::Female,
            _ => anyhow::bail!("Invalid sex"),
        };
        let p_id = match persons.get(name) {
            Some(id) => id.clone(),
            None => {
                let person = kg.np_with_name(sex, name.to_string());
                persons.insert(name, person.clone());
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
            Some(id) => id.clone(),
            None => {
                let person = kg.np_with_name(sex2, name2.to_string());
                persons.insert(name2, person.clone());
                person
            }
        };
        //add relationship
        kg.add_relation(&p_id, &p_id2, rel)?;
    }
    let queries = queries
        .iter()
        .map(|q| {
            let mut words = q.split_whitespace();
            let name = words.next().unwrap();
            let name2 = words.nth(1).unwrap();
            let p_id = match persons.get(name) {
                Some(id) => id,
                None => anyhow::bail!("Person with name {} not found", name),
            };
            let p_id2 = match persons.get(name2) {
                Some(id) => id,
                None => anyhow::bail!("Person with name {} not found", name2),
            };
            kg.get_canonical_relationships(p_id, p_id2)
        })
        .try_collect::<_, Vec<_>, _>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(queries)
    //Ok(queries)
}
///Add persons and relations to the graph from a string
pub fn parse_relations_from_dsl(string: &str, kg: &mut KinGraph) -> Result<(), anyhow::Error> {
    //first trim the string
    let string = string.trim();
    let mut persons = HashMap::<&str, Person>::new();
    let mut statements = Vec::new();
    let mut queries = Vec::new();
    let mut found_query = false;
    for line in string.lines() {
        let line = line.trim();
        if line.starts_with(":QUERY") {
            if found_query {
                anyhow::bail!("Invalid DSL");
            }
            found_query = true;
            continue;
        }
        if found_query {
            queries.push(line);
        } else {
            statements.push(line);
        }
    }
    for line in statements {
        let line = line.trim();
        let mut words = line.split_whitespace();
        let name = words.next().unwrap();
        let sex = words.next().unwrap();
        let sex = match sex {
            "M" => crate::Sex::Male,
            "F" => crate::Sex::Female,
            _ => anyhow::bail!("Invalid sex"),
        };
        let p_id = match persons.get(name) {
            Some(id) => id.clone(),
            None => {
                let person = kg.np_with_name(sex, name.to_string());
                persons.insert(name, person.clone());
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
            Some(id) => id.clone(),
            None => {
                let person = kg.np_with_name(sex2, name2.to_string());
                persons.insert(name2, person.clone());
                person
            }
        };
        //add relationship
        kg.add_relation(&p_id, &p_id2, rel)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tracing::info;

    use super::*;

    #[test]
    fn test_basic_dsl() {
        tracing_subscriber::fmt::init();
        // let dsl = r#"
        // Izy F PARENT Mary F
        // Solomon M CHILD Mary F
        // :QUERY
        // Izy TO Solomon
        // "#;
        // let mut kg = KinGraph::new();

        // let res = query_kin(dsl, &mut kg).unwrap();
        // let mut file = std::fs::File::create("grandchild_rp.dot").unwrap();
        // crate::render_to(&mut file, &kg);
        // println!("RP and grandchild {:?}", res);
        //represnt half-sibling relationship
        // let half_sib_dsl = r#"
        // Mike M CHILD Izy F
        // Mike M CHILD John M
        // Kalob M CHILD Mary F
        // Kalob M CHILD John M
        // :QUERY
        // Kalob TO Mike
        // "#;
        // let mut kg = KinGraph::new();
        // info!("Looking for half sib relationship");
        // let res = query_kin(half_sib_dsl, &mut kg).unwrap();
        // let mut file = std::fs::File::create("half_sib.dot").unwrap();
        // crate::render_to(&mut file, &kg);
        // info!(relationships = ?res, "Half sibling relationship");

        //incest child
        // let incest_dsl = r#"
        // Mike M CHILD Izy F
        // Mike M PARENT Izy F
        // :QUERY
        // Mike TO Izy"#;
        // let mut kg = KinGraph::new();
        // let res = query_kin(incest_dsl, &mut kg).unwrap();
        // println!("Incest");
        // for r in res {
        //     println!("{}", r);
        // }
    }
}
