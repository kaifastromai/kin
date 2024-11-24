import * as PIXI from "pixi.js"
import { KinGraph, KinSex, KinWasmPersonNode, Relation, RelationKind } from "./kin";
import { Sex } from "../kin-wasm/kin";
const COLORS = {
    "darkBlue": 0x3E40E6,
    "purple": 0xAA3EE6,
    "darkPurple": 0x733EE6,
    "lightPurple": 0xAA3EE6,
    "lightBlue": 0x3E75E6,
    "brightPink": 0xE14EE6,
    "depressedPurple": 0xC0ADEA

}
const SCALE = 5;
export class KinGraphView {
    app: PIXI.Application;
    kin_graph: KinGraph;
    x_delta: number = 5
    y_delta: number = 5
    pixi_nodes: Map<number, PIXI.Container> = new Map<number, PIXI.Container>()
    is_grid_drag: boolean = false
    drag_point: [number, number] = [0, 0]
    grid_click: [number, number] = [0, 0]
    constructor(app: PIXI.Application, kin_graph: KinGraph) {
        this.app = app
        this.kin_graph = kin_graph
        this.app.stage.hitArea = app.screen

    }
    //render a background grid
    render_grid(spacing: number) {
        var aspectR = this.app.screen.width / this.app.screen.height
        var half_lengths = { x: this.app.screen.width / 2, y: this.app.screen.height / 2 }
        var x_delta = this.app.screen.width / spacing
        var y_delta = this.app.screen.height / spacing
        x_delta /= aspectR
        this.x_delta = x_delta
        this.y_delta = y_delta
        var extra_x = (this.app.screen.width - x_delta * spacing) / x_delta
        spacing += Math.max(0, Math.floor(extra_x))
        var grid = new PIXI.Graphics()

        //get current cursor position
        for (var i = 0; i < spacing; i++) {
            var x = i * x_delta
            grid.lineStyle(1, 0x000000, 0.15)
            grid.moveTo(x, 0)
            grid.lineTo(x, this.app.screen.height)
        }
        for (var i = 0; i < spacing; i++) {
            var y = i * y_delta
            grid.lineStyle(1, 0x000000, 0.15)
            grid.moveTo(0, y)
            grid.lineTo(this.app.screen.width, y)
        }
        //drag the grid when mouse tragged
        grid.eventMode = 'static'
        grid.hitArea = new PIXI.Rectangle(0, 0, this.app.screen.width, this.app.screen.height)
        grid.pivot.set(half_lengths.x, half_lengths.y)
        grid.position.set(half_lengths.x, half_lengths.y)
        grid.on('pointerdown', (e) => {
            this.is_grid_drag = true
            this.drag_point = [e.global.x, e.global.y]
            console.log("Clicked " + this.drag_point)
            this.grid_click = [grid.position.x, grid.position.y]
        })
        grid.on('pointerup', (e) => {
            this.is_grid_drag = false
            console.log("Released")
        })
        grid.on('pointerupoutside', (e) => {
            this.is_grid_drag = false
            console.log("Released outside")
        }
        )
        grid.on('pointermove', (e) => {
            if (this.is_grid_drag) {
                var pos = e.global
                console.log("dragging " + pos)
                var delta = [pos.x - this.drag_point[0], pos.y - this.drag_point[1]]
                grid.position.set(this.grid_click[0] + delta[0], this.grid_click[1] + delta[1])
                console.log("delta is " + delta)

            }

        })
        this.app.stage.addChild(grid)
        const co = drawNode({ id: 0, name: "John", sex: "M", relations: [] })
        co.position.set(100, 100)
        const co2 = drawNode({
            id: 1, name: "Izy", sex: "F", relations: []
        })
        co2.position.set(200, 200)
        this.app.stage.addChild(co2)
        this.app.stage.addChild(co)
        //draw a link between the two nodes
        var link = new VerticalNodeLink([150, 150], [200, 200])
        link.initDraw()
        this.app.stage.addChild(link.graphics)
    }

}
class KinGraphNode {
    payload: KinWasmPersonNode
    pixi_node: PIXI.Container
    constructor(payload: KinWasmPersonNode) {
        this.payload = payload
        this.pixi_node = new PIXI.Container()
    }

}
//Draws a kin node, which is a rounded rectangle with a name, and 4 points from which to draw relations
function drawNode(node: KinWasmPersonNode) {
    var node_container = new PIXI.Container()
    var node_graphics = new PIXI.Graphics()
    node_graphics.beginFill(0x000000)
    node_graphics.drawRoundedRect(0, 0, 100, 50, 10)
    node_graphics.endFill()
    var name = new PIXI.Text(node.name, { fontSize: 12, fill: 0xFFFFFF })
    name.position.set(10, 10)
    node_container.addChild(node_graphics)
    node_container.addChild(name)
    //draw the relation points
    var relation_points = new PIXI.Container()
    var point_radius = 5
    var top_mid = drawRelationPoint(point_radius, RelationKind.Child)
    top_mid.position.set(50, 0)
    var bottom_mid = drawRelationPoint(point_radius, RelationKind.Parent)
    bottom_mid.position.set(50, 50)
    var left_mid = drawRelationPoint(point_radius, RelationKind.RP)
    left_mid.position.set(0, 25)
    var right_mid = drawRelationPoint(point_radius, RelationKind.Sibling)
    right_mid.position.set(100, 25)
    relation_points.addChild(top_mid)
    relation_points.addChild(bottom_mid)
    relation_points.addChild(left_mid)
    relation_points.addChild(right_mid)
    relation_points.position.set(0, 0)
    node_container.addChild(relation_points)
    return node_container
}
//draw the relation points.
//each point is a colored circle, with a thick purple outline 
const SIBlING_REL_POINT_COLOR = "#585B75"
const RP_REL_POINT_COLOR = "#3E53F5"
const CHILD_REL_POINT_COLOR = "#F5B53D"
const PARENT_REL_POINT_COLOR = "#756B58"
function drawRelationPoint(radius: number, kind: RelationKind): PIXI.Graphics {
    var point = new PIXI.Graphics()
    var outline = new PIXI.Graphics()
    var color = SIBlING_REL_POINT_COLOR;
    switch (kind) {
        case RelationKind.Child:
            color = CHILD_REL_POINT_COLOR;
        case RelationKind.Parent:
            color = PARENT_REL_POINT_COLOR;
        case RelationKind.RP:
            color = RP_REL_POINT_COLOR;
        case RelationKind.Sibling:
            ;

    }
    point.beginFill(color)
    point.drawCircle(0, 0, radius)
    point.endFill()

    outline.lineStyle(2, new PIXI.Color("0xFFFFFF"))
    outline.drawCircle(0, 0, radius)

    //on mouse hover, increase outline thickness
    outline.eventMode = 'static'
    outline.hitArea = new PIXI.Circle(0, 0, radius)
    outline.on('pointerover', () => {
        //change the outline thickness
        outline.clear()
        outline.lineStyle(4, new PIXI.Color("0xFFFFFF"))
        outline.drawCircle(0, 0, radius)
    })
    outline.on('pointerout', () => {
        outline.clear()
        outline.lineStyle(2, "0xFFFFFF")
        outline.drawCircle(0, 0, radius)
    })
    point.addChild(outline)
    return point
}
//A bezier curve that connects two nodes
class VerticalNodeLink {
    start: [number, number]
    end: [number, number]

    graphics: PIXI.Graphics
    constructor(start: [number, number], end: [number, number]) {
        this.start = start
        this.end = end

        this.graphics = new PIXI.Graphics()
    }
    changeEnd(end: [number, number]) {
        this.end = end
        this.updateDraw()
    }
    changeStart(start: [number, number]) {
        this.start = start
        this.updateDraw()
    }
    initDraw() {
        //both control points y axis is the midpoint between the start and end y axis
        var midY = (this.end[1] - this.start[1]) / 2
        const controlStart = [this.start[0], this.start[1] + midY]
        const controlEnd = [this.end[0], this.end[1] - midY]
        this.graphics = drawCubicBezier(this.start, this.end, controlStart, controlEnd)

    }
    updateDraw() {
        this.graphics.clear()
        this.initDraw()
    }

}
function drawCubicBezier(start: [number, number], end: [number, number], controlStart: [number, number], controlEnd: [number, number]) {
    var graphics = new PIXI.Graphics()
    graphics.lineStyle(4, COLORS.darkPurple)
    graphics.moveTo(start[0], start[1])
    graphics.bezierCurveTo(controlStart[0], controlStart[1], controlEnd[0], controlEnd[1], end[0], end[1])
    return graphics
}