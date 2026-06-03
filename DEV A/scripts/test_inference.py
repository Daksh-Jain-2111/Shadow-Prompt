import time
import pandas as pd

from datasets import Dataset, concatenate_datasets

from transformers import (
    AutoTokenizer,
    AutoModelForTokenClassification,
    TrainingArguments,
    Trainer,
    pipeline
)

# =========================================================
# 1. LOAD LOCAL PARQUET DATASETS
# =========================================================

print("\nLoading datasets...\n")

conll_df = pd.read_parquet(
    "data/raw/conll/train.parquet"
)

wiki_df = pd.read_parquet(
    "data/raw/wikiann/en/train.parquet"
)

print("CoNLL Samples:", len(conll_df))
print("WikiANN Samples:", len(wiki_df))


# =========================================================
# 2. CONVERT TO HF DATASETS
# =========================================================

conll_dataset = Dataset.from_pandas(conll_df)
wiki_dataset = Dataset.from_pandas(wiki_df)


# =========================================================
# 3. NORMALIZE LABELS
# REMOVE MISC LABELS FROM CONLL
# =========================================================

def normalize_conll(example):

    new_tags = []

    for tag in example["ner_tags"]:

        # Convert MISC -> O
        if tag in [7, 8]:
            new_tags.append(0)

        else:
            new_tags.append(tag)

    example["ner_tags"] = new_tags

    return example


conll_dataset = conll_dataset.map(normalize_conll)


# =========================================================
# 4. REMOVE UNUSED COLUMNS
# =========================================================

conll_dataset = conll_dataset.remove_columns([
    "id",
    "pos_tags",
    "chunk_tags"
])

wiki_dataset = wiki_dataset.remove_columns([
    "langs",
    "spans"
])


# =========================================================
# 5. MERGE DATASETS
# =========================================================

merged_dataset = concatenate_datasets([
    conll_dataset,
    wiki_dataset
])

print("\nMerged Dataset Created!")
print(merged_dataset)

print("\nSample Example:\n")
print(merged_dataset[0])


# =========================================================
# 6. LABEL DEFINITIONS
# =========================================================

label_names = [
    "O",
    "B-PER",
    "I-PER",
    "B-ORG",
    "I-ORG",
    "B-LOC",
    "I-LOC"
]

id2label = {
    i: label
    for i, label in enumerate(label_names)
}

label2id = {
    label: i
    for i, label in enumerate(label_names)
}


# =========================================================
# 7. LOAD TOKENIZER
# =========================================================

print("\nLoading tokenizer...\n")

tokenizer = AutoTokenizer.from_pretrained(
    "distilbert-base-uncased"
)


# =========================================================
# 8. TOKENIZE + ALIGN LABELS
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

    for i, label in enumerate(examples["ner_tags"]):

        word_ids = tokenized_inputs.word_ids(
            batch_index=i
        )

        previous_word_idx = None
        label_ids = []

        for word_idx in word_ids:

            if word_idx is None:
                label_ids.append(-100)

            elif word_idx != previous_word_idx:
                label_ids.append(label[word_idx])

            else:
                label_ids.append(label[word_idx])

            previous_word_idx = word_idx

        labels.append(label_ids)

    tokenized_inputs["labels"] = labels

    return tokenized_inputs


print("\nTokenizing dataset...\n")

tokenized_dataset = merged_dataset.map(
    tokenize_and_align_labels,
    batched=True
)


# =========================================================
# 9. LOAD MODEL
# =========================================================

print("\nLoading model...\n")

model = AutoModelForTokenClassification.from_pretrained(
    "distilbert-base-uncased",
    num_labels=len(label_names),
    id2label=id2label,
    label2id=label2id
)


# =========================================================
# 10. TRAINING ARGUMENTS
# =========================================================

training_args = TrainingArguments(
    output_dir="./models/checkpoints",

    learning_rate=2e-5,

    per_device_train_batch_size=8,

    num_train_epochs=1,

    weight_decay=0.01,

    logging_steps=10,

    save_strategy="epoch",

    report_to="none"
)


# =========================================================
# 11. TRAINER
# =========================================================

trainer = Trainer(
    model=model,
    args=training_args,
    train_dataset=tokenized_dataset
)


# =========================================================
# 12. TRAIN MODEL
# =========================================================

print("\n====================================")
print("STARTING TRAINING")
print("====================================\n")

trainer.train()

print("\n====================================")
print("TRAINING COMPLETED")
print("====================================\n")


# =========================================================
# 13. SAVE MODEL
# =========================================================

trainer.save_model("./models/final_model")

tokenizer.save_pretrained("./models/final_model")

print("\nModel Saved Successfully!\n")


# =========================================================
# 14. LOAD PIPELINE FOR INFERENCE
# =========================================================

print("\nLoading inference pipeline...\n")

ner_pipeline = pipeline(
    "ner",
    model="./models/final_model",
    tokenizer="./models/final_model",
    aggregation_strategy="simple"
)


# =========================================================
# 15. TEST SENTENCES
# =========================================================

tests = [

    "Rahul Kapoor works at Infosys",

    "Priya Sharma lives in Mumbai",

    "HDFC Bank approved the loan",

    "Aadhaar number is 234523456930",

    "PAN card ABCDE1234F",

    "ICICI Bank transferred money to Raj Mehta",

    "Anjali Verma visited Delhi"
]


# =========================================================
# 16. RUN INFERENCE + LATENCY
# =========================================================

print("\n====================================")
print("RUNNING INFERENCE TESTS")
print("====================================\n")

for text in tests:

    print(f"\nTEXT: {text}")

    start = time.time()

    results = ner_pipeline(text)

    end = time.time()

    print("\nEntities:")

    for r in results:
        print(r)

    print(f"\nLatency: {end - start:.4f} sec")


# =========================================================
# 17. MODEL SIZE CHECK
# =========================================================

import os

model_path = "./models/final_model"

total_size = 0

for path, dirs, files in os.walk(model_path):

    for f in files:

        fp = os.path.join(path, f)

        total_size += os.path.getsize(fp)

size_mb = total_size / (1024 * 1024)

print("\n====================================")
print("MODEL SIZE")
print("====================================")

print(f"\nModel Size: {size_mb:.2f} MB")


# =========================================================
# 18. FINAL STATUS
# =========================================================

print("\n====================================")
print("PIPELINE STATUS")
print("====================================\n")

print("✅ Local parquet loading works")
print("✅ CoNLL + WikiANN merge works")
print("✅ Token alignment works")
print("✅ DistilBERT fine-tuning works")
print("✅ Checkpoint saving works")
print("✅ Inference works")
print("✅ Latency measured")
print("✅ Model size measured")

print("\nPROJECT PHASE COMPLETED SUCCESSFULLY!\n")