import * as PIXI from "pixi.js"
import { KinGraph, KinSex, Relation, RelationKind } from "./kin";
import { Sex } from "../kin-wasm/kin";

const SCALE = 5;
export class KinGraphView {
    app: PIXI.Application;
    kin_graph: KinGraph;
    pixi_nodes: Map<number, PIXI.Container> = new Map<number, PIXI.Container>()
    constructor(app: PIXI.Application, kin_graph: KinGraph) {
        this.app = app
        this.kin_graph = kin_graph
    }
    //Render the kingraph to pixi canvas
    //The idea is simple.
    //We start a given node (the root), and proceed depth first along the graph
    //When we go along a parent edge and the parent is a female, we a draw a line like this:
    //|
    //|___
    //   |
    //   |
    //If it is male, then reflect the graph along verical axis
    //If it is a child, we reflect the graph along horizontal axis
    //If it is a sibling or reproductive partner, then just draw a line
    //The nodes are rendered as circles with the name of the person inside

    render_graph(root_node: number = 0) {
        if (this.kin_graph.nodes == undefined) {
            console.log("no nodes")
            return
        }
        //move transform to center of screen
        this.app.stage.x = this.app.screen.width / 2
        this.app.stage.y = this.app.screen.height / 2
        var root_render = this.render_node(root_node, { x: this.app.screen.width / 2, y: this.app.screen.height / 2 })
        //add to scene
        this.app.stage.addChild(root_render!)
        var visited = new Set<number>()
        var stack = new Array<number>()
        stack.push(root_node)
        visited.add(root_node)
        this.pixi_nodes.set(root_node, root_render!)
        while (stack.length > 0) {
            var current_node = stack[stack.length - 1]
            console.log("Trying to access node " + stack[current_node])
            var node = this.kin_graph.nodes[current_node]
            //find the next node to visit
            var next_node_id = -1
            var cur_relation = node.relations[0]
            for (var i = 0; i < node.relations.length; i++) {
                var relation = node.relations[i]
                if (!visited.has(relation.id)) {
                    next_node_id = relation.id
                    cur_relation = relation
                    break
                }
            }
            if (next_node_id == -1) {
                //go back
                stack.pop()
                console.log("going back")
                continue
            }
            //we have the next node to visit
            var new_pos = { x: 0, y: 0 }
            if (cur_relation.kind == RelationKind.Parent) {
                //render the new node up 10 and to the left or right 10
                var side = (this.kin_graph!.nodes[next_node_id].sex as KinSex) == KinSex.Male ? -1 : 1;
                var g_pos = this.pixi_nodes.get(current_node)!.getGlobalPosition()
                new_pos.x = g_pos.x + side * 10 * SCALE
                new_pos.y = g_pos.y + 10 * SCALE
                console.log("new pos is " + new_pos.x + "," + new_pos.y)

            } else if (cur_relation.kind == RelationKind.Child) {
                //down instead of up
                var side = (this.kin_graph!.nodes[next_node_id].sex as KinSex) == KinSex.Male ? -1 : 1
                var g_pos = this.pixi_nodes.get(current_node)!.getGlobalPosition()
                new_pos.x = g_pos.x + side * 10 * SCALE
                new_pos.y = g_pos.y - 10 * SCALE

            } else if (cur_relation.kind == RelationKind.Sibling || cur_relation.kind == RelationKind.RP) {
                //just to the side
                var g_pos = this.pixi_nodes.get(current_node)!.getGlobalPosition()
                var side = (this.kin_graph!.nodes[next_node_id].sex as KinSex) == KinSex.Male ? -1 : 1
                new_pos.x = g_pos.x + side * 10 * SCALE
                new_pos.y = g_pos.y
            }
            //render the node
            var next_pixi_node = this.render_node(next_node_id, new_pos)
            this.app.stage.addChild(next_pixi_node!)
            console.log("adding node " + next_node_id + " to pixi nodes")
            this.pixi_nodes.set(next_node_id, next_pixi_node!)
            //render the edge
            var edge = this.render_edge(current_node, next_node_id, cur_relation)
            this.app.stage.addChild(edge!)
            //add to visited
            visited.add(next_node_id)
            //add to stack
            stack.push(next_node_id)
        }

    }
    render_edge(node1: number, node2: number, relation: Relation) {
        var p1 = this.kin_graph.nodes![node1]
        var p2 = this.kin_graph.nodes![node2]
        var pxnode1 = this.pixi_nodes.get(node1)!
        var pxnode2 = this.pixi_nodes.get(node2)!
        switch (relation.kind as RelationKind) {
            case RelationKind.Parent:
                console.log("rendering parent edge")
                //draw a line up, and then to the side (depending on node gender)
                var line = new PIXI.Graphics()
                line.lineStyle(2, 0x00FF00)
                //center on pxnode1
                line.moveTo(pxnode1.x, pxnode1.y)
                var point0 = new PIXI.Point(pxnode1.x, pxnode1.y)
                var point1 = new PIXI.Point(pxnode1.x, pxnode1.y - 10 * SCALE)
                var side = (p2.sex as KinSex) == KinSex.Male ? -1 : 1
                //move to the side
                var point2 = new PIXI.Point(pxnode1.x + side * 10 * SCALE, pxnode1.y - 10 * SCALE)
                var polygon = new PIXI.Polygon([point0, point1, point2])
                polygon.closeStroke = false
                line.drawPolygon(polygon)
                return line
            case RelationKind.Child:
                console.log("rendering child edge")
                //same, but down
                var line = new PIXI.Graphics()
                line.lineStyle(2, 0x00FF00)
                //center on pxnode1
                line.moveTo(pxnode1.x, pxnode1.y)
                var point0 = new PIXI.Point(pxnode1.x, pxnode1.y)
                var point1 = new PIXI.Point(pxnode1.x, pxnode1.y + 10 * SCALE)
                var side = (p2.sex as KinSex) == KinSex.Male ? -1 : 1
                //move to the side
                var point2 = new PIXI.Point(pxnode1.x + side * 10 * SCALE, pxnode1.y + 10 * SCALE)
                var polygon = new PIXI.Polygon([point0, point1, point2])
                return line
            default:
                console.log("rendering sibling or rp edge")
                //just a line
                var line = new PIXI.Graphics()
                line.lineStyle(2, 0x000FF)
                line.moveTo(pxnode1.x, pxnode1.y)
                line.lineTo(pxnode2.x, pxnode2.y)
                return line
        }



    }

    render_node(node_id: number, center: { x: number, y: number }) {
        console.log("rendering node " + node_id + " at " + center.x + "," + center.y)
        if (this.kin_graph.nodes == undefined) {
            return
        }
        var node = this.kin_graph.nodes[node_id]
        console.log(node)
        //render circle with name
        var circle = new PIXI.Graphics()
        circle.moveTo(center.x, center.y)
        circle.beginFill(0x000000)
        circle.drawCircle(0, 0, 10)
        circle.endFill()
        var text = new PIXI.Text("NAME")
        //set text size to fit in circle
        text.style.fontSize = 10
        text.style.fill = 0xffffff
        var container = new PIXI.Container()
        container.addChild(circle)
        container.addChild(text)
        return container

    }


}