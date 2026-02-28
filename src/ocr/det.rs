use std::path::Path;
use image::{DynamicImage, GenericImageView, Luma, GrayImage};
use imageproc::{rect::Rect, point::Point};
use ndarray::{Array, ArrayBase, Dim, OwnedRepr};
use ort::inputs;
use ort::value::TensorRef;
use ort::session::{builder::SessionBuilder, Session};

use crate::error::{ReaderError, ReaderResult};

pub struct Det {
    model: Session
}

impl Det {
    pub fn new(model: Session) -> Self {
        Self { model }
    }

    pub fn from_file(model_path: impl AsRef<Path>) -> ReaderResult<Self> {
        let model = SessionBuilder::new()?.commit_from_file(model_path)?;
        Ok(Self { model })
    }

    pub fn find_text_rect(&mut self, img: &DynamicImage) -> ReaderResult<Vec<Rect>> {
        let input = Self::preprocess(img)?;
        let output = self.run_model(&input, img.width(), img.height())?;
        Ok(self.find_box(&output))
    }

    pub fn find_text_img(&mut self, img: &DynamicImage) -> ReaderResult<Vec<DynamicImage>> {
        Ok(self.find_text_rect(img)?
            .iter()
            .map(|r| img.crop_imm(r.left() as u32, r.top() as u32, r.width(), r.height()))
            .collect()
        )
    }

    fn preprocess(img: &DynamicImage) -> ReaderResult<ArrayBase<OwnedRepr<f32>, Dim<[usize; 4]>>> {
        let (w, h) = img.dimensions();
        let pad_w = Self::get_pad_length(w);
        let pad_h = Self::get_pad_length(h);

        let mut input = Array::zeros((1, 3, pad_h as usize, pad_w as usize));
        for pixel in img.pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b, _] = pixel.2 .0;
            input[[0, 0, y, x]] = (((r as f32) / 255.) - 0.485) / 0.229;
            input[[0, 1, y, x]] = (((g as f32) / 255.) - 0.456) / 0.224;
            input[[0, 2, y, x]] = (((b as f32) / 255.) - 0.406) / 0.225;
        }
        Ok(input)
    }

    fn run_model(&mut self, input: &ArrayBase<OwnedRepr<f32>, Dim<[usize; 4]>>, width: u32, height: u32) -> ReaderResult<GrayImage>{
        let pad_h = Self::get_pad_length(height);
        let outputs = self.model.run(inputs!["x" => TensorRef::from_array_view(input.view())?  ])?;
        let output = outputs.iter().next().ok_or(ReaderError::other("no output"))?.1;
        let output = output.try_extract_array::<f32>()?.t().into_owned();
        
        let output: Vec<_> = output.iter().collect();
        let img = image::ImageBuffer::from_fn(width, height, |x, y| {
            Luma([(*output[(x * pad_h + y) as usize] * 255.0).min(255.0) as u8])
        });
        Ok(img)
    }

    fn find_box(&self, img: &GrayImage) ->Vec<Rect> {
        let (w, h) = img.dimensions();
        imageproc::contours::find_contours_with_threshold::<u32>(img, 200)
            .into_iter()
            .filter_map(|x| {
                if x.parent.is_some() { return None; }

                Self::bounding_rect( &x.points )
                    .map(|x| Rect::at((x.left() - 8).max(0), (x.top() - 8).max(0))
                        .of_size((x.width() + 16).min(w), (x.height() + 16).min(h)))
            })
            .collect()
        }

    fn bounding_rect(points: &[Point<u32>]) -> Option<Rect> {
        let (x_min, x_max, y_min, y_max) = points.into_iter()
            .fold(None, |ret, p|{
                match ret {
                    None => Some((p.x, p.x, p.y, p.y)),
                    Some((x_min, x_max, y_min, y_max)) => {
                        Some((x_min.min(p.x), x_max.max(p.x), y_min.min(p.y), y_max.max(p.y)))
                    }
                }
            })?; 
        let width = (x_max - x_min) as u32;
        let height = (y_max - y_min) as u32;
        if width <= 5 || height <= 5 {
            return None;
        }
        Some(Rect::at(x_min as i32, y_min as i32).of_size(width, height))
    }

    const fn get_pad_length(length: u32) -> u32 {
        let i = length % 32;
        if i == 0 {
            length
        } else {
            length + 32 - i
        }
    }
}
