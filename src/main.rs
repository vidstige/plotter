use eframe::{App, CreationContext};
use egui::{Color32, Ui, ImageData, ColorImage, TextureOptions, TextureId, Rect, pos2, Sense, Vec2, Response};
use egui_snarl::{
    ui::{PinInfo, SnarlStyle, SnarlViewer},
    InPin, InPinId, NodeId, OutPin, Snarl,
};
use rand::{distributions::Distribution, rngs::ThreadRng};
use rand_distr::Normal;
use tiny_skia::Pixmap;

const NUMBER_COLOR: Color32 = Color32::from_rgb(0xb0, 0x00, 0x00);
const UNTYPED_COLOR: Color32 = Color32::from_rgb(0xb0, 0xb0, 0xb0);


#[derive(Clone)]
enum Kind {
    F32,
    USize,
    Points2,
    Pixmap,
}

trait Node {
    fn title(&self) -> &str;
    fn inputs(&self) -> &[Kind];
    fn outputs(&self) -> &[Kind];
    fn input_ui(&mut self, ui: &mut Ui, pin: &InPin);
    fn show_body(&mut self, ui: &mut Ui);
}

// --- Constant ----------------------
struct ConstantNode {
    value: f32,    
}
impl ConstantNode {
    fn new(value: f32) -> ConstantNode {
        ConstantNode { value }
    }
}
impl Node for ConstantNode {
    fn title(&self) -> &str { "Number" }
    fn inputs(&self) -> &[Kind] { &[] }
    fn outputs(&self) -> &[Kind] { &[Kind::F32] }
    fn input_ui(&mut self, ui: &mut Ui, pin: &InPin) { }
    fn show_body(&mut self, ui: &mut Ui) {
        ui.add(egui::DragValue::new(&mut self.value));
    }
}

// --- NormalDistribution ----------------------
struct NormalDistributionNode {
    my: f32,
    sigma: f32,
}
impl NormalDistributionNode {
    fn new(my: f32, sigma: f32) -> NormalDistributionNode {
        NormalDistributionNode { my, sigma }
    }
}
impl Node for NormalDistributionNode {
    fn title(&self) -> &str { "Normal distribution" }
    fn inputs(&self) -> &[Kind] { &[Kind::F32, Kind::F32] }
    fn outputs(&self) -> &[Kind] { &[Kind::F32] }
    fn input_ui(&mut self, ui: &mut Ui, pin: &InPin) {
        if pin.remotes.len() > 0 {
            return;
        }
        match pin.id.input {
            0 => ui.add(egui::DragValue::new(&mut self.my)),
            1 => ui.add(egui::DragValue::new(&mut self.sigma)),
            _ => unreachable!("NormalDistributionNode only have two inputs"),
        };
    }
    fn show_body(&mut self, _ui: &mut Ui) { }
}

// --- Pixmap ----------------------
struct PixmapNode {
    pixmap: Pixmap,
}
impl PixmapNode {
    fn new(width: u32, height: u32) -> PixmapNode {
        PixmapNode { pixmap: Pixmap::new(width, height).unwrap() }
    }
}
impl Node for PixmapNode {
    fn title(&self) -> &str { "Pixmap" }
    fn inputs(&self) -> &[Kind] { &[Kind::Pixmap] }
    fn outputs(&self) -> &[Kind] { &[] }
    fn input_ui(&mut self, ui: &mut Ui, pin: &InPin) { }
    fn show_body(&mut self, ui: &mut Ui) {
        let pixmap = &self.pixmap;
        let image_data = ColorImage::from_rgba_premultiplied([pixmap.width() as usize, pixmap.height() as usize], pixmap.data());
        // TODO: Only upload here and allocate texture once
        let texture = ui.ctx().load_texture("pixmap", image_data, TextureOptions::LINEAR);
        let texture_id = TextureId::from(&texture);
        let uv = Rect{ min:pos2(0.0, 0.0), max:pos2(1.0, 1.0)};
        let (rect, _) = ui.allocate_exact_size(Vec2::new(pixmap.width() as f32, pixmap.height() as f32), Sense::hover());
        ui.painter().image(texture_id, rect, uv, Color32::WHITE);
    }
}

// --- Sample2 ----------------------
struct Sample2Node {
    count: usize,
}
impl Sample2Node {
    fn new(count: usize) -> Sample2Node {
        Sample2Node { count }
    }
}
impl Node for Sample2Node {
    fn title(&self) -> &str { "Sample" }
    fn inputs(&self) -> &[Kind] { &[Kind::USize, Kind::F32, Kind::F32] }
    fn outputs(&self) -> &[Kind] { &[Kind::Points2] }
    fn input_ui(&mut self, ui: &mut Ui, pin: &InPin) {
        if pin.remotes.len() > 0 {
            return;
        }
        if pin.id.input == 0 {
            ui.add(egui::DragValue::new(&mut self.count));
        }
    }
    fn show_body(&mut self, ui: &mut Ui) { }
}

struct NodeViewer;

impl SnarlViewer<Box<dyn Node>> for NodeViewer {
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<Box<dyn Node>>) {
        //
        
        for &remote in &to.remotes {
            snarl.disconnect(remote, to.id);
        }

        snarl.connect(from.id, to.id);
    }

    fn title(&mut self, node: &Box<dyn Node>) -> String {
        node.title().to_owned()
    }

    fn inputs(&mut self, node: &Box<dyn Node>) -> usize {
        node.inputs().len()
    }

    fn outputs(&mut self, node: &Box<dyn Node>) -> usize {
        node.outputs().len()
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Box<dyn Node>>,
    ) -> PinInfo {
        snarl[pin.id.node].input_ui(ui, pin);
        PinInfo::circle().with_fill(NUMBER_COLOR)
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Box<dyn Node>>,
    ) -> PinInfo {
        PinInfo::square().with_fill(NUMBER_COLOR)
    }

    fn input_color(
        &mut self,
        pin: &InPin,
        _style: &egui::Style,
        snarl: &mut Snarl<Box<dyn Node>>,
    ) -> Color32 {
        NUMBER_COLOR
    }

    fn output_color(
        &mut self,
        _pin: &OutPin,
        _style: &egui::Style,
        _snarl: &mut Snarl<Box<dyn Node>>,
    ) -> Color32 {
        NUMBER_COLOR
    }

    fn has_body(&mut self, node: &Box<dyn Node>) -> bool {
        // TODO: Only if has body?
        true
    }

    fn show_body(
        &mut self,
        node_id: NodeId,
        inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Box<dyn Node>>,
    ) {
        for input in inputs {
            //let kind = &snarl[input.id.node].inputs()[input.id.input];
            //snarl[node_id].input_value(input.id.input)
            //node.receive();
        }
        snarl[node_id].show_body(ui);
    }

    fn graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<Box<dyn Node>>,
    ) {
        ui.label("Add node");
        if ui.button("Normal distribution").clicked() {
            snarl.insert_node(pos, Box::new(NormalDistributionNode::new(0.0, 1.0)));
            ui.close_menu();
        }
        if ui.button("Number").clicked() {
            snarl.insert_node(pos, Box::new(ConstantNode::new(0.0)));
            ui.close_menu();
        }
        if ui.button("Sample").clicked() {
            snarl.insert_node(pos, Box::new(Sample2Node::new(256)));
            ui.close_menu();
        }
        if ui.button("Pixmap").clicked() {
            snarl.insert_node(pos, Box::new(PixmapNode::new(320, 200)));
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
        snarl: &mut Snarl<Box<dyn Node>>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
        }
    }
}

pub struct PlotterApp {
    snarl: Snarl<Box<dyn Node>>,
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
