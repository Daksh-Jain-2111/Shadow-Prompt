from pathlib import Path

import numpy as np
import pandas as pd
import torch

from datasets import Dataset, concatenate_datasets

from transformers import (
    DistilBertTokenizerFast,
    DistilBertForTokenClassification,
    TrainingArguments,
    Trainer,
    DataCollatorForTokenClassification,
)

# =========================================================
# PATHS
# =========================================================

BASE_DIR = Path(__file__).resolve().parent

CONLL_PATH = (
    BASE_DIR /
    "data" /
    "raw" /
    "conull2003" /
    "train.parquet"
)

WIKIANN_PATH = (
    BASE_DIR /
    "data" /
    "raw" /
    "wikiann" /
    "en" /
    "train.parquet"
)

# =========================================================
# CHECK FILES
# =========================================================

print("\nChecking dataset files...\n")

print("CoNLL Path:", CONLL_PATH)
print("Exists:", CONLL_PATH.exists())

print("\nWikiANN Path:", WIKIANN_PATH)
print("Exists:", WIKIANN_PATH.exists())

if not CONLL_PATH.exists():
    raise FileNotFoundError(f"Missing file: {CONLL_PATH}")

if not WIKIANN_PATH.exists():
    raise FileNotFoundError(f"Missing file: {WIKIANN_PATH}")

# =========================================================
# LOAD DATA
# =========================================================

print("\nLoading datasets...\n")

conll_df = pd.read_parquet(CONLL_PATH)
wikiann_df = pd.read_parquet(WIKIANN_PATH)

print(f"CoNLL Loaded: {len(conll_df)}")
print(f"WikiANN Loaded: {len(wikiann_df)}")

# =========================================================
# DEBUG INFO
# =========================================================

print("\nCoNLL Columns:")
print(conll_df.columns)

print("\nWikiANN Columns:")
print(wikiann_df.columns)

print("\nCoNLL Sample:")
print(conll_df.head(1).to_dict())

print("\nWikiANN Sample:")
print(wikiann_df.head(1).to_dict())

# =========================================================
# KEEP REQUIRED COLUMNS
# =========================================================

required_columns = ["tokens", "ner_tags"]

conll_df = conll_df[required_columns]
wikiann_df = wikiann_df[required_columns]

# =========================================================
# CLEAN DATA
# =========================================================

def clean_dataframe(df):

    cleaned_rows = []

    for _, row in df.iterrows():

        tokens = row["tokens"]
        ner_tags = row["ner_tags"]

        # numpy array -> list
        if isinstance(tokens, np.ndarray):
            tokens = tokens.tolist()

        if isinstance(ner_tags, np.ndarray):
            ner_tags = ner_tags.tolist()

        # tuple -> list
        if isinstance(tokens, tuple):
            tokens = list(tokens)

        if isinstance(ner_tags, tuple):
            ner_tags = list(ner_tags)

        # validate
        if not isinstance(tokens, list):
            continue

        if not isinstance(ner_tags, list):
            continue

        if len(tokens) == 0:
            continue

        if len(tokens) != len(ner_tags):
            continue

        # safe conversion
        tokens = [str(x) for x in tokens]

        try:
            ner_tags = [int(x) for x in ner_tags]
        except:
            continue

        cleaned_rows.append({
            "tokens": tokens,
            "ner_tags": ner_tags
        })

    return pd.DataFrame(cleaned_rows)

print("\nCleaning datasets...\n")

conll_df = clean_dataframe(conll_df)
wikiann_df = clean_dataframe(wikiann_df)

print(f"Clean CoNLL Rows: {len(conll_df)}")
print(f"Clean WikiANN Rows: {len(wikiann_df)}")

if len(conll_df) == 0:
    raise ValueError("CoNLL dataset became empty.")

if len(wikiann_df) == 0:
    raise ValueError("WikiANN dataset became empty.")

# =========================================================
# CONVERT TO HF DATASET
# =========================================================

conll_dataset = Dataset.from_pandas(
    conll_df,
    preserve_index=False
)

