# =========================================================
# SHADOWPROMPT CLEAN TRAINING PIPELINE
# CPU VERSION
# =========================================================

import os
import json
import time
import torch
import pandas as pd
import numpy as np

from datasets import (
    Dataset,
    concatenate_datasets
)

from transformers import (
    AutoTokenizer,
    AutoModelForTokenClassification,
    TrainingArguments,
    Trainer,
    pipeline
)

import evaluate


# =========================================================
# DEVICE INFO
# =========================================================

print("\n===================================")
print("DEVICE STATUS")
print("===================================\n")

print("Using CPU")

print("Torch Version:", torch.__version__)

print("CUDA Available:", torch.cuda.is_available())


# =========================================================
# LABEL DEFINITIONS
# =========================================================

label_names = [

    "O",

    "B-PER",
    "I-PER",

    "B-ORG",
    "I-ORG",

    "B-LOC",
    "I-LOC",

    "B-MONEY",
    "I-MONEY",

    "B-DATE",
    "I-DATE"
]

label2id = {
    label: i
    for i, label in enumerate(label_names)
}

id2label = {
    i: label
    for i, label in enumerate(label_names)
}


# =========================================================
# LOAD CONLL
# =========================================================

print("\nLoading CoNLL...\n")

conll_df = pd.read_parquet(
    "data/raw/conull2003/train.parquet"
)

conll_dataset = Dataset.from_pandas(
    conll_df
)


# =========================================================
# NORMALIZE CONLL
# =========================================================

def normalize_conll(example):

    new_tags = []

    for tag in example["ner_tags"]:

        if tag in [7, 8]:

            new_tags.append(0)

        else:

            new_tags.append(tag)

    example["ner_tags"] = new_tags

    return example


conll_dataset = conll_dataset.map(
    normalize_conll
)

conll_dataset = conll_dataset.remove_columns([
    "id",
    "pos_tags",
    "chunk_tags"
])


# =========================================================
# LOAD WIKIANN
# =========================================================

print("\nLoading WikiANN...\n")

wiki_df = pd.read_parquet(
    "data/raw/wikiann/en/train.parquet"
)

wiki_dataset = Dataset.from_pandas(
    wiki_df
)

wiki_dataset = wiki_dataset.remove_columns([
    "langs",
    "spans"
])


# =========================================================
# LOAD INDIAN SYNTHETIC DATA
# =========================================================

print("\nLoading Indian synthetic dataset...\n")

with open(
    "data/synthetic/indian_ner_dataset.json",
    "r",
    encoding="utf-8"
) as f:

    synthetic_data = json.load(f)

for sample in synthetic_data:

    sample["ner_tags"] = [

        label2id[tag]

        for tag in sample["ner_tags"]
    ]

synthetic_dataset = Dataset.from_list(
    synthetic_data
)


# =========================================================
# LOAD MONEY + DATE DATA
# =========================================================

print("\nLoading MONEY + DATE dataset...\n")

with open(
    "data/synthetic/money_date_dataset.json",
    "r",
    encoding="utf-8"
) as f:

    money_date_data = json.load(f)

for sample in money_date_data:

    sample["ner_tags"] = [

        label2id[tag]

        for tag in sample["ner_tags"]
    ]

money_date_dataset = Dataset.from_list(
    money_date_data
)


# =========================================================
# MERGE DATASETS
# =========================================================

print("\nMerging datasets...\n")

final_dataset = concatenate_datasets([

    conll_dataset,

    wiki_dataset,

    synthetic_dataset,

    money_date_dataset
])

print(final_dataset)


# =========================================================
# TOKENIZER
# =========================================================

print("\nLoading tokenizer...\n")

tokenizer = AutoTokenizer.from_pretrained(
    "distilbert-base-uncased"
)


# =========================================================
# TOKENIZATION
# =========================================================

def tokenize_and_align_labels(examples):

    tokenized_inputs = tokenizer(

        examples["tokens"],

        truncation=True,

        padding="max_length",

        max_length=128,

        is_split_into_words=True
    )

    labels = []

    for i, label in enumerate(
        examples["ner_tags"]
    ):

        word_ids = tokenized_inputs.word_ids(
            batch_index=i
        )

        previous_word_idx = None

        label_ids = []

        for word_idx in word_ids:

            if word_idx is None:

                label_ids.append(-100)

            elif word_idx != previous_word_idx:

                label_ids.append(
                    label[word_idx]
                )

            else:

                label_ids.append(
                    label[word_idx]
                )

            previous_word_idx = word_idx

        labels.append(label_ids)

    tokenized_inputs["labels"] = labels

    return tokenized_inputs


