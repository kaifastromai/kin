import init, { get_graph, get_relation, add_relation, append_from_dsl, Sex, Person } from '../kin-wasm/kin'

interface KinWasmPersonNode {
    id: number
    sex: string,
    relations: WasmRelation[]

}
export enum KinSex {
    Male,
    Female,
}
interface PersonNode {
    id: number
    sex: KinSex,
    name: string | undefined,
    relations: Relation[]
}
export interface WasmRelation {
    kind: string,
    id: number
}
export interface Relation {
    kind: RelationKind,
    id: number

}
export enum RelationKind {
    Parent,
    Child,
    RP,
    Sibling
}
class KinWasmGraph {
    nodes: KinWasmPersonNode[] | undefined
    constructor(json: string) {
        this.nodes = JSON.parse(json).nodes
    }
}
export class KinGraph {
    nodes: PersonNode[] | undefined
    public static async init_kin(): Promise<KinGraph> {
        console.log("initializing kin wasm")
        await init()
        var kg = new KinGraph()
        kg.nodes = []
        console.log("initialized kin wasm")
        return kg

    }
    merge_from_kin_wasm_graph(kin_wasm_graph: KinWasmGraph) {
        //the wasm graph does not contain the names of the person
        //so we need to merge the two graphs
        //If there are any persons with the same id that we have in our graph, update
        //to reflect the relations, but keep the name
        //if there are new persons, add them directly to our graph
        //if there are persons that are not in the wasm graph, remove them from our graph

        if (kin_wasm_graph.nodes == undefined) {
            return
        }
        //first, order the nodes by id
        kin_wasm_graph.nodes.sort((a, b) => a.id - b.id)
        //order our nodes by id
        this.nodes?.sort((a, b) => a.id - b.id)
        //now we can merge the two graphs
        var kwg_len = kin_wasm_graph.nodes.length
        //truncate our graph if it is longer than the wasm graph
        if (this.nodes != undefined && this.nodes.length > kwg_len) {
            this.nodes = this.nodes.slice(0, kwg_len)
        }
        //if longer, add the new nodes 
        if (this.nodes != undefined && this.nodes.length < kwg_len) {

            for (var i = this.nodes.length; i < kwg_len; i++) {

                var node = kin_wasm_graph.nodes[i]
                var relations = kin_wasm_graph.nodes[i].relations.map((r) => {
                    var relation;
                    switch (r.kind) {
                        case "Parent":
                            relation = RelationKind.Parent
                            break;
                        case "Child":
                            relation = RelationKind.Child
                            break;
                        case "RP":
                            relation = RelationKind.RP
                            break;
                        case "Sibling":
                            relation = RelationKind.Sibling
                            break;
                    }
                    return { kind: relation!, id: r.id }

                })
                this.nodes.push({ id: node.id, sex: node.sex == "Male" ? KinSex.Male : KinSex.Female, name: undefined, relations: relations })
            }
        }
        //now we can update the relations
        if (this.nodes != undefined) {
            for (var i = 0; i < this.nodes.length; i++) {
                var node = kin_wasm_graph.nodes[i]
                var our_node = this.nodes[i]
                //update the relations
                var relations = node.relations.map((r) => {
                    var relation;
                    switch (r.kind) {
                        case "Parent":
                            relation = RelationKind.Parent
                            break;
                        case "Child":
                            relation = RelationKind.Child
                            break;
                        case "RP":
                            relation = RelationKind.RP
                            break;
                        case "Sibling":
                            relation = RelationKind.Sibling
                            break;
                    }
                    return { kind: relation!, id: r.id }

                })
                our_node.relations = relations

            }
        }

    }
    load_from_dsl(dsl: string) {
        append_from_dsl(dsl)
        var json = get_graph()
        //convert json string to object
        var wasm_graph = new KinWasmGraph(json)
        this.merge_from_kin_wasm_graph(wasm_graph)

    }
    query(dsl: string) {
        return append_from_dsl(dsl)
    }
    get_relation(p1: PersonNode, p2: PersonNode) {
        var person1 = Person.new_with_id(p1.sex, p1.id)
        var person2 = Person.new_with_id(p2.sex, p2.id)
        return get_relation(person1, person2)
    }


}

