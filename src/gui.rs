use std::sync::{Arc, Mutex};

use eframe::egui::{
    self, load::SizedTexture, Color32, ColorImage, Image, TextureHandle, TextureOptions, Vec2,
};

use crate::rdp::RDPSharedFramebuffer;

pub struct App {
    pub texture_handle: TextureHandle,
    pub rx: tokio::sync::watch::Receiver<Arc<Mutex<RDPSharedFramebuffer>>>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        rx: tokio::sync::watch::Receiver<Arc<Mutex<RDPSharedFramebuffer>>>,
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
                        {
                            let shared_framebuffer = self.rx.borrow().clone();
                            let mut locked = shared_framebuffer
                                .lock()
                                .expect("Failed to lock shared framebuffer");
                            // This slightly manky approach to updating the framebuffer manages to
                            // play full motion video in a debug build.
                            if let Some(mut image) = locked.image.take() {
                                let p_image = image.as_mut_ptr() as *mut Color32;
                                let size = locked.width as usize * locked.height as usize;
                                let pixels =
                                    unsafe { Vec::from_raw_parts(p_image, size, size).clone() };
                                std::mem::forget(image);
                                let image = egui::ColorImage {
                                    pixels,
                                    size: [locked.width as usize, locked.height as usize],
                                };
                                // TODO reset displayed frame when image is None.
                                self.texture_handle
                                    .set(image, egui::TextureOptions::NEAREST);
                            }
                        }
                    }

                    ui.add(
                        Image::new(SizedTexture::new(
                            self.texture_handle.id(),
                            self.texture_handle.size_vec2(),
                        ))
                        .shrink_to_fit(),
                    );
                });
            });
    }
}
