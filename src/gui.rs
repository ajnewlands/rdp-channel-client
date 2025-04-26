use std::mem;

use eframe::egui::{
    self, load::SizedTexture, Color32, ColorImage, Image, TextureHandle, TextureOptions, Vec2,
};

pub struct App {
    pub texture_handle: TextureHandle,
    pub rx: tokio::sync::watch::Receiver<Vec<u8>>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        rx: tokio::sync::watch::Receiver<Vec<u8>>,
        tctx: tokio::sync::oneshot::Sender<egui::Context>,
    ) -> Self {
        let texture_handle =
            cc.egui_ctx
                .load_texture("rdp", ColorImage::example(), TextureOptions::default());
        tctx.send(cc.egui_ctx.clone())
            .expect("Failed to pass egui context to RDP session.");
        // We can then update the image via set partial
        // texture_handle.set_partial(pos, image, options);
        Self { texture_handle, rx }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE) // Remove default borders around the RDP view.
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    // TODO handle possible error.
                    if self.rx.has_changed().unwrap() {
                        // TODO better way to express initial lack of texture.
                        let bytes = self.rx.borrow().clone();
                        if bytes.len() > 0 {
                            let length = bytes.len() / 4;
                            let p = bytes.as_ptr() as *mut Color32;
                            let pixels = unsafe { Vec::from_raw_parts(p, length, length) };
                            mem::forget(bytes);
                            let image = egui::ColorImage {
                                size: [1024, 768], // TODO, temporary nasty hack.
                                pixels,
                            };
                            self.texture_handle
                                .set(image, egui::TextureOptions::NEAREST);
                        }
                    }

                    ui.add(
                        Image::new(SizedTexture::new(
                            self.texture_handle.id(),
                            // TODO hardcoded size.
                            Vec2 {
                                x: 1024.0,
                                y: 768.0,
                            },
                        ))
                        .shrink_to_fit(),
                    );
                });
            });
    }
}
