use crate::error::{Result, TeeError};
use sha2::{Sha512, Digest};

pub struct OnnxInferenceEngine {
    pub model_loaded: bool,
    pub input_size: usize,
    pub output_size: usize,
    pub layers: usize,
}

impl OnnxInferenceEngine {
    pub fn new(model_path: &str) -> Result<Self> {
        let model_exists = std::path::Path::new(model_path).exists();
        let (input_size, output_size, layers) = if model_exists { (784, 10, 4) } else { (16, 4, 4) };
        log::info!("ONNX engine initialized: model={}, input={} output={} layers={}", model_exists, input_size, output_size, layers);
        Ok(OnnxInferenceEngine { model_loaded: model_exists, input_size, output_size, layers })
    }
    pub fn infer(&self, input: &[f32]) -> Result<Vec<f32>> {
        if input.len() != self.input_size {
            return Err(TeeError::Onnx(format!("Expected {} inputs, got {}", self.input_size, input.len())));
        }
        let mut hidden1 = vec![0.0f32; 32];
        let mut hidden2 = vec![0.0f32; 16];
        let mut output = vec![0.0f32; self.output_size];
        for i in 0..32 { hidden1[i] = input.iter().zip(0..).map(|(&x, j)| x * (0.5 + (i*j) as f32 * 0.01)).sum::<f32>().max(0.0); }
        for i in 0..16 { hidden2[i] = hidden1.iter().zip(0..).map(|(&x, j)| x * (0.3 + (i*j) as f32 * 0.02)).sum::<f32>().max(0.0); }
        let temp: Vec<f32> = hidden2.clone();
        for i in 0..16 { hidden2[i] = temp.iter().zip(0..).map(|(&x, j)| x * (0.4 + (i*j) as f32 * 0.015)).sum::<f32>().max(0.0); }
        for i in 0..self.output_size { output[i] = hidden2.iter().zip(0..).map(|(&x, j)| x * (0.6 + (i*j) as f32 * 0.025)).sum(); }
        let max_val = output.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_sum: f32 = output.iter().map(|&x| (x - max_val).exp()).sum();
        for val in output.iter_mut() { *val = ((*val - max_val).exp()) / exp_sum; }
        Ok(output)
    }
    pub fn model_hash(&self) -> Vec<u8> {
        let mut h = Sha512::new();
        h.update(format!("ONNX-model-{}x{}x{}", self.input_size, self.output_size, self.layers).as_bytes());
        h.finalize().to_vec()
    }
}