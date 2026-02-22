use std::collections::HashMap;

use eframe::{
    App, CreationContext, NativeOptions,
    egui::{CentralPanel, Color32, Context, DragValue, Pos2, RichText, TopBottomPanel, pos2},
    run_native,
};
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, LayoutRandom, LayoutStateRandom,
};
use gridgraph_rs::{DimacsInstance, GridGraphError, GridGraphParams, generate_instance};
use petgraph::{
    Directed,
    stable_graph::{DefaultIx, NodeIndex, StableGraph},
};

const DEFAULT_HEIGHT: i32 = 12;
const DEFAULT_WIDTH: i32 = 18;
const DEFAULT_MAX_CAPACITY: i32 = 500;
const DEFAULT_MAX_COST: i32 = 100;
const DEFAULT_SEED: i32 = 123_456;
const GRID_SPACING: f32 = 70.0;
const SOURCE_SINK_OFFSET: f32 = 2.4;
const GRID_NODE_COLOR: Color32 = Color32::from_rgb(59, 130, 246);
const SOURCE_COLOR: Color32 = Color32::from_rgb(74, 222, 128);
const SINK_COLOR: Color32 = Color32::from_rgb(248, 113, 113);

type GridGraph = Graph<NodePayload, EdgePayload>;
type GridGraphView<'a> = GraphView<
    'a,
    NodePayload,
    EdgePayload,
    Directed,
    DefaultIx,
    DefaultNodeShape,
    DefaultEdgeShape,
    LayoutStateRandom,
    LayoutRandom,
>;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions::default();
    run_native(
        "GRIDGRAPH Viewer",
        native_options,
        Box::new(|cc| Ok(Box::new(GridGraphApp::new(cc)))),
    )
}

struct GridGraphApp {
    graph: GridGraph,
    controls: GridControls,
    active_params: GridGraphParams,
    stats: GraphStats,
    last_error: Option<String>,
}

impl GridGraphApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        let params = default_params();
        let instance = generate_instance(params);
        let graph = build_graph(&instance);
        let controls = GridControls::from(params);
        let stats = GraphStats::from(&instance);
        Self {
            graph,
            controls,
            active_params: params,
            stats,
            last_error: None,
        }
    }

    fn regenerate(&mut self) {
        match self.controls.to_params() {
            Ok(params) => {
                let instance = generate_instance(params);
                self.graph = build_graph(&instance);
                self.stats = GraphStats::from(&instance);
                self.active_params = params;
                self.last_error = None;
            }
            Err(err) => self.last_error = Some(err.to_string()),
        }
    }
}

impl App for GridGraphApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut should_regenerate = false;

        TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.heading("GRIDGRAPH interactive viewer");
            ui.label(RichText::new("Drag nodes, pan with right-click, scroll to zoom.").italics());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Height");
                should_regenerate |= ui
                    .add(DragValue::new(&mut self.controls.height).range(1..=250))
                    .changed();
                ui.label("Width");
                should_regenerate |= ui
                    .add(DragValue::new(&mut self.controls.width).range(2..=250))
                    .changed();
                ui.label("Max cap");
                should_regenerate |= ui
                    .add(DragValue::new(&mut self.controls.max_capacity).range(1..=10_000))
                    .changed();
                ui.label("Max cost");
                should_regenerate |= ui
                    .add(DragValue::new(&mut self.controls.max_cost).range(1..=10_000))
                    .changed();
                ui.label("Seed");
                should_regenerate |= ui
                    .add(DragValue::new(&mut self.controls.seed).range(1..=1_000_000))
                    .changed();
            });

            ui.horizontal(|ui| {
                if ui.button("Regenerate graph").clicked() {
                    should_regenerate = true;
                }
                ui.label(format!(
                    "Active grid: {}x{} | nodes {} | arcs {} | max flow {}",
                    self.active_params.height,
                    self.active_params.width,
                    self.stats.node_count,
                    self.stats.arc_count,
                    self.stats.max_flow
                ));
            });

            if let Some(err) = &self.last_error {
                ui.colored_label(Color32::LIGHT_RED, err);
            }
        });

        if should_regenerate {
            self.regenerate();
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut GridGraphView::new(&mut self.graph));
        });
    }
}

#[derive(Clone)]
struct NodePayload {
    id: i32,
    kind: NodeKind,
}

impl NodePayload {
    fn label_text(&self) -> String {
        match &self.kind {
            NodeKind::Grid { row, col } => format!("({row},{col}):{}", self.id),
            NodeKind::Source => format!("source:{}", self.id),
            NodeKind::Sink => format!("sink:{}", self.id),
        }
    }
}

#[derive(Clone)]
enum NodeKind {
    Grid { row: i32, col: i32 },
    Source,
    Sink,
}