wikiann_dataset = Dataset.from_pandas(
    wikiann_df,
    preserve_index=False
)

# =========================================================
# MERGE
# =========================================================

dataset = concatenate_datasets([
    conll_dataset,
    wikiann_dataset
])

print("\nMerged Dataset:")
print(dataset)

if len(dataset) > 0:

    print("\nSample:")
    print(dataset[0])

else:
    raise ValueError("Merged dataset is empty.")

# =========================================================
# LABELS
# =========================================================

label_list = [
    "O",
    "B-PER",
    "I-PER",
    "B-ORG",
    "I-ORG",
    "B-LOC",
    "I-LOC",
    "B-MISC",
    "I-MISC",
]

id2label = {
    i: label for i, label in enumerate(label_list)
}

label2id = {
    label: i for i, label in enumerate(label_list)
}

print("\nLabel Mapping:")
print(label2id)

# =========================================================
# TOKENIZER
# =========================================================

print("\nLoading tokenizer...\n")

tokenizer = DistilBertTokenizerFast.from_pretrained(
    "distilbert-base-uncased"
)

# =========================================================
# TOKENIZATION + LABEL ALIGNMENT
# =========================================================

def tokenize_and_align_labels(examples):

    tokenized_inputs = tokenizer(
        examples["tokens"],
        truncation=True,
        is_split_into_words=True,
        max_length=128,
    )

    aligned_labels = []

    for i, labels in enumerate(examples["ner_tags"]):

        word_ids = tokenized_inputs.word_ids(batch_index=i)

        previous_word_idx = None

        label_ids = []

        for word_idx in word_ids:

            # special tokens
            if word_idx is None:

                label_ids.append(-100)

            # first subword
            elif word_idx != previous_word_idx:

                if word_idx < len(labels):
                    label_ids.append(labels[word_idx])
                else:
                    label_ids.append(-100)

            # remaining subwords
            else:

                label_ids.append(-100)

            previous_word_idx = word_idx

        aligned_labels.append(label_ids)

    tokenized_inputs["labels"] = aligned_labels

    return tokenized_inputs

print("\nTokenizing dataset...\n")

tokenized_dataset = dataset.map(
    tokenize_and_align_labels,
    batched=True,
    remove_columns=dataset.column_names,
)

print("\nTokenization complete.")

# =========================================================
# MODEL
# =========================================================

print("\nLoading model...\n")

model = DistilBertForTokenClassification.from_pretrained(
    "distilbert-base-uncased",
    num_labels=len(label_list),
    id2label=id2label,
    label2id=label2id,
)

# =========================================================
# DATA COLLATOR
# =========================================================

data_collator = DataCollatorForTokenClassification(
    tokenizer=tokenizer
)

# =========================================================
# DEVICE
# =========================================================

device = torch.device(
    "cuda" if torch.cuda.is_available() else "cpu"
)

print(f"\nUsing device: {device}")

model.to(device)

# =========================================================
# TRAINING ARGS
# =========================================================

training_args = TrainingArguments(
    output_dir=str(BASE_DIR / "outputs"),

    num_train_epochs=1,

    per_device_train_batch_size=2,

    learning_rate=2e-5,

    weight_decay=0.01,

    logging_steps=50,

    save_steps=500,

    save_total_limit=2,

    evaluation_strategy="no",

    fp16=torch.cuda.is_available(),

    report_to="none",
)

# =========================================================
# TRAINER
# =========================================================

trainer = Trainer(
    model=model,

    args=training_args,

    train_dataset=tokenized_dataset,

    data_collator=data_collator,
)

# =========================================================
# TRAIN
# =========================================================

print("\nStarting Training...\n")

trainer.train()

# =========================================================
# SAVE MODEL
# =========================================================

SAVE_PATH = BASE_DIR / "saved_model"

print("\nSaving model...\n")

trainer.save_model(str(SAVE_PATH))

tokenizer.save_pretrained(str(SAVE_PATH))

print("\nTraining Complete.")
print(f"\nModel saved at: {SAVE_PATH}")