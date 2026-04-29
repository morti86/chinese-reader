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

If you use local AI models with llama.cpp, you can manage configurations and start/stop llama-server from this app now.
<img width="1479" height="757" alt="image" src="https://github.com/user-attachments/assets/868fc151-da7f-4cae-bf0c-995744d7881f" />
Just show the paths to model and mmproj (if you want vision) and then use the button with coloured square:
<img width="360" height="63" alt="image" src="https://github.com/user-attachments/assets/9d085dfa-48ee-4f72-8efc-e0d376b13542" />
- ⬜️ - local llama.cpp found, not running
- 🟩 - running, ready
- 🟥 - starting, not ready, not responding
- 🟦 - running, ready, but not started from this app
- 🟪 - streaming response
- ⬛️ - not available
For Ollama users to the left of it there is a "toggle keep alive" button so it does not unload a model while reading.

The other buttons (left to right)
- copy AI response to the text tab
- copy response to notes
- toggle Ollama keep alive
- llama.cpp state
- cancel currently running AI prompt
- clear the conversation history

## Improved prompter
<img width="453" height="207" alt="image" src="https://github.com/user-attachments/assets/e606b71d-6115-43bb-ac62-4ba410d3c354" />
Improved prompter lets you choose whether to include the text and/or a loaded image with your prompt.

If you write empty brackets `{}` in the chat, it will replace them with the selected fragment of text, e.g.

_Explain the meaning of the following fragment: {}_

## Deepl
<img width="575" height="42" alt="image" src="https://github.com/user-attachments/assets/28495504-962b-4646-98f6-65338f30d628" />
Here you choose the language and set up your API key.
