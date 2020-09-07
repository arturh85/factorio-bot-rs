<template>
  <div id="entityGraph" style="height: 700px"></div>
</template>

<script lang="ts">
// @ is an alias to /src
import Vue from "vue";
import {FactorioApi} from "@/factorio-bot/restApi";

export default Vue.extend({
  name: "EntityGraph",
  mounted() {
    FactorioApi.entity().then(plan => {
      const d3 = (window as any).d3;
      (window as any).colors = d3.schemeCategory20;
      d3.select("#entityGraph").graphviz({fit: true, width: '100vw', height: '90vh'}).renderDot(plan)
          // .transition(function () {
          //   return d3.transition("main")
          //       .ease(d3.easeLinear)
          //       .delay(500)
          //       .duration(1500);
          // })
          .logEvents(true)
          .on("initEnd", () => {
            console.log("init end");

          });
    })
    //   console.log('PLAN', plan)
    //   // const graph = graphlib.read(plan);
    //   console.log('GRAPH', graph)
    //   console.log('GRAPH edges', graph.edges())
    //   console.log('GRAPH nodes', graph.nodes())
    //   // const elements : ElementDefinition[] = graph.edges().map((edge, index) => ({group: 'edges', data: {source: edge.v, target: edge.w}} as ElementDefinition)).concat(
    //   //     graph.nodes().map(node => ({group: 'nodes', data: {id: node}} as ElementDefinition)));
    //   console.log('ELEMENTS', elements)
    //   // const container = document.getElementById('graph');
    //   console.log('CONTAINER', container)
    //   const cy = cytoscape({
    //     container,
    //     elements,
    //     style: [ // the stylesheet for the graph
    //       {
    //         selector: 'node',
    //         style: {
    //           'background-color': '#666',
    //           'label': 'data(id)'
    //         }
    //       },
    //
    //       {
    //         selector: 'edge',
    //         style: {
    //           'width': 3,
    //           'line-color': '#ccc',
    //           'target-arrow-color': '#ccc',
    //           'target-arrow-shape': 'triangle',
    //           'curve-style': 'bezier'
    //         }
    //       }
    //     ],
    //
    //     layout: {
    //       name: 'grid',
    //       rows: 1
    //     }
    //   });
    //   // dagre.layout(graph);
    //   // this.$data.plan = plan
    //   this.$data.cy = cy
    // })
  },
  components: {
  },
});
</script>