#[derive(Clone)]
struct EdgePayload {
    capacity: i32,
    cost: i32,
}

impl EdgePayload {
    fn label_text(&self) -> String {
        format!("cap {} | cost {}", self.capacity, self.cost)
    }
}

#[derive(Clone, Copy)]
struct GridControls {
    height: i32,
    width: i32,
    max_capacity: i32,
    max_cost: i32,
    seed: i32,
}

impl From<GridGraphParams> for GridControls {
    fn from(params: GridGraphParams) -> Self {
        Self {
            height: params.height,
            width: params.width,
            max_capacity: params.max_capacity,
            max_cost: params.max_cost,
            seed: params.seed,
        }
    }
}

impl GridControls {
    fn to_params(&self) -> Result<GridGraphParams, GridGraphError> {
        GridGraphParams::new(
            self.height,
            self.width,
            self.max_capacity,
            self.max_cost,
            self.seed,
        )
    }
}

#[derive(Clone, Copy)]
struct GraphStats {
    node_count: i32,
    arc_count: i32,
    max_flow: i64,
}

impl From<&DimacsInstance> for GraphStats {
    fn from(instance: &DimacsInstance) -> Self {
        Self {
            node_count: instance.node_count(),
            arc_count: instance.arc_count(),
            max_flow: instance.max_flow(),
        }
    }
}

fn default_params() -> GridGraphParams {
    GridGraphParams::new(
        DEFAULT_HEIGHT,
        DEFAULT_WIDTH,
        DEFAULT_MAX_CAPACITY,
        DEFAULT_MAX_COST,
        DEFAULT_SEED,
    )
    .expect("defaults are valid")
}

fn build_graph(instance: &DimacsInstance) -> GridGraph {
    let base: StableGraph<NodePayload, EdgePayload> = StableGraph::new();
    let mut graph = Graph::from(&base);
    let mut nodes = HashMap::new();
    let params = instance.params();

    let half_width = (params.width as f32 - 1.0) * GRID_SPACING * 0.5;
    let half_height = (params.height as f32 - 1.0) * GRID_SPACING * 0.5;

    for row in 1..=params.height {
        for col in 1..=params.width {
            let id = grid_node_index(row, col, params.width);
            let x = (col as f32 - 1.0) * GRID_SPACING - half_width;
            let y = half_height - (row as f32 - 1.0) * GRID_SPACING;
            let payload = NodePayload {
                id,
                kind: NodeKind::Grid { row, col },
            };
            insert_node(
                &mut graph,
                &mut nodes,
                id,
                payload.clone(),
                payload.label_text(),
                pos2(x, y),
                GRID_NODE_COLOR,
            );
        }
    }

    let center_y = 0.0;
    let source_id = instance.source();
    let source_payload = NodePayload {
        id: source_id,
        kind: NodeKind::Source,
    };
    insert_node(
        &mut graph,
        &mut nodes,
        source_id,
        source_payload.clone(),
        source_payload.label_text(),
        pos2(-half_width - GRID_SPACING * SOURCE_SINK_OFFSET, center_y),
        SOURCE_COLOR,
    );

    let sink_id = instance.sink();
    let sink_payload = NodePayload {
        id: sink_id,
        kind: NodeKind::Sink,
    };
    insert_node(
        &mut graph,
        &mut nodes,
        sink_id,
        sink_payload.clone(),
        sink_payload.label_text(),
        pos2(half_width + GRID_SPACING * SOURCE_SINK_OFFSET, center_y),
        SINK_COLOR,
    );

    for arc in instance.arcs() {
        if let (Some(&from), Some(&to)) = (nodes.get(&arc.from), nodes.get(&arc.to)) {
            let payload = EdgePayload {
                capacity: arc.capacity,
                cost: arc.cost,
            };
            let edge_idx =
                graph.add_edge_with_label(from, to, payload.clone(), payload.label_text());
            if let Some(edge) = graph.g_mut().edge_weight_mut(edge_idx) {
                edge.display_mut().width = edge_penwidth(arc.capacity, params.max_capacity);
            }
        }
    }

    graph
}

fn insert_node(
    graph: &mut GridGraph,
    nodes: &mut HashMap<i32, NodeIndex>,
    id: i32,
    payload: NodePayload,
    label: String,
    location: Pos2,
    color: Color32,
) {
    let idx = graph.add_node_with_label_and_location(payload, label, location);
    if let Some(node) = graph.g_mut().node_weight_mut(idx) {
        node.set_color(color);
    }
    nodes.insert(id, idx);
}

fn edge_penwidth(capacity: i32, max_capacity: i32) -> f32 {
    let ratio = capacity as f32 / max_capacity as f32;
    1.0 + ratio.clamp(0.0, 1.0) * 4.0
}

fn grid_node_index(row: i32, col: i32, width: i32) -> i32 {
    (row - 1) * width + col
}
