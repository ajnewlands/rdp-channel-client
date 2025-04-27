use ironrdp::dvc::{DvcEncode, DvcProcessor};
use ironrdp_core::{impl_as_any, Encode};
use log::error;

#[derive(Debug)]
pub struct GenericChannel {
    name: String,
}

impl GenericChannel {
    pub fn new(name: String) -> Self {
        GenericChannel { name }
    }
}

impl_as_any!(GenericChannel);

impl DvcProcessor for GenericChannel {
    fn channel_name(&self) -> &str {
        &self.name
    }

    fn start(
        &mut self,
        _channel_id: u32,
    ) -> ironrdp::pdu::PduResult<Vec<ironrdp::dvc::DvcMessage>> {
        Ok(Vec::default())
    }

    fn process(
        &mut self,
        _channel_id: u32,
        payload: &[u8],
    ) -> ironrdp::pdu::PduResult<Vec<ironrdp::dvc::DvcMessage>> {
        Ok(Vec::default())
    }
}

pub struct GenericChannelMessage {
    payload: String,
}

impl GenericChannelMessage {
    pub fn from_string(payload: String) -> Self {
        Self { payload }
    }
}

unsafe impl Send for GenericChannelMessage {}

impl Encode for GenericChannelMessage {
    fn name(&self) -> &'static str {
        ""
    }

    fn encode(&self, dst: &mut ironrdp_core::WriteCursor<'_>) -> ironrdp_core::EncodeResult<()> {
        dst.write_slice(self.payload.as_bytes());

        Ok(())
    }

    fn size(&self) -> usize {
        self.payload.as_bytes().len()
    }
}

impl DvcEncode for GenericChannelMessage {}
