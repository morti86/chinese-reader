# chinese-reader
An app that helps you learn Chinese by reading with the help of AI.
Depending on a system, it might require to install ONNX Runtime.

## OCR Model

OCR uses PaddleOCR, a Chinese engine that is superior to Tesseract and other popular OCR engines in recognizing Chinese characters.

Download the following files from Hugging Face
- https://huggingface.co/Kreuzberg/paddleocr-onnx-models/blob/main/PP-OCRv5_server_det_infer.onnx
- https://huggingface.co/Kreuzberg/paddleocr-onnx-models/blob/main/rec/chinese/model.onnx

And put them inside the models folder

Either use the image button to load image from clipboard, or load an image file.

## Deepl

Deepl requires to set up a free API key on Deepl website. You can then translate a selected portion of text.

## AI

Either subscribe to one of the supported providers to get an API key or set up a local Ollama chat. Local models that work best for the goal:

- gemma3 27b
- Qwen3.5 27b

The functions available:

- examples: give a few examples of the usage of selected word
- meaning: explain meaning without context
- grammar: explain grammar of a selected fragment
- explain: explain the meaning of a given fragment, with the whole text in context

## Notes

It allows you to save notes so you can go back to them and review later. Can also add the dictionary or AI output to notes.