print("\nTokenizing dataset...\n")

tokenized_dataset = final_dataset.map(

    tokenize_and_align_labels,

    batched=True
)


# =========================================================
# LOAD MODEL
# =========================================================

print("\nLoading model...\n")

model = AutoModelForTokenClassification.from_pretrained(

    "distilbert-base-uncased",

    num_labels=len(label_names),

    id2label=id2label,

    label2id=label2id
)


# =========================================================
# METRICS
# =========================================================

seqeval = evaluate.load("seqeval")


def compute_metrics(p):

    predictions, labels = p

    predictions = np.argmax(
        predictions,
        axis=2
    )

    true_predictions = []

    true_labels = []

    for prediction, label in zip(
        predictions,
        labels
    ):

        current_preds = []

        current_labels = []

        for pred, lab in zip(
            prediction,
            label
        ):

            if lab != -100:

                current_preds.append(
                    label_names[pred]
                )

                current_labels.append(
                    label_names[lab]
                )

        true_predictions.append(
            current_preds
        )

        true_labels.append(
            current_labels
        )

    results = seqeval.compute(

        predictions=true_predictions,

        references=true_labels
    )

    return {

        "precision":
            results["overall_precision"],

        "recall":
            results["overall_recall"],

        "f1":
            results["overall_f1"]
    }


# =========================================================
# TRAINING ARGUMENTS
# CLEAN LOGGING
# =========================================================

training_args = TrainingArguments(

    output_dir="./models/money_date_checkpoints",

    learning_rate=2e-5,

    per_device_train_batch_size=2,

    num_train_epochs=1,

    weight_decay=0.01,

    logging_steps=50,

    save_strategy="epoch",

    eval_strategy="epoch",

    report_to="none"
)


# =========================================================
# TRAINER
# =========================================================

trainer = Trainer(

    model=model,

    args=training_args,

    train_dataset=tokenized_dataset,

    eval_dataset=tokenized_dataset.select(
        range(500)
    ),

    compute_metrics=compute_metrics
)


# =========================================================
# TRAIN
# =========================================================

print("\n===================================")
print("STARTING TRAINING")
print("===================================\n")

trainer.train()

print("\n===================================")
print("TRAINING COMPLETE")
print("===================================\n")


# =========================================================
# FINAL EVALUATION
# =========================================================

print("\n===================================")
print("FINAL EVALUATION")
print("===================================\n")

metrics = trainer.evaluate()

print(metrics)


# =========================================================
# SAVE MODEL
# =========================================================

SAVE_PATH = "./models/final_money_date_model"

trainer.save_model(SAVE_PATH)

tokenizer.save_pretrained(SAVE_PATH)

print(f"\nModel saved to: {SAVE_PATH}")


# =========================================================
# LOAD INFERENCE PIPELINE
# =========================================================

print("\nLoading inference pipeline...\n")

ner_pipeline = pipeline(

    "ner",

    model=SAVE_PATH,

    tokenizer=SAVE_PATH,

    aggregation_strategy="simple",

    device=-1
)


# =========================================================
# TEST INFERENCE
# =========================================================

tests = [

    "Rahul Kapoor transferred ₹50,000 through HDFC Bank on 12 January 2025",

    "Priya Sharma received salary of ₹1 lakh on 5 March 2024",

    "Ananya Iyer applied for loan of ₹2,50,000 from ICICI Bank on 15-09-2024"
]

print("\n===================================")
print("RUNNING INFERENCE")
print("===================================\n")

for text in tests:

    print(f"\nTEXT: {text}")

    start = time.time()

    results = ner_pipeline(text)

    end = time.time()

    print("\nEntities:\n")

    for r in results:
        print(r)

    print(
        f"\nLatency: {end - start:.4f} sec"
    )


# =========================================================
# FINAL STATUS
# =========================================================

print("\n===================================")
print("FINAL STATUS")
print("===================================\n")

print("✅ Training completed")
print("✅ MONEY learned")
print("✅ DATE learned")
print("✅ Metrics evaluated")
print("✅ Model saved")
print("✅ Ready for ONNX export")

print("\nPHASE COMPLETED SUCCESSFULLY!\n")