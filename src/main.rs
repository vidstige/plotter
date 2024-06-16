use eframe::{App, CreationContext};
use egui::{Color32, Ui, ImageData, ColorImage, TextureOptions, TextureId, Rect, pos2, Sense, Vec2};
use egui_snarl::{
    ui::{PinInfo, SnarlStyle, SnarlViewer},
    InPin, InPinId, NodeId, OutPin, Snarl,
};
use rand::{distributions::Distribution, rngs::ThreadRng};
use rand_distr::Normal;
use tiny_skia::Pixmap;

const NUMBER_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0x00);
const UNTYPED_COLOR: Color32 = Color32::from_rgb(0xb0, 0xb0, 0xb0);

enum Node {
    // Distribution node
    NormalDistribution(f32, f32),

    /// Value node with a single output.
    /// The value is editable in UI.
    Number(f32),

    Pixmap(Pixmap),
}

impl Node {
    fn number_in(&mut self, idx: usize) -> &mut f32 {
        match self {
            Node::NormalDistribution(my, sigma) => match idx {
                0 => my,
                1 => sigma,
                _ => unreachable!(),
            }
            _ => unreachable!(),
        }
    }
}

struct NodeViewer;

impl SnarlViewer<Node> for NodeViewer {
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<Node>) {
        // TODO: Check input/output types
        for &remote in &to.remotes {
            snarl.disconnect(remote, to.id);
        }

        snarl.connect(from.id, to.id);
    }

    fn title(&mut self, node: &Node) -> String {
        match node {
            Node::NormalDistribution(_, _) => "Normal Distribution",
            Node::Number(_) => "Number",
            Node::Pixmap(_) => "Pixmap",
        }.to_owned()
    }

    fn inputs(&mut self, node: &Node) -> usize {
        match node {
            Node::NormalDistribution(_, _) => 2,
            Node::Number(_) => 0,
            Node::Pixmap(_) => 1,
        }
    }

    fn outputs(&mut self, node: &Node) -> usize {
        match node {
            Node::NormalDistribution(_, _) => 1,
            Node::Number(_) => 1,
            Node::Pixmap(_) => 0,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) -> PinInfo {
        match snarl[pin.id.node] {
            Node::NormalDistribution(_, _) => {
                if pin.remotes.len() == 0 {
                    let node = &mut snarl[pin.id.node];
                    ui.add(egui::DragValue::new(node.number_in(pin.id.input)));
                }
                PinInfo::square().with_fill(NUMBER_COLOR)
            },
            Node::Number(_) => {
                unreachable!("Number node has no inputs")
            }
            Node::Pixmap(_) => {
                PinInfo::square().with_fill(NUMBER_COLOR)
            }
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) -> PinInfo {
        match snarl[pin.id.node] {
            Node::NormalDistribution(_, _) => {
                // TODO: Use another color
                PinInfo::square().with_fill(NUMBER_COLOR)
            }
            Node::Number(ref mut value) => {
                assert_eq!(pin.id.output, 0, "Number node has only one output");
                ui.add(egui::DragValue::new(value));
                PinInfo::square().with_fill(NUMBER_COLOR)
            }
            Node::Pixmap(_) => {
                PinInfo::square().with_fill(NUMBER_COLOR)
            }
        }
    }

    fn input_color(
        &mut self,
        pin: &InPin,
        _style: &egui::Style,
        snarl: &mut Snarl<Node>,
    ) -> Color32 {
        match snarl[pin.id.node] {
            Node::NormalDistribution(_, _) => {
                NUMBER_COLOR
            }
            Node::Number(_) => {
                unreachable!("Number node has no inputs")
            }
            Node::Pixmap(_) => {
                NUMBER_COLOR
            }
        }
    }

    fn output_color(
        &mut self,
        _pin: &OutPin,
        _style: &egui::Style,
        _snarl: &mut Snarl<Node>,
    ) -> Color32 {
        NUMBER_COLOR
    }

    fn has_body(&mut self, node: &Node) -> bool {
        match node {
            Node::Pixmap(_) => true,
            _ => false,
        }
    }

    fn show_body(
        &mut self,
        node_id: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        match snarl[node_id] {
            Node::Pixmap(ref pixmap) => {
                let image_data = ColorImage::from_rgba_premultiplied([pixmap.width() as usize, pixmap.height() as usize], pixmap.data());
                // TODO: Only upload here and allocate texture once
                let texture = ui.ctx().load_texture("pixmap", image_data, TextureOptions::LINEAR);
                let texture_id = TextureId::from(&texture);
                let uv = Rect{ min:pos2(0.0, 0.0), max:pos2(1.0, 1.0)};
                let (rect, _) = ui.allocate_exact_size(Vec2::new(pixmap.width() as f32, pixmap.height() as f32), Sense::hover());
                ui.painter().image(texture_id, rect, uv, Color32::WHITE);
            },
            _ => {},
        }
    }

    fn graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        ui.label("Add node");
        if ui.button("Normal distribution").clicked() {
            snarl.insert_node(pos, Node::NormalDistribution(0.0, 1.0));
            ui.close_menu();
        }
        if ui.button("Number").clicked() {
            snarl.insert_node(pos, Node::Number(0.0));
            ui.close_menu();
        }
        if ui.button("Pixmap").clicked() {
            snarl.insert_node(pos, Node::Pixmap(Pixmap::new(320, 200).unwrap()));
            ui.close_menu();
        }
    }

    fn node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
        }
    }

    fn has_on_hover_popup(&mut self, _: &Node) -> bool {
        true
    }

    fn show_on_hover_popup(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        match snarl[node] {
            Node::NormalDistribution(_, _) => {
                ui.label("Normal distribution");
            }
            Node::Number(_) => {
                ui.label("Outputs integer value");
            }
            _ => {}
        }
    }
}

pub struct PlotterApp {
    snarl: Snarl<Node>,
    style: SnarlStyle,
}

impl PlotterApp {
    pub fn new(_ctx: &CreationContext) -> Self {
        let snarl = Snarl::new();
        let style = SnarlStyle::new();
        PlotterApp { snarl, style }
    }
}

impl App for PlotterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //egui_extras::install_image_loaders(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close)
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_switch(ui);
            });
        });

        /*egui::SidePanel::left("style").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui_probe::Probe::new("Snarl style", &mut self.style).show(ui);
            });
        });*/

        egui::CentralPanel::default().show(ctx, |ui| {
            self.snarl
                .show(&mut NodeViewer, &self.style, egui::Id::new("snarl"), ui);
        });
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([320.0, 200.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-snarl demo",
        native_options,
        Box::new(|cx| Box::new(PlotterApp::new(cx))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "egui_snarl_demo",
                web_options,
                Box::new(|cx| Box::new(PlotterApp::new(cx))),
            )
            .await
            .expect("failed to start eframe");
    });
}
