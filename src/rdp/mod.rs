use anyhow::anyhow;
use eframe::egui;
use ironrdp::connector::{self, Credentials};
use ironrdp::dvc::{encode_dvc_messages, DrdynvcClient, DvcEncode};
use ironrdp::pdu::rdp::capability_sets::MajorPlatformType;
use ironrdp::pdu::rdp::client_info::PerformanceFlags;
use ironrdp::session::image::DecodedImage;
use ironrdp::session::{ActiveStage, ActiveStageOutput};
use ironrdp::svc::{ChannelFlags, SvcProcessorMessages};
use ironrdp_tokio::{split_tokio_framed, FramedWrite};
use log::{debug, info};
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use vc::{GenericChannel, GenericChannelMessage};

mod network_client;
pub mod vc;

type UpgradedFramed = ironrdp_tokio::TokioFramed<ironrdp_tls::TlsStream<TcpStream>>;

pub struct RDPSession {
    width: u16,
    height: u16,
    config: connector::Config,
    dynamic_virtual_channels: Option<Vec<String>>,
}

// TODO be nice to have a builder pattern and default port (viz. 3389)
pub struct RDPCredentials {
    username: String,
    password: String,
    domain: Option<String>,
}

#[derive(Default)]
pub struct RDPSharedFramebuffer {
    pub image: Option<Vec<u8>>,
    pub height: u16,
    pub width: u16,
}

impl RDPCredentials {
    pub fn new(username: String, password: String, domain: Option<String>) -> Self {
        Self {
            username,
            password,
            domain,
        }
    }
}

impl RDPSession {
    pub fn from_credentials(credentials: RDPCredentials) -> Self {
        // TODO hard coded default screen size
        let width = 1024;
        let height = 768;

        let config = connector::Config {
            credentials: Credentials::UsernamePassword {
                username: credentials.username,
                password: credentials.password,
            },
            domain: credentials.domain,
            enable_tls: true,
            enable_credssp: true,
            keyboard_type: ironrdp::pdu::gcc::KeyboardType::IbmEnhanced,
            keyboard_subtype: 0,
            keyboard_layout: 0, // the server SHOULD use the default active input locale identifier
            keyboard_functional_keys_count: 12,
            ime_file_name: "".to_string(),
            dig_product_id: "".to_string(),
            desktop_size: connector::DesktopSize { width, height },
            desktop_scale_factor: 0, // Default to 0 per FreeRDP
            bitmap: None,            /*Some(connector::BitmapConfig {
                                         color_depth: 32,
                                         lossy_compression: false, // Minimize computer vision losses?
                                     }), */
            client_build: 1,
            client_name: "rcc".to_string(), // i.e. "RDP Channel Client"
            // NOTE: hardcode this value like in freerdp
            // https://github.com/FreeRDP/FreeRDP/blob/4e24b966c86fdf494a782f0dfcfc43a057a2ea60/libfreerdp/core/settings.c#LL49C34-L49C70
            client_dir: "C:\\Windows\\System32\\mstscax.dll".to_owned(),
            platform: MajorPlatformType::UNIX,
            hardware_id: None,
            license_cache: None,
            no_server_pointer: false,
            autologon: false,
            request_data: None,
            pointer_software_rendering: true,
            performance_flags: PerformanceFlags::DISABLE_FULLWINDOWDRAG,
        };

        Self {
            width,
            height,
            config,
            dynamic_virtual_channels: None,
        }
    }

    pub fn with_dynamic_channels(mut self, dynamic_channels: Option<Vec<String>>) -> Self {
        self.dynamic_virtual_channels = dynamic_channels;
        self
    }

