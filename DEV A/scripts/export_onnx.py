import torch

from transformers import (
    AutoTokenizer,
    AutoModelForTokenClassification
)

MODEL_PATH = "./models/final_money_date_model"

print("\nLoading model...\n")

model = AutoModelForTokenClassification.from_pretrained(
    MODEL_PATH
)

tokenizer = AutoTokenizer.from_pretrained(
    MODEL_PATH
)

model.eval()

print("\nPreparing dummy input...\n")

dummy_input = tokenizer(

    "Rahul Kapoor transferred ₹50,000 on 12 January 2025",

    return_tensors="pt",

    padding="max_length",

    truncation=True,

    max_length=128
)

print("\nExporting ONNX...\n")

torch.onnx.export(

    model,

    (
        dummy_input["input_ids"],
        dummy_input["attention_mask"]
    ),

    "models/model.onnx",

    input_names=[
        "input_ids",
        "attention_mask"
    ],

    output_names=[
        "logits"
    ],

    dynamic_axes={

        "input_ids": {
            0: "batch_size",
            1: "sequence"
        },

        "attention_mask": {
            0: "batch_size",
            1: "sequence"
        },

        "logits": {
            0: "batch_size",
            1: "sequence"
        }
    },

    opset_version=14
)

print("\n✅ ONNX export successful!")