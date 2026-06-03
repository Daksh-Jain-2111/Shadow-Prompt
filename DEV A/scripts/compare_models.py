from transformers import (
    AutoTokenizer
)

import onnxruntime as ort
import numpy as np

# =========================================================
# TOKENIZER
# =========================================================

tokenizer = AutoTokenizer.from_pretrained(
    "./models/final_money_date_model"
)

# =========================================================
# MODELS
# =========================================================

fp32_session = ort.InferenceSession(
    "models/model.onnx"
)

int8_session = ort.InferenceSession(
    "models/model_int8.onnx"
)

# =========================================================
# TEST TEXTS
# =========================================================

texts = [

    "Rahul Kapoor transferred ₹50,000 through HDFC Bank on 12 January 2025",

    "Priya Sharma received salary of ₹1 lakh on 5 March 2024",

    "Ananya Iyer applied for loan of ₹2,50,000 from ICICI Bank on 15-09-2024"
]

# =========================================================
# RUN TESTS
# =========================================================

for text in texts:

    print("\n===================================")

    print("TEXT:")

    print(text)

    print("===================================\n")

    encoded = tokenizer(

        text,

        return_tensors="np",

        padding="max_length",

        truncation=True,

        max_length=128
    )

    inputs = {

        "input_ids":
            encoded["input_ids"],

        "attention_mask":
            encoded["attention_mask"]
    }

    # FP32
    fp32_outputs = fp32_session.run(
        None,
        inputs
    )[0]

    # INT8
    int8_outputs = int8_session.run(
        None,
        inputs
    )[0]

    # Compare
    fp32_pred = np.argmax(
        fp32_outputs,
        axis=-1
    )

    int8_pred = np.argmax(
        int8_outputs,
        axis=-1
    )

    matches = (
        fp32_pred == int8_pred
    ).sum()

    total = fp32_pred.size

    similarity = (
        matches / total
    ) * 100

    print(
        f"Prediction Similarity: "
        f"{similarity:.2f}%"
    )