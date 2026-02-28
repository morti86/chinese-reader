pub mod det;
pub mod rec;

use std::path::Path;
use std::sync::Arc;

use det::Det;
use rec::Rec;
use tokio::sync::RwLock;

use crate::error::ReaderResult;
use crate::utils::get_image;

const REC_MIN: f32 = 0.75;

pub const REC_FILE: &str = "model.onnx";
pub const DET_FILE: &str = "PP-OCRv5_server_det_infer.onnx";
pub const INDEX: &str = "dict.txt";

pub async fn ocr_i(model_path: &str, content: Arc<RwLock<Vec<u8>>>) -> ReaderResult<String> {
    let det_path = model_path.to_owned()+DET_FILE;
    let keys = model_path.to_owned() + INDEX;
    let rec_path = model_path.to_owned()+REC_FILE;
    let mut det = Det::from_file(det_path)?;
    let mut rec = Rec::from_file(rec_path, keys)?.with_min_score(REC_MIN);
    let i_read = content.read().await;
    let img = image::load_from_memory(i_read.as_slice())?;

    let mut res = String::from("");
    for sub in det.find_text_img(&img)? {
        res.push_str(rec.predict_str(&sub)?.as_str());
    }
    Ok(res)

}

pub async fn ocr(model_path: &str) -> ReaderResult<String> {
    let det_path = model_path.to_owned()+DET_FILE;
    let keys = model_path.to_owned() + INDEX;
    let rec_path = model_path.to_owned()+REC_FILE;
    let content = get_image();
    let mut det = Det::from_file(det_path)?;
    let mut rec = Rec::from_file(rec_path, keys)?.with_min_score(REC_MIN);
    let img = image::load_from_memory(content.as_slice())?;

    let mut res = String::from("");
    for sub in det.find_text_img(&img)? {
        res.push_str(rec.predict_str(&sub)?.as_str());
    }
    Ok(res)
}

pub async fn ocr_file(model_path: &str, file_name: impl AsRef<Path>) -> ReaderResult<String> {
    let det_path = model_path.to_owned()+DET_FILE;
    let keys = model_path.to_owned() + INDEX;
    let rec_path = model_path.to_owned()+REC_FILE;
    
    let mut det = Det::from_file(det_path.as_str())?;
    let mut rec = Rec::from_file(rec_path.as_str(),keys.as_str())?;
    let img = image::ImageReader::open(file_name)?;
    let mut res = String::new();
    let r = img.decode()?;
    for sub in det.find_text_img(&r)? {
        res.push_str(rec.predict_str(&sub)?.as_str());
    }
    Ok(res)
}
