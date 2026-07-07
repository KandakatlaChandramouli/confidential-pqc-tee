use crate::error::Result;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnclaveNodeId(pub u64);

#[derive(Debug)]
pub struct EnclaveNode {
    pub id: EnclaveNodeId,
    pub address: String,
    pub model_shard: Option<usize>,
    pub is_leader: bool,
}

pub struct EnclaveMesh {
    pub nodes: Arc<Mutex<HashMap<EnclaveNodeId, EnclaveNode>>>,
    pub leader_id: Arc<Mutex<EnclaveNodeId>>,
}

impl EnclaveMesh {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        let leader = EnclaveNode { id: EnclaveNodeId(0), address: "localhost:8000".into(), model_shard: None, is_leader: true };
        nodes.insert(EnclaveNodeId(0), leader);
        EnclaveMesh { nodes: Arc::new(Mutex::new(nodes)), leader_id: Arc::new(Mutex::new(EnclaveNodeId(0))) }
    }
    pub fn add_node(&self, address: &str, model_shard: Option<usize>) -> Result<EnclaveNodeId> {
        let mut nodes = self.nodes.lock().unwrap();
        let id = EnclaveNodeId(nodes.len() as u64);
        nodes.insert(id.clone(), EnclaveNode { id: id.clone(), address: address.into(), model_shard, is_leader: false });
        Ok(id)
    }
    pub fn distributed_inference(&self, input: &[f32], shards: usize) -> Result<Vec<Vec<f32>>> {
        let shard_size = input.len() / shards.max(1);
        let mut results = Vec::new();
        for s in 0..shards {
            let start = s * shard_size;
            let end = if s == shards - 1 { input.len() } else { (s + 1) * shard_size };
            let shard_input = &input[start..end];
            let shard_result: Vec<f32> = shard_input.iter().map(|&x| x * 1.5 + 0.1).collect();
            results.push(shard_result);
        }
        Ok(results)
    }
    pub fn node_count(&self) -> usize { self.nodes.lock().unwrap().len() }
}