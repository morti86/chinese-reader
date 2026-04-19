# chinese-reader
An app that helps you learn Chinese by reading with the help of AI.
Depending on a system, it might require to install ONNX Runtime.

## OCR Model

OCR uses PaddleOCR, a Chinese engine that is superior to Tesseract and other popular OCR engines in recognizing Chinese characters.

Download the following files from Hugging Face
- https://huggingface.co/monkt/paddleocr-onnx/blob/main/detection/v5/det.onnx
- https://huggingface.co/monkt/paddleocr-onnx/blob/main/languages/chinese/rec.onnx
- https://huggingface.co/monkt/paddleocr-onnx/blob/main/languages/chinese/dict.txt

And put them inside the models folder in your HOME directory (C:\Users\{user name}\models or /home/{user name}/models)

Either use the image button to load image from clipboard, or load an image file.

## AI

Either subscribe to one of the supported providers to get an API key or set up a local Ollama chat. Among local models, Gemma4 is recommended for this task.

The functions available:

- examples: give a few examples of the usage of selected word
- meaning: explain meaning without context
- grammar: explain grammar of a selected fragment
- explain: explain the meaning of a given fragment, with the whole text in context
- translate: to your target language (app language)

## Notes

It allows you to save notes so you can go back to them and review later. Can also add the dictionary or AI output to notes.

## Usage

<img width="716" height="155" alt="image" src="https://github.com/user-attachments/assets/76db5143-457d-4b99-b39c-4d7119ad748a" />

Here pick the third button to manage texts in a database.

<img width="692" height="147" alt="image" src="https://github.com/user-attachments/assets/db8e9d7b-42ac-4aef-8162-77c37b889776" />

Here you can:
- select one of the texts in the base and click the first button to load it
- delete selected text
- add a new text (it will ask you to provide the name), saving the contents that are in the main window
- cancel

