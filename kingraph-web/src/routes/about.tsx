import { A } from "solid-start";
import Counter from "~/components/Counter";
import init, { get_graph, append_from_dsl, add_relation, } from "../../kin-wasm/kin"
import { KinGraph } from "~/kin";
import { createResource, onMount } from "solid-js";
import * as PIXI from 'pixi.js';
import { KinGraphView } from "~/render_kingraph";
export default function About() {

  onMount(async () => {
    console.log("Starting wasm")
    var kg = await KinGraph.init_kin();
    console.log("Kg loaded")
    var kin_dsl = String.raw`
  Izy M PARENT John M
  Gabe M PARENT John M
  JILL F CHILD John M
  CHILL M CHILD John M
  `;
    kg.load_from_dsl(kin_dsl);
    console.log(kg.nodes);
    var canvas_el = document.getElementById("pixi-canvas")!;

    var app = new PIXI.Application({
      background: '#2E2E2E',
      resolution: window.devicePixelRatio,
      autoDensity: true,
      antialias: true,
      width: 1920,
      height: 1080
    });

    console.log("App created")
    canvas_el.appendChild(app.view);
    var kingraphView = new KinGraphView(app, kg);
    kingraphView.render_grid(20, 20)
    kingraphView.render_graph(0)

  })
  return (
    <main >
      <div id="pixi-canvas">
      </div>
    </main>
  );
}
