//! Background worker pool for asset decoding.

use crossbeam_channel::unbounded;
use std::thread::JoinHandle;

use crate::core::{spawn_named_thread, ChannelReceiver, ChannelSender, ZirconError};

use crate::asset::load::{mesh, texture};
use crate::asset::types::{AssetRequest, CpuAssetPayload};

pub struct AssetWorkerPool {
    request_tx: Option<ChannelSender<AssetRequest>>,
    completion_rx: ChannelReceiver<CpuAssetPayload>,
    joins: Vec<JoinHandle<()>>,
}

impl AssetWorkerPool {
    pub fn new(worker_count: usize) -> Result<Self, ZirconError> {
        let worker_count = worker_count.max(1);
        let (request_tx, request_rx) = unbounded();
        let (completion_tx, completion_rx) = unbounded();
        let mut joins = Vec::with_capacity(worker_count);

        for worker_index in 0..worker_count {
            let request_rx = request_rx.clone();
            let completion_tx = completion_tx.clone();
            joins.push(spawn_named_thread(
                format!("zircon-asset-{worker_index}"),
                move || {
                    while let Ok(request) = request_rx.recv() {
                        let payload = process_request(request);
                        let _ = completion_tx.send(payload);
                    }
                },
            )?);
        }

        Ok(Self {
            request_tx: Some(request_tx),
            completion_rx,
            joins,
        })
    }

    pub fn request(&self, request: AssetRequest) -> Result<(), ZirconError> {
        self.request_tx
            .as_ref()
            .expect("asset worker request sender alive")
            .send(request.clone())
            .map_err(|_| ZirconError::ChannelSend(format!("asset request dropped: {request:?}")))
    }

    pub fn request_sender(&self) -> ChannelSender<AssetRequest> {
        self.request_tx
            .as_ref()
            .expect("asset worker request sender alive")
            .clone()
    }

    pub fn completion_receiver(&self) -> ChannelReceiver<CpuAssetPayload> {
        self.completion_rx.clone()
    }
}

impl Drop for AssetWorkerPool {
    fn drop(&mut self) {
        self.request_tx.take();

        for join in self.joins.drain(..) {
            let _ = join.join();
        }
    }
}

fn process_request(request: AssetRequest) -> CpuAssetPayload {
    match request {
        AssetRequest::Texture(source) => match texture::load_texture(&source) {
            Ok(texture) => CpuAssetPayload::Texture(texture),
            Err(message) => CpuAssetPayload::Failure {
                request: AssetRequest::Texture(source),
                message,
            },
        },
        AssetRequest::Mesh(source) => match mesh::load_mesh(&source) {
            Ok(mesh) => CpuAssetPayload::Mesh(mesh),
            Err(message) => CpuAssetPayload::Failure {
                request: AssetRequest::Mesh(source),
                message,
            },
        },
    }
}