    pub async fn connect(
        &self,
        host: &str,
        port: u16,
    ) -> anyhow::Result<(connector::ConnectionResult, UpgradedFramed)> {
        let stream = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .map_err(|e| connector::custom_err!("TCP Connection to RDP Server", e))?;
        let addr = stream
            .peer_addr()
            .map_err(|e| connector::custom_err!("Getting RDP Server address", e))?;

        let mut framed = ironrdp_tokio::TokioFramed::new(stream);

        let mut dynamic_channels = ironrdp::dvc::DrdynvcClient::new();
        for vc in self.dynamic_virtual_channels.clone().unwrap_or_default() {
            dynamic_channels =
                dynamic_channels.with_dynamic_channel(vc::GenericChannel::new(vc.to_owned()));
        }
        let mut connector = connector::ClientConnector::new(self.config.clone())
            .with_server_addr(addr)
            .with_static_channel(dynamic_channels);

        let should_upgrade = ironrdp_tokio::connect_begin(&mut framed, &mut connector).await?;
        let initial_stream = framed.into_inner_no_leftover();

        // Not sure 'server name' is always OK here, given port number suffix?
        // TODO In fact it seems to be very much not OK!
        let (upgraded_stream, server_public_key) = ironrdp_tls::upgrade(initial_stream, &host)
            .await
            .map_err(|e| connector::custom_err!("RDP TLS Upgrade", e))?;
        let upgraded = ironrdp_tokio::mark_as_upgraded(should_upgrade, &mut connector);

        let mut upgraded_framed = ironrdp_tokio::TokioFramed::new(upgraded_stream);

        let mut network_client = network_client::ReqwestNetworkClient::new();

        let connection_result = ironrdp_tokio::connect_finalize(
            upgraded,
            &mut upgraded_framed,
            connector,
            host.into(),
            server_public_key,
            Some(&mut network_client),
            None,
        )
        .await?;

        Ok((connection_result, upgraded_framed))
    }

    pub async fn session_thread(
        framed: UpgradedFramed,
        connection_result: connector::ConnectionResult,
        tx: tokio::sync::watch::Sender<Arc<Mutex<RDPSharedFramebuffer>>>,
        rctx: tokio::sync::oneshot::Receiver<egui::Context>,
    ) -> anyhow::Result<()> {
        let (mut reader, mut writer) = split_tokio_framed(framed);

        let height = connection_result.desktop_size.height;
        let width = connection_result.desktop_size.width;
        let mut image = DecodedImage::new(
            ironrdp::graphics::image_processing::PixelFormat::RgbX32,
            width,
            height,
        );
        let mut active_stage = ActiveStage::new(connection_result);

        let egui_ctx = rctx.await?;
        info!("RDP session waiting for GUI context");
        let shared_frame_buffer = tx.borrow().clone();
        loop {
            let outputs = tokio::select! {
                frame = reader.read_pdu() => {
                    let (action, payload) = frame?;
                    active_stage.process(&mut image, action, &payload)?
                }
            };

            for out in outputs {
                match out {
                    ActiveStageOutput::ResponseFrame(frame) => writer.write_all(&frame).await?,
                    ActiveStageOutput::GraphicsUpdate(_region) => {
                        // We don't want to do any compute in here, because it is called very frequently
                        // for incremental changes. Better to that in the GUI thread in batches.
                        {
                            let mut locked = shared_frame_buffer
                                .lock()
                                .expect("Failed to locked shared framebuffer");
                            // Just take a simple copy of the image buffer which the GUI thread can convert
                            // into a GPU texture.
                            locked.image = Some(image.data().to_vec());
                            locked.width = width;
                            locked.height = height;
                        }
                        if let Err(e) = tx.send(shared_frame_buffer.clone()) {
                            return Err(anyhow!("Failed sending image data to GUI: {}", e));
                        }
                        egui_ctx.request_repaint();
                    }
                    ActiveStageOutput::Terminate(reason) => {
                        return Err(anyhow!("RDP Session terminated with reason: {}", reason));
                    }
                    other => {
                        debug!("Unhandled RDP event: {:?}", other);
                    }
                }
            }
        }
    }
}
