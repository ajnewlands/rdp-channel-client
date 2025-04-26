mod cli;
mod gui;
mod rdp;
use clap::Parser;
use eframe::egui;
use rdp::{RDPCredentials, RDPSession};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Prepare crypto provider for IronRDP
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls default provider");

    let cli = cli::Cli::parse();

    let credentials = RDPCredentials::new(cli.username, cli.password, cli.domain);
    let rdp = RDPSession::from_credentials(credentials);

    // So we can pass a handle to the egui context back to the RDP thread,
    // allowing it to trigger a repaint when the view should update.
    let (tctx, rctx) = tokio::sync::oneshot::channel::<egui::Context>();
    let (tx, rx) = tokio::sync::watch::channel::<Vec<u8>>(Vec::default());
    // TODO handle error in initial thread creation.
    let rdp_session_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();
        // TODO actual RDP session error handling
        let (connection_result, framed) = rt.block_on(rdp.connect(&cli.host, cli.port)).unwrap();
        rt.block_on(RDPSession::session_thread(
            framed,
            connection_result,
            tx,
            rctx,
        ))
    });

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        window_builder: Some(Box::new(|builder| builder.with_resizable(false))),

        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "RDP",
        native_options,
        Box::new(|cc| Ok(Box::new(gui::App::new(cc, rx, tctx)))),
    ) {
        log::error!("Failed to instantiate GUI: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = rdp_session_thread.join().expect("Error joining RDP thread") {
        log::error!("RDP Session error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
